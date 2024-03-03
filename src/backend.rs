use dashmap::DashMap;
use octocrab::models::issues::Issue;
use octocrab::models::{Author, Repository};
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
    pub(crate) repository_map: DashMap<String, Repository>,
    pub(crate) issue_map: DashMap<String, Issue>,
    pub(crate) member_map: DashMap<String, Author>,
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
        //TODO: refresh issues
        // let filter = format!("{} repo:{}/{}", needle, self.owner, self.repo);
        // let page = self
        //     .octocrab
        //     .search()
        //     .issues_and_pull_requests(&filter)
        //     .sort("status")
        //     .per_page(100)
        //     .send()
        //     .await
        //     .map_err(|_| {
        //         tower_lsp::jsonrpc::Error::new(tower_lsp::jsonrpc::ErrorCode::MethodNotFound)
        //     })?;
        let completion_items = self
            .issue_map
            .iter()
            .filter(|issue| issue.title.starts_with(needle)) //TODO: smarter fuzzy match
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

    pub(crate) async fn search_user(
        &self,
        position: Position,
        needle: &str,
    ) -> Result<Vec<CompletionItem>> {
        self.client
            .log_message(MessageType::INFO, format!("search_user: {}", needle))
            .await;
        let completion_items = self
            .member_map
            .iter()
            .filter(|member| member.login.starts_with(needle)) //TODO: smarter fuzzy match
            .map(|member| CompletionItem {
                label: member.get_label(),
                detail: Some(member.get_detail()),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    range: Range {
                        start: Position {
                            line: position.line,
                            character: position.character - needle.len() as u32 - 1,
                        },
                        end: position,
                    },
                    new_text: member.get_edit(),
                })),
                ..CompletionItem::default()
            })
            .collect::<Vec<CompletionItem>>();
        Ok(completion_items)
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

    pub(crate) async fn search_owner(
        &self,
        position: Position,
        needle: &str,
    ) -> Result<Vec<CompletionItem>> {
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
        let completion_items = users
            .into_iter()
            .filter(|member| member.login.starts_with(needle)) //TODO: smarter fuzzy match
            .map(|member| CompletionItem {
                label: member.get_label(),
                detail: Some(member.get_detail()),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    range: Range {
                        start: Position {
                            line: position.line,
                            character: position.character - needle.len() as u32 - 1,
                        },
                        end: position,
                    },
                    new_text: member.get_edit(),
                })),
                ..CompletionItem::default()
            })
            .collect::<Vec<CompletionItem>>();
        Ok(completion_items)
    }

    pub(crate) async fn on_change(&self, params: TextDocumentItem) {
        let rope = ropey::Rope::from_str(&params.text);
        self.document_map
            .insert(params.uri.to_string(), rope.clone());
    }
    pub(crate) async fn initialize(&self) {
        self.initialize_repos().await;
        self.initialize_issues().await;
        self.initialize_members().await;
    }

    async fn initialize_repos(&self) {
        let Ok(repos) = self
            .octocrab
            .current()
            .list_repos_for_authenticated_user()
            .affiliation("organization_member")
            .sort("updated")
            .per_page(100)
            .send()
            .await
        else {
            return;
        };
        repos.into_iter().for_each(|repo| {
            self.repository_map.insert(repo.name.to_owned(), repo);
        });
    }

    async fn initialize_issues(&self) {
        let Ok(issues) = self
            .octocrab
            .issues(&self.owner, &self.repo)
            .list()
            .send()
            .await
        else {
            return;
        };
        issues.into_iter().for_each(|issue| {
            self.issue_map.insert(issue.title.to_owned(), issue);
        });
    }

    async fn initialize_members(&self) {
        let Ok(members) = self
            .octocrab
            .orgs("entur")
            .list_members()
            .page(0_u32)
            .send()
            .await
        else {
            return;
        };
        members.into_iter().for_each(|member| {
            self.member_map.insert(member.login.to_owned(), member);
        });
    }

    pub fn new(client: Client, octocrab: Octocrab, owner: String, repo: String) -> Backend {
        Backend {
            client,
            octocrab,
            owner,
            repo,
            document_map: DashMap::new(),
            repository_map: DashMap::new(),
            issue_map: DashMap::new(),
            member_map: DashMap::new(),
        }
    }
}
