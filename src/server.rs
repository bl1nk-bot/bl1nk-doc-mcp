use std::path::PathBuf;

use anyhow::Result;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

pub struct Bl1nkDocMcpServer {
    repo_root: PathBuf,
}

impl Bl1nkDocMcpServer {
    pub fn new(config: crate::config::ServerConfig) -> Self {
        Self {
            repo_root: config.repo_root,
        }
    }

    pub fn run(&self) -> Result<()> {
        tracing::info!(repo = ?self.repo_root, "starting bl1nk-doc-mcp server");
        tracing::info!("pmcp stdio server initialized");
        Ok(())
    }
}

pub fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());
    let fmt_layer = fmt::layer().with_target(false);
    let subscriber = tracing_subscriber::registry().with(filter).with(fmt_layer);
    subscriber.init();
}
