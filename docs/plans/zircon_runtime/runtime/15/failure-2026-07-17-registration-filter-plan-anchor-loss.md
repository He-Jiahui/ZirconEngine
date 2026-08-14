---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: registration-filter-plan-anchor-loss
origin_plan: docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
fixing_plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/02
fixing_child_dir: docs/plans/zircon_runtime/runtime/15
plan_link_mode: child_record_only
related_code:
  - docs/plans/zircon_runtime/runtime/15/2026-07-17-registration-filter-plan-anchor-current-owner.md
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/provider_boilerplate/registration.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/core_runtime_registration.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/support.rs
tests:
  - cargo test -p zircon_runtime --lib --locked --jobs 1 --color never tests::runtime_absorption::structure_convention::provider_boilerplate::registration::runtime_15_provider_registration_uses_shared_owner -- --exact --test-threads=1
  - cargo test -p zircon_runtime --lib --locked --jobs 1 --color never tests::runtime_absorption::structure_convention::test_file_budget::core_runtime_registration::runtime_15_core_runtime_registration_structure_tests_are_folder_backed -- --exact --test-threads=1
  - cargo test -p zircon_runtime --lib --locked --jobs 1 --color never registration -- --test-threads=1
---

# Runtime15：registration filter plan/status anchors were lost

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md`
- 来源执行切片：M3 RuntimePlugin lifecycle `registration` focused gate
- 修复责任计划：`docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md`
- 交接原因：两项失败都验证 Runtime15 structure/status evidence routing；Frameworks02 不应删减守卫、恢复 archive aggregate 或复制 Runtime15 历史正文来通过上行门。

## 失败现象与复现证据

Frameworks02 Windows managed job `cbbe13aff0db495181c4ec16e984c51f` / run `a4e893e4be0c44859e38fc19b0697986` 执行 `registration` filter，结果 211 passed / 5 failed / 8025 filtered，exit 101。其中两项 Runtime15 structure guards 都在第一个 `Runtime 15 plan` aggregate source 上短路：provider-registration 缺完整 3-anchor tuple，core-runtime-registration 缺完整 6-anchor tuple。

## 最低共享层根因与硬切

两项生产/模块/status truth 均仍存在；断裂的是 current plan aggregation。修复必须新增单一 current child record，并让两个守卫 exact 读取该 owner；不得把完整 tuple 复制回 parent/index/priority plans，也不得继续由 `assert_contains_all` 的 archive fallback 隐式满足。

## 架构修复验收

- 两个 exact tests 通过，并真实检查唯一 current child tuple 与仍存活的 module/provider/lifecycle 公共契约。
- 旧 Rust `plan_status` status/date rows 已由 2026-08-02 receipt-tree hard cut 物理删除，计划 lifecycle 由 Coordinator/Python tooling 持有；不得为本 failure 恢复退役状态路径。
- Frameworks02 `registration` 重跑时这两项消失。

## 禁止临时方案

- 禁止删减 required anchors、排除 filter、恢复 archive aggregate、兼容 alias/shim 或重复父计划正文。

## 修复结果与回传

Open state: `resolving_failure`; no Cargo pass is claimed.

- 2026-08-14 current-source 复核确认 `2026-07-17-registration-filter-plan-anchor-current-owner.md` 是 provider-registration 与 core-runtime-registration 两组完整 tuple 的唯一 current evidence owner；父计划、runtime index 与 priority plans 没有恢复重复正文。
- 两个 structure guard exact 读取该 child record，并继续验证真实 provider/module/lifecycle owner；core guard 对 module convention 验证切片、状态与 folder `mod.rs` 三锚，对 lifecycle doc 验证同组三个锚、两个 focused child owner 与 exact guard 名共 6 锚。共享 `assert_contains_all` 只检查显式 source，没有 label-based archive fallback。required child tuples 分别保持 provider 6/6、core-runtime 7/7。
- current child record、两条 guard 与共享 helper 相对 HEAD 零差异；fresh immutable exact review、managed current-source `registration` gate 与 failure return 尚未完成，因此不声明 fixed/accepted。
