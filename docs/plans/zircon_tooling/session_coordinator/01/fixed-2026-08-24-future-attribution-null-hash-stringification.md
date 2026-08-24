---
handoff_kind: fixed
status: fixed
failure_scope: local
created_at: 2026-08-24
summary_slug: future-attribution-null-hash-stringification
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/ownership_transfers.py
  - tools/session_coordinator/tests/test_ownership_transfers.py
resolved_at: 2026-08-24
---

# future-attribution-null-hash-stringification: 验证失败回写

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：Production archived-clean relocation preview after a future path materialized
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：同一编号计划拥有已集成快照及其前向修复。

## 失败现象与复现证据

- 验证回写：`Production archived-clean relocation preview after a future path materialized` — ownership transfer preview bc01da73e348d36d2851c2c10f46545d9ce1d4816a882e5287fcfdfff350eaa7 followed by c2663c0d178a7515d81645fc32e6636f7d42a5915c114c8d06b134751fb5b859 projected sourceContentHash as the string None for a materialized future path whose attribution content_hash is SQL NULL

## 最低共享层根因

_preview_path converts attribution content_hash with str whenever an attribution row exists, so SQL NULL becomes the sentinel-looking string None instead of preserving the optional value

## 架构修复验收

- Preserve SQL NULL as Python None and JSON null throughout preview storage, reload, fingerprinting, and apply revalidation
- Keep non-null archived/current attribution hashes byte-exact and unchanged
- Prove a materialized future path can be transferred from a non-executable source without stringifying its prior null hash, then records the current hash on the new attribution

## 禁止临时方案

- 不回滚已集成快照来掩盖普通测试失败；应通过前向修复返回 `fixed-*` 记录。
- 不得添加别名、兼容垫片、静默回退、测试旁路或调用点特例。

## 修复结果与回传

- 根因：Ownership transfer preview stringified an attribution SQL NULL content_hash whenever the row existed, producing the sentinel-looking string None instead of an optional null.
- 架构修复：Convert attribution content_hash only when the column is non-NULL, preserving Python None and JSON null through preview persistence, fingerprint reload, and transactional apply while leaving non-null hashes byte-exact.
- 验证：Focused RED reproduced string None; ownership-transfer suite passed 13/13; commit 2a1299f8bf8e5a3012860ff07a6fcf528e4721d8 loaded via rollover 652b47eb1d204463837ce649b17c890b; production preview 91b141228baa4f6da5e3b3fd327d01be preserved sourceContentHash null for two archived SQL-NULL attributions without applying transfer.
- 回传：Coordinator transfer previews now preserve optional attribution identity correctly; consumers may distinguish an unknown prior content hash from the literal string None.
