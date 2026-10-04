---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/probe_buffer
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ReflectionEnvironment.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PlanarReflectionRendering.cpp
  - dev/bevy/crates/bevy_pbr/src/light_probe/environment_map.rs
tests:
  - environment probe_buffer current Rust source eleven of eleven files reviewed, 1549 lines
  - candidate registry lock source guard RED then GREEN
  - scoped rustfmt and diff check passed
  - current-source Cargo, scale counters, F2 pixels and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics environment probe buffer逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`scene_renderer/environment/probe_buffer/**`当前11/11个Rust文件、1,549行，覆盖GPU ABI、persistent resources、slot allocator、PMREM validation/upload及全部tests。固定64-slot LRU与revision命中已经保证稳定cubemap不重复解码/上传；RGBA16F mip payload按revision变化上传，GPU layout和CPU reference parity也有回归。

## 本轮直接止损

`SceneReflectionProbeResources::prepare`原来在最多64个候选的资产加载循环中每个probe都调用`resource_manager.registry()`，即每帧最多64次全局`RwLock`读锁获取。已保持distance/priority/id排序、truncate、missing rejection和slot revision语义不变，把候选先携带cubemap ID，并在一个短registry guard中批量投影revision；guard释放后才执行可能较重的texture load/decode/upload。源码守卫先复现RED再转GREEN，确认registry读取位于candidate upload循环之前且调用点唯一；scoped rustfmt与diff check已过，Cargo待协调器批量执行。

## PERF-MVP-400：generation probe/planar artifact与按需GPU owner

稳定帧仍从environment probes重新filter/collect/sort/truncate，重建`Vec<GpuReflectionProbe>`并写完整active prefix和16-byte header；disabled状态也每帧写相同零header。planar path每camera/frame扫描全部planar probes、重新派生反射camera/matrices并无条件写176-byte uniform。构造`SceneReflectionProbeResources`时无论feature是否启用，都分配64 cubemaps×6 faces×8 mips的128 RGBA16F array、1024 RGBA16F planar 11-mip texture及相关buffers/views；minimal F2和无probe场景也承担VRAM与GPU object成本。

Render11/18应让environment/probe、camera transform/layer、asset revision和planar capture generation发布唯一prepared artifact：scene-static候选/slot与GPU rows按probe generation更新，camera只做可见range或dense selection；相同bytes/header/planar params跳过queue write。probe与planar GPU资源按compiled feature/resource generation single-flight创建，关闭时只绑定device共享neutral，不常驻真实大纹理；capacity按实际需求增长并保留slot identity。slot淘汰、missing/load/PMREM rejection、layer/priority/box projection和capture-camera禁用语义不得变化。

Unreal reflection environment保留`FReflectionCaptureCache`、slot usage与cubemap array resize/remap，仅在desired capacity/size变化时更新；planar reflection由scene proxy持久render target。Bevy的environment map只携带`AssetId`并从`RenderAssets<GpuImage>`取得resident views，probe prepare不重新解码/上传图像。Zircon应复用现有slot/revision基础，而不是另建兼容热路径。

## 验收预算

按probes 0/1/16/64/1k、planar probes 0/1/16/1k、cameras 1/8、stable/camera move/layer/priority/revision/add/remove与feature off/on记录candidate visits/sorts、registry locks、GPU row/matrix builds、buffer writes/bytes、texture/buffer/view creates、VRAM、slot hit/eviction和CPU/GPU p95。当前registry locks≤1/frame；最终stable sort/build/write=0、同generation registry lock=0或≤1 artifact build、changed工作近affected probes/cameras，feature-off真实probe/planar allocation=0，capacity只增长到需求且共享neutral≤1/device。focused Cargo、F2像素、timestamp与DX12 RenderDoc完成前保留在`pending.md`。
