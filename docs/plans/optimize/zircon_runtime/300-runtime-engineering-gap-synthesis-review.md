---
related_code:
  - zircon_runtime/src/core
  - zircon_runtime/src/scene
  - zircon_runtime/src/asset
  - zircon_runtime/src/graphics
  - zircon_runtime/src/render_graph
  - zircon_runtime/src/rhi.rs
  - zircon_runtime/src/ui
  - zircon_runtime/src/text
  - zircon_runtime/src/script
  - zircon_runtime/src/plugin
  - zircon_runtime/src/input
  - zircon_runtime/src/platform
implementation_files:
  - zircon_runtime/src/core
  - zircon_runtime/src/scene
  - zircon_runtime/src/asset
  - zircon_runtime/src/graphics
  - zircon_runtime/src/render_graph
  - zircon_runtime/src/ui
---

# Runtime 工程化差距综合审查（Review-only）

## 审查结论

当前 Runtime 不是“功能很少”，而是已经拥有大量局部实现、测试和 DTO；主要问题是这些局部能力尚未收敛成单一 authority、可证明的产品调用链和可回收的生命周期。现有报告已覆盖大量专题，本报告只汇总当前源码中最需要优先重构的横向差距，不重复已有专题 finding，也不把静态代码存在误判为功能完成。

**总体判断：** Runtime 可作为研究性 engine foundation，但尚不能作为 Unreal/Fyrox/Godot 级别的可持续生产 Runtime。尤其不能根据现有测试数量、feature 名称、descriptor 或渲染 pass 数量宣称性能优于 Unreal；必须先建立真实 workload、GPU/CPU profile、device-loss、压力和长时间运行资格。

## 证据边界

已读取并交叉核对：`zircon_runtime/src/lib.rs`、`zircon_runtime/src/rhi.rs`、`core`、`scene`、`asset`、`graphics`、`render_graph`、`ui`、`text`、`script`、`plugin`、`input`、`platform` 的模块入口和现有专题报告；已有 `docs/plans/optimize/zircon_runtime/` 与 `coverage.md` 中的 Runtime 01-223 当前源码复核作为背景。当前 checkout 有大量 tracked/untracked 并行修改，以下代码位置若处于 dirty 状态必须在实施前重新取指纹，标记为 `source_recheck_required`。

未执行：Cargo、真实 GPU、Editor 产品运行、device loss、跨进程 DLL unload、压力/soak、RenderDoc/PIX/WPR、跨平台、网络双进程和性能基准。因此本文是 E2/E3 静态审查，不是实现验收。

## P0：先于高级功能的基础重构

### RT-P0-01：生命周期没有统一的 Runtime/Product shutdown authority

- **证据：** `zircon_runtime/src/lib.rs:4-40` 仅暴露模块；模块 descriptor 分散在各 owner。`zircon_runtime/src/ui/module.rs:22-49` 将空的 `UiRuntimeDriver` 作为 Immediate driver 注册，`UiConfig::enabled` 没有进入 admission。现有 Runtime193/158/191/200 报告进一步确认 module、service、task、window、UI、dynamic session 存在平行关闭路径。
- **问题：** 模块激活、服务 teardown、进程级 task/timer、GPU work、foreign callback、window/surface 和动态 DLL session 没有一个能证明“停止新工作 -> 等待 in-flight -> 回收资源 -> 发布 terminal receipt”的统一顺序。局部 `cleanup` 或 Drop 成功不能证明整个 Runtime 已 quiescent。
- **重构：** 建立 `RuntimeLifecycleCoordinator` 与 `RuntimeCompositionSnapshot`。每个 owner 必须拥有 activation generation、call lease、cancel acknowledgement、quiescent barrier、terminal disposition；禁止用 sleep polling、裸 `Arc` 或 Drop 忽略错误替代关闭协议。
- **硬切：** 删除只返回 bool/静态成功的 teardown facade；App、dynamic session、Editor gateway 全部只消费 coordinator receipt。
- **验收：** 任务/线程/GPU submit/foreign callback/窗口销毁/插件卸载同时发生时，注入 panic、超时和 device loss，证明无 late publish、无 use-after-unload、无永久 waiter。

### RT-P0-02：World/Scene/Asset 的数据 authority 不一致

- **证据：** 现有 Runtime187/190/204/205/206/207/215/223 交叉确认 legacy entity map、archetype storage、dynamic JSON、NodeRecord、inspection artifact、asset registry、asset index、resource manager 与 renderer prepared map 并存。`offline_bake_frame` (`zircon_runtime/src/graphics/runtime/offline_bake/offline_bake_frame.rs:8-71`) 直接以 mesh 数量和 directional intensity 推导 probe，并不生成可恢复的 bake artifact。
- **问题：** Scene save/load、Play clone、render extract、asset residency、reflection/inspection 可能各自保留不同字段；`TileMap`、`Prefab`、Sprite2D、Skeleton、Terrain 等已有 DTO 不能证明 canonical document roundtrip。资源加载仍可在调用线程读盘、decode 和 publish，renderer 另有 residency 语义。
- **重构：** 建立 `SceneSchemaRegistry`、`WorldReplacementTransaction`、`ExactAssetTypeCatalog`、`AssetLoadCoordinator` 和 `RenderAssetHandle`。所有数据经过 versioned source -> validated compiled artifact -> generation-qualified runtime install；World、Editor document、renderer extract 只通过稳定 identity 和 delta/receipt 交接。
- **硬切：** 禁止 `World::clone` 作为持久化或 Play 语义；禁止 broad `ResourceKind` 代替 exact type；禁止 source-only 或手工 snapshot 伪造 runtime-ready。
- **验收：** 对所有已声明一等资产做 save/load/rename/reimport/failed migration/rollback/Play restore/renderer extract roundtrip，要求未知字段无损、失败不发布半状态。

### RT-P0-03：Graphics 仍是局部 pass 集合，不是可证明的 frame execution system

- **证据：** `zircon_runtime/src/rhi.rs:2-45` 主要是外部 RHI facade；Runtime223/214/213/189 报告确认 RenderGraph 有 access/version/range 基础，但 compiled graph 缺设备级 state/barrier/wait-signal/queue ownership/completion。GPUScene/visibility 数据存在未消费的 producer，若干高级渲染 executor 仍是 no-op 或 CPU 模拟。
- **问题：** clear/history/readback/init、GPU completion、device generation、resource retirement 和 graph execution 仍分散在图外；“有 shader/descriptor/pass”不等价于真实可见结果。高级能力如 virtual geometry、clustered lighting、IBL、bake 需要区分数据结构、CPU planning、真实 GPU consumer 和视觉正确性。
- **重构：** 以 `FrameExecutionPlan` 为唯一 frame authority，显式声明 resource state、subresource、barrier、queue sync、external ownership、history epoch、completion ticket、retirement fence；renderer、UI、readback、capture 都成为 graph resource/side-effect node。
- **硬切：** 删除 no-op executor 的 `stable/complete` 声明；所有 degraded/fallback 必须有 typed disposition，不得静默 `SkipDraw` 或空结果。
- **验收：** real backend 首帧/稳态/resize/device-loss/recovery；RenderGraph golden 与 GPU capture；1/100/10k draw、material variant、probe、shadow、UI workload profile，记录 P50/P95/P99 和 VRAM 峰值。

## P1：规模与长期可演进性

1. **任务与并发**：Runtime158/223 显示 task DTO、TaskGraph、私有 worker、graphics/navigation pools 未统一；需 `TaskScope + typed result + owner/resource vector + cancellation + join`，并用 Loom/故障/长时间压力验证。
2. **时间、确定性与回放**：Runtime211/210 证明 fixed-step、timer、RNG、replay 仍有多 authority、partial commit 和跨域 consumer 缺失；需 frame/tick/RNG/checkpoint 同一 transaction，支持 deterministic replay 和 rollback。
3. **输入与平台**：Runtime220/191 证明 `InputDriver` 仍可为空、窗口/viewport identity 被压缩；需 WindowRegistry、InputIngressBroker、InputUser、device generation、per-window frame boundary，避免 UI/Gameplay/Camera 各自消费同一事件。
4. **脚本/插件**：Runtime216/199 证明 catalog、native ABI、VM、world bridge、App feature 有多套真相；需 trust/admission、per-call lease、fuel/memory/deadline、generation drain、原子 package publish 和 debug map。
5. **UI/Text**：Runtime200/201/202 证明 `UiRuntimeDriver` 是空 ZST、font/locale/translation/runtime surface 分裂；需 per-window UI session、frame identity、typed binding/effect receipt、font artifact、localized catalog、A11y adapter 和 IME 生命周期。
6. **质量与内存**：Runtime196/65 证明 CPU/GPU/foreign/cache/RSS 无全局 budget；需 `MemoryDomain/MemoryTag/ProductQualityPolicy/EffectiveReceipt`，按设备、viewport、刷新率和 thermal/memory pressure 实测。
7. **缺失领域不能靠相邻底座冒充**：Physics、Audio、Animation、Navigation、Network、AI、Gameplay、Terrain、Water、Cloth、Hair、Destruction、Vegetation、Decal、Weather、Media 等报告均显示相邻结构存在但产品 owner/cook/artifact/runtime consumer 不完整。应按 domain contract 独立进入 catalog、scene、scheduler、artifact、debug 和 save/network 生命周期。

## 参考引擎应吸收的约束

- Unreal：吸收 RHI/RDG 的显式 resource lifetime、GPU scene/streaming 的 producer-consumer 纪律和模块/对象生命周期思路；不复制其兼容性债务。
- Bevy：吸收 ECS schedule、system access/conflict 与 App/Plugin composition 的数据驱动边界；不能把其不支持动态卸载的模型当作 Zircon unload 证明。
- Fyrox：吸收 Rust 资源加载、场景序列化和插件/热重载的 owner 设计；仍需补齐 Zircon 的 ABI、server/headless 和产品资格。
- Godot：吸收初始化层级、反向清理、tool/runtime 分层和扩展生命周期；不能把脚本便利性作为持久化 schema 证据。
- Unity Graphics/`dev/Graphics`：吸收 shader/material/variant/cache 与 GPU resource 的版本化和 profiling 思路；闭源行为只引用本地可读取实现，不做臆测。

## 实施依赖与验收顺序

`M0 current-source freeze -> M1 lifecycle/identity/ABI -> M2 World/Asset persistence -> M3 task/time/input -> M4 Frame/RHI/RDG -> M5 domain providers -> M6 Editor integration -> M7 scale/fault/performance`。在 MVP F0-F5 未完成前，只允许设计和补充 review 证据，不开始高级渲染或领域扩张实现。

## 状态与产出记录

- 2026-09-02：本报告完成 review-only 综合归纳；未修改 Runtime production code，未运行动态验证。
- 状态：`in_progress`；现有并行修改使 Runtime 相关源码实施前必须 `source_recheck_required`。
