# bl1nk-doc-mcp: Minimal Specification

## 1. Objective

สร้าง MCP server ด้วย Rust + pmcp เพื่อให้ AI โหลด “หลักฐานขั้นต่ำที่เชื่อถือได้” สำหรับงานหนึ่งงาน โดยไม่ต้องอ่าน repository ทั้งหมด ระบบต้องทำได้ 4 อย่าง:
1. สร้าง context bundle จาก task + git diff + generated knowledge
2. วิเคราะห์ผลกระทบของการเปลี่ยนแปลงต่อ knowledge artifacts
3. บันทึก change ledger แบบ append-only
4. ตรวจว่า AI เรียกใช้ tools ที่จำเป็น และผลลัพธ์ถูกต้องก่อนจบงาน

> MCP นี้ไม่ใช่ agent orchestration platform
> ไม่สร้าง vector database
> ไม่ index code ทั้ง repository
> ไม่ replace CI
> เป็น deterministic repository knowledge gateway เท่านั้น

pmcp รองรับ typed tools, prompt templates และ URI-addressable resources โดย derive schema จาก Rust types ได้ จึงเหมาะกับ server ขนาดเล็กที่ต้องการ schema ชัดและลด boilerplate.

## 2. Architecture

``````text
AI Agent
│
│ MCP
▼
knowledge-mcp
│
├── Git adapter
│   ├── git status
│   ├── git diff
│   └── git log
│
├── Repository adapters
│   ├── Cargo metadata
│   ├── test inventory
│   ├── OpenAPI artifact
│   ├── migration metadata
│   └── generated docs
│
├── Knowledge files
│   ├── AGENTS.md
│   ├── docs/work/CURRENT.md
│   ├── docs/work/CHANGELOG.ndjson
│   ├── docs/work/tasks/
│   ├── docs/generated/
│   └── docs/invariants/
│
└── Telemetry
    ├── tool invocation log
    ├── validation result log
    └── task completion metrics
``````

## 3. Repository Structure

``````
knowledge-mcp/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs
│   ├── server.rs
│   ├── config.rs
│   │
│   ├── domain/
│   │   ├── task.rs
│   │   ├── ledger.rs
│   │   ├── impact.rs
│   │   ├── snapshot.rs
│   │   ├── evidence.rs
│   │   └── metrics.rs
│   │
│   ├── tools/
│   │   ├── status.rs
│   │   ├── context_bundle.rs
│   │   ├── impact.rs
│   │   ├── ledger.rs
│   │   ├── validate.rs
│   │   └── metrics.rs
│   │
│   ├── resources/
│   │   ├── current_state.rs
│   │   ├── task.rs
│   │   ├── ledger.rs
│   │   ├── invariants.rs
│   │   └── generated.rs
│   │
│   ├── prompts/
│   │   ├── start_task.rs
│   │   ├── resume_task.rs
│   │   └── finalize_change.rs
│   │
│   ├── adapters/
│   │   ├── git.rs
│   │   ├── filesystem.rs
│   │   ├── cargo.rs
│   │   └── command.rs
│   │
│   └── telemetry/
│       ├── event_log.rs
│       └── evaluator.rs
│
├── tests/
│   ├── fixtures/
│   │   └── sample-repo/
│   ├── integration/
│   │   ├── context_bundle_test.rs
│   │   ├── impact_test.rs
│   │   ├── ledger_test.rs
│   │   └── evaluation_test.rs
│   └── contract/
│       └── tool_schema_test.rs
│
└── docs/
    ├── spec/
    │   └── knowledge-mcp.md
    └── examples/
        └── task-session.md
``````

## 4. Dependency Policy

``````toml
[dependencies]
pmcp = { version = "2", features = ["macros", "stdio", "logging"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros", "process", "fs"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
schemars = "1"
anyhow = "1"
thiserror = "2"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v7", "serde"] }
tracing = "0.1"
tracing-subscriber = "0.3"

[dev-dependencies]
tempfile = "3"
assert_cmd = "2"
insta = "1"
``````

Production ให้เปิดเฉพาะ PMCP features ที่ใช้จริง ไม่ใช้ full โดยไม่จำเป็น เพราะ framework แยก feature ตาม transport และ capability อยู่แล้ว.

## 5. Core Data Model

### 5.1 Task Contract

``````rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskContract {
    pub id: String,
    pub title: String,
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AcceptanceCheck {
    pub id: String,
    pub description: String,
    pub required: bool,
    pub status: CheckStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Planned,
    InProgress,
    Blocked,
    Verified,
    Completed,
}
``````

### 5.2 Change Ledger Event

``````rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChangeLedgerEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub commit: Option<String>,
    pub task_id: String,
    pub scope: Vec<String>,
    pub intent: String,
    #[serde(default)]
    pub changed_contracts: Vec<String>,
    #[serde(default)]
    pub invariants_added: Vec<String>,
    #[serde(default)]
    pub validations: Vec<ValidationResult>,
    pub status: ChangeStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ValidationResult {
    pub command: String,
    pub passed: bool,
    pub executed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ChangeStatus {
    Draft,
    Verified,
    Blocked,
}
``````

### 5.3 Evidence Model

ทุก tool ต้องคืน evidence source เสมอ

``````rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Evidence {
    pub source_type: EvidenceSourceType,
    pub path: String,
    pub revision: Option<String>,
    pub extracted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSourceType {
    Git,
    TaskContract,
    Ledger,
    GeneratedArtifact,
    Invariant,
    TestOutput,
    CargoMetadata,
}
``````

## 6. MCP Tools

### Tool Design Rule

Tools ต้องเป็น deterministic และ narrow

**ถูก:**
- `get_context_bundle`
- `analyze_change_impact`
- `append_change_ledger`
- `validate_task_completion`

**ไม่ควรมี:**
- `understand_repository`
- `update_all_docs`
- `fix_everything`
- `decide_architecture`

AI เป็นผู้ reasoning MCP เป็นผู้คืน evidence, enforce schema, และตรวจ completion

### 6.1 `repo_status`

ใช้เป็น startup tool ทุก session

``````rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RepoStatusInput {
    pub include_recent_commits: Option<u8>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct RepoStatusOutput {
    pub branch: String,
    pub head_commit: String,
    pub working_tree_clean: bool,
    pub changed_files: Vec<String>,
    pub recent_commits: Vec<CommitSummary>,
    pub last_verified_commit: Option<String>,
    pub evidence: Vec<Evidence>,
}
``````

#### Behavior

1. `git branch --show-current`
2. `git rev-parse HEAD`
3. `git status --porcelain`
4. `git log --max-count=N`
5. อ่าน ledger event ล่าสุดที่ `status=verified`

### 6.2 `get_context_bundle`

เป็น tool หลักของระบบ

``````rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ContextBundleInput {
    pub task_id: String,
    #[serde(default = "default_ledger_limit")]
    pub recent_ledger_limit: u8,
    #[serde(default)]
    pub include_diff: bool,
    #[serde(default)]
    pub include_dependency_neighborhood: bool,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ContextBundleOutput {
    pub task: TaskContract,
    pub repository_state: RepositoryState,
    pub current_snapshot: String,
    pub recent_events: Vec<ChangeLedgerEvent>,
    pub changed_files: Vec<String>,
    pub related_files: Vec<RelatedFile>,
    pub invariants: Vec<Invariant>,
    pub required_validations: Vec<String>,
    pub evidence: Vec<Evidence>,
}
``````

#### Context Loading Policy

- **L0:**
  - `AGENTS.md`
  - `CURRENT.md`
  - `repo_status`
- **L1:**
  - task contract
  - latest ledger events
- **L2:**
  - git diff
  - changed files
- **L3:**
  - direct imports
  - related tests
  - affected contracts
- **L4:**
  - subsystem scope
- **L5:**
  - full repository

`get_context_bundle` default ต้องหยุดที่ L3

หาก AI ต้องการ L4 หรือ L5 ต้องส่ง reason:

``````rust
pub enum ContextExpansionReason {
    UnresolvedDependency,
    CrossModuleContractChange,
    ArchitectureMigration,
    TestFailureOutsideTaskScope,
}
``````

### 6.3 `analyze_change_impact`

``````rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AnalyzeImpactInput {
    pub base_ref: String,
    pub head_ref: String,
    #[serde(default)]
    pub task_id: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct AnalyzeImpactOutput {
    pub change_id: String,
    pub changed_files: Vec<String>,
    pub classifications: Vec<ChangeClassification>,
    pub required_actions: Vec<RequiredAction>,
    pub blocking_actions: Vec<RequiredAction>,
    pub evidence: Vec<Evidence>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct RequiredAction {
    pub kind: RequiredActionKind,
    pub target: String,
    pub reason: String,
    pub required: bool,
    pub status: ActionStatus,
}

#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RequiredActionKind {
    RegenerateOpenApi,
    RegenerateSchemaDocs,
    RegenerateModuleMap,
    UpdateTaskContract,
    AppendLedgerEvent,
    ReviewSecurityInvariant,
    ReviewAdr,
    RunMigrationTest,
    RunAffectedTests,
}
``````

#### Classification Rules

- `migrations/**` changed -> `RegenerateSchemaDocs` -> `RunMigrationTest`
- `src/routes/**` changed -> `RegenerateOpenApi` -> `RunAffectedTests`
- `src/auth/**` changed -> `ReviewSecurityInvariant` -> `RunAffectedTests`
- `Cargo.toml` changed -> `RegenerateModuleMap` -> `RunAffectedTests`
- public Rust type changed -> `RegenerateOpenApi` หรือ contract artifact

เริ่มจาก path-based rules ก่อน ไม่ต้องทำ AST semantic analysis ใน phase แรก

### 6.4 `append_change_ledger`

``````rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AppendLedgerInput {
    pub task_id: String,
    pub intent: String,
    pub scope: Vec<String>,
    pub changed_contracts: Vec<String>,
    pub invariants_added: Vec<String>,
    pub validations: Vec<ValidationResult>,
    pub status: ChangeStatus,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct AppendLedgerOutput {
    pub event: ChangeLedgerEvent,
    pub file: String,
    pub appended: bool,
}
``````

#### Validation

- `task_id` ต้องมี task file อยู่จริง
- `scope` ต้องไม่ว่าง
- `intent` ต้องยาวอย่างน้อย 10 ตัวอักษร
- `status=verified` ต้องมี validation passed อย่างน้อย 1 รายการ
- ห้ามแก้ไข ledger event เก่า
- เขียนเพิ่มท้าย `docs/work/CHANGELOG.ndjson` เท่านั้น

### 6.5 `validate_task_completion`

``````rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ValidateTaskInput {
    pub task_id: String,
    pub base_ref: String,
    pub head_ref: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ValidateTaskOutput {
    pub task_id: String,
    pub passed: bool,
    pub acceptance_checks: Vec<CheckEvaluation>,
    pub impact_actions: Vec<RequiredAction>,
    pub missing_tool_calls: Vec<RequiredToolCall>,
    pub missing_evidence: Vec<String>,
    pub failures: Vec<ValidationFailure>,
    pub score: CompletionScore,
}
``````

#### Completion Gate

**PASS เมื่อ:**
1. task acceptance checks required ทุกข้อผ่าน
2. required impact actions ทุกข้อ resolved
3. มี ledger event ที่สัมพันธ์กับ task
4. AI เรียก `get_context_bundle` ก่อน mutation
5. AI เรียก `analyze_change_impact` เมื่อมี code diff
6. validation command ที่ task ระบุต้องผ่าน
7. generated artifact ไม่มี drift

### 6.6 `get_task_metrics`

``````rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct TaskMetricsInput {
    pub task_id: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct TaskMetricsOutput {
    pub task_id: String,
    pub context_acquisition_ms: u64,
    pub context_files_read: usize,
    pub full_repo_expansion_used: bool,
    pub tool_calls: Vec<ToolCallMetric>,
    pub required_tool_calls_satisfied: bool,
    pub validation_pass_rate: f64,
    pub evidence_coverage_rate: f64,
    pub completion_score: f64,
}
``````

## 7. MCP Resources

Resources เป็น read-only evidence surface

### 7.1 Resource URIs

- `knowledge://repo/current`
- `knowledge://repo/agents`
- `knowledge://task/{task_id}`
- `knowledge://task/{task_id}/context`
- `knowledge://ledger/latest?limit=20`
- `knowledge://ledger/task/{task_id}`
- `knowledge://invariants/{domain}`
- `knowledge://generated/openapi`
- `knowledge://generated/module-map`
- `knowledge://generated/test-inventory`
- `knowledge://impact/{change_id}`

### 7.2 Resource Rules

- `knowledge://repo/current` -> `docs/work/CURRENT.md`
- `knowledge://task/{task_id}` -> `docs/work/tasks/{task_id}.md`
- `knowledge://ledger/latest` -> tail `CHANGELOG.ndjson`
- `knowledge://generated/openapi` -> `docs/generated/openapi.json`
- `knowledge://generated/module-map` -> `docs/generated/module-map.json`

ไม่ expose arbitrary filesystem read ผ่าน MCP อนุญาตเฉพาะ allowlist paths

## 8. MCP Prompts

### 8.1 `start_task`

``````
You are starting task {{task_id}}. Mandatory sequence:
1. Call repo_status.
2. Call get_context_bundle with task_id={{task_id}}.
3. Read all returned invariants.
4. Do not inspect unrelated repository files.
5. Before modifying public behavior, call analyze_change_impact.
6. Before completion, call validate_task_completion.

Evidence hierarchy:
1. compiler, tests, migrations, schemas
2. generated artifacts
3. task contract and invariants
4. prose documentation

Do not claim completion without validation output.
``````

### 8.2 `resume_task`

``````
Resume task {{task_id}} after context loss. Mandatory sequence:
1. Call repo_status.
2. Read knowledge://repo/current.
3. Read knowledge://task/{{task_id}}.
4. Read knowledge://ledger/task/{{task_id}}.
5. Call get_context_bundle.
6. Continue only from unresolved acceptance checks.

Do not reconstruct repository state from memory.
Do not read the whole repository unless context expansion is required.
``````

### 8.3 `finalize_change`

``````
Finalize task {{task_id}}. Required sequence:
1. Call analyze_change_impact for {{base_ref}}..{{head_ref}}.
2. Resolve every required action.
3. Run required validations.
4. Call append_change_ledger.
5. Call validate_task_completion.
6. Report only facts supported by tool output.

Completion is invalid if validate_task_completion.passed is false.
``````

## 9. Tool Invocation Telemetry

### 9.1 Event Schema

``````rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolInvocationEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub session_id: String,
    pub task_id: Option<String>,
    pub tool_name: String,
    pub input_hash: String,
    pub success: bool,
    pub duration_ms: u64,
    pub result_hash: Option<String>,
    pub error_code: Option<String>,
}
``````

### 9.2 Storage

- `docs/work/.telemetry/tool-invocations.ndjson`
- `docs/work/.telemetry/validation-results.ndjson`

Telemetry ไม่ต้อง commit เข้า Git หากมีข้อมูล session-specific แต่ completion report ของ task ต้อง commit

- `docs/work/tasks/TASK-042.report.json`

## 10. Evaluation Model

### 10.1 Metrics

| Metric                       | Formula                               | Target      |
| :--------------------------- | :------------------------------------ | :---------- |
| Required Tool Call Rate      | required calls completed / required calls | 100%        |
| Evidence Coverage            | claims with evidence / total claims   | >= 95%      |
| Validation Pass Rate         | passed validations / required validations | 100%        |
| Context Acquisition Cost     | time before first code mutation       | ลดลงต่อเนื่อง |
| Full Repo Read Rate          | tasks requiring L5 / total tasks      | < 5%        |
| Resume Cost                  | time from resume to first valid action | < 2 min     |
| Knowledge Drift Escape Rate  | drift defects after merge / merged PRs | ใกล้ 0      |
| Completion Accuracy          | task passed validation / task declared complete | 100%        |

### 10.2 Tool Invocation Requirements

| Workflow          | Required Calls                               |
| :---------------- | :------------------------------------------- |
| Start task        | `repo_status`, `get_context_bundle`          |
| Resume task       | `repo_status`, `get_context_bundle`          |
| Code change       | `analyze_change_impact`                      |
| Completion        | `append_change_ledger`, `validate_task_completion` |
| Failure recovery  | `get_context_bundle`, `get_task_metrics`     |

### 10.3 Completion Score

``````rust
pub struct CompletionScore {
    pub required_tool_calls: f64, // 30%
    pub acceptance_checks: f64, // 30%
    pub impact_resolution: f64, // 20%
    pub validation_success: f64, // 15%
    pub evidence_coverage: f64, // 5%
    pub total: f64,
}

total >= 0.95 -> verified
0.80 - 0.94 -> incomplete
< 0.80 -> failed
``````

## 11. PMCP Server Skeleton

``````rust
use pmcp::prelude::*;

#[derive(Clone)]
pub struct KnowledgeServer {
    repo_root: std::path::PathBuf,
    telemetry: TelemetryStore,
}

#[mcp_server(
    name = "knowledge-mcp",
    version = "0.1.0"
)]
impl KnowledgeServer {
    #[mcp_tool(
        name = "repo_status",
        description = "Returns Git state, changed files, recent commits, and last verified change."
    )]
    async fn repo_status(
        &self,
        input: RepoStatusInput,
    ) -> Result<RepoStatusOutput, McpError> {
        tools::status::execute(&self.repo_root, input).await
    }

    #[mcp_tool(
        name = "get_context_bundle",
        description = "Returns the minimum verified evidence required to work on a task without reading the full repository."
    )]
    async fn get_context_bundle(
        &self,
        input: ContextBundleInput,
    ) -> Result<ContextBundleOutput, McpError> {
        tools::context_bundle::execute(&self.repo_root, input).await
    }

    #[mcp_tool(
        name = "analyze_change_impact",
        description = "Analyzes a Git diff and returns required knowledge artifacts, validations, and review actions."
    )]
    async fn analyze_change_impact(
        &self,
        input: AnalyzeImpactInput,
    ) -> Result<AnalyzeImpactOutput, McpError> {
        tools::impact::execute(&self.repo_root, input).await
    }

    #[mcp_tool(
        name = "append_change_ledger",
        description = "Appends an immutable structured change event to the repository change ledger."
    )]
    async fn append_change_ledger(
        &self,
        input: AppendLedgerInput,
    ) -> Result<AppendLedgerOutput, McpError> {
        tools::ledger::execute(&self.repo_root, input).await
    }

    #[mcp_tool(
        name = "validate_task_completion",
        description = "Checks task acceptance criteria, required tool usage, impact actions, ledger state, and validation evidence."
    )]
    async fn validate_task_completion(
        &self,
        input: ValidateTaskInput,
    ) -> Result<ValidateTaskOutput, McpError> {
        tools::validate::execute(&self.repo_root, &self.telemetry, input).await
    }
}
``````

PMCP มี `#[mcp_tool]`, `#[mcp_prompt]`, และ `#[mcp_server]` สำหรับสร้าง schema และ registration จาก Rust types โดยตรง.

## 12. Test Plan

### 12.1 Unit Tests

- `domain/task.rs`
  - parse valid task contract
  - reject missing objective
  - reject invalid acceptance check
- `domain/ledger.rs`
  - append only
  - reject mutation of historical event
  - verified event requires passing validation
- `domain/impact.rs`
  - migration path -> schema regeneration
  - route path -> OpenAPI regeneration
  - auth path -> security invariant review
  - `Cargo.toml` -> module map regeneration

### 12.2 Tool Contract Tests

``````rust
#[test]
fn get_context_bundle_schema_is_stable() {
    let schema = schemars::schema_for!(ContextBundleInput);
    insta::assert_json_snapshot!(
        "get_context_bundle_input_schema",
        schema
    );
}
``````

ตรวจ schema ทุก tool เพื่อป้องกัน breaking change ที่ AI client ใช้ไม่ได้

- `tests/contract/`
  - `repo_status.schema.json`
  - `get_context_bundle.schema.json`
  - `analyze_change_impact.schema.json`
  - `append_change_ledger.schema.json`
  - `validate_task_completion.schema.json`

### 12.3 Integration Tests

- **Fixture:** `sample-repo`
- **Scenario A: Start task**
  - create `TASK-001`
  - call `repo_status`
  - call `get_context_bundle`
  - verify only relevant files returned
  - verify no L5 expansion
- **Scenario B: Migration change**
  - modify `migrations/001.sql`
  - call `analyze_change_impact`
  - expect:
    - `RegenerateSchemaDocs`
    - `RunMigrationTest`
    - `AppendLedgerEvent`
- **Scenario C: Missing ledger**
  - modify route
  - run `validate_task_completion`
  - expect `passed=false`
  - expect `failure=MISSING_LEDGER_EVENT`
- **Scenario D: Incorrect AI workflow**
  - simulate code mutation before `get_context_bundle`
  - validate task
  - expect `missing_tool_calls` contains `get_context_bundle`
- **Scenario E: Resume**
  - append incomplete ledger event
  - create dirty working tree
  - call `get_context_bundle`
  - verify unresolved acceptance checks returned

### 12.4 End-to-End Test

1. Start fixture repository
2. Start `knowledge-mcp` over stdio
3. MCP client discovers tools/prompts/resources
4. Execute:
   - `repo_status`
   - `get_context_bundle`
   - `analyze_change_impact`
   - `append_change_ledger`
   - `validate_task_completion`
5. Assert:
   - tool schemas valid
   - ledger appended
   - telemetry recorded
   - completion score correct

## 13. CI Pipeline

``````yaml
name: knowledge-mcp
on:
  pull_request:
  push:
    branches: [main]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Format
        run: cargo fmt --check
      - name: Lint
        run: cargo clippy --all-targets -- -D warnings
      - name: Unit tests
        run: cargo test --lib
      - name: Integration tests
        run: cargo test --test '*'
      - name: Tool schema snapshot tests
        run: cargo test --test tool_schema_test
      - name: MCP protocol tests
        run: cargo test --test mcp_protocol_test
      - name: Verify no generated drift
        run: git diff --exit-code
      - name: Build release binary
        run: cargo build --release
``````

## 14. Implementation Order

### Phase 1 — Functional Minimum

1. `repo_status`
2. `get_context_bundle`
3. task resource
4. `CURRENT.md` resource
5. `start_task` prompt
6. stdio transport

### Phase 2 — Change Control

1. `analyze_change_impact`
2. `append_change_ledger`
3. impact rules from path classification
4. ledger validation

### Phase 3 — Enforcement

1. telemetry event log
2. `validate_task_completion`
3. completion score
4. CI contract tests

### Phase 4 — Optional Improvements

1. Cargo metadata dependency neighborhood
2. OpenAPI artifact validation
3. migration inspection
4. resource change notifications
5. HTTP transport

PMCP รองรับ stdio สำหรับ local agent และ streamable HTTP สำหรับ hosted deployment โดยใช้ server logic เดิมได้ จึงเริ่ม stdio ก่อนแล้วค่อยเพิ่ม HTTP ได้โดยไม่ต้องแยก implementation.

## 15. Non-Goals

- ไม่ทำ semantic code search ใน phase แรก
- ไม่ทำ vector embedding
- ไม่ทำ RAG pipeline
- ไม่ทำ autonomous code modification
- ไม่ให้ MCP เขียน source code
- ไม่ทำ multi-agent orchestration
- ไม่ทำ AST graph แบบเต็ม repository ตั้งแต่ต้น
- ไม่ให้ AI แก้ `CURRENT.md` โดยตรง

## 16. Definition of Done

- [ ] AI เริ่ม task ด้วย `get_context_bundle` ได้
- [ ] AI resume งานหลัง session ตายได้
- [ ] tool คืน evidence ทุกครั้ง
- [ ] diff ถูกแปลงเป็น required actions ได้
- [ ] ledger append-only และ validate ได้
- [ ] completion ถูก reject เมื่อไม่มี tool call ที่จำเป็น
- [ ] completion ถูก reject เมื่อ impact action ค้าง
- [ ] completion ถูก reject เมื่อ validation ไม่ผ่าน
- [ ] tool schema มี snapshot test
- [ ] integration test ครอบคลุม start, change, resume, finalize
- [ ] CI รัน fmt, clippy, tests, schema contract tests

## 17. Result

ระบบสุดท้ายควรบังคับ workflow นี้:

```
Task
↓
get_context_bundle
↓
AI modifies code
↓
analyze_change_impact
↓
run required validation
↓
append_change_ledger
↓
validate_task_completion
↓
verified / blocked
```

AI ไม่ต้องจำ repository AI ไม่ต้องเดาว่าเอกสารไหนต้องอัปเดต AI ไม่สามารถประกาศว่างานเสร็จได้ หากไม่มี evidence, tool invocation และ validation ครบ
