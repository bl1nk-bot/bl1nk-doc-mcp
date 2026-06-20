#![allow(dead_code, unused_imports)]

mod config;
mod domain;
mod server;
mod tools;
mod resources;
mod prompts;
mod adapters;
mod telemetry;

use anyhow::Result;
use tracing::error;

use crate::config::ServerConfig;
use crate::server::Bl1nkDocMcpServer;

fn main() {
    if let Err(err) = do_main() {
        error!(?err, "bl1nk-doc-mcp failed");
        std::process::exit(1);
    }
}

fn do_main() -> Result<()> {
    crate::server::init_tracing();

    let config = ServerConfig::from_env_or_cwd()?;
    let server = Bl1nkDocMcpServer::new(config);
    server.run()
}
