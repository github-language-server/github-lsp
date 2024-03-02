use dashmap::DashMap;
use octocrab::models::Repository;
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
    pub(crate) repository_map: DashMap<String, Repository>, //TODO: make our own light weight Repository?
    octocrab: Octocrab,
    owner: String,
    repo: String,
}

impl Backend {
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
        //TODO: struggling with getting org members
        //TODO: also merge in repo contributors
        // let org_members = octocrab::instance()
        //     .orgs("entur")
        //     .list_members()
        //     .send()
        //     .await?;
        // println!("{:?}", org_members);
        // for om in org_members {
        //     println!("{}", om.login);
        // }
        // let response: octocrab::Page<octocrab::models::Author> = octocrab
        //     .get("https://api.github.com/orgs/entur/members", None::<&()>)
        //     .await?;
        // println!("{:?}", response);
        Ok(vec![])
    }

    pub(crate) async fn search_wiki(&self, needle: &str) -> Result<Vec<CompletionItem>> {
        self.client
            .log_message(MessageType::INFO, format!("search_wiki: {}", needle))
            .await;
        Ok(vec![])
    }

    pub(crate) async fn search_repo(
        &self,
        position: Position,
        needle: &str,
    ) -> Result<Vec<CompletionItem>> {
        self.client
            .log_message(MessageType::INFO, format!("search_repo: {}", needle))
            .await;
        //TODO: should we enable searching for repos _all_ over github? Maybe?
        let completion_items = self
            .repository_map
            .iter()
            .filter(|repo| repo.name.starts_with(needle)) //TODO: smarter fuzzy match
            .map(|repo| CompletionItem {
                label: repo.get_label(),
                detail: Some(repo.get_detail()),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    range: Range {
                        start: Position {
                            line: position.line,
                            character: position.character - needle.len() as u32 - 1,
                        },
                        end: position,
                    },
                    new_text: repo.get_edit(),
                })),
                ..CompletionItem::default()
            })
            .collect::<Vec<CompletionItem>>();
        Ok(completion_items)
    }

    pub(crate) async fn search_owner(&self, needle: &str) -> Result<Vec<CompletionItem>> {
        self.client
            .log_message(MessageType::INFO, format!("search_owner: {}", needle))
            .await;
        let users = octocrab::instance()
            .search()
            .users(needle)
            // .sort("followers")
            // .order("desc")
            .send()
            .await
            .map_err(|_| {
                tower_lsp::jsonrpc::Error::new(tower_lsp::jsonrpc::ErrorCode::MethodNotFound)
            })?;
        for user in users {
            println!("{}", user.login);
        }
        Ok(vec![])
    }

    pub(crate) async fn on_change(&self, params: TextDocumentItem) {
        let rope = ropey::Rope::from_str(&params.text);
        self.document_map
            .insert(params.uri.to_string(), rope.clone());
    }
    pub(crate) async fn initialize(&self) {
        if let Ok(repos) = self
            .octocrab
            .current()
            .list_repos_for_authenticated_user()
            .affiliation("organization_member")
            .sort("updated")
            .per_page(100)
            .send()
            .await
        {
            for repo in repos {
                self.repository_map.insert(repo.name.to_owned(), repo);
            }
        }
    }

    pub fn new(
        client: Client,
        octocrab: Octocrab,
        owner: String,
        repo: String,
        document_map: DashMap<String, Rope>,
        repository_map: DashMap<String, Repository>,
    ) -> Backend {
        Backend {
            client,
            octocrab,
            owner,
            repo,
            document_map,
            repository_map,
        }
    }
}
