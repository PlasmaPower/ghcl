use std::io;
use std::fs::File;
use std::path::PathBuf;
use std::collections::HashMap;

use clap;
use serde_yaml;
use rpassword;
use errors::*;
use app_dirs2::{AppInfo, get_app_root, AppDataType};

use repository::{Repository, Service, GitProtocol};

const APP_INFO: AppInfo = AppInfo {
    name: "ghcl",
    author: "PlasmaPower",
};

#[derive(Debug, Clone)]
pub struct Authentication {
    pub username: String,
    pub password: String,
}

#[derive(Debug)]
pub struct Options {
    pub repository: Repository,
    pub organization: Option<String>,
    pub track_upstream: bool,
    pub setup_upstream: bool,
    pub remote_name: String,
    pub origin_protocol: GitProtocol,
    pub upstream_protocol: GitProtocol,
    pub authentication: Authentication,
    pub clone_path: String,
    pub quiet: bool,
    pub fork_timeout: u64,
}

#[derive(Debug, Deserialize)]
struct PartialAuthentication {
    username: Option<String>,
    password: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct Config {
    organization: Option<String>,
    track_upstream: Option<bool>,
    setup_upstream: Option<bool>,
    remote_name: Option<String>,
    origin_protocol: Option<GitProtocol>,
    upstream_protocol: Option<GitProtocol>,
    default_service: Option<Service>,
    quiet: Option<bool>,
    fork_timeout: Option<u64>,
    #[serde(default)]
    authentication: HashMap<Service, PartialAuthentication>,
}

fn ask_for(prompt: &'static str, secure: bool) -> io::Result<String> {
    if secure {
        rpassword::prompt_password_stderr(prompt)
    } else {
        rpassword::prompt_response_stderr(prompt)
    }
}

pub fn get_options() -> Result<Options> {
    let matches = clap::App::new("ghcl")
        .version("0.1.0")
        .author("Lee Bousfield <ljbousfield@gmail.com>")
        .about("Automatically forks and clones a GitHub repository")
        .arg(clap::Arg::with_name("repository")
             .value_name("REPOSITORY")
             .required(true)
             .help("Repository to fork and clone"))
        .arg(clap::Arg::with_name("clone_path")
             .value_name("CLONE_PATH")
             .help("Where to clone the repository (defaults to the name of the repo)"))
        .arg(clap::Arg::with_name("config")
             .short("c")
             .long("config")
             .value_name("FILE")
             .help("Sets a custom config file"))
        .arg(clap::Arg::with_name("organization")
             .short("o")
             .long("organization")
             .value_name("ORGANIZATION")
             .help("Fork into an organization"))
        .arg(clap::Arg::with_name("username")
             .short("u")
             .long("username")
             .value_name("USERNAME")
             .help("Your username"))
        .arg(clap::Arg::with_name("password")
             .short("p")
             .long("password")
             .value_name("PASSWORD")
             .help("Your password (insecure - use a personal access token and put it in your config, or input your password when prompted)"))
        .arg(clap::Arg::with_name("fork_timeout")
             .long("fork-timeout")
             .value_name("TIMEOUT")
             .help("The maximum timeout for the fork creation (default: 30)"))
        .arg(clap::Arg::with_name("track_upstream")
             .long("track-upstream")
             .help("Setup master to track upstream (default, imples setup-upstream)"))
        .arg(clap::Arg::with_name("setup_upstream")
             .long("setup-upstream")
             .help("Setup an upstream remote (default)"))
        .arg(clap::Arg::with_name("remote_name")
             .long("remote-name")
             .value_name("REMOTE_NAME")
             .help("The name of the upstream remote to create (default: \"upstream\")"))
        .arg(clap::Arg::with_name("origin_protocol")
             .long("origin-protocol")
             .value_name("GIT_PROTOCOL")
             .possible_values(&["ssh", "https", "SSH", "HTTPS"])
             .help("The git protocol to use for the origin (default: SSH)"))
        .arg(clap::Arg::with_name("upstream_protocol")
             .long("upstream-protocol")
             .value_name("GIT_PROTOCOL")
             .possible_values(&["ssh", "https", "SSH", "HTTPS"])
             .help("The git protocol to use for the upstream (default: HTTPS)"))
        .arg(clap::Arg::with_name("default_service")
             .short("s")
             .long("default-service")
             .value_name("SERVICE")
             .possible_values(&["github", "GitHub", "Github"])
             .help("The service to be used if the repository is in the form user/repo"))
        .arg(clap::Arg::with_name("no_track_upstream")
             .long("no-track-upstream")
             .conflicts_with("track_upstream")
             .help("Don't setup master to track upstream"))
        .arg(clap::Arg::with_name("no_upstream")
             .long("no-upstream")
             .conflicts_with_all(&["upstream", "track_upstream", "remote_name"])
             .help("Don't setup an upstream remote (implies no_track_upstream)"))
        .arg(clap::Arg::with_name("quiet")
             .short("q")
             .long("quiet")
             .help("Don't output status messages"))
        .arg(clap::Arg::with_name("no_quiet")
             .long("no-quiet")
             .conflicts_with_all(&["quiet"])
             .help("Don't be quiet (output status messages)"))
        .get_matches();
    let config_path: Result<PathBuf> = matches.value_of("config").map(PathBuf::from).map(Ok).unwrap_or_else(|| {
        let mut app_dir = get_app_root(AppDataType::UserConfig, &APP_INFO)?;
        app_dir.push("config.yml");
        Ok(app_dir)
    });
    let config_path = config_path?;
    let mut config = match File::open(config_path) {
        Ok(file) => serde_yaml::from_reader(file)?,
        Err(ref err) if err.kind() == io::ErrorKind::NotFound => Config::default(),
        Err(err) => Err(err)?,
    };
    let matches_track_upstream = if matches.is_present("track_upstream") {
        Some(true)
    } else if matches.is_present("no_track_upstream") || matches.is_present("no_upstream") {
        Some(false)
    } else {
        None
    };
    let matches_setup_upstream = if matches.is_present("setup_upstream") || matches.is_present("track_upstream") || matches.is_present("remote_name") {
        Some(true)
    } else if matches.is_present("no_upstream") {
        Some(false)
    } else {
        None
    };
    let matches_quiet = if matches.is_present("quiet") {
        Some(true)
    } else if matches.is_present("no_quiet") {
        Some(false)
    } else {
        None
    };
    if matches_track_upstream != Some(false) && config.track_upstream == Some(true) && config.setup_upstream == Some(false) {
        Err(ErrorKind::ConfigTrackNoSetup)?
    }
    let matches_origin_protocol = match matches.value_of("origin_protocol") {
        Some("https") | Some("HTTPS") => Some(GitProtocol::HTTPS),
        Some("ssh") | Some("SSH") => Some(GitProtocol::SSH),
        _ => None,
    };
    let matches_upstream_protocol = match matches.value_of("upstream_protocol") {
        Some("https") | Some("HTTPS") => Some(GitProtocol::HTTPS),
        Some("ssh") | Some("SSH") => Some(GitProtocol::SSH),
        _ => None,
    };
    let track_upstream = matches_track_upstream.or(config.track_upstream).or(config.setup_upstream).unwrap_or(true);
    let matches_default_service = match matches.value_of("default_service") {
        Some("github") | Some("GitHub") | Some("Github") => Some(Service::GitHub),
        _ => None,
    };
    let repository = Repository::from_arg_string(matches.value_of("repository").unwrap(), matches_default_service.or(config.default_service).unwrap_or(Service::GitHub))?;
    let service = repository.service;
    let mut config_auth = config.authentication.remove(&service);
    let username = matches.value_of("username").map(String::from).or(config_auth.as_mut().and_then(|auth| auth.username.take())).map(Ok).unwrap_or_else(|| ask_for("Username: ", false))?;
    let password = matches.value_of("password").map(String::from).or(config_auth.as_mut().and_then(|auth| auth.password.take())).map(Ok).unwrap_or_else(|| ask_for("Password: ", true))?;
    let clone_path = matches.value_of("clone_path").unwrap_or(&repository.name).into();
    Ok(Options {
        repository: repository,
        organization: matches.value_of("organization").map(String::from).or(config.organization),
        track_upstream: track_upstream,
        setup_upstream: track_upstream || matches_setup_upstream.or(config.setup_upstream).unwrap_or(true),
        remote_name: matches.value_of("remote_name").map(String::from).or(config.remote_name).unwrap_or_else(|| "upstream".into()),
        origin_protocol: matches_origin_protocol.or(config.origin_protocol).unwrap_or(GitProtocol::SSH),
        upstream_protocol: matches_upstream_protocol.or(config.upstream_protocol).unwrap_or(GitProtocol::HTTPS),
        authentication: Authentication { username: username, password: password },
        clone_path: clone_path,
        quiet: matches_quiet.or(config.quiet).unwrap_or(false),
        fork_timeout: matches.value_of("fork_timeout").map(|s| s.parse()).or(config.fork_timeout.map(Ok)).unwrap_or(Ok(30))?,
    })
}
