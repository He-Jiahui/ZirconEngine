# 19-gpu-capability-optimizations 产出记录归档

> 来源：[`19-gpu-capability-optimizations.md`](../19-gpu-capability-optimizations.md) 的 `## 状态与产出记录`。

## 状态与产出记录

| 日期 | 里程碑/切片 | 状态 | 产出 | 验证与证据 | 后续 |
|------|-------------|------|------|------------|------|
| 2026-06-23 | Render index 当前状态总览拆分 | GC-M2 部分完成,GC-M1/3/4 未启动 | 从 docs/plans/zircon_runtime/render/index.md 的第 9 节迁入本计划；本行保留 19 GPU Capability/Bandwidth 的当前事实，render 总索引不再维护计划级明细。 | 文档重组；本次未改生产代码，render/index.md 只保留状态路由说明。 | 仍未完成：capability surface、streaming/separate translucency、cache/quality improvements；验收缺口：需要 feature request/gate 对齐、indirect-count/bindless fallback、bandwidth/cache/product parity tests |
| 2026-06-15 | GC-M1 capability surface and compute base | 未启动: 现状审计已完成,能力请求/gate 修复未实施 | 文档已盘点 binding array gate 探测但未请求、subgroup/timestamp/statistics 能力位缺失、compute reduce 全标量等问题;尚未改 device request 或 capability summary。 | 本文件 `现状与差距` 实读代码列出 `request_device.rs` 与 `capabilities.rs` 断链;计划 16 CN-M1 状态表记录 compute framework 仍未统一。 | 修复 required_features/gate 对齐,新增 subgroup/statistics/timestamp 能力位并接入 diagnostics。 |
| 2026-06-15 | GC-M2 submit hot path: bindless and indirect-count | 部分完成: fixed-count multi-draw/occlusion compact 基础存在,bindless 和 count buffer 未完成 | 计划 03 GS-M4 已有 fixed-count `multi_draw_indexed_indirect`,计划 04 VC-M3 已有 HZB compact replay 和 draw-count buffer;但 `multi_draw_indirect_count` 提交升档和 bindless material texture arrays 仍未接入。 | 计划 03/04 状态表记录 fixed-count replay、compact draw-count/visible-remap 与相关 `cargo check`;本文件明确 `MULTI_DRAW_INDIRECT_COUNT` 已请求但无人消费。 | 接入 count buffer replay、bindless material set、fallback policy 和 product parity tests。 |
| 2026-06-15 | GC-M3 bandwidth: streaming and half-resolution translucency | 未启动: 仍为优化扩展计划 | 常规 texture mip streaming、透明半分辨率段、separate translucency 均未实现。 | 本文件 `现状与差距` 记录 texture streamer 全量加载 mip 链、透明全分辨率直绘;计划 13 TX-M4/计划 12 FX-M2 尚未完成。 | 建立 distance-driven mip residency、separate translucency target 和 bandwidth diagnostics。 |
| 2026-06-15 | GC-M4 cache and quality improvements | 未启动: 静态阴影缓存/GPU 排序/specular AA 均待后续 | cached shadow maps、GPU bitonic sort、Toksvig/specular AA 仍无实现。 | 本文件 `现状与差距` 明确阴影逐帧全量重画、GPU 排序空缺、specular AA 空缺;计划 05/12/13 状态表也记录相关后续项。 | 在 LS/FX/TX 地基完成后分项实施 cache invalidation、GPU sorting 和 normal variance bake。 |

### 参考实现精读笔记

- **bevy `material_bind_groups.rs`**:bindless 不是"全局一张大表"而是 per-material-class slab,槽位回收走 free list;空槽必须填 fallback image,否则 `PARTIALLY_BOUND_BINDING_ARRAY` 缺失的平台直接校验失败——Zircon 的 slab 容量取 limits 下界并静态断言变体位与 gate 同步。
- **bevy `gpu_preprocessing.rs` + `draw_state.rs`**:count 提交的 `max_count` 必须取 CPU plan 上限而非 buffer 容量,否则尾部脏数据可见;bevy 把 batch set 的 count buffer 与 args buffer 分配在同一 prepare 节点,Zircon 经 graph 声明等价。
- **bevy `downsample.wgsl`**:subgroup 路径仍需 workgroup 二段归约(subgroup 大小不等于 workgroup 大小);`SUBGROUP_SUPPORT` def 在 pipeline specialization 注入而非运行时分支——对齐计划 08 模板拼接,不做 uniform 开关。
- **UE `ScenePrivate.h`/`ShadowSetup.cpp`**:`FCachedShadowMapData` 按 LightID 建表、一灯多 shadow map 用数组;失效判据核心是 initializer 等价比较 + MaxDrawDistance/分辨率档位变化——Zircon 以 hash + revision 简化,放弃 UE 的部分更新(scrolling)首版。
- **UE `StreamingManagerTexture.cpp`**:wanted mips 由"最近视点距离 + 屏幕尺寸 + 强制驻留偏置"取 max;预算超限时从优先级最低纹理开始降一档循环——Zircon 同型但去掉 level/材质追踪两层,直接消费 extract 可见集。
- **UE `GPUSort.cpp`**:bitonic 全链 = log²(n) 个 dispatch,键值乒乓双缓冲;首版不做 radix(radix 需多 pass 直方图 + 前缀和,留待 OIT 需求出现再升级)。
- **Unity `CommonMaterial.hlsl`**:`TextureNormalVariance` 用 avg normal length 的 vMF 近似(:241 注释即 Toksvig 谱系);`GeometricNormalFiltering`(:218)的 screenSpaceVariance 系数 0.25 上限钳制防止远景过糊——两个 threshold 参数原样进质量档。

## 风险与回退

- **binding array 平台差异**(A):DX12/Vulkan 桌面普遍可用,GL/WebGL 后端必然 false;bindless 永远是增强档,非 bindless 路径长期保留为一等公民(产物对拍进 CI),不允许任何 feature 仅在 bindless 档实现。
- **压实写的稳定性**(B):subgroup ballot 压实在 subgroup 大小差异平台上顺序不稳定 → draw 顺序变化影响透明段;count 档只用于 opaque/shadow/prepass 等顺序无关 phase,透明段固定走非压实档。
- **重建-换绑抖动**(G):mip 升降级重建 texture 引发 bind group 重建风暴 → 迟滞阈值(升级立即、降级延迟 N 帧)+ 每帧重建数上限;超限任务顺延,统计进 `render_perf_*`。
- **半分辨率边缘瑕疵**(F):深度不连续处上采样鬼影 → 双边核 + 保守 max 深度;不达标时该机制保持默认关,只在带宽受限 profile 开。
- **阴影缓存失效误判**(E):漏失效产生陈旧阴影(正确性事故)→ 失效键宁可过敏感(任何不确定变更一律全量重画);对拍测试覆盖三因子矩阵。
- **GPU 排序规模**(H):bitonic 在 n > 64k 时 pass 数可观 → 容量上限进 profile,超限回退 CPU emitter 粒度排序(计划 12 V1 行为)。
- **查询开销**(D):pipeline statistics 每 pass 一条查询,DX12 上 resolve 有固定开销 → 默认仅在抓帧/诊断模式开启,常规帧只开 timestamp。
- **回退矩阵总则**:每机制独立开关,任意组合下渲染产物有效;能力 gate 全 false 的最低配机器行为与计划 01–17 完成态完全一致(本计划零强制依赖)。

### wgpu 能力跟踪清单(等待上游,不在本计划展开)

| 机制 | 现状 | 跟踪点 |
|------|------|--------|
| memoryless / transient attachment | wgpu 无公开 API(Metal memoryless / Vulkan `TRANSIENT_ATTACHMENT` 未暴露) | tile 内 MSAA resolve 与 G-buffer 带宽归零;上游暴露后接计划 01 瞬态池 |
| Variable Rate Shading(VRS) | wgpu 无 | 上游暴露后以 RenderFeature 接入,与半分辨率透明互补 |
| mesh shader | wgpu experimental 之外,WGSL 无入口 | VG 计划的硬件光栅化路径候选 |
| `SHADER_F16` | wgpu 已有 feature,本工作区未请求 | 归 index §8 第 2 条"fp16 走能力检测";待计划 08 变体体系稳定后评估 |
| sparse binding(真 sparse texture) | wgpu 无;`supports_sparse_texture` 字段现为软件 SVT gate | 上游暴露后 mip 流送可去掉重建-换绑 |
| `TIMESTAMP_QUERY_INSIDE_PASSES` | wgpu 有,计划 17 已决策不需要(pass 边界粒度足够) | 仅记录,不消费 |
