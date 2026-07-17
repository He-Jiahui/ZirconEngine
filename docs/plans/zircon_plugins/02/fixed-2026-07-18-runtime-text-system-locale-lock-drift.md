---
handoff_kind: fixed
status: fixed
created_at: 2026-07-17
summary_slug: runtime-text-system-locale-lock-drift
origin_plan: docs/plans/zircon_plugins/02-sound.md
origin_workflow_node: M1
fixing_plan: docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
origin_child_dir: docs/plans/zircon_plugins/02
fixing_child_dir: docs/plans/zircon_runtime/text/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/Cargo.toml
  - zircon_runtime/src/text/language.rs
  - zircon_runtime/src/text/parallel/raster_pool.rs
  - Cargo.lock
  - zircon_plugins/Cargo.lock
tests:
  - cargo +1.94.1 metadata --locked --format-version 1 --no-deps
  - cargo +1.94.1 metadata --manifest-path zircon_plugins/Cargo.toml --locked --format-version 1 --no-deps
  - cargo +1.94.1 test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sound_runtime --locked --jobs 1 tests::kira_bridge::graph:: -- --nocapture --test-threads=1
resolved_at: 2026-07-18
---


# Text01：system locale 依赖导致双 canonical lock 漂移

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/02-sound.md`
- 来源执行者：`plugins02-sound-m1-kira-core-closeout-r3-20260717`
- 来源执行切片：Sound M1 current-source Kira graph focused acceptance
- 修复责任计划：`docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md`
- 交接原因：最低失败 manifest 是 Runtime Text01 所属的 `zircon_runtime/Cargo.toml`；Sound 不拥有或修改该 manifest。

## 失败现象与复现证据

受管 Sound focused reservation `cba0f6bb6d424790b04200b338aa3fdf`、job
`421c07c12205407d88d4f041d8e4729f`、run `40e81752afd44ccebc003d9c3f525fa3`
以 exit 101 终止，未进入编译或测试执行。Cargo 原始诊断：

```text
error: cannot update the lock file E:\Git\ZirconEngine\zircon_plugins\Cargo.lock because --locked was passed to prevent this
```

Render01 RG-M2 随后在根 workspace 独立复现同一 lifecycle。managed job
`fd0cc41beca14c21ab9121cd4e191557`、run `3496f05915a840949d20770bb0aedcae`
使用 Rust 1.94.1，以 exit 101 / 无 live PID 释放，同样未进入编译或测试：

```text
error: cannot update E:\Git\ZirconEngine\Cargo.lock because --locked was passed.
```

因此本交接必须同时关闭根 workspace 与插件 workspace；Render01 不修改或吸收 Runtime
Text manifest、源码和双 canonical lockfile。

结构化 manifest/lock 比对确认：插件 workspace 133 个 member 及其直接依赖均已与
`zircon_plugins/Cargo.lock` 一致。剩余漂移来自外部 path dependency `zircon_runtime`：当前未提交
manifest 新增可选 `sys-locale = "0.3.2"`，并向 `text` feature 新增 `dep:sys-locale`，但插件锁中
`zircon_runtime` package 的依赖表没有 `sys-locale`；同一 manifest 也使根锁失效。

发现时的当前哈希：

- `zircon_runtime/Cargo.toml`：`8150D7D9BD5B124635577B6ABAB72778D48E38E5E5C66C3042F09576226B83BC`
- `Cargo.lock`：`FED7DA1BF408C9FD58D37768ECC4F92CF72571B51E957C40ADE880CC460822A5`
- `zircon_plugins/Cargo.lock`：`9CF31E50ABCC41EC77EDBEA7A18E37950595A1442224821C482CB7D95C202169`

Text01 父计划已将 locale 切片记录为 `implemented / validation_pending`，并明确 `sys-locale`
仅由现有 `text` feature 可选启用。创建交接时没有 active coordinator lease 覆盖 Runtime manifest。

## 最低共享层根因

Text01 改变共享 path package `zircon_runtime` 的依赖图，却未在同一 current-source acceptance
切片闭合根与插件 workspace lockfile。所有插件 workspace `--locked` 命令因此在依赖解析阶段停止，
无法到达 Sound graph 实现或行为测试。

## 架构修复验收

- Text01 以当前哈希重新认领 Runtime text source 与 `zircon_runtime/Cargo.toml`；其他计划不得重建或回退 locale 实现。
- owner 必须让两个 canonical lockfile 与最终 manifest 形成一致的 current path-package 依赖图。
- 最终刷锁后执行 canonical Rust 1.94.1 根/插件 `cargo metadata --locked --no-deps`，且命令后 lockfile 零漂移。
- 完成 Text01 locale/idempotent-discovery/default-family focused 与 upward 验证、独立复审和 managed milestone commit，回传 immutable SHA 与最终三文件哈希。
- 仅在 fixed return 后 Sound 才登记新的 source-bound Kira graph focused reservation；此前 Sound exit 101 是 dependency-resolution RED，不是 Sound graph 行为结果。

## 禁止临时方案

- 不得移除 `--locked`、手写 lock entry，或用复制/临时 manifest 验证。
- 不得要求 Sound、Render01 或 Shader06 修改或提交 Runtime Text01 source。
- 不得仅为让 Sound 编译而回退 `sys-locale`；Text01 必须接受当前架构并闭合双锁，或完成经复审的 Text-owned 设计修正。
- 不得从本次失败或历史 dependency-only M1.1 记录宣称 Sound M1 Kira hard cut 已 accepted。

## 修复结果与回传

- 根因：Text01 added optional sys-locale to the shared zircon_runtime text dependency graph without regenerating both canonical root and plugin workspace lockfiles, so every locked consumer stopped during dependency resolution.
- 架构修复：Preserved the Text-owned optional locale design, generated both canonical lockfiles through promoted managed Cargo metadata jobs, and kept sys-locale reachable only through the existing text feature; no compatibility shim or caller-side workaround was added.
- 验证：Rust 1.94.1 managed root and plugin metadata generation completed exit 0, followed by exact root and plugin cargo metadata --locked --format-version 1 --no-deps gates exit 0 with unchanged final lock hashes; independent review reported Critical 0, Important 0, Minor 0.
- 回传：Text01 dual-lock dependency closure is current-source consistent and ready for Sound and Render locked reruns.
