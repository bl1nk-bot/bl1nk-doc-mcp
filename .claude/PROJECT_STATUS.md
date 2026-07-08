# bl1nk-doc-mcp — Project Status

**อัปเดต**: 2026-07-08 | **Milestone**: [v0.2.0](https://github.com/bl1nk-bot/bl1nk-doc-mcp/milestones) (due 2026-07-15)

## 📊 ภาพรวม

```
Phase A ██████████ เสร็จ    Task contract system
Phase B ██████████ เสร็จ    Context bundle + Impact analysis
Phase C █████░░░░░ ครึ่งทาง Ledger เสร็จ / Validation ยังไม่เริ่ม
Phase D ░░░░░░░░░░ ยังไม่เริ่ม Resources + Prompts
Phase E █████░░░░░ ครึ่งทาง CI เสร็จ (PR #18) / Snapshot tests ยังไม่เริ่ม
```

| | จำนวน |
|---|---|
| MCP tools ใช้งานได้จริง | **4 / 6** (`repo_status`, `get_context_bundle`, `analyze_change_impact`, `append_change_ledger`) |
| Issues เปิดอยู่ | **6** (P1 × 1, P2 × 4, P3 × 1) |
| PRs เปิดอยู่ | **2** ([#18](https://github.com/bl1nk-bot/bl1nk-doc-mcp/pull/18), [#19](https://github.com/bl1nk-bot/bl1nk-doc-mcp/pull/19)) |
| Tests | 41 ผ่านทั้งหมด (Linux + Windows) |

## 🔀 Open PRs

| PR | เรื่อง | สถานะ | ต้องทำอะไรต่อ |
|---|---|---|---|
| [#18](https://github.com/bl1nk-bot/bl1nk-doc-mcp/pull/18) | DevOps pipeline: CI (Lint + Test matrix ubuntu/windows + cache), auto-label, dependabot, Makefile, hooks, LICENSE | ✅ CI เขียว, reviewer แนะนำ merge, แก้ security findings ครบ | **Merge ได้เลย** — จะปิด [#16](https://github.com/bl1nk-bot/bl1nk-doc-mcp/issues/16) อัตโนมัติ |
| [#19](https://github.com/bl1nk-bot/bl1nk-doc-mcp/pull/19) | Issue templates (feature/bug/task) + PR template กำหนดฟอร์แมต conventional commits | รอ #18 merge ก่อนเพื่อให้ Rust CI รันบน PR นี้ | Merge หลัง #18 |

## 📋 Open Issues (เรียงตามลำดับที่ควรทำ)

| Issue | เรื่อง | Priority | ติดอะไรอยู่ / หมายเหตุ |
|---|---|---|---|
| [#11](https://github.com/bl1nk-bot/bl1nk-doc-mcp/issues/11) | Implement `validate_task_completion` tool | **P1** | 🎯 **งานถัดไป** — `src/tools/validate.rs` ยังเป็น stub; dependencies (#7, #9) เสร็จแล้ว |
| [#13](https://github.com/bl1nk-bot/bl1nk-doc-mcp/issues/13) | Add required MCP resources (9 ตัว) | P2 | `src/resources/*` ทั้ง 5 ไฟล์เป็น stub; dependencies (#7, #8, #10) เสร็จแล้ว |
| [#12](https://github.com/bl1nk-bot/bl1nk-doc-mcp/issues/12) | Implement `get_task_metrics` tool | P2 | ต้องรอ #11 (telemetry เกิดหลัง validate); `src/telemetry/*` เป็น stub |
| [#14](https://github.com/bl1nk-bot/bl1nk-doc-mcp/issues/14) | Add required MCP prompts (3 ตัว) | P2 | ต้องรอ #11; `src/prompts/*` เป็น stub |
| [#15](https://github.com/bl1nk-bot/bl1nk-doc-mcp/issues/15) | Integration + snapshot schema tests | P2 | ทำท้ายสุดหลัง tools ครบ; `tests/integration/`, `tests/contract/` ว่าง, `insta` ยังไม่ถูกใช้ + แทน placeholder tests ใน `tests/unit/` |
| [#16](https://github.com/bl1nk-bot/bl1nk-doc-mcp/issues/16) | Add CI workflow | P3 | ✅ เสร็จแล้วใน PR #18 — ปิดอัตโนมัติเมื่อ merge |

## ✅ เสร็จแล้ว (issue ปิดแล้ว + โค้ดอยู่บน main)

| Issue | เรื่อง | Commit |
|---|---|---|
| [#7](https://github.com/bl1nk-bot/bl1nk-doc-mcp/issues/7) | Task contract system + TASK-001 example | `fddd672` |
| [#8](https://github.com/bl1nk-bot/bl1nk-doc-mcp/issues/8) | `get_context_bundle` tool | `5315e89` |
| [#9](https://github.com/bl1nk-bot/bl1nk-doc-mcp/issues/9) | `analyze_change_impact` tool | `2bf282f` |
| [#10](https://github.com/bl1nk-bot/bl1nk-doc-mcp/issues/10) | `append_change_ledger` tool | `095a9e5` |
| [#1](https://github.com/bl1nk-bot/bl1nk-doc-mcp/issues/1)–[#6](https://github.com/bl1nk-bot/bl1nk-doc-mcp/issues/6) | Bootstrap: domain models, filesystem safety, Git adapter, `repo_status` | `96ad4e1`, `819df51` |

## 🎯 ลำดับงานต่อจากนี้

1. Merge [PR #18](https://github.com/bl1nk-bot/bl1nk-doc-mcp/pull/18) แล้วตามด้วย [PR #19](https://github.com/bl1nk-bot/bl1nk-doc-mcp/pull/19)
2. ทำ [#11](https://github.com/bl1nk-bot/bl1nk-doc-mcp/issues/11) `validate_task_completion` — P1 ตัวสุดท้าย ปลดล็อก #12 และ #14
3. ทำ [#13](https://github.com/bl1nk-bot/bl1nk-doc-mcp/issues/13) resources (ขนานกับ #11 ได้ ไม่ depend กัน)
4. ปิดท้ายด้วย [#12](https://github.com/bl1nk-bot/bl1nk-doc-mcp/issues/12) → [#14](https://github.com/bl1nk-bot/bl1nk-doc-mcp/issues/14) → [#15](https://github.com/bl1nk-bot/bl1nk-doc-mcp/issues/15)

> รายละเอียดสถาปัตยกรรมและ mapping ราย phase ดู [BLUEPRINT.md](../BLUEPRINT.md)
