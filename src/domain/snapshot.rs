use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RepositorySnapshot {
    pub branch: String,
    pub head_commit: String,
    pub working_tree_clean: bool,
    pub changed_files: Vec<String>,
}
