---
related_code:
  - zircon_editor/assets/ui/editor/host/workbench_shell.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_main_band.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_top_toolbar.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_activity_rail.zui
  - zircon_editor/assets/ui/theme/editor_workbench_strict.zui
  - zircon_editor/assets/ui/editor/layout/shell_regions.toml
  - zircon_runtime_interface/src/ui/**
  - zircon_runtime/src/ui/**
  - zircon_editor/src/ui/**
  - zircon_editor/src/scene/selection/**
  - zircon_editor/src/scene/viewport/**
design_references:
  - docs/plans/designment/01-penpot-inspired-interface-design.md
  - docs/plans/designment/02-milestone-execution-and-evidence.md
  - docs/ui-and-layout/editor-workbench-designs/STYLE-NOTES.md
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Widgets/SWidget.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Application/SlateApplication.h
plan_sources:
  - docs/plans/mvp/index.md
  - docs/plans/optimize/zircon_editor/01/2026-08-29-ui-hotspot-ownership-review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/01/2026-08-29-retained-shell-geometry-fast-path.md
  - docs/plans/optimize/zircon_editor/01/2026-08-30-runtime-taffy-retained-parent-product.md
  - docs/plans/optimize/zircon_editor/01/2026-08-30-svg-gpu-cache-observability-and-acceptance.md
  - docs/plans/optimize/zircon_editor/01/2026-08-30-ui-renderer-event-hot-path-recheck.md
implementation_files:
  - docs/plans/zircon_editor/editor_ui/manifests/m0-zui-slate-ui-contract.yaml
  - docs/plans/zircon_editor/editor_ui/evidence/m0-zui-slate-ui-contract.md
  - zircon_editor/assets/ui/editor/host/workbench_shell.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_main_band.zui
  - zircon_editor/assets/ui/theme/editor_workbench_strict.zui
tests:
  - zircon_runtime/tests/zui_penpot_bridge_contract.rs
  - zircon_editor/tests/integration_contracts.rs
  - zircon_editor/src/ui/**/tests/**
doc_type: milestone-detail
status: design-ready
last_refined: 2026-09-02
---

# Penpot / Slate 参考的 Editor UI 与 `.zui` 收敛计划

## 1. 目的

将当前 Editor 的 `.zui` 设计、工作台交互和 UI 性能收敛到一条可验证的单向链：

```text
semantic tokens -> component/catalog -> .zui view -> committed presentation generation
-> arranged geometry / hit index / paint segments -> transient interaction patch
-> typed command/transaction -> journal/persist
```

Penpot 贡献可逆的 authoring projection、语义布局和 token 思想；`dev/UnrealEngine` Slate 贡献 widget tree、arranged geometry、WidgetPath、统一输入、popup/click-outside、focus 和最小 invalidation 的行为标准。不得复制 C++/ClojureScript 实现，也不得新增第二 UI schema、第二 renderer 或 host 控件名特判。

## 2. 当前事实与设计决策

- `.zui` v2 是唯一运行时资产格式；未知字段、events、bindings、repeat、imports、tokens、component contract、style scope 和 raw metadata 必须保留。
- `zircon_runtime_interface::ui` 只放 ABI-safe DTO/契约；布局 pass、dispatch、template/compiler、surface mutation、text/layout 和 render extraction 归 `zircon_runtime`；Editor 只负责 retained host/workbench/业务接线。
- Taffy 是 Flex/Grid/Block/Wrap 的布局权威；Overlay/Canvas/Scroll/Virtual/docking 可以保留专用容器。任何 fallback 必须显式记录 reason。
- `UiSurfaceFrame`/arranged tree 同时服务布局、命中和渲染提取；输入和绘制 consumer 不应各自重建 surface 或 presentation。
- 样式状态优先级固定为 `disabled > pressed > selected/focused > hovered > default`；组件逻辑只产出语义状态，视觉由 selector/token 决定。
- `pointer move/hover/drag/zoom` 属于 transient state；pointer-up/submit/commit 才进入 command/transaction/undo/persist。
- MVP 当前仍为 `in_progress`，F0-F5 仍 blocked；本计划的 foundation 只能作为 design-ready 或直接支撑 F0-F4 的切片，不改变 MVP 状态。

## 3. 里程碑

### M0：契约与基线（本记录）

产出当前计划、M0 manifest/evidence，冻结 owner、设计链、Penpot A0/A1/A2-C/A2-P 顺序、Slate 参考和验证纪律。M0 不改生产代码、不宣称产品验收。

### M1：Shell token、组件状态和布局基线

- 以 `editor_base.zui`、`editor_material.zui`、`editor_workbench_strict.zui` 为基础，统一 surface/border/accent/focus/status/density/control tokens。
- 收敛 `workbench_shell.zui`、`workbench_main_band.zui`、`workbench_top_toolbar.zui`、`workbench_activity_rail.zui` 的 variant/state contract；icon-only 控件继续拥有 label、tooltip 和 keyboard route。
- 用 `shell_regions.toml`、presets、page templates 和现有 autolayout/persistence owner 统一 drawer/pane 尺寸事实。
- 明确窄窗口策略：先折叠/压缩侧栏，保护 center viewport；使用 min/preferred/max/priority/stretch/shrink/clip，不使用页面绝对定位。
- 退出：`.zui` governance/parser、component contract、layout round-trip 和结构 guards 通过；区域尺寸与状态只有一个 authority。

### M2：输入、焦点、选择和提交边界

- 参考 Slate `WidgetPath` 与 `SlateApplication` 的 preview/tunnel、bubble、capture、focus、click-outside/popup 语义，复核 Runtime dispatch 和 Editor retained pointer bridges。
- 统一 activity rail、tabs、splitter、viewport gizmo、Hierarchy、Inspector、popup/menu 的 compiled route id/typed action。
- 建立 `selection -> inspection snapshot -> controlled FieldEditor -> command/transaction -> refresh` 路径；禁止直接修改 ECS/World。
- 同值 hover 变成 O(1) no-op；状态变化只更新 old/new targets，drag 中的预览不污染 history。
- 退出：click/keyboard/capture/outside-dismiss/drag-commit/disabled/read-only focused tests 通过，并能支撑 F4 正常 UI 路径。

### M3：增量布局、damage 和渲染缓存

- 按 dirty domain 发布 immutable presentation generation；stable generation 不做 full presentation clone、reflection/index rebuild、RGBA copy 或 event-time rebuild。
- 复用 retained-parent geometry fast path、refresh batching、pane/shell geometry reuse、spatial/control/hit index 和 compiled paint segments；不在 leaf converter 增加无失效 cache。
- SVG/icon、text measure/shaping、template/compiler/file cache、GPU command batch 的 key 必须覆盖 style/layout/resource generation、DPI、viewport、backend epoch；主题、热重载、资源变化必须显式失效。
- stable frame 不重复全树 layout/extract；单 section damage 只访问相交且发生变化的 segment，正常 ordered stream 不 fallback sort。
- 退出：1/100/1k/10k node 与 same-target/alternating-target pressure 计数、full/patch parity、generation invalidation 和产品 profile 证据齐全。

### M4：Penpot parity 与 E0 壳产品收口

- A0 contract、A1 browser round-trip、A2-C loader/compiler/surface contract 先完成；A2-P 必须等待 F0-F4 gate，且同时具备 Penpot geometry、Runtime structured frame、Editor screenshot 和 tolerance report。
- 完成 E0：tabs、drawer 开合/调整、activity rail、status、menu/shortcut、保存/诊断反馈全部由 Runtime UI retained host 承载。
- 删除无调用者的旧 reflection/full presentation/paint fallback；不以截图替代 frame/interaction evidence。
- 退出：一次 batched Windows validation、产品 bundle smoke、交互 trace、结构化 frame 和截图证据一致；未验收项继续 open。

## 4. 实施边界

首批只修改当前 milestone 明确 owner 文件。优先 `.zui`/theme/layout 与对应 focused tests；只有在确认行为缺口后才修改 `zircon_editor/src/ui/**`，若触及 `zircon_runtime_interface` 或 Runtime UI contract，先补 C3 handoff。保持 `lib.rs`/`mod.rs` thin，禁止兼容 re-export、facade 或临时旁路。

当前工作树已有大量其他 Session 修改；每个 milestone 开始前重新读取 `git status`、目标文件 diff 和 source fingerprint，只 stage 本 milestone 文件，不覆盖、不 reset、不清理未知改动。

## 5. 验证和交付

- 实施切片：`git diff --check`、zui governance/parser/structure guards、已有 focused tests；不默认运行完整 workspace Cargo。
- 里程碑测试：Windows-native，使用 coordinator 分配的 `D:/E:/F:` target-dir 和 `validate-matrix.ps1`；受影响时加入 `zircon_runtime_interface`、`zircon_runtime`、`zircon_editor` scoped check/lib tests 与 integration contracts。
- Penpot plugin 的 typecheck/build/spec 和 `zui_penpot_bridge_contract` 必须记录真实执行结果；环境不可用写 deferred，不虚报通过。
- 性能 evidence 记录 generation、dirty domain、visited nodes、cache hit/miss、rebuild、damage、CPU/p95 和输出路径；不把静态审计当 runtime timing。
- 每个已通过 milestone 单独创建新 git commit。Git push 是共享外部操作；用户所说“推送企微”在当前仓库未找到 remote/脚本定义，执行前必须确认具体 remote、branch 和企微入口。 

## 6. 依赖

`M0 -> M1 -> M2 -> M3 -> M4`。A0/A1/A2-C/A2-P 与 M0/M1/M2/M3/M4 并行关系由其各自 evidence 和 MVP gate 决定；不得越过 F0-F4 直接声称 A2-P 或产品完成。

当前 M0 manifest：`manifests/m0-zui-slate-ui-contract.yaml`。
当前 M0 evidence：`evidence/m0-zui-slate-ui-contract.md`。
---

> 具体实现前，必须按 milestone 重新检查当前 source、现有 open failure 和 owner lease；本文件不授权绕过共享协调器造成文件覆盖。
