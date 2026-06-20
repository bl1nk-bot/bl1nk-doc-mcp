# bl1nk-doc-mcp

MCP server ที่เขียนด้วย Rust และ `pmcp` สำหรับให้ AI agent อ่านเฉพาะ **“หลักฐานขั้นต่ำที่เชื่อถือได้”** ของ task หนึ่งงาน แทนการอ่าน repository ทั้งหมด ระบบนี้คือ **deterministic repository knowledge gateway**

bl1nk-doc-mcp:
- agent orchestration platform
- vector database
- RAG pipeline
- semantic repository index
- CI replacement
- autonomous code modification system

ระบบนี้ทำหน้าที่:
1. คืน repository state ที่ตรวจสอบได้
2. สร้าง task‑scoped context bundle
3. วิเคราะห์ diff เป็น knowledge actions ที่ต้องทำ
4. บันทึก change ledger แบบ append‑only
5. ตรวจ completion จาก tool usage, evidence, impact actions และ validation
6. วัดผลว่า agent ใช้ workflow ถูกต้องหรือไม่

---

## Architecture

```text
AI Agent
  │
  │ MCP over stdio
  ▼
bl1nk-doc-mcp
  │
  ├── Git adapter
  │     ├── git status
  │     ├── git diff
  │     ├── git log
  │     └── git rev-parse
  │
  ├── Knowledge adapters
  │     ├── task contract reader
  │     ├── change ledger reader/writer
  │     ├── generated artifact reader
  │     ├── invariant reader
  │     └── Cargo metadata reader
  │
  ├── MCP capabilities
  │     ├── tools
  │     ├── resources
  │     └── prompts
  │
  └── Telemetry
        ├── tool invocation log
        ├── validation result log
        └── completion metrics
```

---

## Core Workflow

```text
Task
  ↓
repo_status
  ↓
get_context_bundle
  ↓
AI modifies code
  ↓
analyze_change_impact
  ↓
run required validations
  ↓
append_change_ledger
  ↓
validate_task_completion
  ↓
verified / blocked
```

Agent ต้องไม่ประกาศว่างานเสร็จ หาก validate_task_completion.passed != true

---

## Repository Layout

```text
bl1nk-doc-mcp/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── AGENTS.md
├── src/
│   ├── main.rs
│   ├── server.rs
│   ├── config.rs
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── task.rs
│   │   ├── ledger.rs
│   │   ├── impact.rs
│   │   ├── snapshot.rs
│   │   ├── evidence.rs
│   │   ├── validation.rs
│   │   └── metrics.rs
│   ├── tools/
│   │   ├── mod.rs
│   │   ├── status.rs
│   │   ├── context_bundle.rs
│   │   ├── impact.rs
│   │   ├── ledger.rs
│   │   ├── validate.rs
│   │   └── metrics.rs
│   ├── resources/
│   │   ├── mod.rs
│   │   ├── current_state.rs
│   │   ├── task.rs
│   │   ├── ledger.rs
│   │   ├── invariants.rs
│   │   └── generated.rs
│   ├── prompts/
│   │   ├── mod.rs
│   │   ├── start_task.rs
│   │   ├── resume_task.rs
│   │   └── finalize_change.rs
│   ├── adapters/
│   │   ├── mod.rs
│   │   ├── command.rs
│   │   ├── filesystem.rs
│   │   ├── git.rs
│   │   └── cargo.rs
│   └── telemetry/
│       ├── mod.rs
│       ├── event_log.rs
│       └── evaluator.rs
├── docs/
│   ├── spec/
│   │   └── bl1nk-doc-mcp.md
│   ├── generated/
│   │   ├── module-map.json
│   │   ├── test-inventory.json
│   │   └── workspace-map.json
│   ├── invariants/
│   │   ├── repository.md
│   │   └── telemetry.md
│   └── work/
│       ├── CURRENT.md
│       ├── CHANGELOG.ndjson
│       ├── .telemetry/
│       │   ├── tool-invocations.ndjson
│       │   └── validation-results.ndjson
│       └── tasks/
│           ├── TASK-001.md
│           ├── TASK-002.md
│           └── ...
├── tests/
│   ├── fixtures/
│   │   └── sample-repo/
│   ├── unit/
│   │   ├── task_test.rs
│   │   ├── ledger_test.rs
│   │   ├── impact_test.rs
│   │   └── telemetry_test.rs
│   ├── integration/
│   │   ├── repo_status_test.rs
│   │   ├── context_bundle_test.rs
│   │   ├── impact_test.rs
│   │   ├── ledger_test.rs
│   │   └── validation_test.rs
│   └── contract/
│       └── tool_schema_test.rs
└── .github/
    └── workflows/
        └── ci.yml
```

---

## Development Commands

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build
cargo build --release
```

เมื่อ implementation พร้อม ให้เพิ่ม justfile

```bash
just fmt
just lint
just test
just check
just schema
just ci
```

โดย just ci ต้องเทียบเท่ากับ CI pipeline

---

## Required MCP Tools

| Tool                 | Purpose                                      |
| :------------------- | :------------------------------------------- |
| `repo_status`        | คืน Git state และ last verified ledger event |
| `get_context_bundle` | คืน evidence ขั้นต่ำสำหรับ task              |
| `analyze_change_impact` | แปลง Git diff เป็น required actions          |
| `append_change_ledger` | เพิ่ม immutable event ลง ledger              |
| `validate_task_completion` | ตรวจ completion gate                         |
| `get_task_metrics`   | คืน metrics ของ task และ tool usage          |

---

## Required MCP Resources

```text
bl1nk-doc://repo/current
bl1nk-doc://repo/agents
bl1nk-doc://task/{task_id}
bl1nk-doc://task/{task_id}/context
bl1nk-doc://ledger/latest?limit=20
bl1nk-doc://ledger/task/{task_id}
bl1nk-doc://invariants/{domain}
bl1nk-doc://generated/module-map
bl1nk-doc://generated/test-inventory
```

Resources ต้องเป็น read‑only และไม่อนุญาต arbitrary filesystem access

---

## Required MCP Prompts

```text
start_task(task_id)
resume_task(task_id)
finalize_change(task_id, base_ref, head_ref)
```

---

## Quality Rules

1. ทุก tool output ต้องมี evidence
2. ทุก tool input/output ต้อง derive JsonSchema
3. ห้าม expose filesystem path นอก allowlist
4. ห้ามใช้ shell command จาก user input โดยไม่มี validation
5. ห้ามให้ tool แก้ source code
6. `CHANGELOG.ndjson` เป็น append‑only
7. `status=verified` ต้องมี validation ที่ผ่านอย่างน้อยหนึ่งรายการ
8. generated artifact ต้องตรวจ drift ได้
9. schema ของ MCP tools ต้องมี snapshot test
10. completion ต้อง fail หาก required tool calls ไม่ครบ

---

## Definition of Done

```text
- [ ] Agent เริ่ม task ด้วย repo_status และ get_context_bundle ได้
- [ ] Agent resume task หลัง context loss ได้
- [ ] ทุก tool คืน evidence ได้
- [ ] Git diff ถูกแปลงเป็น required actions ได้
- [ ] Ledger append‑only และตรวจ validation ได้
- [ ] Completion fail เมื่อไม่มี required tool calls
- [ ] Completion fail เมื่อ impact action ยังไม่ resolved
- [ ] Completion fail เมื่อ validation ไม่ผ่าน
- [ ] Tool schema มี snapshot test
- [ ] มี integration test สำหรับ start, change, resume, finalize
- [ ] CI รัน fmt, clippy, unit test, integration test, schema test และ release build
```

