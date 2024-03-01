use std::fmt;

use octocrab::models::{issues::Issue, IssueState};
use tokio::process::Command;

#[derive(Debug)]
pub enum GitHubCLIError {
    NoToken,
    NoOwner,
}
impl std::error::Error for GitHubCLIError {}
impl fmt::Display for GitHubCLIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GitHubCLIError::NoOwner => write!(f, "No Owner found"),
            GitHubCLIError::NoToken => write!(f, "No Token found"),
        }
    }
}

pub async fn gh_token() -> Result<String, GitHubCLIError> {
    match std::env::var("GITHUB_TOKEN") {
        Ok(tok) => Ok(tok),
        Err(_) => gh_cli_token().await,
    }
}

async fn gh_cli_token() -> Result<String, GitHubCLIError> {
    let output = Command::new("gh")
        .arg("auth")
        .arg("token")
        .output()
        .await
        .map_err(|_| GitHubCLIError::NoToken)?
        .stdout
        .to_owned();
    let output = String::from_utf8(output)
        .map_err(|_| GitHubCLIError::NoToken)?
        .trim()
        .to_owned();
    Ok(output)
}

pub async fn gh_cli_owner_name() -> std::result::Result<(String, String), GitHubCLIError> {
    let output = Command::new("gh")
        .arg("repo")
        .arg("view")
        .arg("--json")
        .arg("name,owner")
        .arg("--jq")
        .arg(".owner.login,.name")
        .output()
        .await
        .unwrap()
        .stdout
        .to_owned();
    String::from_utf8(output)
        .expect("GH CLI must emit valid utf8")
        .trim()
        .split_once('\n')
        .map(|on| (on.0.to_string(), on.1.to_string()))
        .ok_or(GitHubCLIError::NoOwner)
}

pub(crate) trait GetLabel {
    fn get_label(&self) -> String;
}
pub(crate) trait GetDetail {
    fn get_detail(&self) -> String;
}
pub(crate) trait GetEdit {
    fn get_edit(&self) -> String;
}

impl GetLabel for IssueState {
    fn get_label(&self) -> String {
        if let IssueState::Open = self {
            "Open".into()
        } else {
            "Closed".into()
        }
    }
}

impl GetLabel for Issue {
    fn get_label(&self) -> String {
        format!("{} {} {}", self.number, self.state.get_label(), self.title)
    }
}
impl GetEdit for Issue {
    fn get_edit(&self) -> String {
        let id = self.number;
        format!("[#{id}](../../issues/{id})")
    }
}
impl GetDetail for Issue {
    fn get_detail(&self) -> String {
        let title = self.title.to_string();
        format!(
            "{} {} {}\n{}",
            self.number,
            self.state.get_label(),
            title,
            self.body.as_ref().unwrap_or(&title)
        )
    }
}
