use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Planned,
    InProgress,
    Blocked,
    Verified,
    Completed,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Pending,
    Passed,
    Failed,
    Skipped,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskContract {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub objective: String,
    #[serde(default)]
    pub non_goals: Vec<String>,
    #[serde(default)]
    pub affected_contracts: Vec<String>,
    #[serde(default)]
    pub invariants: Vec<String>,
    #[serde(default)]
    pub acceptance_checks: Vec<AcceptanceCheck>,
    pub status: TaskStatus,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AcceptanceCheck {
    pub id: String,
    pub description: String,
    pub required: bool,
    pub status: CheckStatus,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, thiserror::Error)]
#[error("task contract parse error: {missing_field} — {reason}")]
pub struct TaskContractParseError {
    pub missing_field: String,
    pub reason: String,
}

impl TaskContract {
    pub fn parse_from_json(input: &str) -> Result<Self, TaskContractParseError> {
        serde_json::from_str(input).map_err(|e| TaskContractParseError {
            missing_field: "json".to_string(),
            reason: format!("failed to parse task contract JSON: {e}"),
        })
    }

    pub fn validate(&self) -> Result<(), TaskContractParseError> {
        if self.id.trim().is_empty() {
            return Err(TaskContractParseError {
                missing_field: "id".to_string(),
                reason: "id must not be empty".to_string(),
            });
        }
        if self.objective.trim().is_empty() {
            return Err(TaskContractParseError {
                missing_field: "objective".to_string(),
                reason: "objective must not be empty".to_string(),
            });
        }
        for (idx, check) in self.acceptance_checks.iter().enumerate() {
            if check.id.trim().is_empty() {
                return Err(TaskContractParseError {
                    missing_field: format!("acceptance_checks[{idx}].id"),
                    reason: "acceptance check id must not be empty".to_string(),
                });
            }
            if check.description.trim().is_empty() {
                return Err(TaskContractParseError {
                    missing_field: format!("acceptance_checks[{idx}].description"),
                    reason: "acceptance check description must not be empty".to_string(),
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_task_contract() {
        let json = r#"{
            "id": "TASK-001",
            "title": "Test task",
            "objective": "Do something",
            "status": "planned",
            "acceptance_checks": [
                {"id": "AC-1", "description": "Pass", "required": true, "status": "pending"}
            ]
        }"#;
        let contract = TaskContract::parse_from_json(json).unwrap();
        assert_eq!(contract.id, "TASK-001");
        assert_eq!(contract.objective, "Do something");
        assert_eq!(contract.acceptance_checks.len(), 1);
        assert!(contract.validate().is_ok());
    }

    #[test]
    fn reject_missing_objective() {
        let json = r#"{
            "id": "TASK-001",
            "title": "Test task",
            "status": "planned"
        }"#;
        let contract = TaskContract::parse_from_json(json).unwrap();
        let err = contract.validate().unwrap_err();
        assert_eq!(err.missing_field, "objective");
    }

    #[test]
    fn reject_empty_id() {
        let json = r#"{
            "id": "   ",
            "title": "Test task",
            "objective": "Do something",
            "status": "planned"
        }"#;
        let contract = TaskContract::parse_from_json(json).unwrap();
        let err = contract.validate().unwrap_err();
        assert_eq!(err.missing_field, "id");
    }

    #[test]
    fn reject_invalid_acceptance_check() {
        let json = r#"{
            "id": "TASK-001",
            "title": "Test task",
            "objective": "Do something",
            "status": "planned",
            "acceptance_checks": [
                {"id": "", "description": "", "required": true, "status": "pending"}
            ]
        }"#;
        let contract = TaskContract::parse_from_json(json).unwrap();
        let err = contract.validate().unwrap_err();
        assert!(err.missing_field.starts_with("acceptance_checks[0]"));
    }

    #[test]
    fn reject_invalid_json() {
        let result = TaskContract::parse_from_json("not json");
        assert!(result.is_err());
    }
}
