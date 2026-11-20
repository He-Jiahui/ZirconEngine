# 同一 UUID 的动画图或骨架重建后，关卡播放器可能继续使用旧编译图

## 1. 玩家会看到什么

在项目中保留一个正在播放动画图的实体。把图的输出从片段 A 换成 B，新建的播放器应当和正在播放的播放器选择同一片段。骨架单独重建时，图中的 `Hand` 掩码也应继续作用在同名骨骼上。这里关注的是先提交删除，再带原 UUID 发布新资产这两个操作。项目中就有可导入的图源、骨架源和保存 UUID 的 `.zmeta`。 [场景如何装配播放器][scene-graph] · [内置导入器][importer] · [项目示例][sample]

## 2. 如何从项目输入触发

1. 为实体装上图播放器和骨架组件，让关卡推进一帧。准备两个常量片段，A 把 `Hand.x` 设为 1，B 把它设为 9。先让图输出 A。
2. 保持关卡与实体存活。提交图源删除，确认旧资源已退出注册表。保存并恢复其侧车，再通过项目源发布接口写入输出 B 的图。对于由运行中 World 装配、且没有磁盘引用指向它的图，这两步可以分别通过 `delete_project_source` 和 `publish_generated_project_source` 完成。 [删除][delete] · [源发布][generate] · [侧车身份][meta]
3. 确认图的 UUID 相同，重建前后的修订均为 `1`，已安装载荷的版本标识不同。保持播放，采样下一帧。再用独立运行时加载同一组资产并首次求值，作为冷启动对照。 [修订生成][revision] · [载荷身份][commit-key] · [关卡姿态读取][pose-read]

## 3. ZE 的缓存为何命中

`evaluate_graph` 获取当前图和骨架的快照，然后查询 `(graph_id, skeleton_id)`。条目保存两项数值修订及已编译的图。删除和新发布分属两个资源提交，新记录会从 `1` 计数。两项修订相等时，执行路径直接调用旧 `cached.graph.evaluate`，其中仍保存旧片段引用和按旧骨架顺序生成的掩码。帧首清理的是参数求值结果。Clip evaluator 订阅资源变化并清理它自己的片段和骨架缓存。持久编译图在另一张表里。 [图缓存][graph-cache] · [逐帧处理][frame-reset] · [Clip 失效逻辑][clip-invalidate] · [编译掩码][mask]

下列数值来自固定基线 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37` 的源码推演，供同一用例在修复前后检验。

| 输入变化 | 正确的 `Hand.x` | 基线推演 |
|---|---:|---:|
| 在原记录内改图，修订从 1 升至 2，输出改为 B | 9 | 9 |
| 删除并以同 UUID 重建图，输出改为 B | 9 | 1，仍选择 A |
| 只重建骨架，交换同父的 `Hand`、`Other` 索引，图应给手骨叠加 9 | 10 | 1，旧掩码指向原索引 |

## 4. Unreal 如何处理在用实例

对照版本见 `Engine/Build/Build.version:2–9`，为 6.0.0、UE5、Changelist 0。UAF 在重编开始时冻结已有图实例，在加载新图元数据后解冻，实例从当前入口重新分配节点。经典动画路径在换骨骼网格后重新建立 RequiredBones 和节点的骨骼缓存。

| UE 源码位置 | 在该位置可以核对的行为 |
|---|---|
| `Engine/Plugins/Experimental/UAF/UAFAnimGraph/Source/UAFAnimGraphUncookedOnly/Private/Graph/AnimNextAnimationGraph_EditorData.cpp:798–842` | `FreezeGraphInstances`、写入编译结果、`ThawGraphInstances` |
| `Engine/Plugins/Experimental/UAF/UAFAnimGraph/Source/UAFAnimGraph/Private/Graph/AnimNextAnimationGraph.cpp:253–274` | 对已登记的活实例调用 Freeze/Thaw |
| `Engine/Plugins/Experimental/UAF/UAFAnimGraph/Source/UAFAnimGraph/Private/Graph/AnimNextGraphInstance.cpp:290–339` | 从当前入口句柄重新分配图节点 |
| `Engine/Source/Runtime/Engine/Private/Components/SkeletalMeshComponent.cpp:3575–3591` | 换网格后将 RequiredBones 标为待更新 |
| `Engine/Source/Runtime/Engine/Private/Animation/AnimInstanceProxy.cpp:1180–1208,1452–1479` | 更新骨骼映射后让图节点重新 `CacheBones` |

## 5. 已有测试具体断言了什么

- UE 的 `Animation.AnimNext.Runtime.Graph.ExecuteLatent` 分配一个实例并执行 Update/Evaluate，断言 9 条消息，其中 `UpdateCount == 1`、`SomeLatentInt32 == 33`、`EvaluateCount == 1`。覆盖单个图实例的一次求值。位置为 `Engine/Plugins/Experimental/UAF/UAFTestSuites/Source/UAFAnimGraphTestSuite/Private/AnimNextAnimGraphTraitGraphTest.cpp:636–662`。
- UE 的 `System_Input_Output` 在设置骨骼网格前后编译 Blueprint，两次检查 `BS_UpToDate`，随后生成 Actor，从 Output 组件读取并断言 `WriteSerialNumber != 0`。位置为 `Engine/Plugins/Experimental/UAF/UAFTestSuites/Source/UAFCQTestSuite/Private/UAFModuleTests.cpp:542–583`。
- UE 的 `Simple Modify` 和 `Simple Delete` 检查文件变化数、相对路径、动作，以及事务提交后的哈希或条目状态。用例操作 PNG。位置为 `Engine/Plugins/Tests/EditorTests/Source/EditorTests/Private/UnrealEd/AutoReimportTests.cpp:235–383`。
- ZE 的图播放器已有 `Hand.x=2.5` 的关卡姿态断言。直接 Clip 的删除重建用例也核对修订为 `1`、姿态从 `1` 变为 `9`。本案新增的变量是持续存活的**编译图**及其骨架掩码。 [关卡图断言][graph-test] · [直接 Clip 的重建断言][clip-test]

## 6. 修复与验收

让每个编译条目同时识别生成它的图快照和骨架快照。修订和载荷身份都匹配时复用，一方发生替换时按当前两份快照编译。保存载荷的弱引用并比较分配身份是一种做法，能保留现有每帧共享和缓存容量上限。 [持久缓存的命中位置][graph-cache]

在 `zircon_plugin_animation_runtime` 的 `physics_tick` 集成测试中使用原来的 Level 和实体，分别覆盖图重建、骨架换序、同记录升版。比较当前播放器和新播放器的最终姿态，并检查实际安装的图或骨架快照。定向运行采用 `native-host`、关闭默认特性和 `test` profile。当前结论来自源码，实测结果将在此 PR 更新。 [动画宿主夹具][fixture] · [图资产构造][fixture-asset]

实现范围是持久编译图的命中判据。相邻草稿分别处理经典 Sprite 纹理缓存、姿态旋转混合和直接 Clip 的尾通知。 [纹理缓存 #180][pr180] · [姿态混合 #147][pr147] · [尾通知 #260][pr260]

[clip-invalidate]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/animation_clip_evaluator.rs#L106-L139>
[clip-test]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/tests/cache_invalidation.rs#L4-L73>
[commit-key]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_resource/src/manager/commit.rs#L939-L987>
[delete]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/pipeline/manager/assets/deletion.rs#L10-L75>
[fixture]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/tests/runtime_helpers.rs#L10-L24>
[fixture-asset]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/tests/animation_assets.rs#L90-L129>
[frame-reset]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/animation_evaluation_pipeline.rs#L323-L342>
[generate]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/pipeline/manager/assets/generated_source.rs#L56-L122>
[graph-cache]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/graph_cache.rs#L31-L76>
[graph-test]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/tests/physics_tick.rs#L377-L431>
[importer]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/importer/ingest/asset_importer.rs#L199-L234>
[mask]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/compiled_graph/compile.rs#L135-L174>
[meta]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/asset/project/manager/load_or_create_meta.rs#L9-L22>
[pose-read]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/level_system.rs#L827-L832>
[pr147]: <https://github.com/He-Jiahui/ZirconEngine/pull/147>
[pr180]: <https://github.com/He-Jiahui/ZirconEngine/pull/180>
[pr260]: <https://github.com/He-Jiahui/ZirconEngine/pull/260>
[revision]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/crates/zr_resource/src/manager/commit.rs#L372-L404>
[sample]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/examples/vampire/assets/animation/vampire_idle.graph.zranim.zmeta#L1-L13>
[scene-graph]: <https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/world/project_io/scene_asset.rs#L210-L268>

## 重建图和骨架后的关卡姿态

图用例通过项目接口删除源，恢复原 UUID 的侧车，再导入输出另一段剪辑的图。骨架用例在资源管理器内分两次提交删除和登记，检查换序后掩码仍作用于同名手骨。两例都在同一运行时让原关卡连续消费第 1、2 帧，再让新建关卡首次消费第 3 帧。读回时核对两关卡使用同一份新载荷，并确认原关卡仍保留第 2 帧的姿态。

| 操作 | 持续播放关卡的 `Hand.x`（修前 → 修后） | 新建关卡的 `Hand.x` |
|---|---:|---:|
| 重新登记动画图 | 1 → 9 | 9 |
| 重建骨架并交换同父骨骼的顺序 | 1 → 10 | 10 |

普通覆写对照在同一关卡将图修订从 1 升到 2，手骨位置从 1 更新为 9。三项回归 1/3 → 3/3。运行环境为 Windows x64/MSVC，`zircon_plugin_animation_runtime` 使用 `native-host` 特性和 `physics_tick` 测试目标。关卡测试使用 SDK 宿主夹具。

缓存条目保存用于编译的图和骨架载荷的弱引用。下次使用条目时，运行时比较两份资源快照的修订和载荷身份，替换任一载荷都会重编对应的图。 [关卡回归](https://github.com/He-Jiahui/ZirconEngine/blob/907146fe13fce32e1c8a2a69ba07694b7f4710ce/zircon_plugins/animation/runtime/tests/cache_invalidation/graph_reload.rs)
