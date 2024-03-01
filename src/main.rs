use dashmap::DashMap;
use ropey::Rope;
use serde_json::Value;
use tokio::process::Command;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
    document_map: DashMap<String, Rope>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                //TODO: this is probably much better for performance
                // text_document_sync: Some(TextDocumentSyncCapability::Kind(
                //     TextDocumentSyncKind::INCREMENTAL,
                // )),
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![
                        "[".to_string(),
                        "#".to_string(),
                        ":".to_string(),
                        "/".to_string(),
                    ]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    ..Default::default()
                }),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec!["dummy.do_something".to_string()],
                    work_done_progress_options: Default::default(),
                }),
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
        self.client
            .log_message(MessageType::INFO, "initialized!")
            .await;
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
        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            text: std::mem::take(&mut params.content_changes[0].text),
            version: params.text_document.version,
            language_id: "md".into(), //TODO: is this the way?
        })
        .await
    }

    async fn did_save(&self, _: DidSaveTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file saved!")
            .await;
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
            .expect("Edited doc in docmap");
        let line = rope
            .get_line(position.line as usize)
            .expect("The reported line should exist")
            .to_string();
        let line = line.split_at(position.character as usize).0;
        let word = line
            .chars()
            .rev()
            .take_while(|ch| !ch.is_whitespace())
            .collect::<String>()
            .chars()
            .rev()
            .collect::<String>();
        let parts = word.split_at(1);
        let completions = match parts.0 {
            "#" => self.search_issue_and_pr(parts.1).await,
            "@" => self.search_user(parts.1).await,
            "[" => self.search_wiki(parts.1).await,
            "/" => self.search_repo(parts.1).await,
            ":" => self.search_owner(parts.1).await,
            _ => Ok(vec![]),
        }
        .ok();
        Ok(completions.map(CompletionResponse::Array))
    }
}

impl Backend {
    async fn run_gh_command(args: Vec<&str>) -> Result<String> {
        let output = Command::new("gh")
            .args(args)
            .output()
            .await
            .unwrap()
            .stdout
            .to_owned();
        let output = String::from_utf8(output).unwrap();
        Ok(output)
    }

    async fn search_issue_and_pr(&self, needle: &str) -> Result<Vec<CompletionItem>> {
        self.client
            .log_message(
                MessageType::INFO,
                format!("search_issue_and_pr: {}", needle),
            )
            .await;
        // let mut completion_items: Vec<CompletionItem> = vec![];
        // let issues = Backend::run_gh_command(vec!["issue", "list"]).await?;
        // for issue in issues.lines().collect::<Vec<&str>>() {
        //     let parts = issue
        //         .split('\t')
        //         .map(str::to_owned)
        //         .collect::<Vec<String>>();
        //     let id = parts[0].clone();
        //     let label = parts
        //         .iter()
        //         .take(3)
        //         .map(String::to_owned)
        //         .collect::<Vec<String>>()
        //         .join(" ");
        //     let detail = Backend::run_gh_command(vec!["issue", "view", id.as_str()]).await?;
        //     completion_items.push(CompletionItem::new_simple(label, detail));
        // }
        // Ok(Some(CompletionResponse::Array(completion_items)))
        Ok(vec![])
    }

    async fn search_user(&self, needle: &str) -> Result<Vec<CompletionItem>> {
        self.client
            .log_message(MessageType::INFO, format!("search_user: {}", needle))
            .await;
        Ok(vec![])
    }

    async fn search_wiki(&self, needle: &str) -> Result<Vec<CompletionItem>> {
        self.client
            .log_message(MessageType::INFO, format!("search_wiki: {}", needle))
            .await;
        Ok(vec![])
    }
    async fn search_repo(&self, needle: &str) -> Result<Vec<CompletionItem>> {
        self.client
            .log_message(MessageType::INFO, format!("search_repo: {}", needle))
            .await;
        Ok(vec![])
    }
    async fn search_owner(&self, needle: &str) -> Result<Vec<CompletionItem>> {
        self.client
            .log_message(MessageType::INFO, format!("search_owner: {}", needle))
            .await;
        Ok(vec![])
    }

    async fn on_change(&self, params: TextDocumentItem) {
        let rope = ropey::Rope::from_str(&params.text);
        self.document_map
            .insert(params.uri.to_string(), rope.clone());
    }
}

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "info");
    #[cfg(feature = "runtime-agnostic")]
    use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

    tracing_subscriber::fmt().init();

    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
    #[cfg(feature = "runtime-agnostic")]
    let (stdin, stdout) = (stdin.compat(), stdout.compat_write());

    let (service, socket) = LspService::new(|client| Backend {
        client,
        document_map: DashMap::new(),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
