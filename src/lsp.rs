use serde_json::Value;
use tokio::time::timeout;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::LanguageServer;

use crate::backend::Backend;
use crate::backend::TRIGGER_CHARACTERS;

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(TRIGGER_CHARACTERS.map(String::from).to_vec()),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    ..Default::default()
                }),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),

                    file_operations: None,
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.initialize().await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_change_workspace_folders(&self, _: DidChangeWorkspaceFoldersParams) {
        self.client
            .log_message(MessageType::INFO, "workspace folders changed!")
            .await;
    }

    async fn did_change_configuration(&self, _: DidChangeConfigurationParams) {
        self.client
            .log_message(MessageType::INFO, "configuration changed!")
            .await;
    }

    async fn did_change_watched_files(&self, _: DidChangeWatchedFilesParams) {
        self.client
            .log_message(MessageType::INFO, "watched files have changed!")
            .await;
    }

    async fn execute_command(&self, _: ExecuteCommandParams) -> Result<Option<Value>> {
        self.client
            .log_message(MessageType::INFO, "command executed!")
            .await;

        match self.client.apply_edit(WorkspaceEdit::default()).await {
            Ok(res) if res.applied => self.client.log_message(MessageType::INFO, "applied").await,
            Ok(_) => self.client.log_message(MessageType::INFO, "rejected").await,
            Err(err) => self.client.log_message(MessageType::ERROR, err).await,
        }

        Ok(None)
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file opened!")
            .await;
        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            text: params.text_document.text,
            version: params.text_document.version,
            language_id: "md".into(), //TODO: is this the way?
        })
        .await
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file changed!")
            .await;
        let text = if !params.content_changes[0].text.is_empty() {
            std::mem::take(&mut params.content_changes[0].text)
        } else {
            "\n".into()
        };

        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            text,
            version: params.text_document.version,
            language_id: "md".into(),
        })
        .await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file saved!")
            .await;
        if let Some(text) = params.text {
            self.on_change(TextDocumentItem {
                uri: params.text_document.uri,
                text,
                version: 0,               //TODO: not sure if we should forward version
                language_id: "md".into(), //TODO: is this the way?
            })
            .await
        }
    }

    async fn did_close(&self, _: DidCloseTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file closed!")
            .await;
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let rope = self
            .document_map
            .get(&uri.to_string())
            .ok_or(tower_lsp::jsonrpc::Error::invalid_request())?;

        let line = rope
            .get_line(position.line as usize)
            .ok_or(tower_lsp::jsonrpc::Error::internal_error())?;
        let character_pos = position.character as usize;
        let line = line.to_string();
        let line = line.split_at(character_pos).0;
        let word = line
            .chars()
            .rev()
            .take_while(|ch| !ch.is_whitespace())
            .collect::<String>()
            .chars()
            .rev()
            .collect::<String>();
        let parts = if word.len() <= 1 {
            (word.as_str(), "")
        } else {
            word.split_at(1)
        };
        let fast_ms = tokio::time::Duration::from_millis(200);
        let slow_ms = tokio::time::Duration::from_millis(3000);
        let completions = match parts.0 {
            "#" => timeout(fast_ms, self.search_issue_and_pr(position, parts.1)).await,
            "@" => timeout(fast_ms, self.search_user(position, parts.1)).await,
            "[" => timeout(fast_ms, self.search_wiki(position, parts.1)).await,
            "/" => timeout(fast_ms, self.search_repo(position, parts.1)).await,
            ":" => timeout(slow_ms, self.search_owner(position, parts.1)).await,
            _ => Ok(Ok(vec![])),
        };

        let completions = if let Ok(completions) = completions {
            completions.ok()
        } else {
            Some(vec![])
        };
        Ok(completions.map(CompletionResponse::Array))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let rope = self
            .document_map
            .get(&uri.to_string())
            .ok_or(tower_lsp::jsonrpc::Error::invalid_request())?;

        let line = rope
            .get_line(position.line as usize)
            .ok_or(tower_lsp::jsonrpc::Error::internal_error())?;
        let character_pos = position.character as usize;

        //TODO: cleanup parsing; possible to clean up with treesitter? need to investigate
        let line = line.to_string();
        let line = line.trim();
        // scan backwards
        let mut start = character_pos;
        let mut start_search = start;
        start = usize::MAX;
        loop {
            // look for the start of the (..) link part
            if line.as_bytes()[start_search] == b'(' {
                start = start_search + 1; // skip (
                break;
            }
            if start_search == 0 {
                break;
            }
            start_search -= 1;
        }
        // scan forwards
        let mut end = character_pos;
        let mut end_search = end;
        end = usize::MIN;
        loop {
            // handle hover over the [..] part of a link
            if start == usize::MAX && line.as_bytes()[end_search] == b'(' {
                start = end_search + 1;
            }
            if line.as_bytes()[end_search] == b')' {
                end = end_search; // str[..] slice will exclude ), non inclusive
                break;
            }
            if end_search == line.len() - 1 {
                break;
            }
            end_search += 1;
        }

        if start == usize::MAX || end == usize::MIN || start >= end {
            self.client
                .log_message(
                    MessageType::ERROR,
                    format!(
                        "Hover search failed with invalid start {} end {} for line {}",
                        start, end, line
                    ),
                )
                .await;
            return Ok(None);
        }
        let link: String = line[start..end].into();

        self.on_hover(link).await
    }
}
