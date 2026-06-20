use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ImpactSeverity {
    Minor,
    Major,
    Breaking,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ImpactAnalysis {
    pub task_id: String,
    pub base_ref: String,
    pub head_ref: String,
    pub changed_files: Vec<PathBuf>,
    pub required_actions: Vec<String>,
    pub blocking_actions: Vec<String>,
}
