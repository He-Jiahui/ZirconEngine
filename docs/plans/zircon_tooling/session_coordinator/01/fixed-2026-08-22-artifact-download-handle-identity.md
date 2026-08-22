---
handoff_kind: fixed
status: fixed
failure_scope: local
created_at: 2026-08-19
summary_slug: artifact-download-handle-identity
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/control_plane/artifact_downloads.py
  - tools/session_coordinator/tests/test_artifact_downloads.py
resolved_at: 2026-08-22
---


# artifact-download-handle-identity: 验证失败回写

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：Control-plane workflow artifact download
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：同一编号计划拥有已集成快照及其前向修复。

## 失败现象与复现证据

- 验证回写：`Control-plane workflow artifact download` — Replace workflow artifact path after Path.stat/is_file validation but before Path.open; ArtifactDownloadService reads the replacement object because validation and read do not share a handle.

## 最低共享层根因

Artifact download validates path containment and size by pathname, then reopens that pathname without no-follow/reparse, final-handle-path, file identity, link-count, or durable byte-count checks.

## 架构修复验收

- Windows artifact download opens the file handle first with delete/write sharing denied and reparse-point traversal disabled.
- The opened handle must be a single-link regular non-reparse file whose final DOS path remains under the configured artifact root and whose size matches durable byte_count.
- Range and direct reads use the same verified handle; path-level stat/open replacement cannot change the downloaded object.
- Static escape, reparse, size mismatch, range caps, and normal partial downloads fail closed or retain current bounded behavior.

## 禁止临时方案

- 不回滚已集成快照来掩盖普通测试失败；应通过前向修复返回 `fixed-*` 记录。
- 不得添加别名、兼容垫片、静默回退、测试旁路或调用点特例。

## 修复结果与回传

- 根因：Artifact download validated a pathname and reopened it, allowing the object behind that path to change between metadata validation and read.
- 架构修复：Open one no-follow handle first, verify final path containment, regular non-reparse single-link identity and durable size on that same handle, then perform all range and body reads through it.
- 验证：Immutable managed ticket 2b377c11bd444b4287f3467b65d96fb4 passed the 6 artifact handle/range, 2 security matrix and 8 bounded-load tests; manifest f54be49127606f7f030cefd5891aafafe29d17ea41c6c55328ee45def53de9e1 still exactly matches HEAD commit 08094b9b9e17f6c80372e15c17b01204038b305b.
- 回传：Artifact downloads now bind validation and bytes to one verified handle, including empty files, ranges, replacement attempts, hardlinks and durable-size mismatch.
