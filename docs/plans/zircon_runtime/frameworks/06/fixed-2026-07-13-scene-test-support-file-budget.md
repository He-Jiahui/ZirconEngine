---
handoff_kind: fixed
status: fixed
created_at: 2026-07-12
summary_slug: scene-test-support-file-budget
origin_plan: docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
fixing_plan: docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/06
fixing_child_dir: docs/plans/zircon_runtime/runtime/05
related_code:
  - zircon_runtime/src/scene/tests/support.rs
  - zircon_runtime/src/scene/tests/support/project_fixture.rs
  - zircon_runtime/src/scene/tests
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/global_budget.rs
tests:
  - D:/cargo-targets/zircon-engine/pool/cd364bddb8fbbb51cd141739f3d1894f703664304232c1f3d4452d3669ef2ea7/debug/deps/zircon_runtime-3cf8ac3fa0d196f8.exe tests::runtime_absorption::structure_convention::test_file_budget::global_budget::runtime_15_no_oversized_test_files --nocapture --test-threads=1
resolved_at: 2026-07-13
---


# Runtime 05：scene test support 超过 Runtime 15 文件预算

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md`
- 来源执行切片：Frameworks06 M1 优先 structure convention 全量复验
- 修复责任计划：`docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md`
- 交接原因：最低共享原因位于 Runtime05 所有的 scene test fixture/support owner，Frameworks06 只拥有全局结构门。

## 失败现象与复现证据

Frameworks06 使用 2026-07-12 23:33:46 生成、且包含最新 Material Subsurface child-owner 断言的 Runtime lib-test binary 复验完整结构门。结果为 `1303 passed / 1 failed`；唯一失败是：

```text
Runtime 15 test files should stay below 800 lines; oversized files: scene/tests/support.rs (805 lines)
```

失败测试为 `runtime_15_no_oversized_test_files`，完整结构门耗时 271.18s、exit 101。Material owner 2/2、global production budget 1/1 与 review findings 80/80 同一当前 binary 均为绿色，因此该失败是独立的 Scene test-support ownership 回归。

## 最低共享层根因

Runtime05 当前 fixture/importer 迁移在 `scene/tests/support.rs` 增加 project asset registry lookup、九组 project resource-handle helper、fixture root 解析与 first-wave plugin importer 注册，使该共享测试文件增至 805 行。Frameworks06 尝试取得该文件 lease 时与活动会话 `runtime05-scene-plugin-importer-fixture-20260712` 冲突，未改写其在途代码。

## 架构修复验收

- 将 project asset-reference / resource-handle helpers 抽到 `scene/tests/support/project_handles.rs`，或把 project fixture materialization 拆到等价的单一 child owner；父 `support.rs` 只保留共享 mount/re-export 与通用 fixture orchestration。
- 不提高 800 行预算、不加 allowlist、不删除全局预算测试，也不回退 Runtime05 的 UUID-backed asset registry 新契约。
- 父文件与所有新增 child 均 `< 800` 行，并通过 scoped rustfmt / diff check。
- 先执行精确 `runtime_15_no_oversized_test_files`，再执行完整 `tests::runtime_absorption::structure_convention::`；最终需要当前重编译 binary 的 `1304 passed / 0 failed`。

## 禁止临时方案

- 禁止提高 800 行预算、增加 allowlist、删除全局预算门或弱化完整 structure filter。
- 禁止恢复 UUID asset registry 之前的 fixture 路径、alias、shim、silent fallback 或测试专用旁路。

## 修复结果与回传

- 根因：Runtime05 project fixture and UUID-backed handle orchestration accumulated in scene/tests/support.rs, raising the shared test owner to 805 lines.
- 架构修复：Moved project fixture orchestration into folder-backed scene/tests/support/project_fixture.rs; parent is 701 lines and child is 58 lines, with the UUID-backed importer contract preserved and no budget increase or allowlist.
- 验证：Fresh current-source Runtime binary passed exact scene 1/1、scene products 3/3、animation 45/45、artifact physics 1/1、test budget 1/1、完整 structure 1304/1304（291.97s）和 review 298/298（109.92s）。
- 回传：Runtime05 test-support budget regression is repaired and Frameworks06 can resume its test-file budget gate.
