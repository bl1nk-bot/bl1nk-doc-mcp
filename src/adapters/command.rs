use std::process::{Command, Output};

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct CommandAdapter {
    allowlist: Vec<String>,
}

impl CommandAdapter {
    pub fn new(allowlist: Vec<String>) -> Self {
        Self { allowlist }
    }

    pub fn run(&self, program: &str, args: &[&str]) -> Result<Output> {
        if !self.allowlist.contains(&program.to_string()) {
            anyhow::bail!("command not allowed: {}", program);
        }
        let output = Command::new(program).args(args).output().with_context(|| {
            format!("failed to execute command: {} {}", program, args.join(" "))
        })?;
        Ok(output)
    }
}
