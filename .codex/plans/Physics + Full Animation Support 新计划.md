# Physics + Full Animation Support 新计划（按当前目录结构重写）

## Summary
- 这份计划以当前三包 workspace 为准：`zircon_app`、`zircon_runtime`、`zircon_editor`。`physics` 和 `animation` 继续作为 `zircon_runtime` 顶层内建域，不回退到 `extensions`，也不再拆成新的 workspace crate。
- 现有 `scene` 组件、属性路径、资源类型、资产种类、编辑器 binding 已视为已落地底座；新计划的目标是在此基础上补齐真正可运行的 physics runtime、完整 animation runtime、以及 editor authoring 链路。
- 实施顺序固定为：目录归整与契约扩展 -> runtime/scene/asset 落地 -> editor authoring 落地 -> 全链路验收。

## Key Changes
- `zircon_runtime::physics` 先整理成与现有 runtime 域一致的 folder-backed 结构：`module`、`backend`、`runtime`、`tests`。首个后端放在 runtime 内部，目标是 `Jolt`，通过 feature gate 控制；不新增 `zircon_physics_jolt` 之类外部 crate。
- `zircon_runtime::animation` 同样整理为 `module`、`runtime`、`sequence`、`skeleton`、`graph`、`state_machine`、`tests`。当前扁平文件只保留对外入口，具体行为全部沉到子域。
- `zircon_runtime::core::framework::physics` 从“设置 + 默认材质”扩成完整中性契约：backend 状态、逐 world tick 计划、query/contact DTO、rigid body/collider/joint/material 同步接口。`PhysicsSettings` 继续是配置权威来源。
- `zircon_runtime::core::framework::animation` 从“播放设置 + track path”扩成完整动画契约：sequence 应用、skeletal clip 采样、graph 求值、state machine 迁移、parameter 读写、pose 输出。`AnimationTrackPath` 继续作为 editor/runtime 共享标识，不引入第二套动画绑定路径。
- `zircon_runtime::scene` 继续作为 physics/animation ECS 权威层。现有 `RigidBody`、`Collider`、`Joint`、`Animation*Player`、`AnimationSkeleton` 组件和 property-path 体系不重做，只补 runtime 真正需要的字段与读写路径。调度顺序固定为：`PreUpdate` 同步输入/宿主状态，`FixedUpdate` 跑 physics，`Update/LateUpdate` 写 animation parameter 与 sequence override，animation solve 在 `RenderExtract` 前完成。
- 资产层保持当前格式：`.physics_material.toml` 与 `.zranim` 仍是权威文件，不把 `ImportedAsset` 改造成多输出 importer。模型里的骨骼和 clip 通过现有 `ImportModel` 流程派生为 sibling `*.skeleton.zranim` / `*.clip.zranim` 文件，再走当前单输出导入链；不是让 `import_gltf` 直接返回多个资产。
- 派生动画资产必须有稳定重导入规则：再次导入同一模型时，生成的 skeleton/clip 保持稳定 locator/uuid，现有 scene 和 graph 引用不被打断。
- editor 侧复用现有 animation binding 架构，不新增平行动画 payload 家族。`AnimationCommand` 保持为唯一动画 UI payload，但要补齐 timeline、graph、state-machine 所需变体；`binding_dispatch` 继续负责把字符串路径归一到 `AnimationTrackPath`。
- `zircon_editor::core::editor_event` 目前把 `AnimationCommand` 视为 unsupported，这里要改成正式事件链：`binding_normalization` 接收 animation binding，runtime execution 增加 animation 分支，host event 不再停留在测试态。
- workbench 只新增两个正式视图：`editor.animation_sequence` 和 `editor.animation_graph`。`AnimationSequence` 资产打开到 sequence editor；`AnimationGraph` 和 `AnimationStateMachine` 打开到 graph editor；`AnimationClip`、`AnimationSkeleton`、`PhysicsMaterial` 继续走现有 asset browser/details/preview，不单独做第三个动画 view 或 physics material 专用 editor。
- inspector 继续复用现有 draft/property-path 体系。由 inspector 创建轨道时，产出的必须是 runtime 已接受的同一个 `AnimationTrackPath`，不允许再做 inspector-only 字段映射。

## Public APIs / Interfaces
- `PhysicsManager` 增加 world step、query/contact、backend status、scene sync 相关能力；`PhysicsSettings` 保持项目配置入口。
- `AnimationManager` 增加 sequence、clip、graph、state machine、parameter 与 pose 输出能力；`AnimationTrackPath` 保持 editor/runtime 统一路径模型。
- `AnimationCommand` 扩展为完整动画 authoring 命令面，但仍是唯一动画 binding payload。
- `EditorAssetEvent::OpenAsset` 扩展动画资产路由：`.sequence.zranim -> editor.animation_sequence`，`.graph.zranim` 与 `.state_machine.zranim -> editor.animation_graph`。

## Test Plan
- runtime 契约：physics/animation module 注册、resolver 获取、config 持久化、backend unavailable 降级、builtin module 激活顺序。
- scene/runtime：physics 组件序列化 roundtrip、fixed-step 计划与 world 同步、query/contact DTO、sequence property track 应用、缺失 track 报告、skeletal clip 播放、graph/state-machine 参数求值、`RenderExtract` 前 pose 输出。
- asset：`.zranim` 与 physics material 编解码、`ImportModel` 派生 sibling 动画资产、重导入 locator/uuid 稳定性、graph/state-machine 引用分析。
- editor：animation binding 不再报 unsupported、command -> event -> execution 链路可跑通、打开动画资产能进入正确 view、inspector 创建轨道使用 canonical property path、asset browser 对 physics/animation kind 的筛选与预览保持可用。

## Assumptions
- workspace 形态保持 `zircon_app + zircon_runtime + zircon_editor`，本计划不新增顶层 physics backend crate。
- `Jolt` 是本计划唯一必须交付的 physics backend；backend trait 是为了边界稳定，不要求本轮同时做多后端。
- state machine 编辑承载在 graph editor 内，不单独再开一个 workbench view。
- 模型动画导入采用“导入时派生 sibling `.zranim`”方案；底层 importer 继续保持单输出。
