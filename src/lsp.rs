use serde_json::Value;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::LanguageServer;

use crate::backend::Backend;

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::INCREMENTAL),
                        will_save: None,
                        will_save_wait_until: None,
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            include_text: Some(true),
                        })),
                    },
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![
                        "[".to_string(),
                        "#".to_string(),
                        ":".to_string(),
                        "@".to_string(),
                        "/".to_string(),
                    ]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    ..Default::default()
                }),
                // execute_command_provider: Some(ExecuteCommandOptions {
                //     commands: vec!["dummy.do_something".to_string()],
                //     work_done_progress_options: Default::default(),
                // }),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),

                    file_operations: None,
                }),
                ..ServerCapabilities::default()
            },
            // ..Default::default()
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

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file changed!")
            .await;
        let mut text = self
            .document_map
            .get_mut(&params.text_document.uri.to_string())
            .expect("Did change docs must be opened");
        params.content_changes.iter().for_each(|change| {
            if let Some(range) = change.range {
                let start =
                    text.line_to_char(range.start.line as usize) + range.start.character as usize;
                let end = text.line_to_char(range.end.line as usize) + range.end.character as usize;
                if start < end {
                    text.remove(start..end);
                }
                text.insert(start, &change.text);
                // eprintln!("{}", *text);
            }
        });
        // self.on_change(TextDocumentItem {
        //     uri: params.text_document.uri,
        //     text: text,
        //     version: params.text_document.version,
        //     language_id: "md".into(), //TODO: is this the way?
        // })
        // .await
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
        let character_pos = if position.character as usize >= line.len_chars() {
            line.len_chars() - 1
        } else {
            position.character as usize
        };
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
        let completions = match parts.0 {
            "#" => self.search_issue_and_pr(position, parts.1).await,
            "@" => self.search_user(position, parts.1).await,
            "[" => self.search_wiki(position, parts.1).await,
            "/" => self.search_repo(position, parts.1).await,
            ":" => self.search_owner(position, parts.1).await,
            _ => Ok(vec![]),
        }
        .ok();
        Ok(completions.map(CompletionResponse::Array))
    }
}
