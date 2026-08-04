# 19-gpu-capability-optimizations 产出记录归档

> 来源：[`19-gpu-capability-optimizations.md`](../19-gpu-capability-optimizations.md) 的 `## 状态与产出记录`。

## 状态与产出记录

| 日期 | 里程碑/切片 | 状态 | 产出 | 验证与证据 | 后续 |
|------|-------------|------|------|------------|------|
| 2026-08-03 | GC-M3 material-marked transparent mesh | implementation_complete / secondary_static_review_findings_repaired / managed_validation_pending | alpha-blend `separate_translucency` 透明 mesh 已有专属命令/间接流和克隆资源契约的 half-resolution executor；profile 关闭、MSAA 或 executor 缺失时会在 indirect build 前合并回全分辨率透明流。 | 所有权/序列化、命令分段、graph contract、forward dependencies、fallback merge、scoped Rust parse 与 diff 检查完成；动态图形验收仍由 coordinator 管理。 | coordinator 运行产品渲染、RenderDoc 与新 PNG 后再 accepted closeout。 |
| 2026-08-03 | GC-M2 indirect-count and bindless infrastructure | implementation_in_progress / static_parse_and_diff_check_complete / managed_validation_pending | count -> fixed multi-draw -> per-draw indirect -> direct 提交链及其分档计数、binding-array feature/limit 协商、fallback-filled slab/WGPU table、224-byte material payload ABI、capability-gated template wrapper、按材质资源 ID/revision 去重的 CPU payload registry，以及 GPU Scene primitive 的显式 material-payload slot ABI 已完成；GPU storage row、生产 bind group/pipeline 选择仍待接入。 | scoped Rust parse、模板/registry/ABI 静态测试、rustfmt parse 与 diff check 完成；未直接运行 Cargo、WGPU、RenderDoc 或生成 PNG。 | 继续把 registry/table 接到 GpuScene storage 与独立 bindless pipeline layout，非 bindless 路径保持产品对拍基线。 |
| 2026-08-03 | GC-M4 static shadow cache identity | slice-1 implementation_complete / static_parse_and_diff_check_complete / managed_validation_pending | per-slot 三因子 cache validity、静态 caster revision、完整 shader-visible light parameter hash 及 atlas slot generation 已完成；atlas copy 与 dynamic caster overlay 尚待接入。 | pure Rust regression coverage、scoped Rust parse 与 diff check 完成；未直接运行 Cargo、WGPU、RenderDoc 或生成 PNG。 | 以 cache decision 将 atlas pass 分为 static redraw、static copy 与 dynamic overlay，再做深度对拍。 |
| 2026-08-03 | GC-M3 常规纹理 mip 流送与半分辨率透明实现 | 核心实现及二次静态审查完成；材质标记网格分段和受管动态验收待执行 | `TextureMetadata.streaming_enabled` 默认启用且可由 texture import setting 显式关闭；UI/小纹理/单 mip/runtime-mip/SVT 强制全驻留。`PreparedTexture` 记录连续 `resident_mip_range`。资源准备末尾从权威 `FrameVisibility` 主视图和已准备材质槽采集可见纹理，复用 scratch 集合且自定义投影保守保持最高驻留。`resource_streamer_mip_streaming` 实现可见性去重、screen-coverage wanted-mip、每帧迁移上限、mip 迟滞、最低 mip tail 保障，以及拒绝重叠/失败/过期完成的 per-texture transition state；所有已准备但未观测的纹理以零屏占比进入预算驱逐候选。统一提交上下文的 effective global mip bias 同时驱动采样和流送 wanted mip。`RenderFrameProfile`/diagnostics 记录所有已准备资产纹理的物理常驻字节，`RenderMemoryBudget` 的 1 GiB reference cap 先限制 promotion，并在超额时按低屏占比先行一档驱逐，再交给既有 global mip bias。适格的 source-mip-complete RGBA8 2D/cube 会新建物理 tail texture、GPU copy 共同 resident mip、只上传缺失 mip 并在成功后原子替换；压缩/1D/3D/lightmap 保持全驻留但仍计入预算，未假装支持不完整 layout。半分辨率透明由 profile quality gate 驱动，只在单采样 `Transparent3d` 图注入 half color/depth、深度降采样、渲染器标记的粒子子段和双边深度感知合成；透明 mesh 保持全分辨率，直到材质标记分段完整接入。深度指数核系数由 quality profile 的 `half_resolution_transparency_depth_sigma` 传入 uniform，而非硬编码。MSAA 或功能关闭时保留原全分辨率图。 | 覆盖元数据资格、导入布尔解析、投影屏占比、优先级/迟滞/tail/去重、状态机、持久预算 promotion defer/低优先级驱逐，以及 RGBA8 resident upload 重映射与成功后发布顺序的单测；复审补充离屏已驻留纹理驱逐、粒子半分辨率路由和 quality sigma 传递的静态断言。半分辨率图编译测试覆盖 profile 关闭、单采样资源/读写声明和 4x MSAA 回退；本会话完成 scoped `rustfmt --check` 与 `git diff --check`。按协调规则未直接运行 Cargo、WGPU、RenderDoc 或生成 PNG。 | 继续实现材质标记的透明 mesh 分段；随后由 coordinator wakeup 运行 resource-streamer tests、产品渲染、RenderDoc 和新 PNG 证据。 |
| 2026-06-23 | Render index 当前状态总览拆分 | GC-M2 部分完成,GC-M1/3/4 未启动 | 从 docs/plans/zircon_runtime/render/index.md 的第 9 节迁入本计划；本行保留 19 GPU Capability/Bandwidth 的当前事实，render 总索引不再维护计划级明细。 | 文档重组；本次未改生产代码，render/index.md 只保留状态路由说明。 | 仍未完成：capability surface、streaming/separate translucency、cache/quality improvements；验收缺口：需要 feature request/gate 对齐、indirect-count/bindless fallback、bandwidth/cache/product parity tests |
| 2026-06-15 | GC-M1 capability surface and compute base | 未启动: 现状审计已完成,能力请求/gate 修复未实施 | 文档已盘点 binding array gate 探测但未请求、subgroup/timestamp/statistics 能力位缺失、compute reduce 全标量等问题;尚未改 device request 或 capability summary。 | 本文件 `现状与差距` 实读代码列出 `request_device.rs` 与 `capabilities.rs` 断链;计划 16 CN-M1 状态表记录 compute framework 仍未统一。 | 修复 required_features/gate 对齐,新增 subgroup/statistics/timestamp 能力位并接入 diagnostics。 |
| 2026-06-15 | GC-M2 submit hot path: bindless and indirect-count | 部分完成: fixed-count multi-draw/occlusion compact 基础存在,bindless 和 count buffer 未完成 | 计划 03 GS-M4 已有 fixed-count `multi_draw_indexed_indirect`,计划 04 VC-M3 已有 HZB compact replay 和 draw-count buffer;但 `multi_draw_indirect_count` 提交升档和 bindless material texture arrays 仍未接入。 | 计划 03/04 状态表记录 fixed-count replay、compact draw-count/visible-remap 与相关 `cargo check`;本文件明确 `MULTI_DRAW_INDIRECT_COUNT` 已请求但无人消费。 | 接入 count buffer replay、bindless material set、fallback policy 和 product parity tests。 |
| 2026-06-15 | GC-M3 bandwidth: streaming and half-resolution translucency | 未启动: 仍为优化扩展计划 | 常规 texture mip streaming、透明半分辨率段、separate translucency 均未实现。 | 本文件 `现状与差距` 记录 texture streamer 全量加载 mip 链、透明全分辨率直绘;计划 13 TX-M4/计划 12 FX-M2 尚未完成。 | 建立 distance-driven mip residency、separate translucency target 和 bandwidth diagnostics。 |
| 2026-06-15 | GC-M4 cache and quality improvements | 未启动: 静态阴影缓存/GPU 排序/specular AA 均待后续 | cached shadow maps、GPU bitonic sort、Toksvig/specular AA 仍无实现。 | 本文件 `现状与差距` 明确阴影逐帧全量重画、GPU 排序空缺、specular AA 空缺;计划 05/12/13 状态表也记录相关后续项。 | 在 LS/FX/TX 地基完成后分项实施 cache invalidation、GPU sorting 和 normal variance bake。 |

### 2026-08-03 GC-M3 material-marked transparent mesh update

- 状态：`implementation_complete / secondary_static_review_findings_repaired / managed_validation_pending`。材质自有 `[overrides] separate_translucency = true` 仅在 alpha-blend runtime material 上生效；未标记 mesh 继续使用原全分辨率透明流。
- 完成：mesh build 将该标记带入 command，命令缓冲和间接参数将其从普通透明流拆开；half-resolution mesh executor 只回放该流。图注入从选中 `mesh.transparent` pass 克隆完整资源契约，再替换 scene color/depth target，保留前向或延迟路径的既有 light/shadow 依赖。
- 二次静态复审前向修复：当 profile 关闭、MSAA 开启或 plugin 未提供 half-resolution mesh pass 时，标记 mesh 在间接批处理前合并回常规透明流；同时修正默认材质 fixture 的错误正向序列化断言。静态覆盖包含材质所有权/序列化、command split 和 fallback merge、single-sample graph resource contract、forward light dependencies 与 4x MSAA fallback。
- 本会话未直接执行 Cargo、WGPU、RenderDoc 或 PNG 生成。后续继续 GC-M1 非验收实现，由 coordinator-managed 动态验收验证图形结果。

### 2026-08-03 GC-M1 pipeline-statistics diagnostic baseline

- 状态：`implementation_complete / static_parse_and_diff_check_complete / managed_validation_pending`。WGPU `PIPELINE_STATISTICS_QUERY` 仅在既有 GPU timing diagnostics 开关启用且设备实际协商该 feature 时创建；默认渲染路径不创建 QuerySet、不分配读回请求。
- 完成：RHI WGPU 层提供五槽 query 生命周期（vertex、clipper、primitive-out、fragment、compute invocations），以共享 bounded readback queue 异步解析并同名聚合。首个消费者是 HZB 每个实际 compute mip pass，scope 在 compute pass 内开始和结束，不在 encoder 范围伪测量。frame profile 的 pass 条目新增独立 `pipeline_statistics`，不影响 GPU 时间预算；诊断期间禁用 graph parallel recording，避免共享 query-slot 分配竞争。
- 静态覆盖：feature gate、五槽解码、同一 graph pass 多 mip 聚合、profile late-result merge、HZB scope 传递，以及改动文件的 Rust 语法解析和 `git diff --check`。未直接执行 Cargo、WGPU、RenderDoc 或 PNG 生成。

### 2026-08-03 GC-M2 indirect-count fallback and bindless material foundation

- 状态：`implementation_in_progress / static_parse_and_diff_check_complete / managed_validation_pending`。本切片完成了 count 提交 gate 的正确降级、bindless allocator/table、payload ABI、capability-gated 模板 wrapper、CPU registry 与 primitive slot ABI；尚未把 table/registry 写入 GPU Scene storage、draw bind group 及生产 mesh pipeline。
- 完成：`MULTI_DRAW_INDIRECT_COUNT` 独立于 core fixed-count multi-draw 能力协商。提交降级链已收敛为 `count -> fixed multi-draw -> per-draw indirect -> direct`：压实 replay 仅在 count feature 已协商时调用 `multi_draw_indexed_indirect_count`；否则在支持 multi-draw 时回放 CPU batch 上限的 fixed-count stream；仅支持 indirect 的设备保留参数缓冲并逐条 `draw_indexed_indirect`；没有 `INDIRECT_FIRST_INSTANCE` 时保留 direct。四档 API 调用计数贯穿 replay、shadow atlas 多 slot 合并和 prepared queue stats。压实前完整清零输出 args 区，故 fixed-count 回退只能读到零实例尾项，保留 GPU culling 正确性且不访问未经协商的 API。`BindlessMaterialSlab` 使用启动期定容的 slot/free-list/hash-dedup 表；slot 0 与所有空槽固定为 fallback，lease generation 拒绝延迟释放回收后的新槽位。`BindlessMaterialBindingTable` 用协商容量创建 group-2 texture/sampler array，持有资源 owner，且仅在分配、最终释放或 mip-view 替换时重建 bind group。`GpuBindlessMaterialPayload` 以 224-byte/16-byte-aligned 行复用既有 192-byte 标准属性并追加六个标准纹理索引与两个 fallback reserve slot。模板请求只有同时携带 bindless feature 与非零协商容量时才注入 `enable wgpu_binding_array` 和 wrapper；反向组合显式拒绝。prepare 侧将稳定 `ResourceId` 贯穿 `PendingMeshDraw`，`BindlessMaterialPayloadRegistry` 按资源 ID/revision 保持 slot、按帧回收并仅导出脏行，GPU Scene primitive ABI 预留 96-byte 行中的 `material_payload_slot` 和明确的 16-byte tail，当前普通路径写 invalid sentinel。
- 静态覆盖：WGPU optional count feature 映射、framework count/indirect gate、三种 indirect replay 分支和 direct eligibility、分档统计 bridge，以及 slab 的 fallback 填充、去重、容量回收、fallback 不占动态槽和 stale-lease 回收保护。设备请求仅在 bindless 三项 feature 全部可用时协商 texture/sampler array 的共同容量（最小值、上限 1000）；RHI `RenderDeviceLimits`、framework diagnostics 与 capability summary 都携带该上限，gate 要求共同容量至少为 2。table layout 静态检查 texture/sampler array 的相同固定容量；payload 检查标准 uniform row 与 fallback reserve；模板检查 capacity/feature 配对和 binding-array header；registry 检查同帧去重、revision 原位更新与帧末回收复用；GPU Scene layout test 检查新的显式 field offset。完成 scoped Rust syntax parse 与 `git diff --check`；未直接运行 Cargo、WGPU、RenderDoc 或生成 PNG。
- 后续：把 registry 的脏 payload row 写入 GPU Scene storage，并只为 bindless variant 建立包含该 storage 的独立 scene/pipeline layout；随后在 prepare 期将标准纹理的 slot lease 接入 table。非 bindless group-2 路径保留且后续受管产品对拍。

### 2026-08-03 GC-M4 static shadow cache identity

- 状态：`slice-1 implementation_complete / static_parse_and_diff_check_complete / managed_validation_pending`。本切片只建立命中判据和 atlas 代际，不假装已经跳过 shadow rendering。
- 完成：`ShadowCache` 以 `ShadowSlotKey(light_id, face_index)` 持有 `ShadowCacheEntry`，只有 `light_params_hash`、`static_caster_revision`、`atlas_slot_generation` 三项完全相等才返回 `ReuseStaticDepth`；任何一项变化或缺失 entry 都返回带原因的 `RedrawStaticDepth`。`static_shadow_caster_revision` 对 static、geometry/material revision 均权威的 caster 按稳定 key 排序后计算 revision；存在可移动或无版本 caster 时返回 `None`，调用方必须强制重画。`shadow_light_params_hash` 覆盖最终 `GpuShadowSlot` 的 view-projection、atlas scale/bias 和 depth/normal-bias/PCF flags 的原始位模式。atlas allocator 保持当前 slot 的 generation：同 key、同保留 allocation 复用 generation；slot 被移除并重新分配则生成新值。
- 静态覆盖：命中与三种失效因子、slot retain、caster 顺序无关/内容敏感、动态或无版本 caster fail-closed、GPU slot 三组字段的 hash 敏感性，以及 allocator reuse/reallocation generation 行为。完成 scoped Rust syntax parse 与 `git diff --check`；未直接运行 Cargo、WGPU、RenderDoc 或 PNG 生成。
- 后续：将 frame extract 中可验证的 static caster 集和 `ShadowFramePlan` slot generation 组合为 cache input；再分裂 atlas record 为 static redraw/copy/dynamic overlay，并以静态场景深度 bit-exact 对拍验收。

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
