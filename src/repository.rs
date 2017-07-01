use serde_json;
use regex::Regex;
use reqwest::{self, Client, StatusCode, Response};
use serde::de::DeserializeOwned;

use options::Authentication;
use errors::*;

#[derive(Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Service {
    GitHub,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Repository {
    pub service: Service,
    pub user: String,
    pub name: String,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum GitProtocol {
    SSH,
    HTTPS,
}

fn github_repo_json_git_url(json: serde_json::Value, git_protocol: GitProtocol) -> Result<String> {
    Ok(match git_protocol {
        GitProtocol::SSH => json.get("ssh_url").ok_or(ErrorKind::MissingKey("ssh_url"))?.as_str().ok_or(ErrorKind::MalformedKey("ssh_url"))?.into(),
        GitProtocol::HTTPS => json.get("clone_url").ok_or(ErrorKind::MissingKey("clone_url"))?.as_str().ok_or(ErrorKind::MalformedKey("clone_url"))?.into(),
    })
}

fn github_res<D: DeserializeOwned>(mut res: Response) -> Result<D> {
    match *res.status() {
        StatusCode::Ok | StatusCode::Created | StatusCode::Accepted | StatusCode::NoContent => {
            Ok(res.json()?)
        }
        _ => {
            let json: serde_json::Value = res.json()?;
            if let Some(message) = json.get("message").and_then(|v| v.as_str()) {
                return Err(ErrorKind::APIError(message.into()).into());
            }
            Err(ErrorKind::RawAPIError(json).into())
        }
    }
}

impl Repository {
    pub fn from_arg_string(string: &str, default_service: Service) -> Result<Repository> {
        if let Some(captures) = Regex::new(r#"(?:(?:https?:)?//)?(?:www\.)?github\.com/([a-zA-Z-2-9\-_]+)/([a-zA-Z0-9\-_]+)(?:/tree/[a-zA-Z0-9\-_]+)?(?:[?#].*)?"#)?.captures(string) {
            if let (Some(user), Some(name)) = (captures.get(1), captures.get(2)) {
                return Ok(Repository {
                    service: Service::GitHub,
                    user: user.as_str().to_string(),
                    name: name.as_str().to_string(),
                });
            }
        }
        let mut slash_found = false;
        let mut user = String::new();
        let mut name = String::new();
        for c in string.chars() {
            if c == '/' {
                if slash_found {
                    Err(ErrorKind::FailedToParseRepository)?
                }
                slash_found = true;
                continue;
            }
            if !c.is_alphanumeric() && c != '-' && c != '_' {
                Err(ErrorKind::FailedToParseRepository)?
            }
            if slash_found {
                name.push(c);
            } else {
                user.push(c);
            }
        }
        if user.is_empty() || name.is_empty() {
            Err(ErrorKind::FailedToParseRepository)?
        }
        Ok(Repository {
            service: default_service,
            user: user,
            name: name,
        })
    }

    pub fn get_git_url(&self, git_protocol: GitProtocol) -> Result<String> {
        match self.service {
            Service::GitHub => {
                let res = reqwest::get(&format!("https://api.github.com/repos/{}/{}", self.user, self.name))?;
                let json = github_res(res)?;
                Ok(github_repo_json_git_url(json, git_protocol).chain_err(|| "failed to get git URL from JSON")?)
            }
        }
    }

    pub fn fork(&self, authentication: Authentication, organization: Option<&str>, git_protocol: GitProtocol) -> Result<String> {
        let http_client = Client::new()?;
        match self.service {
            Service::GitHub => {
                let mut params_map = serde_json::Map::new();
                if let Some(org) = organization {
                    params_map.insert("organization".into(), serde_json::Value::String(org.into()));
                }
                let res = http_client.post(&format!("https://api.github.com/repos/{}/{}/forks", self.user, self.name))
                    .json(&serde_json::Value::Object(params_map))
                    .basic_auth(authentication.username, Some(authentication.password))
                    .send()?;
                let json = github_res(res)?;
                Ok(github_repo_json_git_url(json, git_protocol).chain_err(|| "failed to get git URL from JSON")?)
            }
        }
    }
}
