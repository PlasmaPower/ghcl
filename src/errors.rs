use std::io;
use std::num;

use git2;
use app_dirs;
use serde_json;
use serde_yaml;
use regex;
use reqwest;

error_chain! {
    errors {
        ConfigTrackNoSetup {
            description("config specifies track: true but setup: false, which is not possible")
        }
        BranchNotNamed {
            description("default branch does not have a name (or was not valid UTF-8)")
        }
        RemoteNotNamed {
            description("upstream remote does not have a name (or was not valid UTF-8)")
        }
        FailedToParseRepository {
            description("failed to parse the repository name")
        }
        ForkTimedOut(wait: u64) {
            description("fork timed out (new forked repository not cloneable)")
            display("fork timed out (new forked repository not cloneable in {} seconds)", wait)
        }
        MissingKey(key: &'static str) {
            description("received JSON missing key")
            display("received JSON missing key: {}", key)
        }
        MalformedKey(key: &'static str) {
            description("received JSON with malformed key (expected string)")
            display("received JSON malformed key (expected string): {}", key)
        }
        APIError(message: String) {
            description("API error")
            display("API error: {}", message)
        }
        RawAPIError(json: serde_json::Value) {
            description("unparsable API error")
            display("unparsable API error: {:?}", json)
        }
    }
    foreign_links {
        AppDirs(app_dirs::AppDirsError);
        Io(io::Error);
        Yaml(serde_yaml::Error);
        Git(git2::Error);
        Regex(regex::Error);
        Reqwest(reqwest::Error);
        ParseInt(num::ParseIntError);
    }
}
