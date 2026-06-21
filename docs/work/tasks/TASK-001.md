---
id: "TASK-001"
title: "Create task contract system"
status: "in_progress"
---

# Task: Create task contract system

## Objective
Implement a task contract parser/validator and establish a canonical task contract format for the bl1nk-doc-mcp project.

## Non-goals
- Do not implement tool execution handlers.
- Do not modify repository state beyond task contracts.

## Affected contracts
- `src/domain/task.rs`
- `docs/work/tasks/TASK-001.md`

## Invariants
- Task contracts are read-only and validated deterministically.
- Acceptance checks can be `pending`, `passed`, `failed`, or `skipped`.

## Acceptance checks

- [ ] `AC-001` TaskContract can be parsed from valid Markdown frontmatter — **required**
- [ ] `AC-002` Parser rejects missing `id` field — **required**
- [ ] `AC-003` Parser rejects missing `objective` field — **required**
- [ ] `AC-004` Acceptance checks can be parsed and validated — **required**
- [ ] `AC-005` Status field accepts only valid TaskStatus values — **required**

## Evidence
- `repo_status` output showing working tree clean.
- Unit tests in `src/domain/task.rs` covering parse, reject missing fields, and validate acceptance checks.
