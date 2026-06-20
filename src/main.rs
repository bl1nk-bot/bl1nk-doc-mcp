#![allow(dead_code, unused_imports)]

mod adapters;
mod config;
mod domain;
mod prompts;
mod resources;
mod server;
mod telemetry;
mod tools;

use anyhow::Result;
use tracing::error;

use crate::config::ServerConfig;
use crate::server::Bl1nkDocMcpServer;

fn main() {
    if let Err(err) = tokio_main() {
        error!(?err, "bl1nk-doc-mcp failed");
        std::process::exit(1);
    }
}

fn tokio_main() -> Result<()> {
    crate::server::init_tracing();

    let config = ServerConfig::from_env_or_cwd()?;
    let server = Bl1nkDocMcpServer::new(config);

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(server.run())
}
