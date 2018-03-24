#![recursion_limit="128"] // for error-chain
use std::io;
use std::thread;
use std::process;
use std::io::prelude::*;
use std::borrow::Borrow;
use std::time::Duration;

extern crate clap;
extern crate git2;
#[macro_use] extern crate error_chain;
#[macro_use] extern crate serde_derive;
extern crate serde_yaml;
extern crate app_dirs2;
extern crate regex;
extern crate reqwest;
extern crate rpassword;
extern crate serde;
extern crate serde_json;

mod repository;

mod errors;
use errors::*;

mod options;
use options::{get_options, Options};

mod git_operations;
use git_operations::*;

#[cfg(test)]
mod tests;

fn handle_retry<F: FnMut(&Options, &mut bool) -> Result<git2::Repository>>(options: &Options, mut f: F) -> Result<git2::Repository> {
    let mut stderr = io::stderr();
    let mut total_wait = 0;
    let mut timeout = Duration::from_secs(2);
    loop {
        let mut progressed = false;
        let res = f(options, &mut progressed);
        let should_retry = !progressed && if let &Err(Error(ErrorKind::Git(ref err), _)) = &res {
            err.class() == git2::ErrorClass::Net
        } else {
            false
        };
        if should_retry && total_wait > options.fork_timeout {
            return res.chain_err(|| ErrorKind::ForkTimedOut(total_wait));
        }
        if should_retry {
            if !options.quiet {
                writeln!(stderr, "Fork not yet created, waiting {} seconds", timeout.as_secs()).ok();
            }
            thread::sleep(timeout);
            total_wait += timeout.as_secs();
            timeout *= 2;
            continue;
        } else {
            return res;
        }
    }
}

fn main_inner() -> Result<()> {
    let mut stderr = io::stderr();
    let options = get_options().chain_err(|| "Failed to get options")?;
    if !options.quiet {
        writeln!(stderr, "Forking repository...").ok();
    }
    let fork_git_url = options.repository.fork(options.authentication.clone(), options.organization.as_ref().map(Borrow::borrow), options.origin_protocol.clone())
        .chain_err(|| "Failed to fork repository")?;
    if !options.quiet {
        writeln!(stderr, "Cloning repository...").ok();
    }
    let repo = handle_retry(&options, |options, progressed| clone_repo(&fork_git_url, &options.clone_path, &options.authentication, options.quiet, progressed)).chain_err(|| "Failed to clone repository")?;
    if options.setup_upstream {
        let mut remote = setup_upstream(&repo, &options.remote_name, &options.repository.get_git_url(options.upstream_protocol).chain_err(|| "Failed to get upstream git URL")?)
            .chain_err(|| "Failed to setup upstream")?;
        if options.track_upstream {
            if !options.quiet {
                writeln!(stderr, "Fetching and tracking upstream...").ok();
            }
            let mut master = get_head_branch(&repo)?;
            fetch_remote(&mut remote, &master, &options.authentication, true).chain_err(|| "Failed to fetch upstream")?;
            track_upstream(&mut master, &remote).chain_err(|| "Failed to set master to track upstream")?;
            hard_reset_fetch_head(&repo).chain_err(|| "Failed to hard reset to upstream")?;
        }
    }
    if !options.quiet {
        writeln!(stderr, "Done!").ok();
    }
    Ok(())
}

fn main() {
    if let Err(e) = main_inner() {
        let mut stderr = io::stderr();
        writeln!(stderr, "Error:     {}", e).expect("Failed to write to stderr");
        for e in e.iter().skip(1) {
            writeln!(stderr, "caused by: {}", e).expect("Failed to write to stderr");
        }
        if let Some(backtrace) = e.backtrace() {
            writeln!(stderr, "backtrace: {:?}", backtrace).expect("Failed to write to stderr");
        }
        process::exit(2);
    }
}
