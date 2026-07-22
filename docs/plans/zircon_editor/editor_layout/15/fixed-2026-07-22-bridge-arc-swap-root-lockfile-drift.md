---
handoff_kind: fixed
status: fixed
created_at: 2026-07-22
summary_slug: bridge-arc-swap-root-lockfile-drift
origin_plan: docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
fixing_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
origin_child_dir: docs/plans/zircon_editor/editor_layout/15
fixing_child_dir: docs/plans/zircon_plugins/01
plan_link_mode: child_record_only
related_code:
  - Cargo.toml
  - Cargo.lock
  - zircon_editor/Cargo.toml
  - zircon_runtime/Cargo.toml
tests:
  - cargo +1.94.1 test -p zircon_editor --lib --locked --jobs 1 --no-run --message-format short --color never
resolved_at: 2026-07-22
---


# Plugins01：bridge ArcSwap 根 lockfile 依赖边漂移

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md`
- 来源执行者：`editor-layout15-native-keyboard-return-r3-20260722`
- 来源执行切片：native-keyboard window contract failure upward gate，snapshot `680`
- 修复责任计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 交接原因：`zircon_runtime` 的 ArcSwap bridge 迁移由 Plugins01 持有，workspace manifest 与根 lockfile 必须作为同一依赖合同原子更新。

## 失败现象与复现证据

受管 Windows reservation `ab379d52a2c741d89b5846d0b998e63c`、job `c59f02eec88e4e528a0d79bc00e99561`、run `d306f6f5a7074c5ebb416f78782679c0` 执行原始 `zircon_editor --lib --locked --no-run` 向上门，在 Rust 编译前稳定 exit `101`：

```text
error: cannot update the lock file E:\Git\ZirconEngine\Cargo.lock because --locked was passed to prevent this
```

当前 workspace、`zircon_editor` 与 `zircon_runtime` manifest 均声明 `arc-swap`；`Cargo.lock` 已有 `arc-swap 1.9.2` package 和 `zircon_editor` dependency edge，却漏掉 `zircon_runtime` package dependency edge。

## 最低共享层根因

Plugins01 bridge 将 `arc-swap.workspace = true` 接入 `zircon_runtime/Cargo.toml` 后，根 lockfile 只完成 package/editor 一侧更新，未原子更新 runtime consumer dependency list。`--locked` 正确拒绝使用该不一致依赖图。

## 架构修复验收

- 根 `Cargo.lock` 的 `zircon_runtime` package dependency list 必须包含且只包含一个 `arc-swap` edge，并继续复用已有 `arc-swap 1.9.2` package 记录。
- 受管 Windows `zircon_editor --lib --locked --no-run` 必须越过 lockfile 解析并进入 Rust 编译；不得移除 `--locked`。
- 原始 Layout15 native-keyboard upward gate 必须在同一 current-source 上重跑，证明恢复的是共享依赖合同而非单一调用点。

## 禁止临时方案

- 不得删除 `--locked`、改用 `--offline` 或提交手工生成的第二份 lockfile。
- 不得回退 ArcSwap bridge、改写依赖版本或把 `arc-swap` 复制成 crate-local 非 workspace 版本。
- 不得把只越过 lockfile 的结果登记为 native-keyboard 功能验收。

## 修复结果与回传

- 根因：Plugins01 ArcSwap bridge manifest migration added the zircon_runtime workspace dependency without adding its consumer edge to the root lockfile zircon_runtime package dependency list.
- 架构修复：Added exactly one arc-swap edge to the existing zircon_runtime Cargo.lock package record and reused the existing arc-swap 1.9.2 package; no version or feature workaround was introduced.
- 验证：Static TOML checks found one zircon_runtime arc-swap dependency and one arc-swap 1.9.2 package. Managed Windows job 8e229f6cd2c749f495b0f701e0c07bc0 run b410b0de35d14f2d9980be50241c640e reran cargo +1.94.1 test -p zircon_editor --lib --locked --jobs 1 --no-run, compiled arc-swap 1.9.2 and entered zircon_runtime/zircon_editor compilation. It exited 101 only on existing Plugins01 E0631 import.rs:66 and E0308 registration_replay.rs:392; the lockfile error was absent. Coordinator auto-finished/released the job and observed the process tree exit.
- 回传：Root lockfile ArcSwap dependency graph is fixed and returned to Layout15. The original upward gate now reaches Rust workspace compilation; remaining blockers belong to the existing Plugins01 bridge-import and registration-replay failure owners, not this lockfile contract.
