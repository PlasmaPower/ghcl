# ghcl: GitHub CLone

Automatically fork, clone, and setup an upstream remote for a GitHub repository. By default, the master branch will track the upstream remote.

Intended as a "quick-start" for contributing to a GitHub repository.
More services such as GitLab and BitBucket are coming soon™. The code is ready for multiple services, but the APIs for them haven't been implemented.

This project is licensed under the terms of the MIT license. See `LICENSE.txt` for more details

## Installation

Currently, all methods of installation require building from source, and a working Rust installation.
To quickly setup Rust (and Cargo), use [rustup](https://rustup.rs).

- Through crates.io: `cargo install ghcl`
- Through GitHub:
  ```sh
  git clone https://github.com/PlasmaPower/ghcl.git
  cd ghcl
  cargo build --release
  sudo cp target/release/ghcl /usr/bin
  ```

## Arguments

```
ghcl 0.1.0
Lee Bousfield <email redacted here to prevent spam>
Automatically forks and clones a GitHub repository

USAGE:
    ghcl [FLAGS] [OPTIONS] <REPOSITORY> [CLONE_PATH]

FLAGS:
    -h, --help                 Prints help information
        --no-quiet             Don't be quiet (output status messages)
        --no-track-upstream    Don't setup master to track upstream
        --no-upstream          Don't setup an upstream remote (implies no_track_upstream)
    -q, --quiet                Don't output status messages
        --setup-upstream       Setup an upstream remote (default)
        --track-upstream       Setup master to track upstream (default, imples setup-upstream)
    -V, --version              Prints version information

OPTIONS:
    -c, --config <FILE>                       Sets a custom config file
    -s, --default-service <SERVICE>           The service to be used if the repository is in the form user/repo [values: github, GitHub, Github]
        --fork-timeout <TIMEOUT>              The maximum timeout for the fork creation (default: 30)
    -o, --organization <ORGANIZATION>         Fork into an organization
        --origin-protocol <GIT_PROTOCOL>      The git protocol to use for the origin (default: SSH) [values: ssh, https, SSH, HTTPS]
    -p, --password <PASSWORD>                 Your password (insecure - use a personal access token and put it in your config, or input your password when prompted)
        --remote-name <REMOTE_NAME>           The name of the upstream remote to create (default: "upstream")
        --upstream-protocol <GIT_PROTOCOL>    The git protocol to use for the upstream (default: HTTPS) [values: ssh, https, SSH, HTTPS]
    -u, --username <USERNAME>                 Your username

ARGS:
    <REPOSITORY>    Repository to fork and clone
    <CLONE_PATH>    Where to clone the repository (defaults to the name of the repo)
```

## Config

The config is located in:

- Linux: `~/.config/ghcl/config.yml` (or `$XDG_CONFIG_HOME/ghcl/config.yml`)
- MacOS: `~/Library/Application Support/ghcl/config.yml`
- Windows: `%APPDATA%\PlasmaPower\ghcl`

Contents (all of which are optional, and can be overriden by arguments):

| Key               | Type/valid values   | Description                                                                                    |
|-------------------|---------------------|------------------------------------------------------------------------------------------------|
| organization      | String              | the organization to clone repositories to                                                      |
| track_upstream    | bool                | should the master branch be setup to track upstream? (if true, setup_upstream cannot be false) |
| setup_upstream    | bool                | should the upstream remote be created?                                                         |
| remote_name       | String              | the name of the upstream remote to create (only used if setup_upstream is true)                |
| origin_protocol   | HTTPS or SSH        | the protocol to use for the origin remote                                                      |
| upstream_protocol | HTTPS or SSH        | the protocol to use for the upstream remote (only used if setup_upstream is true)              |
| default_service   | only github for now | the service to use if the repository is in the form of "user/repository"                       |
| quiet             | bool                | should status messages be outputed?                                                            |
| fork_timeout      | integer             | the maximum total timeout for attempting to clone after a fork                                 |
| authentication    | map - see below     | authentication (usually username + password) for each service                                  |

Authentication is a map of service (currently only "github") to a username and password.
With GitHub, you can (and it's recommended to) use a personal access token with the "repo" permission instead of an actual password.
That way, it has limited permissions, and can be easily revoked.

Example config:

```yaml
organization: myOrg
track_upstream: false
setup_upstream: true
remote_name: my-upstream
origin_protocol: SSH
upstream_protocol: HTTPS
default_service: github
quiet: true
fork_timeout: 30
authentication:
  github:
    username: ExampleUser
    password: efbfd4e43d8e77c1dc24... # personal access token
```

## This program doesn't fit my workflow!

This program is opinionated, and not intended to fit every workflow.
However, if it's just a small option that's missing, feel free to create an issue or pull request.
