# VG/HGI 后续完善计划（参照 Unreal Nanite/Lumen）

## Summary
- 以现有插件化切割为基线：VG/HGI 的重状态继续留在 `zircon_plugins`，`zircon_runtime` 只保留中立渲染 DTO、RenderGraph/RHI 宿主能力、执行器注册与只读输出。
- 参考 Unreal Nanite 的资源/流送/剔除/反馈链路，补齐 VG 的 cook、场景抽取、页驻留、NodeAndClusterCull、VisBuffer64 与反馈闭环。
- 参考 Unreal Lumen 的 SceneData、Card Capture、Surface Cache、ScreenProbeGather、RadianceCache，补齐 HGI 的场景表示、表面缓存、屏幕探针、体素 fallback、历史与最终合成。
- 分层推进：先收束当前 renderer plugin 基线和旧路径测试，再做 VG/HGI 各自闭环，最后做跨功能组合与文档验收。

## Key Changes
1. **M0：稳定当前插件渲染基线**
   - 保留现有 `RenderFeatureDescriptor`、中立 readback DTO、executor registration 形态，不恢复旧 runtime 专有路径，也不让 runtime 依赖 `zircon_plugins`。
   - 将仍依赖旧 runtime fixture 的 VG/HGI renderer 测试迁到插件拥有的测试源，先处理 `crate::asset`、`crate::core`、`SceneRenderer`、`ViewportRenderFrame` 等旧 owner 假设。
   - 把 VG/HGI `lib.rs` 中当前占位 executor 逐步接入插件内部 renderer helper，但只通过中立执行上下文与输出 DTO 交换数据。

2. **M1：VG Nanite-like 数据资产与 CPU 参考层**
   - 以 Unreal `NaniteResources.cpp`、`NaniteResources.h` 为参考，强化 cook 输出：稳定 root、hierarchy node、cluster、page、page dependency、debug metadata，不追求 UE 二进制兼容。
   - 在 VG 插件内扩展 CPU reference traversal：加入 camera/frustum/SSE/forced mip/page residency 的统一策略，让 `CpuDebug`、`BaselineGpu`、`FlagshipGpu` 共用同一份可验证决策。
   - 保持 cook 仍在资产层，插件只消费中立 cooked VG 数据与场景实例快照。

3. **M2：VG 场景抽取、页驻留与反馈闭环**
   - 以 Unreal `NaniteStreamingManager.cpp`、`NaniteFeedback.cpp` 为参考，在 VG 插件内实现 page request、priority、budget、LRU/age、pending completion、root page table readback。
   - 场景抽取从 cooked model primitive 和实例快照生成 live VG instance，不再把手写 fixture 当真实场景来源。
   - 插件禁用时不得初始化 VG 重状态；启用时通过 capability/feature descriptor 暴露 debug/profile/readback 数据。

4. **M3：VG NodeAndClusterCull、VisBuffer64 与 raster 路径**
   - 以 Unreal `NaniteCullRaster.cpp`、`NaniteShared.h`、`NaniteVisibility.cpp` 为参考，实现 bounded CPU/GPU-equivalent cull plan：VisitNode、StoreCluster、frustum、near、backface、SSE、budget、debug visit log。
   - 引入中立 `VisBuffer64` 输出语义，由 VG 插件填充 visibility/debug readback；runtime 只承载中立输出，不知道插件内部资源布局。
   - Baseline 路径优先保证 deterministic 软件/简化 GPU 结果；Flagship 路径再接硬件 raster、multi-pass hierarchy culling、material visibility 查询。

5. **M4：HGI Lumen-like Scene Representation**
   - 以 Unreal `LumenSceneData.h`、`LumenScene.cpp` 为参考，把 HGI 真值从 authored probes/traces 迁到 scene representation：mesh/card candidates、light inputs、material/emissive/normal/depth/motion、history invalidation。
   - `RenderHybridGiExtract` 保留设置、预算和 debug 入口；旧 `RenderHybridGiProbe` / `RenderHybridGiTraceRegion` 降级为测试 fixture/debug 输入，后续删除生产依赖。
   - HGI 插件独立运行，不强依赖 ray tracing feature；未来有 `ray_tracing_policy` 时只作为可选加速能力。

6. **M5：HGI Surface Cache、Card Capture 与 Probe/Radiance Cache**
   - 以 Unreal `LumenSceneCardCapture.cpp`、`LumenSurfaceCache.cpp` 为参考，实现 card allocation、page table、surface atlas、capture feedback、dirty/invalidation。
   - 以 `LumenScreenProbeGather.cpp`、`LumenRadianceCache.cpp` 为参考，实现 screen probe placement、trace scheduling、radiance cache update、temporal filter、history rejection。
   - 加入 voxel/low-detail fallback，避免无 probe 或 cache miss 时直接退化成黑帧或静态常量 GI。

7. **M6：VG/HGI 组合与最终收束**
   - HGI 可消费 VG 产生的中立 scene/material/visibility 信息，但不得直接读取 VG 插件私有状态。
   - 补齐统计、debug profile、readback snapshot 和文档：更新 VG Nanite foundation 与 HGI Lumen scene representation 文档，记录 Unreal 参考点、接口边界、测试证据。
   - 与 rendering optional feature 计划保持兼容：`virtual_geometry`、`hybrid_gi` 可独立启停，组合启用只增加能力，不改变 runtime 根包所有权。

## Public Interfaces
- `zircon_runtime` 只允许新增或扩展中立类型：render feature capability、pass execution context、plugin renderer outputs、scene/material/light extract、debug/readback DTO。
- VG 插件拥有 VG runtime state、page residency、GPU/readback helpers、Nanite-like traversal/cull/raster 实现。
- HGI 插件拥有 scene representation、surface cache、probe/radiance cache、history、resolve 与 GPU/readback helpers。
- 不新增 runtime 到插件的依赖，不保留旧路径兼容 shim，不在 editor/UI 层做额外改动，除非后续单独开启 editor plugin UI 任务。

## Test Plan
- 每个里程碑先跑插件包 focused tests：`zircon_plugin_virtual_geometry_runtime`、`zircon_plugin_hybrid_gi_runtime`。
- M0 后必须恢复 renderer/test-source promotion 的通过状态，并用搜索门禁确认无旧 VG/HGI runtime owner 路径残留。
- VG 验证覆盖 cook determinism、CPU reference traversal、page request/residency、VisBuffer64 readback、Baseline/Flagship/CpuDebug 一致性。
- HGI 验证覆盖 scene representation、card/page allocation、surface cache invalidation、screen probe/radiance cache、history rejection、fallback resolve。
- runtime focused tests 覆盖 executor propagation、plugin extension registration、neutral readback DTO；workspace 全量测试若仍被 editor/UI identity drift 阻塞，记录为外部 blocker，不把它混入 VG/HGI 完成标准。

## Assumptions
- 当前活跃的插件化切割和中立 renderer executor/readback 设计作为既定基线。
- `dev/UnrealEngine` 只作为架构和行为参考，不做 UE 代码复制，也不追求数据格式兼容。
- 优先完成 runtime/plugin 渲染闭环；editor 可视化面板、复杂 UI 和 runtime-interface identity drift 不纳入本轮 VG/HGI 主线。
- 由于当前工作树已有多会话改动，实施时只触碰 VG/HGI、必要的中立 render DTO、资产 cook 和相关文档，避免覆盖其他会话成果。
