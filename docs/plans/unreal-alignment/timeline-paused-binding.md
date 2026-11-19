# TL-01 Draft 候选：暂停序列的采样准入未观察世界绑定失效

> 取证说明：UE 的本机路径与文件 SHA256 是用户提供参考树的依据；本轮 D: 审计快照/工作记录仅本地保存，不随本 PR 发布，也不冒充 GitHub 可读证据。UE/ZE 测试均未在本轮执行。

## 1. 差异、路径行号、符号与可达调用

- 状态：**只读源码调查；独立 Draft 问题说明候选，未创建远端 PR，未修改或运行单测/修复**。本文件仅此一问题。源码因果链置信度高（约 0.90）；生产帧结果仍须等价测试实测。
- ZE 本地 HEAD `cb495bb1c952a24c86f4513b8099bc4942c3a87d`；固定 GitHub 可引用 baseline **`e89349ca51b4e9d024c194cd3bdf645bd25b6c37`**。已只读核对这两提交的引擎 crates 无差异，并核对本问题 owner 路径相对 baseline 无未提交差异。以下 ZE 链接均固定 e893，而非本地 cb495。
- UE 本地版本为 **6.0.0 / BranchName UE5**，依据 [Build.version:1–10](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Build/Build.version#L1-L10>)；这是本地源对照，不是 UE 可执行验收。

| ZE 可达层次 | 证据 | 结论 |
|---|---|---|
| 真正注册的帧系统 | [runtime_system.rs:25–60](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/runtime_system.rs#L25-L60>) | `animation.evaluate` 注册到 `PostUpdate`，进入 `tick_animation_world`；非仅测试 helper。 |
| 世界扫描与请求选择 | [tick.rs:60–89](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/tick.rs#L60-L89>)、[parameter_apply.rs:358–417](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/parameter_apply.rs#L358-L417>) | 暂停播放器仅在 `SequenceProjectionRevision` 改变时入队；否则第 394 行直接跳过。 |
| 暂停修订的实际内容 | [parameter_apply.rs:63–67](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/parameter_apply.rs#L63-L67>)、[parameter_apply.rs:385–395](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/parameter_apply.rs#L385-L395>) | 只有 `player_change` 与 `asset_revision`，不含 World 目标目录/绑定修订。 |
| 请求才会加载与应用 | [tick.rs:163–190](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/tick.rs#L163-L190>)、[tick.rs:331–337](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/tick.rs#L331-L337>) | 没有请求便不会调用 `apply_loaded_sequences`。 |
| 正确的下游绑定失效检查 | [sequences.rs:47–82](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/sequences.rs#L47-L82>) | 缓存失效检查位于已入队 sample 的循环内；检查本身无法唤醒被前级跳过的暂停实例。 |
| Runtime 已有明确的缺目标重试契约 | [compiled.rs:46–75](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/animation/sequence/compiled.rs#L46-L75>)、[compiled.rs:118–130](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/animation/sequence/compiled.rs#L118-L130>) | 缺目标保留为 `missing_tracks`；目录修订改变使 `is_current_for` 返回 false，要求重新编译。 |

## 2. 有界实际影响与可复现实例

只限定 **同一 World / 同一 replacement epoch、已成功加载的有效序列、`property_tracks=true`、`playing=false`，且目标首次编译时不存在、随后创建或重命名到可解析路径**。不声称所有动画都受影响，不声称 UE 会自动处理所有外部对象变化。

建议冻结下面的生产入口证据；数值结论目前是源码推导，不是已执行输出：

1. 用现有 [sequence_asset_for_entity:16–46](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/tests/animation_assets.rs#L16-L46>) 构造目标 `LateTarget` 的一秒、30fps 序列；其 `t=0.5` 位移为 `[2,0,0]`。注册为 Ready 资源，保持资源版本不变。
2. 另建 `SequenceClock` 作为播放器 owner，设置暂停、`time_seconds=0.5`；此时 World 没有 `LateTarget`。经 `tick_level_seconds(..., 0.0)` 完成一次正常帧，缺目标编译产物被保留，扫描修订被提交。
3. 通过 `spawn_node` / `rename_node` 创建 `LateTarget`，默认位移 `[0,0,0]`；**不改 `SequenceClock` 的播放器、不重注册序列、不替换 World**。World 的目标目录修订已发生变化。
4. 再经相同真实帧入口 tick。契约期望：同一暂停时间重试新目标，位移变为 `[2,0,0]`，时钟仍为 `0.5`。当前 `playing=false` 且 `revision_new == revision_old`，使 `playing || revision_changed` 为 false，不生成序列 sample；据源码，新目标仍为 `[0,0,0]`。
5. 再增加相同 tick 不会恢复；修改播放器/资产或重建 projection 可触发恢复。播放中实例每帧入队，是必要对照组。

这不是“暂停必须每帧持续写属性”的新需求，而是 runtime **已经宣告的缺目标目录变更重试**被上游缓存遮蔽。现有 [compiled_sequence_retries_missing_target_only_after_topology_catalog_changes:183–217](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/animation/sequence/tests/cases.rs#L183-L217>) 已覆盖下游失效判定；缺的是与真实暂停准入的组合。

## 3. 根因及 UE 的解决机制

ZE 有两个缓存层，却只把数据/播放器修订接到了前级：

- World 的 [scene_binding_catalog_generation:13–17](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/world/compiled_binding/scene_binding_topology.rs#L13-L17>) 明确用于重试原先不存在的目标；[generation.rs:48–67](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/world/compiled_binding/generation.rs#L48-L67>) 独立推进目录修订。名称替换也会 [推进绑定代数:115–135](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/world/typed_api/component_mutation_effects.rs#L115-L135>)。
- `Ref::last_changed` 是 [该组件的 tick:26–49](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/ecs/change_detection/wrappers.rs#L26-L49>)，不是全世界目录修订。新增另一个目标不会自然改变暂停播放器修订。
- [修订提交:280–296](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/tick.rs#L280-L296>) 不等待缺目标变为存在；因此后续稳定 tick 持续命中前级跳过条件。修复下游 `is_current_for` 不足以解决。

UE 的可沿用点是“**绑定失效独立于时间前进**”，不是照搬其 UObject/ECS：

1. [RequestInvalidateBinding:1626–1631](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/MovieScene/Private/MovieSceneSequencePlayer.cpp#L1626-L1631>) 可以只失效绑定，不改播放位置。
2. [FMovieSceneObjectCache::InvalidateInternal:305–327](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/MovieScene/Private/Evaluation/MovieSceneEvaluationState.cpp#L305-L327>) 广播失效；[RegisterObjectCacheEvents:734–739](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/MovieScene/Private/Evaluation/MovieSceneEvaluationState.cpp#L734-L739>) 接到 Linker。
3. [Linker:229–234](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/MovieScene/Private/EntitySystem/MovieSceneEntitySystemLinker.cpp#L229-L234>) 转入 [FInstanceRegistry:102–115](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/MovieScene/Public/EntitySystem/MovieSceneInstanceRegistry.h#L102-L115>) 的按实例失效集合。
4. [Runner:1074–1086](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/MovieScene/Private/EntitySystem/MovieSceneEntitySystemRunner.cpp#L1074-L1086>) 与 [Runner:1120–1127](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/MovieScene/Private/EntitySystem/MovieSceneEntitySystemRunner.cpp#L1120-L1127>) 把绑定失效作为必须执行实例化阶段的独立原因；[UnlinkStaleObjectBindings:22–62](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/MovieScene/Private/EntitySystem/MovieSceneEntityInstantiatorSystem.cpp#L22-L62>) 标记重链接。
5. 编辑器 [FSequencer::ForceEvaluate:3826–3829](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Editor/Sequencer/Private/Sequencer.cpp#L3826-L3829>) 可按当前时间重评估；[Tick:1309–1315](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Editor/Sequencer/Private/Sequencer.cpp#L1309-L1315>) 的脏评估也不要求 Playing。这里只据源码确认机制，不声称运行过 UE。

UE 文件 SHA256（仅短符号/路径说明，无 UE 实现段落复制）：

| 文件 | SHA256 |
|---|---|
| [MovieSceneSequencePlayer.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/MovieScene/Private/MovieSceneSequencePlayer.cpp>) | `D64AF5EC39FA89C0597521E63E4EFC453FB9ADF2D4C76F5009EF9EF7E28860C0` |
| [MovieSceneEvaluationState.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/MovieScene/Private/Evaluation/MovieSceneEvaluationState.cpp>) | `FDB41A8A50B95994512A71E3D334D10CAA7E86063D76CD279EF94B2976163840` |
| [MovieSceneEntitySystemLinker.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/MovieScene/Private/EntitySystem/MovieSceneEntitySystemLinker.cpp>) | `4D08D68F2A093DF2E151D2A931759AED32CE59CB1EB710A2FF3E32225B2DA637` |
| [MovieSceneInstanceRegistry.h](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/MovieScene/Public/EntitySystem/MovieSceneInstanceRegistry.h>) | `792FA8626FA3C297631C93AABEA73EBB23F721D74743E682BCEB6F234E5826B5` |
| [MovieSceneEntitySystemRunner.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/MovieScene/Private/EntitySystem/MovieSceneEntitySystemRunner.cpp>) | `E83089C9092A35E37CA7D8FD420F673EB81C747388089B49FF1C1FD8E2A11D8D` |
| [MovieSceneEntityInstantiatorSystem.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/MovieScene/Private/EntitySystem/MovieSceneEntityInstantiatorSystem.cpp>) | `B0816B3D96D2A684B90E79EDC0A505805A8D8721B9CAF716164C7573E1E60787` |
| [Sequencer.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Editor/Sequencer/Private/Sequencer.cpp>) | `BD4DF7C63861A7B7FB37ACA17B9DF6272E8F8748E45E3A32B089D681658C2127` |

## 4. Rust 沿用 UE 逻辑的方式

- 在决定跳过暂停序列**之前**比较该序列依赖的 World 绑定失效凭据；变化时在原 `time_seconds` 生成一次重采样请求。复用现有 compile → `is_current_for` → apply，不引入新的蒙太奇、镜头、嵌套序列或时间控制系统。
- 最小正确方案可将 World 的**专用目录修订 token**纳入 `SequenceProjectionRevision`；不要拿任意属性写入都会推进的粗粒度 World generation 代替，否则会破坏暂停静默优化。
- 目录 token 当前是 runtime 内部只读 API，插件不能直接调用其 `pub(crate)` 方法。需由 runtime owner 提供受限只读凭据接口（或等价的 runtime-owned 失效摘要），再由插件比较；不是把 World 内部结构移入插件。
- 失效请求与现有 revision stage 一起提交；保留 deferred entity 不提交、World owner 变化重置和 replacement epoch 防旧世界写入逻辑。稳定的后续帧应回到零请求。
- 可以先采用一次目录变化唤醒所有暂停序列的保守正确策略，后续若做细粒度依赖优化须单独证明；本问题不夹带性能重构。

## 5. ECS、ABI 与数据差异

- UE 以 GUID + instance handle、对象缓存事件、实体重链接实现；ZE 以 `EntityId`、组件变更 tick、不可变 source IR、已编译属性写入器实现。不复制 UObject、模板评估器或 UE 私有数据布局。
- 本问题不要求给持久化 [AnimationSequenceAsset:68–76](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/framework/animation/asset/sequence.rs#L68-L76>) 或 [AnimationSequencePlayerComponent:25–32](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/components/scene/animation.rs#L25-L32>) 增加字段；不变更序列二进制版本、Scene DTO、native/cdylib ABI 或资源句柄身份。
- 失效凭据只留在 World owner 及插件内部 projection。若扩大 Rust 可见性，必须审查 API 名称/语义；不得跨稳定 ABI 传 World、引用或 Rust trait object。
- 编译器、路径/稳定 ID 优先级、缺轨诊断与属性写入器保持现状；这里修的是“让已有正确失效检查可达”，不是重写绑定解析规则。

## 6. 具体 ZE 修改草案、测试映射与可行门

**顺序：独立问题 Draft → 等价单测 RED → 修复评估可行门 → 最小修复/同测试 GREEN → 真实产品验收。此次停在调查，不执行这些步骤。**

建议最小 owner 改动：

1. Runtime 绑定 owner 给动画插件提供专用、同 World 生命周期的只读目录修订凭据；现有低层缺目标重试测试必须保持成立。
2. 在 [SequenceProjectionRevision:63–67](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/parameter_apply.rs#L63-L67>) 添加 transient 凭据，并在 [scan_sequence_players:385–395](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/parameter_apply.rs#L385-L395>) 的跳过条件前取值；保持时间不推进。
3. 单元用例落到已有 [parameter_apply 测试文件](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/tests/parameter_apply.rs>)；真实 tick 回归落到已有 [cache_invalidation 测试文件](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/tests/cache_invalidation.rs>)。后者由 [physics_tick:10–11](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/tests/physics_tick.rs#L10-L11>) 包含，无需新增宽泛 harness。
4. 不变更 UI、编译源 IR、轨道数据和 Step 边界语义；不将此问题扩大到恢复动画、持续覆写暂停属性、事件分发或蒙太奇。

测试迁移/落点表（严格区分已有 UE 自动化与源码推导）：

| 来源及已有测试 | 输入与已有期望 | ZE 现有落点 / 本问题用途 |
|---|---|---|
| **已有 UE 自动化，但不是本缺陷等价用例**：[FMovieScenePlaybackCapabilitiesInvalidateTest / System.Engine.Sequencer.Capabilities.Invalidate:257–304](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/MovieScene/Private/Tests/MovieScenePlaybackCapabilitiesTests.cpp#L257-L304>) | 创建可失效 capability，执行一次 `InvalidateCachedData`，其计数从 0 到 1；覆盖 inline/raw/shared。 | 证明实际存在失效机制测试，不冒充“暂停缺目标后出现”的迁移。ZE 机制级控制可落到上述 projection 单元，但不是逐项迁移该容器测试。 |
| **已有 ZE 基础测试**：[compiled_sequence_retries_missing_target_only_after_topology_catalog_changes:183–217](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/animation/sequence/tests/cases.rs#L183-L217>) | 缺目标产物初始 current；创建节点后不再 current。 | 作为 unchanged 低层控制保留，不宣称已覆盖前级扫描和生产 tick。 |
| **已有 ZE 生产入口控制**：[level_tick_applies_loaded_animation_sequences_to_world_properties:245–323](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/tests/physics_tick.rs#L245-L323>) | 播放中序列在 0.5 秒写位移 2；暂停/速率/循环影响 continuous-frame demand。 | 复用同一个 runtime、Ready 资源注册及 tick 入口；新用例必须改为“暂停 + 目标后出现”，不得只再调用编译 helper。 |
| **源码推导，非 UE 测试迁移**：本文件第 2 节 | 首帧缺目标；目录改变后同时间请求数增加一次，新目标位移为 2、时钟仍 0.5；再 tick 请求数不增加。当前源码预测不增加且位移仍 0。 | 分别新增 projection 与真实 tick 回归，并加播放中恢复对照；精确测试名可为 `paused_sequence_retries_late_target_once_after_binding_catalog_change`。 |

本缺陷的完全对应 UE 自动化未找到。已搜索范围：MovieScene 的 Private/Tests（含 Capabilities、Compiler、TrackInstancePropertyBindings）、MovieSceneTracks 的 Private/Tests（含 PartialEvaluation、CompiledDataManager、Decomposer）、Sequencer 源内自动化宏，以及 LevelSequence 的测试文件名；未把不等价的 capability/属性写入测试包装成等价单测。相关 UE 测试文件 SHA256：

- [MovieScenePlaybackCapabilitiesTests.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/MovieScene/Private/Tests/MovieScenePlaybackCapabilitiesTests.cpp>)：`986913870DA7B30C8AC792D2D01E96FD9D77AA76B9D82ED2009A11D4104FC0F9`。
- [MovieScenePartialEvaluationTests.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/MovieSceneTracks/Private/Tests/MovieScenePartialEvaluationTests.cpp>)：`8120ECC65F301E1499CD86BEA29B7F523AB7B557841FD8CC175AF9D9F0EEA1DB`；该用例针对分阶段评估/子序列存活，不是本缺陷等价用例。

可行门依赖：先明确 runtime token API/owner 与 paused binding retry 的验收口径；然后在获授权的实现工作树，以相同冻结用例证明 RED，确认 `zircon_plugin_animation_runtime` 的 `native-host`、默认 features 关闭、`--lib` 定向单元与 `--test physics_tick` 定向集成可编译执行。复用既有 [Cargo 配置:8–22](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/Cargo.toml#L8-L22>)，不推断现有所有测试均可编译，不绕过实际编译依赖。尤其 [runtime_helpers.rs:109–144](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/tests/runtime_helpers.rs#L109-L144>) 会强制物理路径为 `D:/cargo-targets/zircon-local/win32/<task>/target` 与同级 `build`，smoke 位于 `build/fixtures` 下，且 `TEMP/TMP/TMPDIR` 全等于 smoke；它读取 `CARGO_TARGET_DIR`、`CARGO_BUILD_BUILD_DIR`、`ZIRCON_RENDERING_SMOKE_OUTPUT_DIR`，并校验测试二进制物理位置。因此“普通 Cargo 加 filter 即能跑”尚未成立，环境/目录准备是实在的前置门，禁止为通过而放松断言。构建目录、范围声明、真实 CI/产品门由集成方按当前授权执行，不能用本调查替代。

去重：已读取 [近期 PR 快照](<D:/CodexData/worktrees/ze-ue-audit-20261117/reports/ue-module-audit-20261117/github-pr-snapshot.json>) 全 15 条（#131–#145）；SHA256 `B374EE7808D92536D2680A200DE47479F5A458015F568562465B209C0AB0EF30`，文件 UTC 修改时间 `2026-11-17T10:08:52.8827817Z`。标题/body/head 未发现 animation/timeline/sequence/paused/binding 同题，物理父局部写回、渲染仿射、glTF 和旧流程 PR 均非此根因。**仅此快照内无重复，不代表查过完整历史或并发新 PR**；发布前仍需集成方刷新去重。另已核对旧 [08c-animation-runtime-review:192–196](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md#L192-L196>)：其 P1-16 是逐帧加载/解释、统一 clock、binding generation 等宽泛改造背景，未给出本例“暂停准入只观察 player/asset，遮蔽已有目录失效检查”的具体条件/回归。本候选披露该背景，只报这个可冻结的实现错误，不重报性能或高级缺失。未访问远端、未创建 Draft。

## 7. 第一次帧级回归：编译成功，但临时目录检查挡在 Core 之前

本轮 tests-only `09cd9a059363a5e9860f613c9703b39afec21d03`（[六例独立集成测试](https://github.com/He-Jiahui/ZirconEngine/blob/09cd9a059363a5e9860f613c9703b39afec21d03/zircon_plugins/animation/runtime/tests/sequence_tick.rs#L49-L100)）在 Windows x64/MSVC 1.94.1、插件包默认关闭、无额外包级 features、`test` profile 独立编成。测试代码使用明确的现有 Tasks/Time/Foundation/Asset/Scene 与动画 descriptor、[产品实际采用的 portable 注册](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_app/src/web/mod.rs#L23-L34)、真实资源管理器和 Level；它不走旧 D11 SDK TestRuntime，不放松其 descriptor 检查，也不代表 wasm 下载、桌面插件 catalog 或整个产品启动通过。既有 [#216](https://github.com/He-Jiahui/ZirconEngine/pull/216)、[#217](https://github.com/He-Jiahui/ZirconEngine/pull/217) 保持独立。

同一编译产物 SHA-256 `217c03a06605eee4b60d574ce2a92616b258cfb32333fc7be07bbda4e72f0f3b` 的 6 个 `sequence_tick_` 用例实际执行，0 过 / 6 失败：**每个都先在隔离配置的临时目录字面等价检查停止**，没有一个进入 Core 或第一次动画 sample。没有把失败退出码当作 #165 行为证据。拟分别固定迟生、改名、共享资源及繁忙的另一条轨道；对照原有绑定、正播放和稳定暂停的独立写入不得被覆盖。泊在播放器 `playing=false`，不是让 World 虚拟时钟暂停后依然运行回调。

修正原调查 §4 的一个候选：不能把全局目录代直接加给所有暂停玩家——一个无关实体出现就可能把已经绑定、被编辑的值重新覆盖。原下游 compile 的 `is_current_for` 有自己的失效域；一个编译产物可能被多个玩家共享，且当前 cache 会随本帧有请求的 asset 集合清理，故两者均需对照。UE `System.Engine.Sequencer.Capabilities.Invalidate`、`System.Engine.Sequencer.PartialEvaluation` 分别覆盖失效通知和明确排队后的分段评估，**不是**普通暂停运行时因新 Actor 自动求值的证明；Editor 的 ForceEvaluate 与空闲 Runtime 的排队门也不同。

## 8. [实测失败] 实际 PostUpdate 的条件与取样

修复的是夹具临时目录尾部的表示：重新确认系统返回的目录与显式 TEMP 是同一无别名的物理目录，配置必须位于其下。自启动真实 Core 起，全部行为断言与上次相同。此次六例均进入真实 Level 帧，带非零 delta；`playing=false` 的播放器时钟仍是 `0.5`，资源和 World 身份保持。初次已经存在的目标可采样到位移 X=2；正播放者在目标出现后也能写入，另一个繁忙资产不应使已绑定的静止轨覆写外部设置的 X=17。新增和改名后目标已能按路径解析，却没有入队，X 仍为 0（应为 2）。两位暂停玩家共用资产时，新帧仍只含另一个正播放实例的请求。

本次 Git 输入 `06fbf4bdff659009063a21fe08394f171a1b3dff`；[实际回归用例](https://github.com/He-Jiahui/ZirconEngine/blob/06fbf4bdff659009063a21fe08394f171a1b3dff/zircon_plugins/animation/runtime/tests/sequence_tick.rs)，该输入的测试产物 SHA-256 `bdf5bf859ff3a0fd1ae42ce24e972214d5aa3c5843cf4a5635b1ad427b1721cf`。独立集成测试实际运行 **3 过 / 3 败**。

| 冻结的用例 | 本次结果 |
|---|---|
| `sequence_tick_paused_keeps_bound_target_quiet` | 通过 |
| `sequence_tick_paused_retries_renamed_target` | 失败 |
| `sequence_tick_paused_retries_shared_asset_after_eviction` | 失败 |
| `sequence_tick_paused_retries_spawned_target` | 失败 |
| `sequence_tick_playing_retries_spawned_target` | 通过 |
| `sequence_tick_samples_initial_paused_target` | 通过 |

| 实际阶段（日志标记） | 本帧玩家 | 本帧序列请求 |
|---|---:|---:|
| `initial_bound` | 1 | 1 |
| `after_rename` | 1 | 0 |
| `shared_retries_after_eviction` | 3 | 1 |
| `after_spawn` | 1 | 0 |

这是 Windows 集成层经产品 portable 注册调用同一 PostUpdate 求值器、使用 native 资产管理器的结果；未跑真正的 Web 加载、原生插件扫描、桌面产品或 UE 原测。

## 9. [实测失败] 实际 PostUpdate 的条件与取样

前六例及其生产入口保持不变。增加的混合轨用正常 Ready 资源、独立的真实目标和暂停播放器，区分缺失轨第一次变得可写与已有轨仍须保持当前用户编辑。该控制限定 ZE 的不引入回归边界，不是断言所有 UE 轨道均如此。

本次 Git 输入 `10b017713921ef346ddef9735ff27ccd4223dda6`；[实际回归用例](https://github.com/He-Jiahui/ZirconEngine/blob/10b017713921ef346ddef9735ff27ccd4223dda6/zircon_plugins/animation/runtime/tests/sequence_tick.rs)，该输入的测试产物 SHA-256 `08d56301497eb05385e178482fc95d1fd83353ae13987b245ca7fe6327b7b493`。独立集成测试实际运行 **4 过 / 4 败**。

| 冻结的用例 | 本次结果 |
|---|---|
| `sequence_tick_mixed_explicit_seek_and_asset_revision_resample_all_tracks` | 通过 |
| `sequence_tick_mixed_late_bindings_preserve_independent_edits` | 失败 |
| `sequence_tick_paused_keeps_bound_target_quiet` | 通过 |
| `sequence_tick_paused_retries_renamed_target` | 失败 |
| `sequence_tick_paused_retries_shared_asset_after_eviction` | 失败 |
| `sequence_tick_paused_retries_spawned_target` | 失败 |
| `sequence_tick_playing_retries_spawned_target` | 通过 |
| `sequence_tick_samples_initial_paused_target` | 通过 |

| 实际阶段（日志标记） | 本帧玩家 | 本帧序列请求 |
|---|---:|---:|
| `initial_bound` | 1 | 1 |
| `after_rename` | 1 | 0 |
| `shared_retries_after_eviction` | 3 | 1 |
| `after_spawn` | 1 | 0 |

两份私有候选均未提交到生产。第一版不能保证扫描时记下的修订与随后取得的真实载荷成对；第二版在同内容、同修订的资源卸载又重驻留时会丢掉暂停实例仍待解决的绑定。后一种情况可以在两帧之间按正常资源生命周期发生，不要求制造竞态。这是针对候选的源码反证，不是本次用例运行过候选。下一道门是同一真实资源最后一个租约释放、按同修订重新装入并证实 active 身份变化，再新增目标；不应为此每帧重写已有轨或默认扩大热更新范围。

这是 Windows 集成层经产品 portable 注册调用同一 PostUpdate 求值器、使用 native 资产管理器的结果；未跑真正的 Web 加载、原生插件扫描、桌面产品或 UE 原测。
