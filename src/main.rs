use backend::Backend;
use octocrab::Octocrab;
use tower_lsp::{LspService, Server};

use crate::gh::{gh_cli_owner_name, gh_token};

mod backend;
mod gh;
mod lsp;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let token = gh_token().await?;
    let octocrab = Octocrab::builder().personal_token(token.clone()).build()?;
    let owner_repo = gh_cli_owner_name().await?;

    tracing_subscriber::fmt().init();

    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());

    let (service, socket) =
        LspService::new(|client| Backend::new(client, octocrab, owner_repo.0, owner_repo.1));
    Server::new(stdin, stdout, socket).serve(service).await;
    Ok(())
}
