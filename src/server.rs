use std::path::PathBuf;

use anyhow::Result;
use pmcp::{Server, ServerCapabilities};
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

use crate::tools::status::RepoStatusTool;
use crate::tools::context_bundle::ContextBundleTool;
use crate::tools::impact::AnalyzeImpactTool;
use crate::tools::ledger::AppendLedgerTool;

pub struct Bl1nkDocMcpServer {
    repo_root: PathBuf,
}

impl Bl1nkDocMcpServer {
    pub fn new(config: crate::config::ServerConfig) -> Self {
        Self {
            repo_root: config.repo_root,
        }
    }

    pub async fn run(&self) -> Result<()> {
        tracing::info!(repo = ?self.repo_root, "starting bl1nk-doc-mcp server");

        let server = Server::builder()
            .name("bl1nk-doc-mcp")
            .version("0.1.0")
            .capabilities(ServerCapabilities::tools_only())
            .tool("repo_status", RepoStatusTool::new(self.repo_root.clone()))
            .tool("get_context_bundle", ContextBundleTool::new(self.repo_root.clone()))
            .tool(
                "analyze_change_impact",
                AnalyzeImpactTool::new(self.repo_root.clone()),
            )
            .tool(
                "append_change_ledger",
                AppendLedgerTool::new(self.repo_root.clone()),
            )
            .build()?;

        tracing::info!("pmcp stdio server initialized");
        server.run_stdio().await?;
        Ok(())
    }
}

pub fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());
    let fmt_layer = fmt::layer().with_target(false);
    let subscriber = tracing_subscriber::registry().with(filter).with(fmt_layer);
    subscriber.init();
}
