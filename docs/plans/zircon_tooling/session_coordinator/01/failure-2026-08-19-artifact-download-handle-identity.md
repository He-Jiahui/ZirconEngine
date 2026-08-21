---
handoff_kind: failure
status: open
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

- RED：测试在 pathname metadata 返回后用同尺寸 `evil` 文件替换 artifact；修复前 `ArtifactDownloadService` 随后的 `Path.open()` 返回 `evil`，断言 `safe` 失败。
- 修复：Windows 先以 `CreateFileW` 打开 read handle，拒绝 write/delete sharing，并设置 `FILE_FLAG_OPEN_REPARSE_POINT`；从同一 handle 验证非 directory/reparse、单链接、EOF size、final DOS path containment 和 durable `byte_count`，再转为 Python binary stream。非 Windows 使用 `os.open(O_NOFOLLOW)`、`fstat` 和 descriptor final path。
- 读取一致性：range 计算、seek 和 bounded read 全部发生在同一 verified handle 上；short read 也 fail closed 为不泄露路径的 `artifact_not_found`。
- GREEN r1：`python -m unittest tools.session_coordinator.tests.test_artifact_downloads -v` 为 5/5，覆盖 pathname replacement seam、真实 Windows hardlink、durable size mismatch、escape、206/416 range 与 opaque headers；ticket `6e43431c7cdd4805beefeb2a49367f5a` 也通过 5 + 2 + 8 项消费者回归。
- 自审 RED r2：同 handle short-read 检查暴露零字节普通下载回归。无 Range 时 `_range()` 将 size 0 归一为 end 0，计算 count 1，合法空 body 因 `len(body) != count` 被误报 `artifact_not_found`。
- 修复 r2：无 Range 的 end 保留为 `size - 1`；空 artifact 因此读取 count 0 并返回 200 空 body，而任何 byte range 仍按 RFC 边界返回 416 `bytes */0`。
- GREEN r2：`python -m unittest tools.session_coordinator.tests.test_artifact_downloads tools.session_coordinator.tests.test_control_security_matrix tools.session_coordinator.tests.test_control_load -v` 为 16/16，包含 6 个 handle/range 回归、2 个 security matrix 和 8 个 bounded-load 消费者测试。
- 消费者证据：同一批 `test_control_security_matrix` 为 2/2、`test_control_load` 为 8/8。`test_control_http` 的 5 个失败均为 current source 已知的 token-free runtime/旧 403 预期漂移（空 token 断言、wake 无认证、origin/UI 404、elevation 409），未出现 artifact handle/body 回归；不得把 P0 auth 迁移混入本三路径修复。
- 尚待受管收口：immutable validation ticket、candidate/commit/rollover 完成前保持 `status: open`。
