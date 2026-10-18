# ZirconEngine Bevy-Style 自研 ECS 与场景编辑模式计划

## Summary
- 采用你选择的 **Bevy 风格自研路线**：不直接引入 `bevy_ecs`，继续让 `zircon_runtime::scene::World` 作为场景权威存储，但补齐 Bevy 式的 schedule、resource、event、system/plugin 组织方式。
- `bevy_ecs` 和 `bevy_app` 作为结构参考；`bevy_dylib` 只属于 Bevy 动态链接优化，不纳入本次 runtime ECS 设计。
- Editor 场景编辑模式继续保持“编辑器拥有视口交互、运行时只提供中立场景数据”的边界，视觉和交互目标对齐 `ecs.md` 中 `simple_cad.gif` 的层级面板、中央视口、顶部工具条、右侧检查器和底部显示模式控件。

## Key Changes
- 在 `zircon_runtime::scene` 下新增本地 ECS 子系统：`ResourceStore`、`Events<T>`、`SystemStage`/schedule runner、system descriptor、plugin/system registration API。
- 保留现有稳定 `EntityId = u64`、场景序列化、编辑器选择、属性路径和 render extract 外部接口；组件权威仍在当前 `World` 的 typed component storage 中，避免一次性大迁移。
- 将现有 stage 调整为 Bevy 风格顺序：`First -> PreUpdate -> FixedUpdate -> Update -> PostUpdate -> Last -> RenderExtract`；把现有 `LateUpdate` 硬切到 `PostUpdate`，同步更新 runtime hooks、测试和文档，不保留双名兼容层。
- 把现有 transform propagation、active hierarchy、node cache rebuild、render extract 准备逻辑收束为明确的内部 systems，由 schedule 驱动，而不是散落在 spawn/setter 后的隐式调用。
- Editor 新增 scene edit mode projection：从 runtime `World` 派生 hierarchy rows、inspector fields、viewport toolbar state、scene stats 和 HUD，不向 editor 暴露内部 ECS 存储细节。

## Implementation Plan
- Milestone 1: Runtime ECS 基础层  
  建立本地 `scene::ecs` 模块，提供 resource/event API、schedule order、system descriptor、stage runner；更新 `WorldDriver` 让每帧通过 schedule 执行内部 systems 和插件 scene hooks。
- Milestone 2: World 行为收束  
  把层级有效性、active-in-hierarchy、world matrix、node cache、render extract 缓存刷新改为 schedule-driven systems；保留现有 public methods，但它们只标记 dirty state，由 systems 统一刷新。
- Milestone 3: Plugin 与 Hook 对齐  
  将 `SceneRuntimeHookDescriptor.stage` 更新到新 stage 集合；hook order 继续按 `stage -> order -> id` 排序，并接入同一个 stage runner，保证插件行为与内部 systems 顺序可测。
- Milestone 4: Editor 场景编辑模式  
  优化现有 viewport toolbar、hierarchy、inspector 和 overlay projection，使第一屏接近 `ecs.md` 图示：左侧对象树、中央网格/对象/工具条、右侧检查器/性能信息、底部 2D/3D/材质/线框类切换。选择、拖拽、gizmo、undo 仍复用现有 `SceneViewportController` 和 command history。
- Milestone 5: 文档与清理  
  更新 runtime ECS、editor viewport boundary、module docs；删除旧 `LateUpdate` 命名和散落式刷新路径；记录 Bevy 参考点、刻意偏离点和验证命令。

## Test Plan
- Runtime: 覆盖 schedule 顺序、resource/event 生命周期、hook 排序、spawn/despawn、层级 transform、active propagation、render extract、scene roundtrip、dynamic component/property path。
- Editor: 覆盖 hierarchy projection、inspector projection、viewport selection sync、handle drag undo、toolbar state、HUD/stats projection，以及“编辑器覆盖层不污染 runtime 默认 render extract”。
- Validation: 运行 `zircon_runtime`、`zircon_editor`、`zircon_app` 相关测试，再跑 workspace 级验证；若现有 dirty workspace 中有无关失败，记录失败来源，不回退用户已有改动。

## Assumptions
- 本次不引入 `bevy_ecs` 依赖；目标是先把当前 ECS 做成 Bevy 思想下的 Zircon 原生实现。
- 不伪造 GPU/renderer 性能数据；editor 只展示当前能真实采集的对象数、更新耗时、render extract 或 pointer pick 耗时。
- 不改变资产和 editor 对外使用的稳定 `u64 EntityId`。
- 当前工作区已有大量未归属改动，实施时只触碰 ECS、scene viewport、相关测试和文档范围内的文件。
