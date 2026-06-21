# BLUEPRINT: bl1nk-doc-mcp

> สรningen ส gren จาก codebase จริง + open issues เวอร์ชัน 2026-06-22

---

## 1. สbv现状 (Current State)

### 1.1 ส 작업ที่แล้วเสร็จ
| Phase | Commit | 成果 |
|-------|--------|------|
| Phase 1 bootstrap | 96ad4e1 | domain models, filesystem safety layer |
| Phase 3 Git adapter | 819df51 | `GitGateway`, `ProductionGitAdapter`, `repo_status` tool + integration tests |
| Repo hygiene | 1a1fb80 | `.gitignore`, `.gitattributes`, `.editorconfig`, 清除 target tracking |
| Project tracking | 9252fd5 | GitHub issues #7-16, milestone v0.2.0, labels P1/P2/P3/testing, `PROJECT_STATUS.md` |

### 1.2 ไฟล์ที่มีอยู่แล้ว (แต่เป็น stub/半成品)

**มี domain models และ logic:**
- `src/domain/evidence.rs` — `Evidence`, `EvidenceSourceType`, `ValidationResult`, `TaskMetrics`
- `src/domain/impact.rs` — `ImpactAnalysis`, `ImpactSeverity` (model only, no logic)
- `src/domain/ledger.rs` — `ChangeLedgerEvent`, `ChangeStatus`, `is_valid()`
- `src/domain/metrics.rs` — `Metrics` (model only)
- `src/domain/task.rs` — `TaskContract`, `AcceptanceCheck`, `TaskStatus`, `CheckStatus`
- `src/domain/snapshot.rs` — stub
- `src/domain/validation.rs` — stub

**มี implementations จริง:**
- `src/adapters/filesystem.rs` — `SafeRepositoryFs` + unit tests (8 tests)
- `src/adapters/git.rs` — `GitGateway`, `ProductionGitAdapter`, `FakeGitAdapter` + unit tests (2 tests)
- `src/adapters/command.rs` — stub
- `src/adapters/mod.rs` — module wiring
- `src/tools/status.rs` — `repo_status_impl`, `RepoStatusTool`, helper functions + unit tests (6 tests)
- `src/lib.rs` — module exports
- `src/main.rs` — binary entry
- `src/server.rs` — MCP server wiring
- `src/config.rs` — config

**เป็น stub (ว่างเปล่า / 1 byte):**
- `src/tools/context_bundle.rs`
- `src/tools/impact.rs`
- `src/tools/ledger.rs`
- `src/tools/validate.rs`
- `src/tools/metrics.rs`
- `src/resources/current_state.rs`
- `src/resources/generated.rs`
- `src/resources/invariants.rs`
- `src/resources/ledger.rs`
- `src/resources/task.rs`
- `src/prompts/start_task.rs`
- `src/prompts/resume_task.rs`
- `src/prompts/finalize_change.rs`
- `src/domain/snapshot.rs`
- `src/domain/validation.rs`
- `src/adapters/cargo.rs`
- `src/adapters/command.rs`

**Tests ปัจจุบัน:**
- Integration: 4 tests (`tests/repo_status_test.rs`)
- Unit: 3 placeholder tests (`tests/unit.rs` + `tests/unit/*`)
- หลังเพิ่ม: 21 unit tests + 4 integration = 25 tests รวม (semua ผ่าน)

---

## 2. Issue → Blueprint Mapping

 milestone: **v0.2.0** (due 2026-07-15)

### P1 — Critical (Blocking ม Rights อื่น)

| # | Issue | Target Files | Dependencies |
|---|-------|--------------|--------------|
| 7 | Create task contract system (Task domain model + TASK-001 example) | `src/domain/task.rs`, `docs/work/tasks/TASK-001.md` | ต้อวaggerings ก、 logic (ไม่มี) |
| 8 | Implement `get_context_bundle` tool | `src/tools/context_bundle.rs`, `src/resources/task.rs`, `src/resources/ledger.rs` | #7 (ต้องรู้จัก task contract) |
| 9 | Implement `analyze_change_impact` tool | `src/tools/impact.rs`, `src/domain/impact.rs` | #8 (ต้องมี context bundle ก่อน) |
| 11 | Implement `validate_task_completion` tool | `src/tools/validate.rs`, `src/domain/validation.rs` | #7, #9 |

### P2 — High Priority

| # | Issue | Target Files | Dependencies |
|---|-------|--------------|--------------|
| 10 | Implement `append_change_ledger` tool | `src/tools/ledger.rs` | #7 (ต้องใช้ `ChangeLedgerEvent`) |
| 12 | Implement `get_task_metrics` tool | `src/tools/metrics.rs`, `src/domain/metrics.rs`, `src/telemetry/` | #11 (ต้องมี metrics หลัง validate) |
| 13 | Add required MCP resources | `src/resources/current_state.rs`, `src/resources/generated.rs`, `src/resources/invariants.rs`, `src/resources/ledger.rs` | #7, #8, #10 |
| 14 | Add required MCP prompts | `src/prompts/start_task.rs`, `src/prompts/resume_task.rs`, `src/prompts/finalize_change.rs` | #8, #9, #11 |
| 15 | Add integration tests and snapshot schema tests | `tests/integration/`, `tests/contract/` | ทughtools แล code |

### P3 — Medium

| # | Issue | Target Files | Dependencies |
|---|-------|--------------|--------------|
| 16 | Add CI workflow (fmt, clippy, test, release build) | `.github/workflows/ci.yml` | ทัว API |

---

## 3. Execution Phases

### Phase A: Task Contract Foundation (#7)
- ตรวจสอบ `src/domain/task.rs` มี model ครบ
- เพิ่ม parser/validator สำหรับ `TaskContract`
- สร้าง `docs/work/tasks/TASK-001.md` เป็นตัวอย่าง canonical
- เพิ่ม integration test สำหรับ parse/validate task contract

### Phase B: Context Bundle & Impact (#8, #9)
- Implement `get_context_bundle` — อ่าน task contract, ledger, generated artifacts
- Implement `analyze_change_impact` — รับ Git diff -> `ImpactAnalysis` -> required actions
- เช็คว่าทั้งหมด derive `JsonSchema`
- เพิ่ม integration tests สำหรับ start -> change flow

### Phase C: Ledger & Validation (#10, #11)
- Implement `append_change_ledger` — append-only write ไปที่ `docs/work/CHANGELOG.ndjson`
- Implement `validate_task_completion` — ตรวจ required tool calls, impact actions, validations
- เพิ่ม metric telemetry ตาม `src/domain/metrics.rs`

### Phase D: Resources & Prompts (#13, #14)
- Implement 9 MCP resources (read-only, no arbitrary FS access)
- Implement 3 MCP prompts (start_task, resume_task, finalize_change)
- ทดสอบ resources + prompts ผ่าน integration tests

### Phase E: Testing & CI (#15, #16)
- เพิ่ม integration tests สำหรับ resume + finalize workflows
- เพิ่ม snapshot tests สำหรับ MCP tool schemas (`insta`)
- สร้าง `.github/workflows/ci.yml`

---

## 4. Verification Gates

ก่อน merge ทุก phase ต้อง check:

1. `cargo fmt --check` ผ่าน
2. `cargo clippy --all-targets -- -D warnings` ผ่าน
3. `cargo test` รวม integration + unit ผ่าน
4. ทุก tool input/output derive `JsonSchema`
5. No arbitrary filesystem access (ใช้ `SafeRepositoryFs` เท่านั้น)
6. Coverage อยู่เหนือ 80% สำหรับ code ใหม่

---

## 5. Blueprint Principles

1. **Deterministic ก่อน สnv才** — ทัว output ต้องมี evidence, ไม่อนุญาต arbitrary access
2. **Minimal surface area** — MCP server ไม่ใช่ monolith, แต่แค่ gateway ผ่าน task-scoped bundles
3. **Append-only ledger** — `CHANGELOG.ndjson` ไม่สามารถแก้ไขกลับได้
4. **TDD ธรรมดาา** — แก้ไข tools ก่อน tests, ก、才เร็ว
5. **Separation of concerns** — adapters ตลก`, domain ikut logic, tools เป็น MCP handlers, resources/prompts เป็น Hermes-facing

---

## 6. Next Immediate Step

```text
Start: Phase A
  └─ Issue #7: Create task contract system
       ├─ ตรวจสอบ `src/domain/task.rs` model
       ├─ เพิ่ม parser/validator
       └─ สร้าง `docs/work/tasks/TASK-001.md`
```

ถ้าขร才刚刚 เริ่มจาก #7 ได้เลย
