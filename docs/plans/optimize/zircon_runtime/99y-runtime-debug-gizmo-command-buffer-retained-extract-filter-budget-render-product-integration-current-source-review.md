---
title: Runtime Debug Gizmo、Command Buffer、Retained Asset、Extract、View Filter、Budget、Render 与 Product Integration Current Source Review
category: zircon_runtime
report_id: Runtime124
review_date: 2026-08-23
baseline_head: 6ce24f25e46d8f370aa5b5d4e8487f53103b43c0
baseline_epoch: 375
supersedes:
  - docs/plans/optimize/zircon_runtime/49-runtime-debug-gizmo-command-buffer-retained-extract-filter-budget-render-product-integration-review.md
related_code:
  - zircon_runtime/src/core/framework/gizmos
  - zircon_runtime/src/core/framework/render/overlay.rs
  - zircon_runtime/src/core/framework/navigation/gizmo.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/buffers
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_gizmo
  - zircon_runtime/src/dynamic_api/session/extract_stats.rs
  - zircon_editor/src/core/editor_extension/viewport_overlay_provider.rs
  - zircon_editor/src/scene/modes/viewport_overlay_builder.rs
  - zircon_editor/src/scene/viewport
  - zircon_plugins/navigation/editor
  - zircon_plugins/ai/editor
tests:
  - zircon_runtime/src/tests/gizmos/mod.rs
  - zircon_runtime/src/graphics/tests/render_product_ui.rs
  - zircon_runtime/src/graphics/tests/project_render/project_scenes.rs
  - zircon_editor/src/tests/editor_event/runtime/extensions_registration/overlay_lifecycle.rs
  - zircon_plugins/navigation/editor/src/tests.rs
  - zircon_plugins/navigation/editor/src/tests/viewport_overlay_provider.rs
  - zircon_plugins/ai/editor/src/tests.rs
plan_sources:
  - docs/plans/mvp/index.md
  - docs/plans/optimize/zircon_runtime/49-runtime-debug-gizmo-command-buffer-retained-extract-filter-budget-render-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/49/2026-08-20-shared-immutable-gizmo-asset-commands.md
  - docs/plans/optimize/zircon_editor/58-editor-scene-viewport-host-render-product-surface-lifecycle-frame-currentness-multi-viewport-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/59-editor-scene-viewport-interaction-controller-input-picking-selection-highlight-gizmo-transaction-cancel-generation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/67-editor-scene-viewport-transform-manipulation-gizmo-pivot-coordinate-space-grid-snapping-workplane-numeric-surface-vertex-alignment-preference-transaction-product-integration-current-source-review.md
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - .codex/plans/全系统重构方案.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/DrawDebugHelpers.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/DrawDebugHelpers.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/LineBatchComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Components/LineBatchComponent.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Debug/DebugDrawService.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Debug/DebugDrawService.cpp
  - dev/bevy/crates/bevy_gizmos/src/config.rs
  - dev/bevy/crates/bevy_gizmos/src/gizmos.rs
  - dev/bevy/crates/bevy_gizmos/src/retained.rs
  - dev/bevy/crates/bevy_gizmos_render/src/lib.rs
  - dev/bevy/crates/bevy_gizmos_render/src/retained.rs
  - dev/bevy/crates/bevy_gizmos_render/src/pipeline_3d.rs
  - dev/godot/editor/scene/3d/node_3d_editor_gizmos.h
  - dev/godot/editor/scene/3d/node_3d_editor_gizmos.cpp
  - dev/godot/editor/scene/3d/node_3d_editor_plugin.cpp
  - dev/Fyrox/fyrox-impl/src/renderer/debug_renderer.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Volume/VolumeGizmoDrawer.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugDisplaySettingsUI.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 99y · Runtime Debug Gizmo Current Source Review

## 1. 结论

Runtime49 的核心结论在当前源码中仍成立：`core::framework::gizmos` 有 typed command、稳定记录顺序、immediate buffer、CPU reference tessellation 和 retained transform 入口，不是空壳；但它仍不是 Zircon 产品实际使用的 debug-draw 系统。对 production source 反查后，`GizmoBuffer`、`GizmoAsset`、`RetainedGizmo`、`extract_gizmo_overlay` 与 `append_gizmo_overlay` 仍然只有定义和 dedicated tests，**production consumer 为 0**。Editor camera/light、Runtime Navigation、Navigation Editor、AI Editor 与 Virtual Geometry 继续直接构造 `SceneGizmoOverlayExtract` / `OverlayLineSegment`，各自复制 primitive 展开、颜色、可见性和生命周期政策。

本轮确认一项真实但很窄的进展：`GizmoAsset.commands` 已从按值 `Vec<GizmoCommand>` 改为 `Arc<[GizmoCommand]>`，asset/retained clone 不再复制稳定 command stream，serde shape 也由测试保持。这只关闭了 repeated clone 的局部成本，没有形成 asset handle、generation、instance identity、config ownership、TTL、remove、world/session scope 或 module quiescence，所以 `GIZMO-P1-006` 只能从 Open 改判为 **Partial**，不能 Closed；child output 中 release benchmark 的 Windows 实测和 combined managed validation 仍明确是 pending。

Editor 侧也出现局部生命周期底座：viewport overlay provider 有 ID 注册、duplicate 拒绝、capability toggle、factory/extract panic containment 和 quarantine。它仍直接返回 `Vec<SceneGizmoOverlayExtract>`，context 没有 view/world/frame generation、budget、deadline 或 receipt，也没有 provider unregister、admission close、in-flight drain 和 code/resource lease。因此 `GIZMO-P1-010` 为 **Partial**，资格门 `GZ04` 也只能 Partial；这不等于 plugin unload 已安全。

其余关键缺陷未收敛。七组 config 中只有 `enabled` 与 `color_policy` 被 extract 消费；group、line width、depth bias、render layer 与 screen scale 都是 false surface。Axis 仍只变换 origin，Sphere/Circle 不随 transform 缩放，Circle normal 没有 inverse-transpose/plane policy，AABB 仍只变换 min/max；NaN/Inf、负半径、巨量 command 和 unchecked `usize -> u32` 均未 fail-close。最终 renderer 仍逐帧 CPU 展开和 `create_buffer_init`，每个 icon 独立 buffer/draw，scene/grid/selection/wire/gizmo/handle 仍拆成多个 LoadStore pass；`PERF-MVP-333` 的父 P0 没有被本报告重复计数，也没有被局部 Arc 改动解决。

当前裁决为 **0 项本地 P0；54 P1 Open、2 P1 Partial、0 Closed；14 P2 Open；35 Gate Fail、1 Gate Partial**。本轮新增 finding 为 0，只对 Runtime49 账本做 current-source 复核、措辞纠正、owner 收束和实施排序。本文不把源码存在、ignored benchmark、DTO 像素测试或 callback quarantine 写成工程资格，更不构成性能或表现优于 Unreal 的证据。

本轮只做静态 review 和文档记录，没有修改 production、tests、Cargo、ABI 或参考源码；没有运行 Cargo、Editor、WGPU、RenderDoc、GPU timestamp、device loss、plugin unload、soak 或 benchmark。MVP 仍未完成，`source_recheck_required` 保持 true；后续实现开始前必须重新冻结共享工作树。

## 2. 审查边界与物理冻结

### 2.1 Focused 集合

| 范围 | 文件 / 行 / bytes / tests | fingerprint |
|---|---:|---|
| canonical gizmo owner | 6 / 800 / 20,385 / 0 | `370dccbd41b5e8a5ae38dfac7a06c165c756e7f976d673a8894c58413b185646` |
| Runtime/Graphics/Editor/plugin 产品链 | 34 / 4,670 / 162,841 / 7 | `d65f762b8064e6a97f1b0a33a29a97cbcccbe9ff5c0fb58ee0068523fb4b002c` |
| focused tests | 7 / 3,468 / 125,250 / 58 | `aebc0e523364871b68a2d0daf9d98397b6ff7b0bdb20b24bd97c68483918a351` |
| 五引擎参考实现 | 18 / 11,533 / 463,587 / 0 | `f4d41747fb7c930fa34cac37378a9cd1ecfa913f12ff58df4aa94ea4ec4c89c0` |

Zircon 三组是不重叠集合，共 47 个 focused 文件、8,938 行、308,476 bytes、65 个 `#[test]`，总 fingerprint 为 `5380c6b6d7051313afdc79641a1e888966781e8e0a7fe0d1402028c48b041193`。其中 dedicated gizmo 文件当前有 12 个 test，最后一个为 ignored release performance evidence；旧报告“十个 dedicated test”的计数已过时。

fingerprint 算法：仓库相对路径转 `/`、小写、ordinal 排序去重；每项编码为 `lowercase-path + NUL + lowercase per-file SHA-256`，以 LF 连接且末尾无 LF，再对 UTF-8 payload 计算 SHA-256。它只冻结本轮实际读取集合，不是 runtime identity、ABI、artifact 或 release identity。

### 2.2 Currentness 与证据等级

- 会话注册基线为 `6ce24f25e46d8f370aa5b5d4e8487f53103b43c0` / epoch 375；本报告读取当前 working tree，并保留其他会话在 index 和 gizmo test 中的改动。
- Runtime49 child slice 证明共享 immutable command storage 已实现，但同时写明 Cargo regression、combined managed validation 与 Windows release timing pending；本报告只承认源码和 focused test contract，不升级为 accepted milestone。
- Editor provider lifecycle 的 duplicate/toggle/panic tests 是有效的局部 E3 证据；没有真实 native unload/unregister/quiescence 测试。
- WGPU tests 直接注入最终 `SceneGizmoOverlayExtract`，证明部分 overlay/UI composition，不证明 canonical API、budget、retained reuse 或 domain cutover。
- 参考引擎结论来自本地实现源码。Bevy/Godot selected slice 未找到直接 focused unit tests；Unreal 测试树未做全量证明，因此本文不声称参考引擎没有测试。

## 3. 当前纵向调用链

```text
当前孤立 generic 路径：
GizmoBuffer -> GizmoAsset/RetainedGizmo -> extract_gizmo_overlay
            -> SceneGizmoOverlayExtract -> append_gizmo_overlay
            -> zero production caller

当前真实产品路径：
Editor Camera/Light ---------------------+
Runtime Navigation ----------------------+
Navigation Editor provider --------------+-> SceneGizmoOverlayExtract
AI Editor provider ----------------------+   -> per-frame CPU vertices
Virtual Geometry debug ------------------+   -> per-frame GPU buffers
                                             -> multiple overlay passes
```

### 3.1 Canonical owner 当前事实

| 合同 | 当前事实 | 裁决 |
|---|---|---|
| command surface | Line/Ray/LineStrip/Rect/Circle/Sphere/Cube/Aabb/Axis 九类 typed command；稳定 Vec 顺序 | 可保留最小 source vocabulary，不等于可扩展注册表 |
| config | group、enabled、line width、depth bias、render layer、color policy、screen scale | extract 只消费 enabled/color；五项 false surface |
| immediate | disabled push 不写入，clear 由 caller 显式调用 | 无 frame writer、seal、late receipt、自动清空或 producer identity |
| retained | `Arc<[GizmoCommand]>` 共享 immutable snapshot；instance 有 transform/config | 无 handle/generation/remove/TTL/world/module owner；asset snapshot 丢 config |
| extract request | bare `EntityId`、closed kind、selected bool、两个引用 Vec | 无 runtime/world/view/frame generation、budget、bounds、receipt |
| compiler | CPU 展开所有 command；Circle 固定 32 段；每次新 Vec | 无 validation、checked accounting、visibility/LOD、compiled generation |
| final DTO | owner/kind/selected + owned lines/wires/icons/picks | 仍是领域可直接写入的万能 DTO；selected 不进入 renderer |

### 3.2 几何正确性仍未闭合

| Primitive | 当前 retained transform | 错误面 |
|---|---|---|
| Line/Ray/LineStrip | point/delta path基本可用 | space、large-world、finite/range仍无合同 |
| Axis | 只变换 origin | rotation 后仍绘制 global axis；scale policy缺失 |
| Circle | center/normal变换，radius原值 | non-uniform/negative scale、normal inverse-transpose、ellipse policy缺失 |
| Sphere | center变换，radius原值 | uniform scale也错误；non-uniform下sphere/ellipsoid未裁决 |
| Aabb | 变换 min/max 后重建 AABB | rotation/shear/negative scale会丢角、倒置或产生错误包围盒 |

当前没有上述失败面的行为测试；现有 source-string test 只检查 Mat4 读取次数和临时 Vec 文本形状，是 implementation-coupled false green。NaN radius 可一路展开为 NaN vertex，huge LineStrip/serde Vec 没有 decode/submit limit。

### 3.3 产品 producer 仍全部绕过 canonical API

| Producer | 当前输出 | 直接后果 |
|---|---|---|
| Editor camera/light | `render_packet.rs` 直接组装 line/icon/pick DTO | Editor policy、pick 与 visual 脱离统一 source/config/budget |
| Runtime Navigation | 每 triangle 输出三条边 | 相邻三角形共享边重复，稳定 navmesh 仍 O(T) 展开 |
| Navigation Editor | provider 直接返回 overlay DTO | provider lifecycle局部增强，但不进入 runtime compiler/lifetime |
| AI Editor | 私有 24 段 circle 与 perception shape | 与 generic 32 段 policy分叉，无统一质量或 hard cap |
| Virtual Geometry | 私有 box/cross builders并 clone source overlays | geometry、颜色、阈值与 generation 独立演化 |

`SceneGizmoKind` 仍硬编码 Camera、DirectionalLight、VirtualGeometryBvh、VirtualGeometryVisBuffer、NavigationMesh、AiPerception。新增首方或第三方 domain 必须改 Runtime enum，而不是注册 qualified descriptor。

### 3.4 Renderer 与 diagnostics 仍是临时提交模型

- scene gizmo prepare 每帧重建完整 line vertex Vec，`build_line_buffer` 再 `create_buffer_init`；stable source 没有 CPU artifact/GPU arena 命中路径。
- icon 每项调用 `build_icon_buffer`，prepared state 保存 `Vec<PreparedIconDraw>`，record 时逐 icon bind/draw。
- line pipeline 固定 `LineList + LessEqual + zero bias`；公开 width、depth bias、screen scale、render layer 都没有 shader/pipeline consumer。
- selection、wireframe、grid、scene gizmo、handle 以多个 LoadStore pass提交；顺序测试只冻结 DTO/pass order，没有证明 `PERF-MVP-333` 的单 pass、零稳定重建或像素等价。
- GPU count 使用 `vertices.len() as u32`，estimate line count 使用 unchecked sum；没有 typed overflow/partial acceptance。
- `extract_stats.rs` 只估算 scene-gizmo DTO slice payload bytes，不记录 Vec capacity、compiled artifact、upload bytes、buffer create、draw/pass、resident generation、culled/dropped 或 producer/category。

## 4. 五引擎参考差异

| 参考 | 已核对的工程合同 | Zircon 应吸收 | 不照搬项 |
|---|---|---|---|
| Unreal | DrawDebug world owner、persistent/lifetime/depth/thickness；LineBatch 有 RemainingLifeTime/DepthPriority/BatchID、flush/tick expiry/bounds；DebugDrawService 按 name 注册并返回 delegate handle，支持显式 unregister 与线程安全注册 | world/session owner、lifetime/remove、depth/style、batch identity、对称注册注销、卸载前 drain | 不复制 UObject/global singleton/宏体系，也不把 CPU line batch 当性能终点 |
| Bevy | typed config group store；pixel width/perspective/style/joints/depth bias/RenderLayers 有 extract/render consumer；immediate storage 按 schedule 清理、deferred per-system merge；retained 用 Asset Handle + Transform/config，GPU asset按generation prepare | typed group/config、schedule seal、并行 producer merge、asset generation、per-view layer/style extract、稳定 retained GPU asset | TypeId 不能作为 Zircon 跨 DLL/plugin 稳定 identity；不把每个 API 形状机械改成 ECS system param |
| Godot | gizmo visual mesh/lines/handles 与 collision segments/triangles分离；selected/hidden/BVH node；plugin priority/name/on-top/hide/select policy；plugin sorted add/remove与BVH insert/update/remove | visual/pick identity共享但 representation/budget分离，plugin descriptor/lifecycle，spatial index与精确更新 | 不复制 Godot inheritance/editor node层级或裸对象生命周期 |
| Fyrox | DebugRenderer 复用一个 `GpuGeometryBuffer`，每帧 `set_lines` 并返回 `RenderPassStatistics` | 即使简单实现也应复用 GPU object并返回真实统计；Zircon目标进一步做到generation dirty upload | 不把单一动态线缓冲当最终 retained/culling/budget架构 |
| Unity Graphics | Volume gizmo 使用 `Gizmos.matrix`、wire/solid preference、Active/Selected/NonSelected过滤；sphere明确一个scale-axis政策；Debug settings UI支持register/lazy init/reset/unregister/foldout persistence | transform政策必须显式，产品category settings需要注册、重置、持久化和状态过滤 | 本地 Graphics mirror不是完整 Unity engine；只将其作为可见政策/UI生命周期证据 |

五套参考并不要求 Zircon 复制某一个类层级，但共同证明工程级 debug draw 至少需要：qualified source identity、owner/lifetime、真实 config consumer、view filter、几何政策、budget/receipt、稳定 artifact/GPU storage、diagnostics 和对称扩展生命周期。性能优势只能来自更短的 compiled path、delta update 和更低 pass/draw 成本，不能来自删除这些语义。

## 5. Canonical owner 边界

| 事实 | Canonical owner | Runtime124 只拥有的纵切面 |
|---|---|---|
| debug source/category/producer identity、immediate/retained lifetime | Runtime124 | descriptor、writer、handle、generation、remove/TTL、seal/drain |
| primitive validation、bounds、tessellation、style/view compilation、budget/receipt | Runtime124 | source到`CompiledDebugDrawArtifact`的唯一compiler |
| GPU arena、overlay pass、device loss、render stats | Render10/17 + PERF-MVP-333 | Runtime124定义artifact/receipt合同，不复制renderer owner |
| world/entity/view qualified identity与large-world transform | Runtime24/23 | Runtime124消费，不另造裸ID或空间系统 |
| camera/view/layer/current frame product | Runtime/Editor viewport owner，Editor58 | Runtime124只消费sealed view facts |
| pointer picking、selection、highlight、cancel | Runtime47、Editor59 | Runtime124发布pick representation和generation，不拥有输入路由 |
| transform handle、pivot/space/snap/workplane/numeric transaction | Editor67 | Runtime可提供纯数学kernel；session、policy、undo/redo归Editor |
| plugin/native code admission、lease、quiescence | Runtime lifecycle / Plugins01 | Runtime124 contribution必须接入统一lease，不能发明第二套卸载系统 |
| Navigation/AI/VG领域语义与LOD颜色 | 各domain owner | domain发布typed source；Runtime124拥有共同primitive compiler |

Editor provider 不能继续以最终 `SceneGizmoOverlayExtract` 作为扩展边界。目标边界应是 provider 提交带 qualified producer/category/owner/generation 的 source contribution；Runtime compiler 在已封印的 view facts 下统一验证、过滤、预算、编译和出具 receipt。Editor 的选择/操作事务保持在 Editor，不被 Runtime debug draw service吸收。

## 6. Runtime49 P1 当前裁决

状态只表示本轮 focused source 是否满足旧账目标；Partial 不代表可发布。

| ID | 状态 | 当前源码证据 |
|---|---|---|
| GIZMO-P1-001 | Open | canonical API仍为0个production consumer |
| GIZMO-P1-002 | Open | 六类真实producer仍直接写最终overlay DTO |
| GIZMO-P1-003 | Open | 七组config仍只消费enabled/color |
| GIZMO-P1-004 | Open | generic group仍是raw String；Editor provider ID不是runtime category descriptor |
| GIZMO-P1-005 | Open | asset snapshot仍丢失buffer config，retained恢复独立/default config |
| GIZMO-P1-006 | Partial | `Arc<[GizmoCommand]>`消除clone copy；handle/generation/instance owner/lifetime仍缺 |
| GIZMO-P1-007 | Open | immediate buffer仍无frame seal、late receipt或自动clear |
| GIZMO-P1-008 | Open | retained仍无TTL/remove token/owner detach |
| GIZMO-P1-009 | Open | retained仍无runtime/world/session scope |
| GIZMO-P1-010 | Partial | Editor provider已有注册/toggle/quarantine；无unregister/admission drain/lease/quiescence |
| GIZMO-P1-011 | Open | `SceneGizmoKind`仍是六项closed enum |
| GIZMO-P1-012 | Open | overlay owner仍是bare `EntityId` |
| GIZMO-P1-013 | Open | generic extract仍无view/camera/eye/frame identity |
| GIZMO-P1-014 | Open | 无统一project/user/session/view config store或generation |
| GIZMO-P1-015 | Open | render layer仍未进入view filter或renderer |
| GIZMO-P1-016 | Open | screen scale policy仍未进入compiler/shader |
| GIZMO-P1-017 | Open | 2 px width仍未进入固定LineList pipeline |
| GIZMO-P1-018 | Open | depth bias仍未消费，pipeline bias恒零 |
| GIZMO-P1-019 | Open | `selected`仍不被renderer读取 |
| GIZMO-P1-020 | Open | color policy仍在CPU flatten，style变化重建geometry |
| GIZMO-P1-021 | Open | 无depth-tested/xray/on-top双通道合同 |
| GIZMO-P1-022 | Open | 无dash/dot/joint/cap/AA policy与GPU实现 |
| GIZMO-P1-023 | Open | primitive set仍九类且不可注册扩展 |
| GIZMO-P1-024 | Open | 无2D/screen/text/label/arc/image domain |
| GIZMO-P1-025 | Open | generic command仍不能统一生成wire/icon/fill/pick representation |
| GIZMO-P1-026 | Open | position/vector/color/transform仍无finite admission |
| GIZMO-P1-027 | Open | radius/size/normal/length仍无range/degenerate policy |
| GIZMO-P1-028 | Open | retained Axis仍只变换origin |
| GIZMO-P1-029 | Open | retained Sphere/Circle仍不缩放radius |
| GIZMO-P1-030 | Open | retained AABB仍只变换min/max后重建 |
| GIZMO-P1-031 | Open | Circle normal仍无non-uniform scale合同 |
| GIZMO-P1-032 | Open | source/asset仍无cached bounds |
| GIZMO-P1-033 | Open | 无frustum/distance/occlusion/LOD before expansion |
| GIZMO-P1-034 | Open | Circle/Sphere仍固定32段 |
| GIZMO-P1-035 | Open | stable primitive仍每帧CPU展开 |
| GIZMO-P1-036 | Open | line vertex仍每帧`create_buffer_init` |
| GIZMO-P1-037 | Open | icon仍一项一buffer/一draw |
| GIZMO-P1-038 | Open | overlay仍拆成多个LoadStore pass |
| GIZMO-P1-039 | Open | 无command/primitive/vertex/byte/time/residency hard budget |
| GIZMO-P1-040 | Open | LineStrip与serde owned Vec仍无bounded decode/submit |
| GIZMO-P1-041 | Open | reserve/accounting仍无checked admission与allocation receipt |
| GIZMO-P1-042 | Open | GPU count仍有unchecked `usize as u32` |
| GIZMO-P1-043 | Open | push/extract仍无accepted/dropped/truncated/stale terminal receipt |
| GIZMO-P1-044 | Open | 无producer/category/view source/visible/culled/dropped诊断 |
| GIZMO-P1-045 | Open | extract stats仍只估DTO slice bytes |
| GIZMO-P1-046 | Open | 无build/schema/generation-bound capture/export/replay |
| GIZMO-P1-047 | Open | generic API仍只有caller-private `&mut GizmoBuffer`，无并行writer/barrier |
| GIZMO-P1-048 | Open | provider BTreeMap只局部稳定；canonical merge仍无phase/layer/owner sequence合同 |
| GIZMO-P1-049 | Open | 无source/config/camera/style/GPU generation失效图 |
| GIZMO-P1-050 | Open | Navigation仍每triangle输出三边并复制共享边 |
| GIZMO-P1-051 | Open | AI、VG、Navigation与generic仍有私有shape builders |
| GIZMO-P1-052 | Open | 无multi-camera/stereo/dynamic-resolution/large-world方案 |
| GIZMO-P1-053 | Open | 无headless/shipping/remote principal/strip policy |
| GIZMO-P1-054 | Open | serde command/asset仍无schema version/migration/unknown policy |
| GIZMO-P1-055 | Open | Editor仍无统一category tree/search/reset/persist/per-view status |
| GIZMO-P1-056 | Open | tests仍未走canonical API到真实Runtime/Editor producer和WGPU产品链 |

## 7. Runtime49 P2 当前裁决

| ID | 状态 | 当前源码证据 |
|---|---|---|
| GIZMO-P2-001 | Open | `linestrip`命名仍未hard-cutover为`line_strip` |
| GIZMO-P2-002 | Open | width/depth bias/screen scale单位与范围仍未形成schema合同 |
| GIZMO-P2-003 | Open | group仍接受任意String，无canonical校验 |
| GIZMO-P2-004 | Open | render layer仍是无语义tuple wrapper |
| GIZMO-P2-005 | Open | overlay color仍无linear/sRGB/HDR/premultiply合同 |
| GIZMO-P2-006 | Open | command仍无interned source/callsite/debug marker |
| GIZMO-P2-007 | Open | extract request仍每次持有两个Vec引用 |
| GIZMO-P2-008 | Open | source serde enum与compiled/cook artifact仍未分离 |
| GIZMO-P2-009 | Open | source-string implementation test仍存在 |
| GIZMO-P2-010 | Open | default test仍把字段存在混同consumer policy覆盖 |
| GIZMO-P2-011 | Open | Axis/Sphere/Circle/AABB transform failure仍无回归测试 |
| GIZMO-P2-012 | Open | 无unknown/version/fuzz/size-limit fixture corpus |
| GIZMO-P2-013 | Open | 无1/1k/100k、stable/1%-changed CPU/GPU分布benchmark |
| GIZMO-P2-014 | Open | 无canonical Editor/WGPU/多平台current-source资格证据 |

## 8. 目标架构

```text
Registered DebugDrawSourceDescriptor
  -> DebugDrawFrameWriter(s) / RetainedDebugDrawStore
  -> seal(frame, world_generation, source_generation)
  -> validate + checked accounting + source receipt
  -> bounds/layer/view/LOD visibility
  -> deterministic budget resolver
  -> CompiledDebugDrawArtifact
       geometry_generation
       style_generation
       view_facing_generation
       visual/pick representations
       ranges + bounds + diagnostics
  -> persistent GPU arena / dirty upload
  -> one ordered overlay product pass
  -> terminal frame/category/producer receipts
```

### 8.1 Identity 与生命周期

`DebugDrawSourceDescriptor` 至少包含 qualified producer/category key、owner module/provider generation、domain capability、default config schema、priority、build/shipping/remote policy。Immediate writer 绑定 runtime/world/frame generation，在 barrier seal；late write返回 typed result。Retained entry使用 generational handle，明确 Frame/Duration/Persistent lifetime、remove token、entity/world owner、module detach和last-use retirement。Unregister 必须先关闭admission，再等待writer/callback/compiled/GPU引用清零。

### 8.2 Geometry、style 与 view分代

source command 不是最终 vertex。Compiler 先做finite/range/transform政策与bounds，再生成可复用 geometry artifact；style、selection、depth、line width、screen-size和camera-facing部分独立分代。Circle/Sphere/AABB/Axis必须先冻结local-space和transform政策，unsupported shear或scale必须fail-close而不是静默画错。

### 8.3 Budget 与确定性降级

预算至少覆盖 source items、expanded primitives、CPU bytes/time、GPU vertices/upload/residency、icons/draw/pass，并能按 project/profile/view/category/producer分层。Resolver 以显式 priority、phase、category、owner和stable sequence排序；超限必须确定性 drop/truncate并给 producer receipt，不能依赖 Vec 到达顺序、OOM或整数wrap。

### 8.4 产品与Editor接线

Runtime Navigation先作为runtime consumer，Navigation或AI provider作为Editor/plugin consumer；两者都只提交typed source。Camera/light、VG、其他domain随后hard-cutover，删除私有 common tessellation。Editor settings只拥有用户/viewport policy UI，消费runtime category catalog和真实availability；transform handle操作仍走Editor67 transaction，不把编辑器会话塞入debug draw service。

## 9. 分层重构路线

### M124-0 · Truth、RED tests 与 deletion matrix

- 冻结所有 direct DTO producer、private builder、renderer consumer、tests和删除目标；标明每项最终owner。
- 先写 Axis/Sphere/Circle/AABB、NaN/huge payload、zero production consumer、stable rebuild/buffer-create 的 RED tests/counters。
- child Arc slice完成managed validation与Windows timing前不得标记accepted；它不是后续architecture blocker。

### M124-1 · Qualified descriptor、config 与 source contract

- 引入稳定 qualified producer/category identity和descriptor registry；duplicate、disabled、unavailable、unregister均有typed status。
- 删除所有无consumer的公开config字段，或让width/depth/layer/scale/color真实进入compiler/render合同。
- `SceneGizmoKind` hard-cutover为registered key；不保留长期closed-enum兼容shim。

### M124-2 · Immediate/retained authority

- 建立per-frame chunk writer、deterministic seal/merge与late receipt；避免全局热锁。
- 建立generational retained handle、TTL/remove/world/entity/module owner和retirement。
- 接入统一plugin admission/call/resource lease；实现provider unregister和quiescence测试。

### M124-3 · Geometry correctness 与 bounded compiler

- 先冻结space/transform政策，再修Axis/Sphere/Circle/AABB；补rotation/uniform/non-uniform/negative/shear golden。
- finite/range/degenerate/size-limit checked admission；所有乘加、reserve和draw count checked。
- 生成bounds、adaptive segment和visual/pick representation，共享identity但独立budget。

### M124-4 · View compiler、generation 与 budget

- 接入qualified world/view/camera/eye/frame、layer、frustum、distance、LOD和large-world relative transform。
- geometry/style/view-facing/GPU artifact独立generation；stable frame CPU tessellation为0。
- 完成四级budget、确定性overflow和producer/category/view receipt/diagnostics。

### M124-5 · Persistent GPU product renderer

- 与 Render10/17、`PERF-MVP-333` 一起建立persistent arena、dirty range、icon atlas instances和device-loss rebuild。
- 在保持子层顺序/像素等价的前提下，把overlay attachments收敛到目标pass数量。
- 统计buffer create、upload bytes、draw/pass、resident bytes、GPU timestamp；不以CPU估算代替。

### M124-6 · Domain 与 Editor hard cutover

- Runtime Navigation + 一个Editor/plugin provider先接canonical service并删除对应direct DTO/private tessellation。
- 依次迁移Camera/Light、AI、VG；Navigation改为unique-edge/indexed stable artifact。
- Editor category settings接runtime catalog，提供search/reset/persist/per-view override和真实status。

### M124-7 · 产品资格

- 1/1k/100k与stable/1%-changed记录CPU p50/p95、alloc/RSS、tessellation、upload、buffer/draw/pass与GPU timestamp。
- canonical API -> Runtime/Editor producer -> compiler -> WGPU像素/RenderDoc全链；覆盖multi-view、resize/rebase、device loss、world unload、plugin unload。
- Windows MVP先通过；Linux/macOS按平台资格补齐。所有门禁与MVP证据通过后才允许关闭Runtime49/124账本。

## 10. 资格门当前状态

| Gate | 状态 | 当前判定 |
|---|---|---|
| GZ01 | Fail | canonical API没有Runtime或Editor/plugin production consumer |
| GZ02 | Fail | AI/VG/Nav/Editor仍有private common tessellation |
| GZ03 | Fail | 五项公开config仍无consumer |
| GZ04 | Partial | Editor provider可注册/冲突诊断；runtime category不qualified且无unregister |
| GZ05 | Fail | 无frame seal与late typed result |
| GZ06 | Fail | retained无remove/TTL/world/entity/module lifetime |
| GZ07 | Fail | 无unload writer/callback/artifact lease drain |
| GZ08 | Fail | overlay仍用bare entity/view identity |
| GZ09 | Fail | render layer未消费 |
| GZ10 | Fail | screen-size政策未消费 |
| GZ11 | Fail | fixed LineList不消费width |
| GZ12 | Fail | 无xray/on-top/depth bias政策 |
| GZ13 | Fail | geometry/style/view-facing generation未分离 |
| GZ14 | Fail | Axis rotation错误且无golden |
| GZ15 | Fail | Sphere/Circle scale错误且无政策 |
| GZ16 | Fail | AABB rotation/shear错误且无OBB政策 |
| GZ17 | Fail | NaN/Inf/negative/huge未fail-close |
| GZ18 | Fail | generic visual/pick artifact未统一identity/budget |
| GZ19 | Fail | visibility未发生在CPU展开/GPU上传前 |
| GZ20 | Fail | circle/arc固定段数，无pixel-error hard cap |
| GZ21 | Fail | stable frame仍CPU tessellate |
| GZ22 | Fail | stable frame仍create GPU buffers/full upload |
| GZ23 | Fail | icon buffer/draw随icon一一增长 |
| GZ24 | Fail | 未达到PERF-MVP-333目标pass |
| GZ25 | Fail | 无commands/bytes/vertices/time/residency hard budget |
| GZ26 | Fail | 无确定性overflow与drop receipt |
| GZ27 | Fail | 仍有unchecked sum/cast |
| GZ28 | Fail | diagnostics不含producer/category/view全链统计 |
| GZ29 | Fail | 无build/schema/config/source generation capture/replay |
| GZ30 | Fail | 无规模与stable/change CPU/GPU/RSS分布证据 |
| GZ31 | Fail | Navigation仍按triangle重复边 |
| GZ32 | Fail | AI/VG/Nav未共享canonical compiler |
| GZ33 | Fail | 无multi-camera/stereo/resize/rebase generation资格 |
| GZ34 | Fail | 无BuildSet/shipping/remote principal fail-close |
| GZ35 | Fail | Editor无统一category settings与真实status |
| GZ36 | Fail | canonical WGPU/Editor/RenderDoc/device-loss/unload全链缺失 |

## 11. 验收约束

后续任何实施不得通过保留旧DTO双写、兼容module、`pub use` facade或第二套provider registry来“渐进兼容”。每个milestone需要先明确old caller/deletion matrix，再以hard cutover删除旧路径。性能主张必须同时给出相同功能、相同视图/像素语义下的CPU/GPU/RSS分布和RenderDoc/counter证据；只减少command clone、只通过synthetic test或只证明某帧有像素均不能宣称优于Unreal。

本报告是 review complete、implementation pending。Runtime124 文档完成不等于用户的全工程 review 目标完成，也不改变MVP或父性能任务状态。

## 12. 状态与产出记录

| Milestone | Status | Date | Evidence |
|---|---|---|---|
