use backend::Backend;
use dashmap::DashMap;
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
    #[cfg(feature = "runtime-agnostic")]
    use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

    tracing_subscriber::fmt().init();

    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
    #[cfg(feature = "runtime-agnostic")]
    let (stdin, stdout) = (stdin.compat(), stdout.compat_write());

    let (service, socket) = LspService::new(|client| {
        Backend::new(
            client,
            octocrab,
            owner_repo.0,
            owner_repo.1,
            DashMap::new(),
            DashMap::new(),
        )
    });
    Server::new(stdin, stdout, socket).serve(service).await;
    Ok(())
}
