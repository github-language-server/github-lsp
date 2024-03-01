use serde_json::Value;
use tokio::process::Command;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use tracing::info;

#[derive(Debug)]
struct Backend {
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string()]),
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
            ..Default::default()
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

    async fn did_open(&self, _: DidOpenTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file opened!")
            .await;
    }

    async fn did_change(&self, _: DidChangeTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file changed!")
            .await;
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

    async fn completion(
        &self,
        _completion_parms: CompletionParams,
    ) -> Result<Option<CompletionResponse>> {
        /*
        Showing 4 of 4 open issues in entur/helm-charts\n
        #142  Automatically update examples as part of release                about 4 months ago
        #126  `ingress.class` annotation is deprecated           enhancement  about 5 months ago
        #101  Use internalPort to specify k8s' grpc probe ports               about 4 months ago
        #42   Add integration test for cron job                  enhancement  about 1 year ago
        */
        let output = Command::new("gh")
            .arg("issue")
            .arg("list")
            .output()
            .await
            .unwrap()
            .stdout
            .to_owned();
        let output = String::from_utf8(output).unwrap();

        self.client.log_message(MessageType::INFO, &output).await;
        let iss: Vec<&str> = output.split_terminator('\n').skip(2).collect();
        let compls: Vec<CompletionItem> = iss
            .into_iter()
            .map(|entry| CompletionItem::new_simple(entry.into(), entry.into()))
            .collect();
        /*
        Add integration test for cron job #42
        Open • AlexanderBrevig opened about 1 year ago • 0 comments
        Labels: enhancement\n\nNo description provided\n\n
        View this issue on GitHub: https://github.com/entur/helm-charts/issues/42
        */
        // Ok(Some(CompletionResponse::Array(vec![
        //     CompletionItem::new_simple("Hello".to_string(), "Some detail".to_string()),
        //     CompletionItem::new_simple("Bye".to_string(), "More detail".to_string()),
        // ])))
        Ok(Some(CompletionResponse::Array(compls)))
    }
}

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "info");
    // env_logger::init();
    info!("github-lsp starting");
    #[cfg(feature = "runtime-agnostic")]
    use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

    tracing_subscriber::fmt().init();

    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
    #[cfg(feature = "runtime-agnostic")]
    let (stdin, stdout) = (stdin.compat(), stdout.compat_write());

    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}
