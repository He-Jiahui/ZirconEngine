# 10-renderer-family 产出记录归档

> 来源：[`10-renderer-family.md`](../10-renderer-family.md) 的 `## 状态与产出记录`。

## 状态与产出记录

| 日期 | 里程碑/切片 | 状态 | 产出 | 验证与证据 | 后续 |
|------|-------------|------|------|------------|------|
| 2026-06-23 | Render index 当前状态总览拆分 | RF-M1/M2 部分完成,RF-M3/RF-M4 未启动 | 从 docs/plans/zircon_runtime/render/index.md 的第 9 节迁入本计划；本行保留 10 Renderer Family 的当前事实，render 总索引不再维护计划级明细。 | 文档重组；本次未改生产代码，render/index.md 只保留状态路由说明。 | 仍未完成：LOD Group、custom renderer registry、RendererCommon 基座；验收缺口：需要 LOD cross-fade、renderer registry 消费端和 batching reason diagnostics |
| 2026-06-15 | RF-M1 RendererCommon baseline | 部分完成: extract 字段和若干公共语义存在,统一 renderer 基座未落地 | Mesh/sprite/particle extract 已能提供 layer、source entity、material、shadow/velocity 相关字段;计划 03/04/05/06 已在 GPUScene、visibility、shadow 和 velocity 中消费这些字段。 | 计划 03/04/05/06 状态表分别记录 source entity、GPUScene instance source、shadow caster filtering、particle/mesh velocity 相关证据。 | 建立 `RendererCommon` 组件/DTO,把 layer、sorting、material override、shadow、motion flags 收束到单一基座。 |
| 2026-06-15 | RF-M2 batching policy execution | 部分完成: GPUScene/indirect 机制存在,renderer 级策略未定稿 | 计划 03 GS-M4 已有 indirect batcher、multi-draw replay 与 diagnostics;但本计划定义的 static/dynamic/instancing 策略组件面仍未落地。 | 计划 03 GS-M4 状态表记录 indirect stats、WGPU replay 与 `cargo check` 证据。 | 补 renderer batching policy、batching reason diagnostics 和 material/renderer override。 |
| 2026-06-15 | RF-M3 LOD Group | 未启动: 仍停留在计划描述 | 当前只具备距离阈值式 LOD 描述,无屏占比、过渡或 LOD Group 资产。 | 本文件 `现状与差距` 明确 LOD 仅为距离阈值选 mesh。 | 实施 LODGroup asset、screen-size evaluation、cross-fade 与 renderer stats。 |
| 2026-06-15 | RF-M4 custom renderer registry | 未启动: 仍缺注册入口 | 新增渲染器类型仍需改内建枚举,无插件/运行时 renderer 注册位。 | 本文件 `现状与差距` 明确 custom renderer 无注册位。 | 与计划 12 trail/projector/terrain 等消费者联动建立 renderer family registry。 |

### 参考实现精读笔记

**`dev/UnrealEngine/.../Engine/Private/PrimitiveSceneProxy.cpp`**

- `FPrimitiveSceneProxyDesc::InitializeFromPrimitiveComponent`(约 :285 起)一次性拷贝组件开关全集:`CastShadow`、`bCastDynamicShadow`、`bCastHiddenShadow`、`bCastShadowAsTwoSided`、`bRenderInMainPass`、`bRenderInDepthPass`、`Mobility`、`TranslucencySortPriority`、`MinDrawDistance`/`CachedMaxDrawDistance`、`ShadowCacheInvalidationBehavior` —— "组件字段 → 渲染快照"单向拷贝即 Zircon 的 extract 复合 `RendererCommon`,字段清单是 `RendererCommon` 的母本(`bCastShadowAsTwoSided` → `TwoSided`;`bShadowIndirectOnly` + `bRenderInMainPass=false` 的组合 → `ShadowsOnly`)。取舍:UE 拷 50+ 个 bool,Zircon 只收敛 9 字段,editor-only/raytracing/foliage 类不进契约。
- proxy 构造列表里 `bCastDynamicShadow(InProxyDesc.bCastDynamicShadow && InProxyDesc.CastShadow && !InProxyDesc.GetShadowIndirectOnly())`(:474)—— 派生开关在构造期一次解析、运行期只读;Zircon 对应:`needs_velocity`/shadow relevance 在 extract→relevance 派生一次,prepare/queue 不再看原始枚举。
- `FPrimitiveSceneProxy::GetViewRelevance(const FSceneView*)`(:816)默认返回空 `FPrimitiveViewRelevance`,派生类按视图填位 —— 印证 relevance 是"每视图位表"而非全局 bool,Zircon 由计划 04 `PrimitiveRelevance` 承接,本计划只供字段。
- 双路径:`DrawStaticElements(FStaticPrimitiveDrawInterface* PDI)`(头文件 :391)静态路径进缓存命令,`GetDynamicMeshElements(..., FMeshElementCollector&)`(:448)每帧动态收集 —— Zircon 对应 `is_static` → 计划 02 `CachedMeshDrawCommands` 与每帧 dynamic 列表的分流;本计划的 `is_static` 失效即 UE `ShadowCacheInvalidationBehavior` 同型的代际驱动。取舍:Zircon 不做虚函数双路径,extract 数据驱动单路径 + 缓存命中。

**`dev/Graphics/.../Runtime/Passes/DrawObjectsPass.cs`**

- `m_FilteringSettings = new FilteringSettings(renderQueueRange, layerMask)`(:92)—— pass 级过滤只有 queue 段 + layer mask 两维,Zircon 等价物即 `RenderQueueValue` 段 + `RenderLayer`(归 09),本计划保证两字段在 `RendererCommon`/`MeshDrawBatchKey` 中可用。
- sort 选择(:204–206):opaque 用 `cameraData.defaultOpaqueSortFlags`、透明用 `SortingCriteria.CommonTransparent`、2D 用 `SortingLayer | RenderQueue | OptimizeStateChanges | CanvasOrder` 组合 —— 排序准则按 phase 配置而非全局唯一;Zircon 由 09 的 `sort_key` 位段一次表达,本计划不造第二套。
- depth priming 时 `m_RenderStateBlock.depthState = new DepthState(false, CompareFunction.Equal)`(:221)—— early-z 后主 pass 关深度写、用等值测试;本计划"帧时序与集成点"第 5 条直接采用该型。

**`dev/Graphics/.../Runtime/Passes/RenderObjectsPass.cs` + `ScriptableRendererFeature.cs`**

- `overrideMaterial`/`overrideMaterialPassIndex`/`overrideShader`(:21–31)与 `SetDepthState(bool, CompareFunction)`(:57)—— URP 的覆写是整材质/整 shader 替换;Zircon 取舍:`MaterialOverrideSet` 只做 per-slot 句柄覆写、不换 shader 模板(正文风险节既定),避免变体爆炸。
- `ScriptableRendererFeature` 的 `public abstract void Create()`(:50)+ `AddRenderPasses(ScriptableRenderer, ref RenderingData)`(:64)双钩子 —— "一次构造 + 每帧注入"的扩展模型;Zircon 映射为 `RendererTypeRuntime` 注册(一次)+ `prepare/queue` 钩子(每帧),且注入只经 `RenderFeatureDescriptor`,无运行时动态加 pass 的旁路。

**`dev/bevy/crates/bevy_sprite/src/sprite.rs`**

- `struct Sprite`(:19)字段 `image: Handle<Image>`、`texture_atlas: Option<TextureAtlas>`、`color`、`flip_x/flip_y`、`custom_size: Option<Vec2>`、`rect: Option<Rect>`、`image_mode: SpriteImageMode` —— 与现 `RenderSpriteSnapshot`(`core/framework/render/sprite/sprite.rs`)逐字段同构,确认 sprite 契约无需重设计,RF-M1 仅追加 `common: RendererCommon` 并删 `render_layer_mask` 散字段。

## 风险与回退

- 动态合批的 CPU 搬运可能负优化:严格阈值 + stats 暴露收益,默认保守,可被 quality profile 关闭。
- 静态合批与 GPU instancing 在同组对象上竞争:互斥优先级固定且可解释,不做运行时自动切换抖动。
- 材质覆写引爆变体数:覆写只换绑定不换 shader 模板,超出参数集的覆写在导入期诊断。
