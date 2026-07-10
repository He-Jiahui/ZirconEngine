# 15-terrain-vegetation 产出记录归档

> 来源：[`15-terrain-vegetation.md`](../15-terrain-vegetation.md) 的 `## 状态与产出记录`。

## 状态与产出记录

| 日期 | 里程碑/切片 | 状态 | 产出 | 验证与证据 | 后续 |
|------|-------------|------|------|------------|------|
| 2026-06-23 | Render index 当前状态总览拆分 | 全部未启动 | 从 docs/plans/zircon_runtime/render/index.md 的第 9 节迁入本计划；本行保留 15 Terrain/Vegetation 的当前事实，render 总索引不再维护计划级明细。 | 文档重组；本次未改生产代码，render/index.md 只保留状态路由说明。 | 仍未完成：terrain renderer/plugin skeleton、editor delta、grass scatter、tree/imposter；验收缺口：需要等 03/04/08/10/13 地基稳定后进入实现和产品场景验收 |
| 2026-06-15 | TV-M1 terrain data plane and renderer plugin skeleton | 未启动: 等待 mesh/GPUScene/texture 地基稳定 | 当前引擎无 terrain renderer、heightfield asset、patch mesh 或 terrain material path。 | 本文件 `现状与差距` 明确无任何 terrain/植被能力。 | 在计划 03/10/13 基础完成后创建 terrain plugin skeleton 与 patch renderer。 |
| 2026-06-15 | TV-M2 editor contract and incremental updates | 未启动: 等待 TV-M1 | 无 terrain editing delta、height/weight map update 或 editor brush contract。 | 本文件将 terrain/vegetation 定位为上层消费者,依赖前置计划。 | 定义编辑器 brush operation、dirty tiles、undo/redo 和 runtime upload。 |
| 2026-06-15 | TV-M3 grass scatter layer | 未启动: 等待 GPU instancing/LOD/texture array | grass scatter、density map、wind animation 与 culling 均未实现。 | 计划 03/10/13 状态表显示 GPUScene/renderer/texture 地基仍有后续验收项。 | 基于 GPUScene instance batches 实现 grass scatter 与 visibility culling。 |
| 2026-06-15 | TV-M4 tree and imposter | 未启动: 等待 LOD Group 与 texture pipeline | tree billboard/imposter、LOD transition、wind 和 shadow integration 未实现。 | 计划 10 RF-M3、计划 13 TX-M3 均未完成。 | 等 LOD Group、array/cubemap/atlas 能力后再实施 tree/imposter。 |

### 参考实现精读笔记

以下符号均为实际读到的代码(`dev/UnrealEngine/Engine/Source/Runtime/Landscape/Private/`):

| 参考符号 | 要点 | Zircon 对应物 | 取舍 |
|------|------|------|------|
| `LandscapeRender.cpp` proxy 构造中的 `LODScreenRatioSquared` 预计算 | 以 `LOD0ScreenSize / CVarStaticMeshLODDistanceScale` 起步,先除 `LOD0Distribution` 再逐级除 `LODDistribution`(均夹 >1.01),逐级平方存表;并存 `LODSettings.LOD0ScreenSizeSquared` / `LOD1ScreenSizeSquared` / `LODOnePlusDistributionScalarSquared` / `LastLODScreenSizeSquared` | `TerrainLodSettings::screen_ratio_squared` 同型除数链 | 不引入 `r.StaticMeshLODDistanceScale` 抵消逻辑(Zircon 无该 CVar);除数链与夹值保留 |
| `FLandscapeRenderSystem::ComputeSectionsLODForView` + `FLandscapeSectionInfo::ComputeLODForView` | 每 view 计算并缓存逐 section 的小数 LOD(`TResourceArray<float>`,按 ViewStateKey 持久或按 view 临时) | prepare 期每 view 的 section 小数 LOD 计算 | 不做 per-view 持久 state map,按帧重算(section 数量级小,缓存复杂度不值) |
| `SectionLODBiasBuffer` / `SectionLODBiasSRV`(`RHICmdList.LockBuffer` 写 float 数组,顶点工厂按 section 索引取值) | 小数 LOD 经 GPU buffer 进顶点着色器驱动 morph | b9 `section_lod` storage buffer | 同型;Zircon 经 graph 声明的 prepare 写入,不裸锁 buffer |
| `LODSettings.VirtualShadowMapInvalidationLimitLOD` 的阈值反插值循环(在 `LODScreenRatioSquared[i]` 区间内线性求小数 LOD) | 屏幕比例平方 → 小数 LOD 的区间反插值即 UE 的"连续 LOD"数学 | `compute_fractional_lod` 的插值公式 | 直接采用 |
| `LandscapeGrass.cpp` `FGrassBuilderBase` / `FAsyncGrassBuilder`(ctor 携带 `SqrtSubsections`/`SubX`/`SubY`/`InHaltonBaseIndex`) | grass 按 component 细分子块异步构建;Halton 与 jitter grid 两条散布路径 | `scatter_chunk` worker 任务 | 只采 jitter grid 路径,Halton 分支不采(分布质量足够,实现简单) |
| jitter grid 路径:`MaxJitter1D = FMath::Clamp<float>(PlacementJitter, 0.0f, .99f) * Div * .5f`,每 cell 两次 `RandomStream.GetFraction()` 求偏移;保留条件 `Weight > AllowedDensityRange.Min && Weight <= Max && Weight >= RandomStream.GetFraction()`;`SampleLandscapeAtLocationLocal` 返回位置 + 权重 | 密度过滤 = 权重与随机数比较;jitter 上限 0.99 防越 cell | 散布算法的 jitter 公式与密度保留判定逐条同型 | `RandomStream` 是整 component 单流、顺序耦合;Zircon 改为 per-cell 独立 `hash64(seed, chunk, cell)`,换取 chunk 并行与增量重建的确定性 |
| `bAlignToSurface` 分支:用相邻 instance 位置 `(PosX1-PosX2) ^ (PosY1-PosY2)` 叉积求法线再构造对齐矩阵 | 贴地姿态来自局部位置差分 | `align_to_surface` 直接用 `TerrainHeightFieldQuery` 法线 | 高度场法线已有解析重建,不需邻居位置缓存 |
| `InstanceBuffer.AllocateInstances` + `SetInstance(InstanceIndex, OutXForm, RandomStream.GetFraction())` 烘焙输出 | 实例集合一次性写入 instance buffer | `Vec<FoliageInstance>` → `GpuScene` instance span(计划 03) | Zircon 无独立 HISM buffer,统一走 GpuScene |

## 风险与回退

- 本计划依赖面最广(03/04/08/10/13):启动条件 = 阶段 B 完成 + 计划 13 TX-M3 落地;在此之前不开工,避免私有旁路。
- speedtree 资产格式导入不做承诺:风模型参数化为通用材质参数,`.st` 导入器列为远期独立项。
- 地形物理(碰撞 heightfield)只暴露查询接口,物理对接归 Physics 计划。
