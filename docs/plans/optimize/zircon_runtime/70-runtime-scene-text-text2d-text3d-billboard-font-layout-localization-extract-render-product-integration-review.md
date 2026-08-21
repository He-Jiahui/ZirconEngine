---
title: Runtime Scene Text、Text2D、Text3D、Billboard、Font、Layout、Localization、Extract、Render 与 Product Integration 工程化差距
category: zircon_runtime
report_id: Runtime70
review_date: 2026-08-20
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_runtime/Cargo.toml
  - zircon_runtime/runtime-feature-presets.toml
  - zircon_runtime/src/asset/assets/font.rs
  - zircon_runtime/src/asset/assets/scene
  - zircon_runtime/src/asset/importer/ingest/import_font_asset
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/dynamic_api/session/hud.rs
  - zircon_runtime/src/dynamic_api/session/extract.rs
  - zircon_runtime/src/dynamic_api/session/menu.rs
  - zircon_runtime/src/scene/components/scene
  - zircon_runtime/src/scene/world/bootstrap.rs
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/src/scene/world/project_io
  - zircon_runtime/src/scene/world/property_access
  - zircon_runtime/src/scene/world/records.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/typed_api/fixed_components.rs
  - zircon_runtime/src/text
  - zircon_runtime/src/graphics/scene/scene_renderer/ui
  - zircon_editor/src/core/commands/defaults.rs
  - zircon_editor/src/ui/host/scene_inspection_publication.rs
  - zircon_editor/src/ui/workbench/event/node_kind_id.rs
  - zircon_editor/src/ui/workbench/event/node_kind_from_id.rs
tests:
  - zircon_runtime/src/asset/tests/assets/font.rs
  - zircon_runtime/src/text/glyph_artifact/tests.rs
  - zircon_runtime/src/text/font/database/tests
  - zircon_runtime/src/text/layout/line_break/tests.rs
  - zircon_runtime/src/text/shaping/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests
  - zircon_runtime/src/scene/tests
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_runtime/11b-runtime-text-font-shaping-layout-editing-ime-review.md
  - docs/plans/optimize/zircon_runtime/11c-gpu-ui-renderer-atlas-sdf-batch-clip-submit-review.md
  - docs/plans/optimize/zircon_runtime/43-dynamic-runtime-session-registry-ffi-frame-event-extract-host-request-world-sync-ui-shader-prewarm-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/61-runtime-scene-world-level-registry-lifecycle-project-io-snapshot-clone-serialization-schema-transaction-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/62-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/63-runtime-scene-reflection-type-schema-registry-dynamic-component-property-address-inspection-artifact-subscription-editor-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/64-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/65-runtime-scalability-quality-profile-device-profile-capability-tier-dynamic-resolution-frame-budget-lod-feature-fallback-product-integration-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/33-localization-string-table-culture-translation-import-export-fallback-pseudo-localization-preview-authoring-review.md
  - docs/plans/optimize/zircon_app/06-vampire-roguelite-example-project-asset-script-gameplay-evidence-product-integration-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Classes/ActorFactories/ActorFactoryTextRender.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/TextRenderComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/TextRenderActor.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Components/TextRenderComponent.cpp
  - dev/bevy/crates/bevy_sprite/src/text2d.rs
  - dev/bevy/crates/bevy_sprite_render/src/text2d/mod.rs
  - dev/bevy/crates/bevy_text/src/text.rs
  - dev/Fyrox/fyrox-ui/src/text.rs
  - dev/Fyrox/fyrox-ui/src/formatted_text.rs
  - dev/Fyrox/fyrox-ui/src/formatted_text/run.rs
  - dev/Fyrox/fyrox-ui/src/formatted_text/textwrapper.rs
  - dev/godot/scene/3d/label_3d.h
  - dev/godot/scene/3d/label_3d.cpp
  - dev/godot/editor/scene/3d/gizmos/label_3d_gizmo_plugin.h
  - dev/godot/editor/scene/3d/gizmos/label_3d_gizmo_plugin.cpp
  - dev/Graphics/Tests/SRPTests/Projects/MultipleSRP_Tests/Assets/TextMesh Pro/Shaders/TMP_SDF.shader
  - dev/Graphics/Tests/SRPTests/Projects/MultipleSRP_Tests/Assets/TextMesh Pro/Shaders/TMP_SDF Overlay.shader
  - dev/Graphics/Tests/SRPTests/Projects/MultipleSRP_Tests/Assets/TextMesh Pro/Shaders/TMP_SDF-Surface.shader
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Samples~/Common/TextMesh Pro/Resources/Fonts & Materials/TMP_Node.hlsl
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 70 · Runtime Scene Text、Text2D、Text3D、Billboard、Font、Layout、Localization、Extract、Render 与 Product Integration 工程化差距

## 1. 结论

Zircon的文本底座不是临时空壳。`FontAsset`已经表达family、face、variation、fallback、composite font与render strategy，import/load/acquire链真实存在；`zircon_runtime::text`具有字体匹配、fallback、BiDi、horizontal/vertical shaping、line break、rich text、glyph artifact、SDF/MSDF、bitmap atlas、retry、cache与大量单元测试；GPU UI也有真实atlas upload、SDF与native fallback、clip、pixel framebuffer测试。这些应被保留并收敛为中立的共享文本服务，不能为Scene Text再造第二套font database、shaper或atlas。

但当前产品只有UI文本，没有场景文本。`NodeKind`只有Empty、Camera、Cube、Mesh与五类light；`SceneEntityAsset`、`SceneNode`、`NodeRecord`、fixed component snapshot、project I/O、reflection、property access、render extract和Editor create command都没有Text2D、Text3D、SceneText或Label3D。全仓对`Text2D/Text2d/Text3D/Text3d/Label3D/SceneText/WorldText/BillboardText`的第一方Scene/asset/graphics/editor精确检索为0。一个项目无法创建、保存、重开、选择、检查、脚本修改或渲染世界中的文字。

现有GPU文本不能通过“加一个transform”升级为场景文本。`ScreenSpaceUiTextBatch`是UI私有类型，身份为UI tree/node/source range，几何是`UiFrame`和clip frame；atlas instance只有`screen_rect_px`，shader直接把像素坐标映射到clip space并写`z = 0`，UI/atlas/SDF pipeline均无depth attachment。它没有entity、world transform、view、bounds、visibility、render layer、material、depth、shadow、motion history、billboard、fixed-screen-size或picking合同。

本轮登记0项新增P0、48项P1、12项P2与48项资格门。缺少Scene Text是未实现能力，而当前Editor/Runtime没有把它公开为可用产品，因此不虚构数据损坏或假成功P0。Engine内硬编码`gameplay.hud_text`/`vampire.hud_text`、固定颜色尺寸和按HP/XP/Orbit/Lance/Pulse字符串分类的产品污染，继续由Runtime43的`DYN-P1-051`、`DYN-GATE-036`、App06及开放的Runtime09 UI bridge failure唯一拥有；本篇只要求Scene Text与runtime UI layer分域，不重复累计。

## 2. 审查边界与物理冻结

### 2.1 Owner边界

| 领域 | Canonical owner | Runtime70责任 | 不得重复登记 |
|---|---|---|---|
| Font、shaping、layout、rich text | Runtime11B | Scene source消费中立layout/glyph artifact、generation与typed outcome | 字体cook、global database、fallback、Unicode、IME、rich parser父P0/P1 |
| UI GPU text、atlas、SDF | Runtime11C | 复用atlas/sampling能力但建立独立world pipeline | UI clip、UI draw order、UI atlas upload父问题 |
| Visibility、material、residency | Runtime09B/09C/09D、Runtime64 | Scene Text提供qualified bounds、phase/material/residency需求与receipt | 通用culling、PSO、streaming、device resource父问题 |
| Scene schema、hierarchy、reflection | Runtime61/62/63 | 定义SceneText具体component/schema/field conservation与extract | 通用World transaction、bounds空间、reflection publication父问题 |
| Localization | Editor33 | 消费qualified localized text identity与culture generation | string table/import-export/fallback/pseudo-localization owner |
| Editor authoring | Editor03/05 | SceneText create/inspect/gizmo/picking/preview adapter | 通用undo、selection、inspector customization父问题 |
| Product UI/HUD | Runtime43、App06、Runtime09 failure | 定义overlay UI与world text的组合边界 | 删除Vampire/HUD/menu硬编码与项目UI bridge |

`zircon_runtime::scene`拥有SceneText实例与持久runtime source；`zircon_runtime::text`拥有中立font/shaping/layout service；`zircon_runtime::core::framework::render`只发布SceneText extract DTO；`zircon_runtime::graphics`拥有world text GPU generation与submission；`zircon_editor`拥有authoring document、preview和gizmo。不得新增第四个root package，不得让UI tree成为Scene权威，也不得让graphics私有glyph instance进入Scene持久化。

### 2.2 Zircon物理冻结

本轮聚焦470个Zircon文件，共107,076行、3,757,609 bytes；按相对路径小写、排序去重，以`path|lowercase SHA-256`逐行LF连接且末尾无LF计算，指纹为`c45fae6a5346516d080765015564da290e226e7806972291e4cb99041a0f1088`。入选范围含1,164个Rust test attribute与16个ignored attribute，但这些几乎全部验证共享文本和UI renderer，不包含任何SceneText组件、save/reopen、world-space depth、billboard、picking或产品像素资格。

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Font asset、Scene schema/component/project I/O、Editor入口 | 72 / 16,907 / 621,510 | asset、node kind、record、snapshot、reflection、property、extract、create command逐字段审查 |
| 共享text/font/shaping/layout/raster/atlas | 286 / 63,068 / 2,156,067 | 可复用能力、public/crate-private边界、generation、cache、artifact与测试逐模块审查 |
| UI GPU text与Dynamic HUD consumer | 112 / 27,101 / 980,032 | batch identity、screen rect、shader、depth state、product fallback和测试边界逐调用点审查 |
| 去重合计 | **470 / 107,076 / 3,757,609** | fingerprint如上；1,164 test、16 ignored；33个入选路径dirty |

冻结时33个入选路径dirty，包括Scene project I/O、frame extract、UI render、font importer/database、shaping/layout/rich tests与Editor inspection；结论绑定当前共享working copy。实施前必须重算指纹、精确零搜索、schema字段守恒与shader/pipeline状态。本轮不修改production/tests，不运行Cargo、Editor、GPU capture、pixel、fault、soak或benchmark，符合MVP gate下review-only授权。

### 2.3 参考物理冻结

五类参考共19个文件、9,405行、328,432 bytes，指纹为`155154a89f0d135ef34f9c10058f89730dbee46e4112077f8e5640c3f7c79fd1`。参考只提供职责与资格证据，不要求Zircon复制类名、对象模型或渲染API。

| 参考 | 可采用证据 | 不可机械照搬 |
|---|---|---|
| Unreal | `UTextRenderComponent`是PrimitiveComponent，拥有localized `FText`、Font、Material、alignment、world size、scale、spacing、color；scene proxy参与relevance/occlusion/static-dynamic/RT/PSO，bounds与localization revision均闭环 | UObject、MID缓存、RHI proxy与宏反射原样复制 |
| Bevy | `Text2d`要求Transform/Visibility/Anchor/TextLayout/TextFont/TextColor，布局按change、camera render layer和target scale更新，计算AABB；extract尊重ViewVisibility并输出glyph/background/shadow/decoration sprite | ECS required-component与RenderWorld schedule原样复制 |
| Godot | `Label3D`是GeometryInstance3D，拥有font/layout/language/BiDi、billboard/fixed-size、depth/double-side、alpha/material policy；dirty deferred shaping生成mesh/AABB，Editor gizmo用triangle mesh picking | RenderingServer RID、Variant与surface生成方式原样复制 |
| Fyrox | 本地快照只有UI `Text`/`FormattedText`，含rich run、wrap、measure/arrange、glyph atlas与shadow；没有first-class scene text节点 | 不把参考缺席当目标上限，也不复用UI Widget作为Scene Node |
| Unity Graphics | TMP SDF、Overlay与Surface shader分别表达Transparent GUI depth、`ZTest Always` overlay和lit surface变体，并含stencil/cull/blend/outline/bevel/underlay/perspective参数 | 本地镜像不含完整TextMeshPro runtime/Editor，不能据shader文件宣称完整架构 |

## 3. 可保留的真实底座

### 3.1 Font asset与typed load链

`FontAsset`、`FontMarker`、`ImportedAsset::Font`、importer、artifact cache、`load_font_asset`与`acquire_font_asset`都是真实能力。family member、variable instance、fallback与composite descriptor也足以成为SceneText依赖的起点。重构重点是把字体字节/cook与owner问题交回Runtime11B，并让Scene component引用qualified Font handle，而不是发明`String font_path`或复制字体数据库。

### 3.2 Shaping、layout与glyph artifact

共享text模块已有horizontal/vertical shaping、BiDi、language normalization、line break、rich runs、visual cluster/source mapping、font generation、layout cache和glyph artifact。SceneText应复用同一个service和generation，新增空间布局策略与scene artifact owner；不应从字符逐个生成quad，也不应在renderer中再次shaping。

### 3.3 Bitmap/SDF/MSDF atlas与失败底座

atlas allocator、page residency、dirty upload、retry、persistent slot、render contract和GPU plan有大量测试，UI产品也有真实framebuffer路径。目标是让world text renderer消费共享glyph residency与sampling contract，同时独立拥有world vertex/projection/depth/material pipeline。复用atlas不等于复用`ScreenSpaceUiTextBatch`。

### 3.4 Scene layer与Editor基础

World已经有typed component store、active state、render layer、world transform、mobility、snapshot、project I/O、reflection与Editor command/inspection框架。SceneText应按现有component/schema模式接入这些唯一authority，不能通过dynamic JSON component或产品脚本字符串旁路。

## 4. 当前产品链逐层事实

| 层 | 当前事实 | 工程级缺口 |
|---|---|---|
| Source | FontAsset完整度较高，UI text保存raw/resolved string | 没有SceneText source、localized identity、style/span或spatial policy |
| Scene | NodeKind、SceneEntityAsset、SceneNode/Record支持mesh/sprite/light等 | 无Text2D/Text3D component、snapshot、clone、project I/O、reflection或property path |
| Layout | SharedTextLayoutService、session/cache/glyph artifact存在 | 输入/输出围绕UI style/frame，Scene没有artifact identity、dirty frontier或world bounds |
| Extract | Mesh、sprite、light进入RenderFrameExtract | 无SceneText extract、entity/view/transform/layer/visibility/bounds/generation |
| GPU | UI atlas/SDF/native fallback真实工作 | screen rect、clip-space z=0、无depth/material/history/world projection |
| Editor | create node、inspection、selection/gizmo框架存在 | 无创建、Inspector、gizmo、triangle picking、preview、undo/save/reopen闭环 |
| Product | project UI优先，否则engine menu/HUD fallback | engine按产品component ID和显示字符串拼UI，overlay与world text没有分层合同 |
| Evidence | text/UI单元和framebuffer测试很多 | SceneText精确测试为0，无world pixel、depth、picking、culture reload、scale/fault证据 |

## 5. 新增P0

本轮没有新增P0。当前没有SceneText产品surface、schema或“已支持”能力，因此缺席按P1工程能力差距登记。若后续在component、Editor菜单或capability catalog暴露SceneText而仍没有save/reopen、render或typed failure，应由Runtime61/Editor capability owner升级为假成功或数据丢失P0；不得提前用本篇重复累计。

## 6. 目标架构

| 组件 | 所属 | 责任 |
|---|---|---|
| `SceneTextSource` | Runtime Scene | literal、localized text identity、argument binding、rich span reference与source revision |
| `SceneTextStyleDescriptor` | Runtime text/framework | font handle/family/fallback、size、line/letter spacing、alignment、wrap、language/direction、paint/effect policy |
| `SceneTextSpatialPolicy` | Runtime Scene | Text2D/Text3D模式、world units、anchor、billboard、fixed-screen-size、depth/double-side、distance/LOD policy |
| `SceneTextComponent` | Runtime Scene | source/style/spatial/render policy引用与component generation；不持有GPU对象 |
| `SceneTextLayoutArtifact` | Runtime text | shaped glyphs、lines、source map、local geometry/bounds、font/culture/style generation与typed completeness |
| `SceneTextDeltaExtract` | Scene -> render framework | created/changed/removed、entity/world/view/layer/bounds、artifact/material/effective policy引用 |
| `PreparedSceneTextGeneration` | Graphics | resolved glyph residency、instance ranges、pipeline/material key、device generation、last-good与failure |
| `SceneTextViewProjection` | Graphics | world/billboard/fixed-size投影、current/previous history、clip/depth、多view/XR一致性 |
| `SceneTextSubmissionReceipt` | Graphics/diagnostics | requested/effective mode、font/layout/atlas/material/view generation、culled/fallback/error/cost reason |
| `SceneTextAuthoringAdapter` | Editor | create/inspect/gizmo/pick/preview/localization binding/undo/save/reopen，复用runtime compiler |

Billboard和fixed-screen-size是SceneText的view projection policy；始终置顶的HUD、菜单、字幕与输入控件属于runtime UI layer。两者可共享font/layout/atlas，但不能共享Scene/UI身份、持久化、hit test、accessibility、depth或ordering authority。

## 7. P1差距与重构定义

### 7.1 Source、Scene与Authoring

| ID | 当前差距 | 重构完成定义 |
|---|---|---|
| SCENETEXT-P1-001 | 没有`SceneTextComponent`或任何first-class scene text source | 建立typed component与唯一Scene owner；不存在时capability为Unavailable，不以dynamic JSON或UI command冒充 |
| SCENETEXT-P1-002 | 没有Text2D/Text3D空间模式与坐标合同 | 以typed spatial mode定义Canvas/world plane、basis、anchor、unit、transform继承与非法组合校验 |
| SCENETEXT-P1-003 | Scene只可能保存最终raw string，没有localized text identity | component消费Editor33提供的qualified table/key/namespace/argument generation，literal与localized source显式区分 |
| SCENETEXT-P1-004 | Scene没有Font handle，`FontAsset` render strategy又耦合`UiTextRenderMode` | 建立consumer-neutral FontRenderStrategy与qualified Font handle/dependency；UI/Scene分别解析effective pipeline |
| SCENETEXT-P1-005 | 没有可持久style/span descriptor | 保存font、weight/style/variation、color、outline、line/letter spacing与stable styled source range，不保存GPU glyph |
| SCENETEXT-P1-006 | alignment、wrap width、language、direction、writing mode没有Scene schema | 复用Runtime11B语义类型并建立versioned field validation/default migration，禁止renderer猜默认 |
| SCENETEXT-P1-007 | billboard、fixed-size、depth、double-side、alpha、layer、sort与distance policy无source truth | `SceneTextSpatialPolicy`与render policy分层，requested/effective及unsupported reason进入receipt |
| SCENETEXT-P1-008 | SceneEntityAsset/NodeRecord/snapshot/project I/O没有文本字段 | 建立versioned schema、unknown-field policy、legacy migration和memory/document/disk/reopen字段守恒 |
| SCENETEXT-P1-009 | reflection/property path/change subscription无法读写文本 | 注册stable type/property ID、typed validation、change tick与inspection revision，derived字段只读 |
| SCENETEXT-P1-010 | clone、transaction、prefab、dynamic reload与world replacement不携带文本 | Runtime61/63事务对exact source/style/policy/generation守恒，reload失败保留last-good且不混代 |
| SCENETEXT-P1-011 | Editor没有Create Text、Inspector、gizmo、picking或preview | Editor03/05接入create/undo/redo/save，显示local bounds/anchor/billboard并用真实triangle/pick proxy选择 |
| SCENETEXT-P1-012 | world signage/dialogue没有semantic/accessibility投影政策 | source可声明decorative/semantic、locale与reading order；需要可访问的文本投影到同代UI/a11y surface而非重复字符串 |

### 7.2 Layout、Artifact与Invalidation

| ID | 当前差距 | 重构完成定义 |
|---|---|---|
| SCENETEXT-P1-013 | 共享service的关键session/artifact/render state主要为crate-private UI消费 | 冻结consumer-neutral internal contract；Scene adapter复用唯一service，不公开UI tree DTO或复制shaper |
| SCENETEXT-P1-014 | `ResolvedTextGlyphArtifact`绑定`UiResolvedStyle/UiResolvedTextLayout` | 抽取中立source/style/layout artifact，UI与Scene各自投影；source cluster与font generation保持一致 |
| SCENETEXT-P1-015 | 没有SceneText artifact identity/schema/compiler generation | artifact key包含source/style/font/culture/layout policy/compiler版本，invalid/stale/missing有typed状态 |
| SCENETEXT-P1-016 | 没有按entity/span/font/culture更新的dirty frontier | 文字、style、transform、font reload、culture切换分别标脏必要阶段；steady frame不重shape全部SceneText |
| SCENETEXT-P1-017 | UI pixel frame不能定义world unit、pixel density与camera scale | layout以canonical logical unit输出，再由Text2D/Text3D projection解析world/pixel scale；finite/range/budget校验明确 |
| SCENETEXT-P1-018 | world scaling下没有hinting/raster strategy | 参考Bevy按world transform与target scale选择hinting/SDF/bitmap策略，跨尺度切换有hysteresis和effective receipt |
| SCENETEXT-P1-019 | layout bounds未扩张outline/shadow/effect且不成为Scene spatial bounds | 产出local ink/layout/effect bounds，转换为world/conservative billboard bounds供cull/pick/LOD共享 |
| SCENETEXT-P1-020 | Font lease、glyph demand、culture reload与layout publish没有事务 | 消费Runtime11B/64 generation lease；完整prepare后原子发布，失败保留last-good且不混用旧font新layout |

### 7.3 Scene Extract、World Render与GPU

| ID | 当前差距 | 重构完成定义 |
|---|---|---|
| SCENETEXT-P1-021 | `RenderFrameExtract`没有SceneText数据 | 增加中立`SceneTextDeltaExtract`与stable entity/artifact generation，不把graphics或Editor类型反向泄漏 |
| SCENETEXT-P1-022 | active、render layer、view visibility、mobility和culling链完全缺席 | SceneText与mesh/sprite共享qualified activation/layer/view合同，隐藏实例不shape/prepare/submit昂贵工作 |
| SCENETEXT-P1-023 | 无world transform、camera/view、billboard/fixed-size解析 | 每view解析basis/scale/anchor并携带view generation；camera cut、resize、DPI、orthographic/perspective可区分 |
| SCENETEXT-P1-024 | UI batch身份是tree/node/source range，不能代表Scene entity | 使用qualified World/Entity/Component/Artifact/View generation；移除或重建后旧batch绝不命中新实例 |
| SCENETEXT-P1-025 | glyph atlas shader只消费`screen_rect_px`并写clip-space `z=0` | world vertex path消费local glyph rect、world/view/projection与current/previous transform；UI shader保持独立 |
| SCENETEXT-P1-026 | UI/atlas/SDF pipeline全部无depth attachment | Text3D明确depth test/write/bias/occlusion policy，Text2D与overlay走独立phase；reversed-Z和MSAA一致 |
| SCENETEXT-P1-027 | 无2D canvas sort、opaque/mask/transparent world phase与ordering合同 | 由mode/alpha/material生成phase key；transparency sort、render layer、z/order与stable tie-break可验证 |
| SCENETEXT-P1-028 | 无material domain、blend、cull、double-side、stencil或alpha coverage | 建立restricted text material contract与pipeline key；requested/effective状态不靠shader名字或固定blend猜测 |
| SCENETEXT-P1-029 | 无lighting、shadow、fog、exposure、post-process或unlit政策 | basic unlit/lit与cast/receive shadow、fog/exposure participation显式；高级PBR/effect延后但不能静默误入 |
| SCENETEXT-P1-030 | 无current/previous transform、glyph layout history与motion vector | history key含entity/artifact/view generation；camera cut、first frame、text mutation和reload明确invalidate |
| SCENETEXT-P1-031 | atlas采样没有world perspective derivative、mip与斜视质量合同 | SDF/bitmap按projected scale、derivative、filter与atlas padding选择；极端透视不闪烁/糊成块且成本可测 |
| SCENETEXT-P1-032 | 无multi-view、split-screen、reflection/shadow view或XR策略 | 同一artifact可产生per-view projection/visibility，view mask与glyph residency共享但history不串view |
| SCENETEXT-P1-033 | 无distance fade、screen-size LOD、hysteresis或glyph density budget | projected bounds驱动quality tier与fade，避免每像素/每帧抖动；requested/effective和fallback reason可见 |
| SCENETEXT-P1-034 | Scene picking/gizmo无法获得glyph/layout geometry | CPU保留qualified coarse bounds与可选triangle proxy；Editor picking不读取GPU私有buffer或UI frame |
| SCENETEXT-P1-035 | UI path按batch持有String/Vec并面向screen rect，没有world retained instance/batching | 建立persistent glyph instance arena、dirty range upload、font/atlas/material/phase兼容batch与fence-safe回收 |
| SCENETEXT-P1-036 | 无SceneText GPU residency、device loss、last-good或retirement | prepared generation绑定device/atlas/font/artifact；loss/reload原子重建，old allocation在GPU completion后退役 |

### 7.4 Product、Diagnostics与资格

| ID | 当前差距 | 重构完成定义 |
|---|---|---|
| SCENETEXT-P1-037 | Dynamic HUD按产品component ID与显示字符串拼接UI | Runtime43/App06删除硬编码；SceneText不得吸收HUD fallback，项目UI和world labels通过正式provider消费 |
| SCENETEXT-P1-038 | `current_ui_extract`只选project UI或单一menu/HUD fallback，无layer composition | runtime UI layer stack可组合project HUD/menu/subtitle/debug；SceneText独立进入Scene render并明确合成顺序 |
| SCENETEXT-P1-039 | 无runtime/script/plugin创建与修改SceneText的typed API | 反射、Zr、plugin与native bridge只使用stable component schema/qualified handles，权限、budget和错误一致 |
| SCENETEXT-P1-040 | 无per-entity layout/render completeness诊断 | `SceneTextSubmissionReceipt`关联source/font/culture/layout/atlas/material/view/device generation和fallback/cull reason |
| SCENETEXT-P1-041 | 无capability/quality profile对SceneText requested/effective解析 | Runtime65定义bitmap/SDF/MSDF、shadow、lit、fixed-size、XR与budget tier，unsupported不静默换视觉语义 |
| SCENETEXT-P1-042 | 无每文本字节/span/glyph/line/atlas/world实例预算 | source ingest、shape、layout、glyph demand、GPU instance和per-view work在分配前admit，超限typed且可恢复 |
| SCENETEXT-P1-043 | 没有change-frontier与规模成本合同 | steady frame CPU/GPU工作随dirty/visible glyph增长；不clone所有String/Vec、不全World scan/sort或每帧建buffer |
| SCENETEXT-P1-044 | 无create-save-close-reopen-clone-prefab测试 | 真实Font+localized/literal SceneText经过Editor transaction、disk、reload、Play fork后字段/identity/generation守恒 |
| SCENETEXT-P1-045 | 无world-space pixel/depth/occlusion/billboard/picking测试 | perspective/orthographic、前后遮挡、double-side、billboard、fixed-size、alpha phase与Editor picking有GPU golden |
| SCENETEXT-P1-046 | 无Arabic/CJK/emoji/vertical/BiDi/culture/font reload的Scene测试 | 复用Runtime11B corpus并验证world bounds、source map、fallback、atomic republish和last-good视觉结果 |
| SCENETEXT-P1-047 | 无missing/corrupt font、atlas full、OOM、device loss、reload race与cancel矩阵 | fault测试验证typed outcome、budget释放、old generation退役、无黑帧/错字/跨entity污染 |
| SCENETEXT-P1-048 | 无10K/100K label、长文本、crowd nameplate的capture/soak/参考基线 | 报告shape/layout/extract/upload/draw、CPU/GPU/frame、RSS/VRAM、atlas churn与top offender；同条件比较后才谈优势 |

## 8. P2延后项

| ID | 延后能力 | 前置资格 |
|---|---|---|
| SCENETEXT-P2-001 | Extruded/beveled true 3D glyph mesh与侧面材质 | flat world text source/artifact/bounds/material/depth/picking先闭环 |
| SCENETEXT-P2-002 | 沿Spline/曲面/圆柱/球面的path text | stable glyph layout、local geometry、per-glyph transform与bounds先完成 |
| SCENETEXT-P2-003 | world rich text inline image/widget/interaction hotspot | source range、artifact identity、picking与UI/Scene ownership先稳定 |
| SCENETEXT-P2-004 | animated outline/glow/underlay/bevel与per-span material graph | restricted basic material、PSO、budget与temporal history先完成 |
| SCENETEXT-P2-005 | full PBR、GI、reflection、subsurface/transmission文字材质 | basic lit/unlit、shadow/fog/exposure与Runtime09E/F/H父管线先资格化 |
| SCENETEXT-P2-006 | ray tracing/path tracing geometry与procedural glyph intersection | Runtime28 AS owner、SceneText bounds/material/residency/generation先完成 |
| SCENETEXT-P2-007 | mesh/task shader或compute-generated glyph geometry | RHI capability、传统vertex fallback、persistent instance arena先完成 |
| SCENETEXT-P2-008 | runtime variable-font axis animation与逐glyph deformation | immutable layout artifact、dirty range、history与font generation先完成 |
| SCENETEXT-P2-009 | GPU feedback/direct-storage glyph page streaming与GPU decompression | 普通async glyph demand、budget、last-good、device loss先完成 |
| SCENETEXT-P2-010 | speech bubble/nameplate避让、屏幕拥挤求解与occlusion placement | Scene/UI分域、projected bounds、multi-view和deterministic budget先完成 |
| SCENETEXT-P2-011 | XR depth-aware legibility、foveation补偿与stereo comfort policy | 基础multi-view/XR投影、history、scale与accessibility先完成 |
| SCENETEXT-P2-012 | 超大世界double-precision/rebase下的massive signage优化 | Runtime23空间合同、render-relative transform与SceneText history先完成 |

## 9. 分层实施顺序

### M0 · Capability Truth与合同冻结

- 保持SceneText为Unavailable，直到component/schema/compiler/extract/renderer/Editor consumer同时存在；
- 冻结`SceneTextSource/Style/SpatialPolicy/LayoutArtifact/Extract/PreparedGeneration/Receipt`字段与owner；
- 复核Runtime11B/11C、09B/09C/09D、61/62/63/64/65、Editor03/05/33的唯一依赖，不建立平行manager。

### M1 · Scene Source、Schema与Editor最小闭环

- 新增versioned SceneText component、NodeKind/record/snapshot/project I/O/reflection/property；
- Editor支持create、inspect、undo/redo、save/reopen、gizmo与coarse picking；
- literal与localized identity、Font handle和render policy经过字段守恒测试。

### M2 · 中立Layout Artifact

- 从UI DTO中抽取中立text source/style/layout/glyph artifact合同；
- 建立font/culture/style generation、dirty frontier、typed completeness与local/effect bounds；
- UI保持行为parity，Scene不产生第二套shaping或font database。

### M3 · Scene Extract与World GPU Pipeline

- 增加created/changed/removed extract、qualified entity/view/layer/bounds；
- 建立world/billboard/fixed-size projection、depth/material/phase、persistent instance arena与batch；
- 接入visibility、residency、device generation、last-good和fence retirement。

### M4 · Product、Localization与Failure资格

- Editor33 culture generation驱动原子re-layout/re-publish；
- project UI、subtitle/HUD与SceneText分层合成，删除engine product字符串适配；
- 完成save/reopen、Unicode、pixel/depth/picking、fault/device-loss与10K/100K规模矩阵。

### M5 · Competitive与高级能力

- 先在同场景同画质同平台同硬件下达到correctness、memory、fault和soak门；
- 再按P2选择extrusion、path text、advanced material、RT、XR与GPU streaming；
- 未取得可复核capture和统计结果前，不宣称表现或性能优于Unreal。

## 10. 验收门禁

| Gate | 验收内容 |
|---|---|
| SCENETEXT-G01 | SceneText capability在component/schema/compiler/extract/renderer/Editor provider齐全前保持Unavailable |
| SCENETEXT-G02 | Runtime Scene、shared text、render framework、graphics与Editor owner没有循环或第二authority |
| SCENETEXT-G03 | literal/localized source、style、spatial policy、artifact、prepared generation identity不可混用 |
| SCENETEXT-G04 | Runtime11B/11C、09B/09C/09D、61/62/63/64/65、Editor03/05/33父finding不被重复实现或累计 |
| SCENETEXT-G05 | schema有version、validator、migration、unknown-field与support window |
| SCENETEXT-G06 | memory/document/disk/reopen/clone/Play fork对全部SceneText字段守恒 |
| SCENETEXT-G07 | stable property ID、reflection revision、change tick与derived read-only policy一致 |
| SCENETEXT-G08 | dynamic reload/world replacement失败保留last-good且旧新generation不混合 |
| SCENETEXT-G09 | literal与localized table/key/arguments显式区分，最终字符串不反向成为持久identity |
| SCENETEXT-G10 | Font handle/family/fallback/variation均引用qualified asset generation，无裸source path |
| SCENETEXT-G11 | alignment/wrap/language/direction/writing mode在UI/Scene共享语义但不共享owner DTO |
| SCENETEXT-G12 | invalid UTF-8边界、NaN/Inf size、超限span/glyph/line在分配前typed拒绝 |
| SCENETEXT-G13 | create/inspect/undo/redo/save/reopen/gizmo/picking使用同一component/schema |
| SCENETEXT-G14 | semantic/decorative policy和a11y投影不复制或漂移本地化字符串 |
| SCENETEXT-G15 | Font/culture/style/source变化只使必要artifact和instance dirty |
| SCENETEXT-G16 | steady unchanged SceneText不重新shape、layout或clone完整String/Vec |
| SCENETEXT-G17 | artifact key包含source/style/font/culture/layout/compiler generation且clean build确定性 |
| SCENETEXT-G18 | glyph artifact source cluster、visual order与layout line source map经过Unicode corpus |
| SCENETEXT-G19 | local ink/layout/effect bounds区分明确并覆盖outline/shadow/decoration |
| SCENETEXT-G20 | world/fixed-size/billboard projected bounds在view变化后保守且不无限膨胀 |
| SCENETEXT-G21 | font reload或culture切换完整prepare后原子publish，失败保留同代last-good |
| SCENETEXT-G22 | bitmap/SDF/MSDF/hinting切换有hysteresis、quality policy和effective receipt |
| SCENETEXT-G23 | RenderFrameExtract携带qualified entity/component/artifact/view/layer/bounds generation |
| SCENETEXT-G24 | remove/reuse/reload后旧SceneText batch不能命中新entity或新artifact |
| SCENETEXT-G25 | hidden、inactive、layer不交或culled实例不执行昂贵shape/prepare/submit |
| SCENETEXT-G26 | perspective/orthographic下world、billboard、fixed-size transform与anchor正确 |
| SCENETEXT-G27 | Text3D depth test/write/bias与reversed-Z、MSAA、occlusion结果有GPU golden |
| SCENETEXT-G28 | Text2D、Text3D与overlay UI使用独立phase/ordering，不能靠统一z=0管线 |
| SCENETEXT-G29 | alpha mask/transparent、cull/double-side、stencil/material key与pipeline状态一致 |
| SCENETEXT-G30 | lit/unlit、shadow、fog、exposure requested/effective明确，unsupported不静默变更 |
| SCENETEXT-G31 | current/previous transform/layout history在first frame、mutation、reload、camera cut正确invalidate |
| SCENETEXT-G32 | perspective斜视、远近缩放、mip/filter/atlas padding下无明显闪烁、bleed或糊块 |
| SCENETEXT-G33 | split-screen、多camera、reflection/shadow view与XR不串visibility、scale或history |
| SCENETEXT-G34 | distance LOD/fade/hysteresis按projected bounds工作且不逐帧振荡 |
| SCENETEXT-G35 | Editor coarse bounds/triangle proxy picking不读取GPU私有buffer或UI frame |
| SCENETEXT-G36 | same font/atlas/material/phase的实例进入persistent arena和共享batch，不逐label建buffer/pass |
| SCENETEXT-G37 | dirty range upload、instance slot reuse、fragmentation/overflow与fence retirement可观测 |
| SCENETEXT-G38 | device loss/atlas rebuild/font reload后prepared generation原子替换且old GPU allocation安全退役 |
| SCENETEXT-G39 | Runtime/Plugin/Zr/native API只通过stable component schema与qualified handles修改SceneText |
| SCENETEXT-G40 | Engine source无`gameplay.hud_text`、`vampire.hud_text`或按显示字符串分类的产品规则 |
| SCENETEXT-G41 | project UI layer stack与SceneText render可同时存在，菜单/HUD/subtitle/debug组合有确定顺序 |
| SCENETEXT-G42 | SubmissionReceipt可关联source/font/culture/layout/atlas/material/view/device与cost/fallback/cull reason |
| SCENETEXT-G43 | 每source字节/span/glyph/line/atlas page/world instance/per-view work均有admission与预算释放证据 |
| SCENETEXT-G44 | 真实项目create-save-close-reopen-Play-reload对literal/localized/font/style/policy完全守恒 |
| SCENETEXT-G45 | Arabic/CJK/emoji/vertical/BiDi/fallback/culture/font reload在world pixel与bounds中通过 |
| SCENETEXT-G46 | missing/corrupt font、atlas full、OOM、device loss、reload race、cancel均typed且无跨entity污染 |
| SCENETEXT-G47 | 10K/100K labels与代表性nameplate/signage长时soak无无界cache、allocation、receipt或atlas churn |
| SCENETEXT-G48 | 同场景同画质同硬件报告CPU/GPU/frame/RSS/VRAM/upload/visual parity；未胜出不得宣称优于Unreal |

## 11. 禁止的临时实现

- 禁止把`ScreenSpaceUiTextBatch`加transform后改名为Text3D；
- 禁止让Scene component持有`UiFrame`、UI tree/node ID、GPU buffer、atlas slot或shader string；
- 禁止在renderer中逐字符shape、逐label创建buffer/pass或每帧clone完整String/Vec；
- 禁止用dynamic JSON component、`gameplay.*`字符串或示例产品规则代替first-class SceneText schema；
- 禁止保存最终localized display string来替代table/key/argument identity；
- 禁止把billboard/fixed-size文字塞进HUD overlay来绕过depth、visibility、picking和world bounds；
- 禁止复制font database、fallback、shaper、line breaker、SDF generator或atlas manager；
- 禁止以source-string test、shader包含断言、单帧截图或CPU proxy宣称world text已完成；
- 禁止为通过性能门删除Unicode、fallback、depth、shadow、picking、fault或accessibility语义；
- 禁止在同场景同画质同硬件的correctness/fault/memory/soak证据前宣称优于Unreal。

## 12. 当前状态

| 项目 | 状态 | 证据 |
|---|---|---|
| Runtime70静态审查 | review_complete | 470个Zircon文件、107,076行、3,757,609 bytes；19个参考文件、9,405行 |
| Finding账本 | review_complete | 0 P0 / 48 P1 / 12 P2；48项资格门 |
| SceneText production能力 | unavailable | first-party Scene/asset/graphics/editor精确类型与consumer均为0 |
| 可保留基础 | partial | Font asset/import/load、shared shaping/layout/glyph、UI atlas/SDF/GPU framebuffer真实存在 |
| Production/tests修改 | pending | 本轮未修改production/tests/Cargo，未运行动态验证 |
| Source currentness | recheck_required | baseline HEAD `bea1acf91`，入选范围33个dirty路径；实施前必须重算指纹 |
