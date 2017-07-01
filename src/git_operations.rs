use io::{self, Write};
use std::path::Path;
use std::sync::atomic::{self, AtomicBool};

use git2::{self, Repository, Remote, Branch, ResetType, FetchOptions, RemoteCallbacks};
use git2::build::RepoBuilder;

use options::Authentication;
use errors::*;

fn get_fetchoptions<'a>(quiet: bool, auth: &'a Authentication, progressed: &'a AtomicBool) -> FetchOptions<'a> {
    let mut options = FetchOptions::new();
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(move |url, username, allowed| {
        progressed.store(true, atomic::Ordering::Relaxed);
        let config = git2::Config::open_default()?;
        let mut cred_helper = git2::CredentialHelper::new(url);
        cred_helper.config(&config);
        if allowed.contains(git2::SSH_KEY) {
            let user = username.map(|s| s.to_string())
                               .or_else(|| cred_helper.username.clone())
                               .unwrap_or("git".to_string());
            git2::Cred::ssh_key_from_agent(&user)
        } else if allowed.contains(git2::USER_PASS_PLAINTEXT) {
            git2::Cred::userpass_plaintext(&auth.username, &auth.password)
        } else if allowed.contains(git2::DEFAULT) {
            git2::Cred::default()
        } else {
            Err(git2::Error::from_str("no authentication available"))
        }
    });
    if !quiet {
        let mut stage = 0;
        callbacks.transfer_progress(move |progress| {
            progressed.store(true, atomic::Ordering::Relaxed);
            let mut stderr = io::stderr();
            let objects = (progress.received_objects(), progress.total_objects());
            let deltas = (progress.indexed_deltas(), progress.total_deltas());
            if objects.0 != objects.1 {
                if stage == 0 {
                    stage = 1;
                } else {
                    write!(stderr, "\r").ok();
                }
                let percent = if objects.1 == 0 { 0 } else { 100*objects.0/objects.1 };
                write!(stderr, "Receiving objects: {}% ({}/{})", percent, objects.0, objects.1).ok();
                return true;
            }
            if stage == 1 {
                writeln!(stderr, "").ok();
                stage = 2;
            } else {
                write!(stderr, "\r").ok();
            }
            let percent = if deltas.1 == 0 { 0 } else { 100*deltas.0/deltas.1 };
            write!(stderr, "Receiving deltas: {}% ({}/{})", percent, deltas.0, deltas.1).ok();
            true
        });
        callbacks.sideband_progress(move |message| {
            progressed.store(true, atomic::Ordering::Relaxed);
            let mut stderr = io::stderr();
            write!(stderr, "\rremote: ").ok();
            stderr.write_all(message).ok();
            true
        });
    }
    options.remote_callbacks(callbacks);
    options
}

pub fn clone_repo<P: AsRef<Path>>(url: &str, location: P, auth: &Authentication, quiet: bool, progressed: &mut bool) -> Result<Repository> {
    let mut progressed_atomic = AtomicBool::new(*progressed);
    let repo = RepoBuilder::new().fetch_options(get_fetchoptions(quiet, auth, &mut progressed_atomic)).clone(url, location.as_ref())?;
    *progressed = progressed_atomic.into_inner();
    if !quiet {
        writeln!(io::stderr(), "").ok();
    }
    Ok(repo)
}

pub fn setup_upstream<'a>(repository: &'a Repository, name: &str, orig_url: &str) -> Result<Remote<'a>> {
    Ok(repository.remote(name, orig_url)?)
}

pub fn get_head_branch<'a>(repository: &'a Repository) -> Result<Branch<'a>> {
    Ok(Branch::wrap(repository.head()?))
}

pub fn fetch_remote(remote: &mut Remote, branch: &Branch, auth: &Authentication, quiet: bool) -> Result<()> {
    let mut progressed = AtomicBool::new(false);
    remote.fetch(&[branch.name()?.ok_or(ErrorKind::BranchNotNamed)?], Some(&mut get_fetchoptions(quiet, auth, &mut progressed)), None)?;
    if !quiet {
        writeln!(io::stderr(), "").ok();
    }
    Ok(())
}

pub fn track_upstream(branch: &mut Branch, remote: &Remote) -> Result<()> {
    let upstream_name = remote.name().ok_or(ErrorKind::RemoteNotNamed)?.to_string() + "/" + branch.name()?.ok_or(ErrorKind::BranchNotNamed)?;
    Ok(branch.set_upstream(Some(&upstream_name))?)
}

pub fn hard_reset_fetch_head(repo: &Repository) -> Result<()> {
    let fetch_head = repo.revparse_single("FETCH_HEAD")?;
    Ok(repo.reset(&fetch_head, ResetType::Hard, None)?)
}
