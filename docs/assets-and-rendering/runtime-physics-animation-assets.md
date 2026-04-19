---
related_code:
  - zircon_framework/src/physics/mod.rs
  - zircon_framework/src/animation/mod.rs
  - zircon_manager/src/lib.rs
  - zircon_resource/src/marker.rs
  - zircon_resource/src/identity/asset_reference.rs
  - zircon_runtime/src/asset/assets/animation.rs
  - zircon_runtime/src/asset/assets/physics_material.rs
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/asset/importer/service/import_animation_asset.rs
  - zircon_runtime/src/asset/importer/service/import_physics_material.rs
  - zircon_runtime/src/asset/artifact/store.rs
  - zircon_runtime/src/asset/project/manager/asset_kind.rs
  - zircon_runtime/src/asset/editor/manager/reference_analysis.rs
  - zircon_runtime/src/asset/editor/manager/preview_refresh/preview_palette.rs
  - zircon_editor/src/core/editor_event/runtime/execution/common.rs
  - zircon_editor/src/core/host/resource_access.rs
  - zircon_editor/src/tests/editing/asset_workspace.rs
  - zircon_editor/src/tests/editor_event/runtime.rs
implementation_files:
  - zircon_framework/src/physics/mod.rs
  - zircon_framework/src/animation/mod.rs
  - zircon_resource/src/marker.rs
  - zircon_resource/src/identity/asset_reference.rs
  - zircon_runtime/src/asset/assets/animation.rs
  - zircon_runtime/src/asset/assets/physics_material.rs
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/asset/importer/service/import_animation_asset.rs
  - zircon_runtime/src/asset/importer/service/import_physics_material.rs
  - zircon_runtime/src/asset/artifact/store.rs
  - zircon_runtime/src/asset/project/manager/asset_kind.rs
  - zircon_runtime/src/asset/editor/manager/reference_analysis.rs
  - zircon_runtime/src/asset/editor/manager/preview_refresh/preview_palette.rs
  - zircon_editor/src/core/editor_event/runtime/execution/common.rs
  - zircon_editor/src/core/host/resource_access.rs
plan_sources:
  - user: 2026-04-20 继续正在runtime/editor/framework实现完整的物理和动画系统
  - user: 2026-04-20 physics和animation吸收进runtime
  - .codex/plans/Physics  Full Animation Support Plan.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
tests:
  - zircon_runtime/src/asset/tests/assets/animation.rs
  - zircon_runtime/src/asset/tests/assets/physics_material.rs
  - zircon_runtime/src/asset/tests/assets/importer.rs
  - zircon_runtime/src/asset/tests/assets/artifact_store.rs
  - zircon_runtime/src/asset/tests/project/manager.rs
  - zircon_runtime/src/asset/tests/editor/manager.rs
  - zircon_editor/src/tests/editing/asset_workspace.rs
  - zircon_editor/src/tests/editor_event/runtime.rs
  - cargo test -p zircon_runtime --locked --offline --target-dir target/codex-shared-b asset::tests:: -- --nocapture
  - cargo test -p zircon_editor --locked --offline --target-dir target/codex-shared-b asset_kind_filter_event_accepts_physics_and_animation_asset_kinds -- --nocapture
doc_type: module-detail
---

# Runtime Physics And Animation Assets

## Purpose

这一层文档只描述本轮已经真正落地的“runtime 吸收 + asset/editor 链路”部分，不提前宣称完整物理仿真或完整动画求值已经完成。

当前完成的是：

- `zircon_framework` 持有 physics / animation 的中性契约与 DTO
- `zircon_resource` 持有新的资源种类标记
- `zircon_runtime::asset` 持有 physics material 与 animation binary asset 的 source/import/artifact/runtime payload 流程
- `zircon_editor` 资产工作区与事件层已经能识别这些新种类并把它们显示成 canonical `ResourceKind`

## Ownership Split

这批能力遵守固定六层主脊：

- `zircon_framework`
  - `physics::PhysicsMaterialMetadata`、`PhysicsCombineRule`
  - `animation::AnimationTrackPath`、`AnimationParameterValue`
  - 这里只定义共享协议，不做 importer、runtime payload 或 editor workflow
- `zircon_manager`
  - 继续只做 handle / resolver / public-surface 聚合；physics combine rule 也从 framework 复用
- `zircon_runtime`
  - `asset/assets/` 定义 runtime-authoritative physics/animation asset DTO
  - `asset/importer/service/` 负责 source 文件识别与解析
  - `asset/artifact/store.rs` 负责 `library/` artifact 布局与二进制/JSON 持久化
  - `asset/editor/manager/` 负责 catalog、reference graph 与 preview palette
- `zircon_editor`
  - 只消费 `ResourceKind`、catalog/details 和 canonical asset filters，不直接拥有 asset schema

## Asset Kinds And Formats

本轮新增的 `ResourceKind`：

- `PhysicsMaterial`
- `AnimationSkeleton`
- `AnimationClip`
- `AnimationSequence`
- `AnimationGraph`
- `AnimationStateMachine`

权威 source 格式固定为：

- physics material
  - source: `*.physics_material.toml`
  - artifact: `library/physics/materials/*.json`
- animation skeleton
  - source: `*.skeleton.zranim`
  - artifact: `library/animation/skeletons/*.bin`
- animation clip
  - source: `*.clip.zranim`
  - artifact: `library/animation/clips/*.bin`
- animation sequence
  - source: `*.sequence.zranim`
  - artifact: `library/animation/sequences/*.bin`
- animation graph
  - source: `*.graph.zranim`
  - artifact: `library/animation/graphs/*.bin`
- animation state machine
  - source: `*.state_machine.zranim`
  - artifact: `library/animation/state_machines/*.bin`

动画 source 文件不是 TOML/JSON 资产；它们直接是版本化二进制 payload，前面包一层 magic/version envelope，再走 `bincode` 写入和读取。

## Binary Schema Notes

动画 schema 当前已经覆盖：

- skeleton
  - bone 名称、父骨索引、本地 TRS
- clip
  - skeleton 引用、时长、按骨骼分组的 translation/rotation/scale channel
- sequence
  - `EntityPath + ComponentPropertyPath` 绑定树
  - track 通过 shared `AnimationTrackPath` 生成 canonical runtime/editor 路径
- graph
  - clip / blend / output 节点
  - graph parameter 默认值
- state machine
  - entry state、state -> graph 绑定、transition condition

为了兼顾二进制稳定性和 editor/runtime 的共享 DTO，二进制编码没有直接依赖人类可读序列化分支：

- `AnimationChannelValueAsset` 用显式 tag + payload 结构编码，避免 `serde` 的 human-readable enum 方案在 `bincode` 上失效
- `AssetReference` 在 binary 路径走结构化字段，在 human-readable 路径继续保留 legacy locator-only 兼容
- clip / graph / state-machine 内部引用在 binary 文件里都先落到稳定的 binary reference form，再回填到 runtime `AssetReference`

## Reference Graph And Editor Surface

`zircon_runtime::asset::editor::manager::reference_analysis` 现在对新资产种类建立了第一波直接引用图：

- `AnimationClip -> AnimationSkeleton`
- `AnimationGraph -> AnimationClip`（仅 clip node）
- `AnimationStateMachine -> AnimationGraph`
- `AnimationSequence` 当前不直接持有 asset reference，只持有 property path
- `PhysicsMaterial` 当前没有直接 asset dependency

editor 侧已经跟进这些种类：

- `parse_asset_kind_filter(...)` 接受 `PhysicsMaterial`、`AnimationSkeleton`、`AnimationClip`、`AnimationSequence`、`AnimationGraph`、`AnimationStateMachine`
- `AssetWorkspaceState` 能按这些 kind 过滤 catalog
- `resource_access.rs` 的 ready-handle 诊断路径能正确显示这些 canonical kind 名称
- preview palette 已给 physics/animation 资产分配稳定颜色/缩略图类别，避免它们在资产面板里退回 unknown bucket

## Validation Scope

本轮新增的直接验证点集中在三条链：

- schema 和 importer
  - physics material TOML roundtrip
  - animation binary roundtrip
  - `*.physics_material.toml` / `*.sequence.zranim` importer decode
- runtime asset pipeline
  - artifact store 对 physics JSON artifact 和 animation binary artifact 的读写
  - project manager 对 physics/animation source 目录的扫描、meta sidecar 和 `library/animation/**` / `library/physics/materials/**` 落盘
  - editor asset manager 对 physics/animation catalog kind 和 direct reference graph 的投影
- editor authoring surface
  - `AssetWorkspaceState` 对 physics/animation `ResourceKind` 的过滤
  - editor event runtime 对 physics/animation asset kind filter string 的接受和状态投影

还没有完成的层次仍然包括：

- physics backend 驱动、scene rigidbody/collider/joint 运行时
- sequence evaluator / skeletal pose solve / graph runtime / state-machine runtime
- inspector canonical property model 与 sequence editor 的统一 authoring 面
