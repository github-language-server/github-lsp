use dashmap::DashMap;
use octocrab::models::issues::Issue;
use octocrab::models::{Author, Repository};
use octocrab::params::State;
use octocrab::Octocrab;
use ropey::Rope;
use tower_lsp::jsonrpc::{self, Result};
use tower_lsp::lsp_types::{
    CompletionItem, CompletionTextEdit, Hover, HoverContents, MarkupContent, MarkupKind,
    MessageType, Range, TextDocumentItem, TextEdit,
};
use tower_lsp::{lsp_types::Position, Client};

use crate::gh::wiki::WikiArticle;
use crate::gh::{self, GetDetail, GetEdit, GetLabel};

pub const TRIGGER_CHARACTERS: [char; 5] = ['[', '#', ':', '@', '/'];

#[derive(Debug)]
pub struct Backend {
    pub(crate) client: Client,
    pub(crate) document_map: DashMap<String, Rope>,
    pub(crate) repository_map: DashMap<String, Repository>,
    pub(crate) issue_map: DashMap<String, Issue>,
    pub(crate) member_map: DashMap<String, Author>,
    pub(crate) wiki_map: DashMap<String, WikiArticle>,
    octocrab: Octocrab,
    owner: String,
    repo: String,
}

impl Backend {
    const PER_PAGE: u8 = 100;

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
            wiki_map: DashMap::new(),
        }
    }

    pub(crate) async fn initialize(&self) {
        self.initialize_issues().await;
        self.initialize_members().await;
        self.initialize_repos_as("owner").await;
        self.initialize_repos_as("organization_member").await;
        self.initialize_wiki().await;
    }

    pub async fn on_hover(&self, link: String) -> Result<Option<Hover>> {
        let mut text = String::new();
        //FIX: probably will cause issues for someone, maybe?
        if link.contains("github.com") {
            let link = link.replace("https://github.com/", "");
            let parts = link.split('/');
            let identifier = parts
                .last()
                .ok_or("No issue part in URL")
                .map_err(|_| jsonrpc::Error::method_not_found())?;
            if link.contains("issues") {
                let issue = self
                    .issue_map
                    .iter()
                    .filter(|issue| issue.get_label().starts_with(&format!("#{} ", identifier)))
                    .last()
                    .ok_or("No issue")
                    .map_err(|_| jsonrpc::Error::method_not_found())?;
                text = issue.get_detail().to_string();
            } else if link.contains("wiki") {
                text = format!("# Wiki article {}", identifier);
            } else if link.contains('/') {
                let repository = self
                    .repository_map
                    .iter()
                    .filter(|repo| repo.get_label() == link)
                    .last()
                    .ok_or("No repo")
                    .map_err(|_| jsonrpc::Error::method_not_found())?;
                text = repository.get_detail().to_string();
            } else {
                let users = octocrab::instance()
                    .search()
                    .users(identifier)
                    .per_page(1)
                    .page(0u32)
                    .send()
                    .await
                    .map_err(|_| {
                        tower_lsp::jsonrpc::Error::new(
                            tower_lsp::jsonrpc::ErrorCode::MethodNotFound,
                        )
                    })?;
                let user = &users.items[0];
                text = format!("# User {}", user.login.to_owned());
            }
        }
        let hover = Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: text,
            }),
            range: None,
        };
        Ok(Some(hover))
    }

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
        let completion_items = self
            .issue_map
            .iter()
            .filter(|issue| issue.get_detail().contains(needle)) //TODO: smarter fuzzy match
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
            .filter(|member| member.get_detail().contains(needle)) //TODO: smarter fuzzy match
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

    pub(crate) async fn search_wiki(
        &self,
        position: Position,
        needle: &str,
    ) -> Result<Vec<CompletionItem>> {
        self.client
            .log_message(MessageType::INFO, format!("search_wiki: {}", needle))
            .await;
        let completion_items = self
            .wiki_map
            .iter()
            .filter(|member| member.title.contains(needle)) //TODO: smarter fuzzy match
            .map(|member| CompletionItem {
                label: member.title.to_owned(),
                detail: None,
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
            .filter(|repo| repo.get_detail().contains(needle)) //TODO: smarter fuzzy match
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
        let needle = needle.replace(':', "");
        if needle.is_empty() {
            return Ok(vec![]);
        }
        let users = octocrab::instance()
            .search()
            .users(&needle)
            // .sort("followers")
            // .order("desc")
            .send()
            .await
            .map_err(|_| {
                tower_lsp::jsonrpc::Error::new(tower_lsp::jsonrpc::ErrorCode::MethodNotFound)
            })?;
        let completion_items = users
            .into_iter()
            .filter(|member| member.login.starts_with(&needle))
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

    async fn initialize_repos_as(&self, affiliation: &str) {
        self.client
            .show_message(
                MessageType::INFO,
                format!("initializing repos with affiliation `{}`", affiliation),
            )
            .await;
        let mut page: u8 = 0;
        let mut repos: Vec<Repository> = vec![];
        while let Ok(mut page_repos) = self
            .octocrab
            .current()
            .list_repos_for_authenticated_user()
            .affiliation(affiliation)
            .sort("updated")
            .per_page(Backend::PER_PAGE)
            .page(page)
            .send()
            .await
        {
            if page_repos.items.is_empty() {
                break;
            }
            repos.append(page_repos.items.as_mut());
            page += 1;
        }
        if repos.is_empty() {
            self.client
                .log_message(
                    MessageType::WARNING,
                    format!("No repos found with affiliation {}", affiliation),
                )
                .await;
            return;
        };
        repos.into_iter().for_each(|repo| {
            let _ = self.repository_map.insert(repo.name.to_owned(), repo);
        });
    }

    async fn initialize_issues(&self) {
        self.client
            .show_message(MessageType::LOG, "initializing issues")
            .await;
        let mut page: u8 = 0;
        let mut issues: Vec<Issue> = vec![];
        while let Ok(mut page_issues) = self
            .octocrab
            .issues(&self.owner, &self.repo)
            .list()
            .state(State::All)
            .per_page(Backend::PER_PAGE)
            .page(page)
            .send()
            .await
        {
            if page_issues.items.is_empty() {
                break;
            }
            issues.append(page_issues.items.as_mut());
            page += 1;
        }
        if issues.is_empty() {
            self.client
                .log_message(MessageType::WARNING, "No issues found")
                .await;
            return;
        };
        issues.into_iter().for_each(|issue| {
            self.issue_map.insert(issue.title.to_owned(), issue);
        });
    }

    async fn initialize_wiki(&self) {
        self.client
            .show_message(MessageType::INFO, "initializing wiki")
            .await;
        let wikis = gh::wiki::find_wiki_articles(&self.owner, &self.repo).await;
        match wikis {
            Ok(articles) => articles.into_iter().for_each(|article| {
                self.wiki_map.insert(article.title.to_owned(), article);
            }),
            Err(_) => {
                self.client
                    .log_message(MessageType::WARNING, "No wiki found")
                    .await;
            }
        }
        //TODO: load local .md files and make relative links?
    }

    async fn initialize_members(&self) {
        self.client
            .show_message(MessageType::INFO, "initializing members")
            .await;
        let mut page: u8 = 0;
        let mut members: Vec<Author> = vec![];
        while let Ok(mut page_members) = self
            .octocrab
            .orgs(self.owner.to_owned())
            .list_members()
            .per_page(Backend::PER_PAGE)
            .page(page)
            .send()
            .await
        {
            if page_members.items.is_empty() {
                break;
            }
            members.append(page_members.items.as_mut());
            page += 1;
        }
        if members.is_empty() {
            self.client
                .log_message(MessageType::WARNING, "No members found")
                .await;
            return;
        };
        members.into_iter().for_each(|member| {
            self.member_map.insert(member.login.to_owned(), member);
        });
    }
}
