use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Metrics {
    pub task_id: String,
    pub tool_invocations: u64,
    pub validations_passed: u64,
    pub validations_failed: u64,
}
