---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - docs/plans/zircon_runtime/render/07-postprocess-color-pipeline.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/VolumetricFog.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/LightCookieManager.cs
  - dev/bevy/crates/bevy_core_pipeline/src/oit
  - dev/bevy/crates/bevy_pbr/src/light_probe/irradiance_volume.rs
tests:
  - complete advanced_lighting directory forty of forty current Rust files reviewed, 6538 lines
  - empty froxel local-volume fallback upload source contract RED then GREEN
  - OIT counted-prefix clear source contract RED then GREEN
  - light-cookie explicit atlas-slot contract RED then GREEN
  - scoped rustfmt and diff check passed
  - current-source Cargo, F2 pixels, GPU timestamps and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics advanced lighting逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`scene_renderer/advanced_lighting/**`当前40/40个Rust文件、6,538行：froxel 17、irradiance volume 3、light cookie 6、OIT 4、planar filter 3、subsurface 3、transmission 3及root 1。现有WGSL parse/ABI、froxel数值与真实WGPU链、OIT排序、planar mip、SSS descriptor及fallback测试覆盖了基础合同；本记录只声明全目录静态审查、三组源码守卫和局部修复，不声明current-source动态验收。

## 本轮直接止损

`FroxelMediaInjectPipeline`原在无局部雾体或质量档关闭局部雾体时，每dispatch仍创建CPU `Vec`、填入一个48B零体积并新建storage buffer上传。现把零体积buffer提升为pipeline持久fallback，空路径上传从208B降至160B，并删除每帧零体积CPU/GPU分配；真实局部体积上传路径不变。

OIT fragment-store原同时清空count和整个layer buffer。resolve只读取atomic count限定的本帧前缀，而该前缀在fragment store中逐元素覆盖，因此layer clear是冗余全量GPU写。现仅清count；默认4 fragments/pixel时，每个OIT帧避免约63.3MiB（1080p）或253.1MiB（4K）清零。源码合同同时锁定count clear仍存在。

light-cookie审查另发现blit把`metadata.misc[2]`当atlas slot，但该字段属于灯光体积参与位且frame plan始终写0，多个cookie会覆盖第0格。现由`CookieAtlasEntry`显式携带稳定slot，blit按slot定位viewport，并补排序/去重后slot 0、1回归。该项是正确性修复，可避免后续像素与RenderDoc验收建立在错误atlas上。

## PERF-MVP-403：PreparedAdvancedLightingFrame与持久GPU工作区

froxel三个pass分别解析同一相机的Volume设置、派生camera/grid/matrix并复制资源句柄和诊断String；media inject还先克隆layer-visible `FogVolumeData`，再转换为第二个GPU Vec。各pass每帧创建params buffer和bind group，light scatter另建两组binding。Volume重复求值已由PERF-MVP-363负责，matrix与诊断metadata分别回链PERF-MVP-346/343；本任务要求最终消费者只借用一次准备的advanced artifact。

cookie atlas每camera/frame clone cookies、建`BTreeMap`去重排序、全清1024x1024 atlas、对最多64个resident cookie逐个创建bind group和draw，即使asset/slot完全稳定也重做。Unity HDRP的`LightCookieManager`用persistent atlas的`IsCached`/`NeedsUpdate`只刷新dirty texture，Zircon应按cookie/asset revision generation维护stable slots、cached binding与dirty blit ranges，稳定帧plan/clear/bind/draw均为0。

irradiance volume在draw构建前已收集可见mesh positions并做volumes x positions containment、resident Arc clone和uniform write；graph中的`irradiance.volume_bind`又完整执行一次相同选择与写入。应在draw前唯一发布selected volume/resource-generation handle，graph executor只消费/验证该artifact，不再创建第二份positions Vec、克隆volume或重复queue write。selection进一步接PERF-MVP-377的scene-generation spatial index；稳定generation不得重复矩阵逆转置或uniform写。

planar filter对每个mip创建params buffer、output view、bind group和独立compute pass，1024纹理为11套对象/dispatch，report又为每mip分配Vec/String诊断。SSS setup/scatter分别解析相同profile table和逆view-projection，scatter再深clone profiles并每帧创建profile/params buffers及bind group，recombine再建binding；64-tap Burley只通过tile indirect裁剪GPU工作。两者应复用texture-generation mip view/binding bundle、camera dynamic-uniform ring和单一resolved profile artifact，详细per-mip/per-pass rows仅诊断启用时生成。

Render18作为feature owner发布`PreparedAdvancedLightingFrame`：resolved fog/grid/matrices、packed visible fog volumes、cookie plan/dirty slots、selected irradiance resident handle、resolved SSS table及planar mip bundle均带scene/camera/asset/feature generation。Render01只编译dense pass-to-artifact handles；Plugins01允许重CPU筛选/打包在有界prepare job single-flight完成，render线程只record ready/last-good。OIT/froxel/SSS/planar动态参数走帧并行安全的ring/arena，不能用单buffer覆盖in-flight帧。

## 验收预算

按cameras 1/8、meshes 0/1/1k/100k、fog/irradiance volumes 0/1/16/1k、cookies 0/1/64、OIT 1080p/4K与fragments 1/4/8、planar 128/1024、SSS profiles 0/1/16、stable/1% changed/resize/reload/off-on记录Volume resolves、position/volume visits、plan builds/sorts/clones、matrix/table builds、CPU alloc bytes、buffer/texture/view/bind creates、clear/upload/blit bytes、pass/draw/dispatch、artifact hit及CPU/GPU p95。当前空froxel fallback per-frame buffer/upload=0，OIT layer clear bytes=0，cookie slot唯一；最终stable heavy CPU build/write/GPU object create=0、cookie clear/blit=0、irradiance selection<=1/camera generation、SSS table<=1、planar views<=1/texture generation，feature-off真实资源=0。focused Cargo、F2像素、GPU timestamp和DX12 RenderDoc完成前保留在`pending.md`，不进入`review.md`。
