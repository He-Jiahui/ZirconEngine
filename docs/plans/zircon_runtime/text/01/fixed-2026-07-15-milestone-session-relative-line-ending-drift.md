---
handoff_kind: fixed
status: fixed
created_at: 2026-07-15
resolved_at: 2026-07-15
summary_slug: milestone-session-relative-line-ending-drift
origin_plan: docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_runtime/text/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
related_code:
  - tools/session_coordinator/git_finalize.py
  - tools/session_coordinator/tests/test_git_finalize.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_git_finalize -v
  - python -m tools.session_coordinator --json milestone commit --session-id runtime-text01-fr-m2-closeout-20260714 --run-id b5fedc3825764dc79b3c785291a40910 --milestone M3 --summary "Complete Text01 FR-M3 composite font default package acceptance"
---


# Tooling01: Session-relative tracked-path comparison misclassifies clean LF files

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md`
- 来源执行切片：Text01 FR-M3 milestone commit, required before Frameworks05 M3 physical text-owner hard cut
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：最低共享根因位于协调器的 Session-relative Git dirty-path 判定，而非 Text01 25 文件提交清单或文本运行时代码。

## 失败现象与复现证据

协调器重启并加载 `finalize_path_unchanged` 修复后，重新提交 Text01 M3 action `653a436b742a4e91934ebd18920b99e3`，仍以 `finalize_owned_path_omitted` 失败。协调器额外认定 8 个已由 `06a67343` 提交的 M2 文件仍为当前 Session 脏改动，但对这些路径执行 `git status --short -- <paths>`、`git diff --numstat -- <paths>` 与 `git diff --cached --numstat -- <paths>` 均为空。

代表路径 `zircon_runtime/src/graphics/text/shaping/fallback_spans.rs` 的当前工作树为 2834 bytes、89 个 LF、0 个 CRLF；`git cat-file --filters --path=<path> HEAD:<path>` 在 `core.autocrlf=true` 下输出 2923 bytes、89 个 CRLF。`_require_owned_scope` 将当前原始字节 SHA-256 与 `_head_worktree_hash()` 的 smudged CRLF SHA-256 比较，因此把 Git 认为 clean 的 LF 文件错误加入 `owned_dirty`。

被误判的 8 个路径为：

- `docs/plans/zircon_runtime/text/01/2026-07-14-fr-m2-variable-font-product-acceptance.md`
- `docs/zircon_runtime/graphics/text/font-variation-instances.md`
- `zircon_runtime/src/graphics/scene/scene_renderer/ui/render/text_advances.rs`
- `zircon_runtime/src/graphics/scene/scene_renderer/ui/text/prepare_report.rs`
- `zircon_runtime/src/graphics/scene/scene_renderer/ui/text/resolved_batches.rs`
- `zircon_runtime/src/graphics/text/shaping/fallback_spans.rs`
- `zircon_runtime/tests/runtime_text_multilingual_product_framebuffer/proof_assertions.rs`
- `zircon_runtime/tests/runtime_text_multilingual_product_framebuffer/proof_commands.rs`

## 最低共享层根因

`tools/session_coordinator/git_finalize.py::_head_worktree_hash` 使用 `git cat-file --filters` 取得 HEAD 的 checkout/smudge 表示，再与 `hash_file()` 的当前原始工作树字节比较。该比较没有使用 Git 的 clean-filter 等价关系；当已提交文件保留 LF、而 `core.autocrlf=true` 使 HEAD 的 filtered 输出变成 CRLF 时，Git-clean 文件被判定为 Session-relative dirty。

## 架构修复验收

- 在 `test_git_finalize.py` 增加 Windows line-ending 回归：当前 LF 文件与 HEAD 在 Git clean-filter 语义下相同，不得进入 `owned_dirty`；真实内容变化仍必须进入。
- `python -m unittest tools.session_coordinator.tests.test_git_finalize -v` 全量通过。
- 重新加载协调器服务后，原样重试上述 Text01 M3 25 文件 milestone commit，必须成功且不得扩大 manifest。
- 提交后 `git show --name-only --format= HEAD` 必须精确等于既有 25 文件 manifest，随后 Frameworks05 M3 hard-cut gate 才可恢复。

## 禁止临时方案

- 不得把 8 个 Git-clean M2 文件重新塞入 M3 manifest。
- 不得放宽 `finalize_owned_path_omitted`、归属校验或 staged-scope 校验。
- 不得增加 line-ending 特判路径、兼容 shim、silent fallback、重复真相、test-only bypass 或调用点例外。

## 修复结果与回传

- 根因：Raw worktree and filtered HEAD hashes misclassified Git-clean LF files under core.autocrlf, and preexisting global-diff false positives were not filtered.
- 架构修复：Use git diff --quiet HEAD path identity for every attributed tracked path, removing Git-clean global-diff entries while restoring real Session-relative deltas.
- 验证：Focused autocrlf regression 1/1, git-finalize suite 27/27, workflow-commit suite 11/11, coordinator reloaded as ac51d3adb684498fa02137a6a4e701fb.
- 回传：Text01 may retry the unchanged 25-file FR-M3 milestone manifest; no compatibility shim or scope expansion was introduced.
