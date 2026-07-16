---
related_code:
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/access.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/surface.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/surface.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/plans/zircon_runtime/runtime/15/fixed-2026-07-14-ui-text-project-asset-manager-access-consumer-drift.md
  - docs/plans/zircon_runtime/runtime/15/fixed-2026-07-14-ui-text-manager-access-cross-frame-retention.md
tests:
  - managed Windows default-feature cargo build -p zircon_runtime --locked (job c2db4e7bfe0647678e6334648b6df811; passed)
  - rustfmt --edition 2021 --check --config skip_children=true zircon_runtime/src/graphics/scene/scene_renderer/ui/construct.rs zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/surface.rs
  - python -m unittest tools.tests.test_frameworks_05_manager_access_lifetime tools.tests.test_frameworks_05_layer_direction -v
  - git diff --check -- docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md docs/plans/zircon_runtime/frameworks/05/2026-07-14-ui-text-project-asset-manager-access-consumer-drift.md docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md docs/plans/zircon_runtime/runtime/15/fixed-2026-07-14-ui-text-manager-access-cross-frame-retention.md docs/zircon_runtime/core/manager.md docs/zircon_runtime/graphics/scene/scene_renderer/ui/text.md tools/tests/test_frameworks_05_manager_access_lifetime.py zircon_runtime/src/graphics/scene/scene_renderer/ui/construct.rs zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
doc_type: failure-repair-detail
---

# Frameworks05 UI Text Manager Access Consumer Correction

Plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
Parent milestone: M4
Status: accepted
Date: 2026-07-14
Files: ["docs/plans/zircon_runtime/frameworks/05/2026-07-14-ui-text-project-asset-manager-access-consumer-drift.md", "docs/plans/zircon_runtime/runtime/15/fixed-2026-07-14-ui-text-manager-access-cross-frame-retention.md", "docs/plans/zircon_runtime/runtime/15/fixed-2026-07-14-ui-text-project-asset-manager-access-consumer-drift.md", "docs/zircon_runtime/graphics/scene/scene_renderer/ui/text.md", "tools/tests/test_frameworks_05_manager_access_lifetime.py", "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs", "zircon_runtime/src/graphics/scene/scene_renderer/ui/construct.rs", "zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs", "zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs"]

## 状态与完成项目

| 切片 | 状态 | 完成证据 |
|---|---|---|
| UI text asset-manager access lifetime hard cut | `frameworks_05_ui_text_manager_access_cross_frame_retention_accepted` | 独立 review 驳回构造期 concrete Arc 留存；长期 text owner 已改存 versioned access，构造/每帧 prepare 精确一次 bounded resolve 与两条错误出口由加强后的守卫锁定；fresh managed default-feature Runtime build 已通过，Failure 已原子回传，整改后独立复审为 0 Critical / 0 Important。 |

## 根因与修复

Frameworks05 已把 UI renderer 参数硬切为 `ProjectAssetManagerAccess`，但调用点仍把 access object 直接传给要求 `Arc<ProjectAssetManager>` 的 text subsystem，并对已经 infallible 的 constructor 保留 `?`。这在 Runtime15 Render owner 拆分的 fresh build 中产生 E0308/E0277。

初次修复只解决了 constructor 类型漂移，独立 review 随后证明 text subsystem 会跨帧保存 concrete Arc，因此该边界不足。最低真实修复现在由 `ScreenSpaceUiTextSystem` 持有 `ProjectAssetManagerAccess`：构造初始化与每帧 `prepare` 分别解析一次，Arc 只存在于当前 bounded operation。`ScreenSpaceUiRenderer::record` 返回 `GraphicsError`，普通 scene 与 render-graph 两条调用链均显式传播，不增加兼容 API、旧 resolver、Arc adapter、silent fallback 或双 owner。

## 验证

- 红态：managed job `ff2fa0c62ede4e858ef24e01382c4263` 在 `ui/construct.rs:81` 报 E0308/E0277。
- 绿态：managed Windows job `9dac70c034fb4aa18155d370f77073e1` 完成 `cargo build -p zircon_runtime --locked`，通过，耗时 7m51s。
- 独立 review：Critical 0 / Important 1 / Minor 0，发现跨帧 concrete manager retention，阻止协调器提交。
- 新 lifetime guard 红态准确失败于 `asset_manager: Arc<ProjectAssetManager>`；实现后 current-source 1/1 通过，并已加强为分别限定 constructor/prepare 方法且要求各自恰好一次 `resolve()`；scoped rustfmt 与 diff check 通过。
- Frameworks05 current-source layer suite 24/24 通过；Failure artifacts 115/115 schema/link 通过；本节点计划输出布局无新增违规。
- Editor11 binary visibility 外部阻断已由其责任会话解除。fresh managed Windows job `c2db4e7bfe0647678e6334648b6df811` 完成 `cargo build -p zircon_runtime --locked` 并返回 0，已覆盖本轮 UI owners 的默认特性编译门。更宽 Runtime lib 测试仍受共享 Cargo 池排队影响，不用它替代专门回归或原子 Failure 回传证据。
- 来源 Editor Layout15 的 managed job `6af3a291b7754a6d86a91f61fe56e12e` 已成功编译 Runtime 与 Editor lib，证明原 E0308/E0277 不再出现；随后仅在其测试 owner 的 `RetainedUiProjection.nodes` 外部 E0609 停止，该问题不归本修复。
- render-graph 的字符串错误出口已经由共享 Render18 原子提交 `53c48d1c4f9de77ff4c3836bcd2fa83c4ac0986f` 收录；本 failure-repair 提交只携带仍未入库的 Runtime Text owners、精确守卫、模块文档和 canonical fixed records。
- 最终独立复审：Critical 0 / Important 0 / Minor 0；唯一措辞问题已整改为两个 handoff 均归属 Runtime15。代码复核确认 constructor/prepare 方法范围内各恰好一次 resolve，fixed 记录不再指向已移动的 failure，也不再保留 open/pending 矛盾。

## 剩余范围

本 failure-repair 切片已 accepted。它只关闭 UI text manager-access lifetime 与 Runtime15 下的两个对应 handoff，不宣称 Frameworks05 M4 总里程碑、Runtime15 Render owner 切片或全 workspace 已完成；M4 总记录继续服从 `M1 → M2 → M3 → M4` 拓扑，不以本原子提交绕过前置里程碑。
