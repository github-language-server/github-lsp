use dashmap::DashMap;
use octocrab::Octocrab;
use ropey::Rope;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionTextEdit, MessageType, Range, TextDocumentItem, TextEdit,
};
use tower_lsp::{lsp_types::Position, Client};

use crate::gh::{GetDetail, GetEdit, GetLabel};

#[derive(Debug)]
pub struct Backend {
    pub(crate) client: Client,
    pub(crate) document_map: DashMap<String, Rope>,
    octocrab: Octocrab,
    owner: String,
    repo: String,
}

impl Backend {
    // async fn run_gh_command(args: Vec<&str>) -> Result<String> {
    //     let output = Command::new("gh")
    //         .args(args)
    //         .output()
    //         .await
    //         .unwrap()
    //         .stdout
    //         .to_owned();
    //     let output = String::from_utf8(output).unwrap();
    //     Ok(output)
    // }

    pub(crate) async fn search_issue_and_pr(
        &self,
        position: Position,
        needle: &str,
    ) -> Result<Vec<CompletionItem>> {
        self.client
            .log_message(
                MessageType::INFO,
                format!("search_issue_and_pr: {}", needle),
            )
            .await;
        let filter = format!("{} repo:{}/{}", needle, self.owner, self.repo);
        let page = self
            .octocrab
            .search()
            .issues_and_pull_requests(&filter)
            .sort("status")
            .per_page(10) //TODO: figure out a setting for this
            .send()
            .await
            .map_err(|_| {
                tower_lsp::jsonrpc::Error::new(tower_lsp::jsonrpc::ErrorCode::MethodNotFound)
            })?;
        let completion_items = page
            .items
            .iter()
            .map(|issue| CompletionItem {
                label: issue.get_label(),
                detail: Some(issue.get_detail()),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    range: Range {
                        start: Position {
                            line: position.line,
                            character: position.character - needle.len() as u32 - 1,
                        },
                        end: position,
                    },
                    new_text: issue.get_edit(),
                })),
                ..CompletionItem::default()
            })
            .collect::<Vec<CompletionItem>>();
        Ok(completion_items)
    }

    pub(crate) async fn search_user(&self, needle: &str) -> Result<Vec<CompletionItem>> {
        self.client
            .log_message(MessageType::INFO, format!("search_user: {}", needle))
            .await;
        Ok(vec![])
    }

    pub(crate) async fn search_wiki(&self, needle: &str) -> Result<Vec<CompletionItem>> {
        self.client
            .log_message(MessageType::INFO, format!("search_wiki: {}", needle))
            .await;
        Ok(vec![])
    }

    pub(crate) async fn search_repo(&self, needle: &str) -> Result<Vec<CompletionItem>> {
        self.client
            .log_message(MessageType::INFO, format!("search_repo: {}", needle))
            .await;
        Ok(vec![])
    }

    pub(crate) async fn search_owner(&self, needle: &str) -> Result<Vec<CompletionItem>> {
        self.client
            .log_message(MessageType::INFO, format!("search_owner: {}", needle))
            .await;
        Ok(vec![])
    }

    pub(crate) async fn on_change(&self, params: TextDocumentItem) {
        let rope = ropey::Rope::from_str(&params.text);
        self.document_map
            .insert(params.uri.to_string(), rope.clone());
    }

    pub fn new(
        client: Client,
        octocrab: Octocrab,
        owner: String,
        repo: String,
        document_map: DashMap<String, Rope>,
    ) -> Backend {
        Backend {
            client,
            octocrab,
            owner,
            repo,
            document_map,
        }
    }
}
