---
handoff_kind: failure
status: open
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

Production preview `c2663c0d178a7515d81645fc32e6636f7d42a5915c114c8d06b134751fb5b859`
confirmed the typed drift: the newly materialized fixed path retained SQL NULL in
its original future attribution, but the API emitted `sourceContentHash: "None"`.

The focused RED constructs the same lifecycle: reserve a missing path, materialize
it, make the source non-executable, and preview transfer to a successor. Before the
repair, `assertIsNone(candidate.source_content_hash)` fails with `'None' is not
None`. The repair converts the column only when its value is non-NULL, preserving
the optional value through JSON serialization, stored preview reload, fingerprint,
and apply revalidation. Apply then writes the actual current hash to the successor
attribution.

The focused regression passes `1/1`; the complete ownership-transfer suite passes
`13/13`, including future-path CAS, archived clean transfer, stale attribution,
unowned clean rejection, foreign leases, and real child-record-only failure return.
Python compilation and `git diff --check` pass.

Open state: `implementation and regression suite accepted / maintenance commit,
source rollover, and production null projection verification pending`.
