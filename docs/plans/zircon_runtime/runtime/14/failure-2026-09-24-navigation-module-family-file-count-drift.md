---
handoff_kind: failure
status: open
created_at: 2026-09-24
summary_slug: navigation-module-family-file-count-drift
origin_plan: docs/plans/optimize/zircon_tooling/13-repository-codex-skill-hook-structural-audit-governance-security-currentness-review.md
fixing_plan: docs/plans/zircon_runtime/runtime/14-runtime-module-family-closeout.md
origin_child_dir: docs/plans/optimize/zircon_tooling/13
fixing_child_dir: docs/plans/zircon_runtime/runtime/14
plan_link_mode: child_record_only
related_code:
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/module_family_boundary.py
  - tools/tests/test_runtime_module_family_boundary.py
  - docs/crates/zircon_runtime/navigation/runtime.md
  - zircon_runtime/src/navigation/runtime/world_scan.rs
  - zircon_runtime/src/navigation/runtime/world_scan/capacity_tests.rs
  - zircon_runtime/src/tests/runtime_absorption/root_entries/module_families/navigation.rs
  - zircon_runtime/src/tests/runtime_absorption/root_entries/module_families/mirror_docs.rs
  - docs/plans/zircon_runtime/runtime/14/2026-07-09-runtime-module-family-closeout-output-records.md
tests:
  - python -B -m unittest tools.tests.test_runtime_module_family_boundary.RuntimeModuleFamilyBoundaryTests.test_navigation_family_includes_folder_backed_operation_owners -v
  - python -B -m unittest discover -s tools/tests -p test_runtime_*.py
  - managed zircon_runtime --lib runtime_navigation_boundary_file_set_requires_doc_update
  - managed zircon_runtime --lib runtime_14_module_family_mirror_docs_match_structure_audit_counts
---

# Runtime14：导航新增测试文件与模块族清单计数漂移

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_tooling/13-repository-codex-skill-hook-structural-audit-governance-security-currentness-review.md`
- 来源执行切片：Tooling13 当前工作树的完整 Runtime 静态批次。
- 修复责任计划：`docs/plans/zircon_runtime/runtime/14-runtime-module-family-closeout.md`
- 交接原因：Runtime14 拥有 navigation 模块族文件集合、结构守卫和模块文档的同步合同；Tooling13 消费该结构审计结果，不应跳过新增文件或替其他 owner 改动导航实现。

## 失败现象与复现证据

2026-09-24 在当前工作树执行 `python -B -m unittest discover -s tools/tests -p 'test_runtime_*.py'`：实际运行 1429 项，只有 `test_navigation_family_includes_folder_backed_operation_owners` 失败（1428 通过、1 失败）。再次精确执行该单测，实际运行 1 项、失败 1 项：`tools/tests/test_runtime_module_family_boundary.py:27` 期望 16，`navigation["rust_file_count"]` 实际为 17。Tooling13 UI 架构聚焦测试独立运行 4/4 通过；该完整批次不能标为绿色。

审计 `module_family_boundary.py` 的 `_rust_file_count` 对 `zircon_runtime/src/navigation/**/*.rs` 执行 `rglob`；额外文件为 `zircon_runtime/src/navigation/runtime/world_scan/capacity_tests.rs`，由 `world_scan.rs` 的 `#[path = "world_scan/capacity_tests.rs"] mod capacity_tests;` 接入。当前新增文件未跟踪、未见协调器 attribution；[Runtime846 容量记录](../../../optimize/zircon_runtime/169/2026-09-20-navigation-projection-capacity.md) 的旧文件哈希与本次当前哈希也不同。此处不领用该 foreign 源码或假定其最终形态。

本次只读检查的 SHA-256：新增测试 `5bf43737941ff7512c8e0e751a5f6b1f046efe318fbcc039663d5d322607da84`；审计脚本 `a82cdd5fd7cfe8597dacaefaa47259e549b82ace39052b6a7402cac8342e426a`；Python 守卫 `0cda8ec3e98f931a5115649fabb18a2755181f334327369be1f233e20d664a96`；导航模块文档 `3503e98f8277eaead629408bd5b8982be29896c34323b410d48a0b9cdd52b05d`。原始 Runtime846 记录与当前哈希不同，不据此复用其结果。

## 最低共享层根因

Runtime846 当前树引入一个目录内 Rust 测试文件，但 Runtime14 的精确 16 文件结构基线、Python 守卫断言和当前导航模块文档没有对这次结构增长进行所有权审查与同步。此前 16 个文件的快照仍为历史证据，不能自动当作当前事实。

## 架构修复验收

- 先由 Runtime846 源码 owner 核对并归属新增文件的最终哈希、模块接入与测试意图；若该文件不应存在，按其 owner 的决定修复实际文件集合，不由本交接回滚。
- 在确认现行 owner 集合后，Runtime14 原子同步精确结构白名单、导航模块文档和 Python/Rust 镜像守卫；不得仅为通过测试机械抬高数字。
- 重新实际执行原始精确失败测试、完整 Runtime 静态批次，以及受管 Windows `--locked` 的 Runtime14 navigation/模块结构 Rust 验收；保留独立 C/I/M=0 审查与当前源码快照。

## 禁止临时方案

- 不过滤未跟踪 `.rs` 文件、移除结构守卫或跳过失败项，不把 1428 项通过误报成完整批次通过。
- 不改动、认领或删除 Runtime846 的 foreign 源码；不把直接 Python 诊断或尚未执行的受管 Cargo 票据当作验收通过。
- 不加兼容路径、别名或局部绕过；不更改历史版本的 16 文件快照。

## 修复结果与回传

Open state: `reproduced / Runtime846 foreign source attribution pending / Runtime14 inventory repair pending / managed upward acceptance pending`。此记录由 `failure-roll-01a084c8-runtime14-navigation-family-r1` 仅持有 handoff 文件；源码未编辑，不声明 fixed、return、closeout 或通知成功。


## 2026-09-26 rolling repair status

Session `failure-roll-01a0df1a-runtime14-r2` prepared the current 23/17/33/8 inventory mirror, exact navigation directory guard, documentation links, and numbered output record. The original reproduction `python -B -m unittest tools.tests.test_runtime_module_family_boundary -v` executed 1 test and passed. Focused `rustfmt +1.94.1 --check`, `git diff --check`, and the handoff structural validator passed locally. No managed Rust or upward acceptance ticket has passed; no return, closeout, commit, or WeCom success is claimed.

The broad `python -B -m unittest discover -s tools/tests -p test_runtime_*.py` executed 1429 tests and failed 2. Both failures are in the foreign, untracked `tools/tests/test_runtime_rg_a4_state_tracking_contract.py`: `test_prepare_scope_validates_texture_shape_before_projection_and_history` and `test_tracker_has_compact_scope_owners_and_a_work_receipt`. This is linked to `docs/plans/astra/performance/02-render-graph-state-tracking.md` and its `astra-rg-a4-followup-20260926` Session; the Runtime14 broad batch remains failed and is not counted as acceptance.

`zircon_runtime/src/navigation/runtime/world_scan/capacity_tests.rs` remains untracked and lacks source attribution. Its current SHA-256 `5bf43737941ff7512c8e0e751a5f6b1f046efe318fbcc039663d5d322607da84` differs from Runtime846's recorded `967017...45ee`; line-ending normalization does not explain the mismatch. Runtime846's record still says managed validation pending. Preserve that owner's provenance and performance gate before binding this file into Runtime14 acceptance.
