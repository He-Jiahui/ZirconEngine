---
handoff_kind: failure
status: open
created_at: 2026-07-30
summary_slug: runtime-frame-capture-sibling-module-projection
origin_plan: docs/plans/zircon_runtime/render/17-performance-and-profiling.md
fixing_plan: docs/plans/mvp/03-f2-scene-runtime.md
origin_child_dir: docs/plans/zircon_runtime/render/17
fixing_child_dir: docs/plans/mvp/03
plan_link_mode: child_record_only
related_code:
  - zircon_app/src/entry/runtime_entry_app/surface_present/redraw.rs
  - zircon_app/src/entry/runtime_entry_app/frame_capture.rs
tests:
  - managed cargo test -p zircon_runtime --lib graphics::tests::render_product_post_process_full_chain::render_product_post_full_chain_all_effects_on --locked -- --exact --test-threads=1
---

# MVP 03：runtime frame capture sibling module projection

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/render/17-performance-and-profiling.md`
- 来源执行切片：Render17 current-source WGPU full-chain chromatic diagnostic gate
- 修复责任计划：`docs/plans/mvp/03-f2-scene-runtime.md`
- 交接原因：失败位于 MVP F2 当前拥有的 `runtime_entry_app` 产品帧捕获模块边界；Render17 只消费该应用层 crate，不能在渲染计划内改写其模块投影。

## 失败现象与复现证据

2026-07-30 12:25 CST，Render17 受管 job
`d425b47aef704483adab085546417704` 在运行上列 exact gate 前编译当前共享源码，`zircon_app` 自然终止：

```text
error[E0433]: could not find `frame_capture` in `super`
  --> zircon_app/src/entry/runtime_entry_app/surface_present/redraw.rs:148:16
148 -         super::frame_capture::write_runtime_frame_png(
```

`frame_capture` 定义于 `runtime_entry_app` 根模块；`redraw.rs` 位于其 `surface_present` 子模块，因而该路径解析为 `surface_present::frame_capture`，而不是根模块的 sibling。Render17 测试体尚未执行，故本轮不是色彩链路的通过或失败证据。

## 最低共享层根因

MVP F2 新增的产品截图写出调用未从 `surface_present` 正确投影到 `runtime_entry_app::frame_capture`。这是应用层模块所有权/可见性错误，不是 WGPU、Render17 后处理或测试配置错误。

## 架构修复验收

- `surface_present::redraw` 通过唯一的 `runtime_entry_app` 根模块路径调用帧捕获 writer，不创建 alias、重复 writer 或 test-only fallback。
- MVP F2 的聚焦运行时帧捕获测试通过。
- 重跑本 handoff 的原始 Render17 受管 exact gate；它至少必须越过 `zircon_app` 的 E0433 编译边界，之后 Render17 再独立判定色彩结果。

## 禁止临时方案

- 不得在 `surface_present` 复制 `frame_capture`、添加兼容 alias，或关闭首帧 PNG 写出以掩盖编译错误。
- 不得将 Render17 的截图/色彩验收改为跳过 `zircon_app` 编译。

## 修复结果与回传

### 2026-08-27 受管验证前置阻塞

- 以 Coordinator01 failure-cleanup session 发起
  `zircon_app` focused lib-test
  `frame_capture_projects_to_the_runtime_entry_root_sibling`；该 gate 会真实编译
  `zircon_app`，用于直接复核旧 E0433 边界。
- 协调器在 validation ticket/Cargo job 创建前以
  `unmanaged_artifacts_detected` fail-closed。唯一当前路径为
  `D:\ZirconBuilds\tooling15-wave137-runtime-20260827-054615`，其 artifact cleanup
  reservation 于 `2026-08-26T21:55:50.161328+00:00` 仍存在；对应 Tooling15 bootstrap
  进程仍存活，MVP03 未删除、终止或接管该 owner 的产物。
- 本次没有启动 Cargo/rustc，也不构成新的动态 GREEN。HEAD 中仍保留唯一
  `super::super::frame_capture::write_runtime_frame_png(...)` 调用和两条 committed
  source guard；failure 继续保持 `open`，待 artifact governance 恢复后由 FIFO
  重提一次 focused gate。

### 2026-08-24 受管验证前置阻塞

- validation ticket：`d1ec49cbdb304826a21bb59a7faccbba`
- validation copy：`a5356e6b9f4742a693f895ec67f2ac37`
- 终态：`failed`，阶段 `closure_planning`，Cargo 未启动。
- 精确外部依赖链：`zircon_editor/src/tests/ui/boundary/global_material_surface_assets.rs` 引用了当前副本缺失的 `zircon_editor/assets/ui/editor/animation_editor.zui`；durable error code 为 `validation_copy_compile_time_resource_missing`。
- 本 failure 的两条实现路径仍与 snapshot `2089` 一致；不得把 Editor 资产缺失误记为 frame-capture GREEN。等待该外部 compile-time resource 恢复后，仅按 FIFO 重提同一 focused test。

Open state: `MVP03 source repair complete; managed upward validation pending`; no pass is claimed.

- `surface_present::redraw` now calls the one root-owned writer through
  `super::super::frame_capture::write_runtime_frame_png(...)`; it does not create a
  `surface_present` alias, duplicate writer, or test-only capture fallback.
- Current-source static review confirms the root `frame_capture` module exports the writer and
  the redraw call resolves through the sibling path. The local source guard covers that call
  shape; this is not Cargo evidence.
- The declared managed Render17 exact gate must still rebuild `zircon_app` from a fresh source
  snapshot and run past the previous E0433 boundary. Only its terminal result can determine the
  downstream render outcome or move this canonical handoff to `fixed-*`.

### 2026-08-28 current-source managed replay

- Managed job `09b9495deb684503850deaa5b0bdf774` ran
  `cargo test -p zircon_app --locked --lib frame_capture_projects_to_the_runtime_entry_root_sibling`
  in the retained D-drive pool and released normally with no live process PIDs.
- Cargo rebuilt the current dependency graph and did not reproduce the original
  `surface_present::frame_capture` E0433. Compilation stopped before the focused test executed on
  the separately owned `zircon_runtime_host/src/foreign_output/item_count.rs`: its match over
  `WorldQueryResult` does not yet cover `TransformSnapshot` (E0004).
- This replay proves the prior artifact-governance and validation-copy closure blockers are no
  longer the admission boundary, but it is not a focused GREEN. The canonical frame-capture
  failure remains `open` until the external world-query consumer repair is integrated and the
  same managed test executes successfully; MVP03 does not absorb or patch that owner here.
