---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/core
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_resources/runtime_prepare.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/environment_cubemap.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_write_scene_uniform/write_scene_uniform.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_target/finish_viewport_frame.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
  - docs/plans/zircon_runtime/render/13-texture-pipeline.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/bevy/crates/bevy_render/src/pipelined_rendering.rs
  - dev/bevy/crates/bevy_render/src/lib.rs
  - dev/bevy/crates/bevy_render/src/texture/gpu_image.rs
tests:
  - scene renderer core ninety-two of ninety-two Rust files reviewed, 9985 current lines
  - cubemap full-chain encoding and irradiance scratch source guard RED then GREEN
  - rustfmt and scoped git diff check passed
  - current-source focused Cargo reservation pending
  - F2 counters, GPU timestamps and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics scene renderer core逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`zircon_runtime/src/graphics/scene/scene_renderer/core/**`当前92/92个Rust文件、9,985行，包括构造/layout/bind-group、compiled-scene 26文件、插件prepare/readback、history、uniform、surface/offscreen、capture与runtime-output存储。compiled-scene细项已由PERF-MVP-373..378和独立证据覆盖；本轮补齐其余66文件后确认两项新的MVP根因：插件runtime-prepare把任意collector的串行工作直接放入render submission线程（PERF-MVP-379），source cubemap变更帧在同一线程同步完成全量f32→f16编码和碎片上传（PERF-MVP-380）。

无surface/headless路径的`finish_viewport_frame`会调用`read_texture_rgba`创建readback buffer、提交copy、`map_async`后`PollType::wait_indefinitely`并逐行复制整张RGBA；这与既有PERF-MVP-023完全相同，不重复编号。构造期固定BRDF LUT约1,678万积分样本已由PERF-MVP-351覆盖；`reset_last_runtime_outputs`每帧丢弃报告Vec容量归既有PERF-MVP-343的generation-owned sealed report，不再拆出重复工作项。

## 已直接止损

`SceneEnvironmentCubemap::upload_cubemap_texels`原对每个face×mip切片分别分配RGBA16F编码Vec；现在每条source或PMREM mip-chain只分配一份按face/mip清空复用的byte scratch，capacity限制在单面base mip，不放大到整条cubemap链。irradiance原每face先分配RGBA32F texel Vec、再分配RGBA16F byte Vec；现在直接编码到一份跨六面复用的byte scratch。源码门禁先观察RED再转GREEN，并运行rustfmt和scoped diff check。该改动只减少变更帧的CPU分配，不把同步转换与多次`write_texture`冒充为已解决。

## PERF-MVP-379：插件prepare在提交线程串行且无预算

`execute_runtime_prepare_passes`依注册顺序同步调用全部collector，并把`Device`、`Queue`、当前`CommandEncoder`、`ResourceStreamer`和完整`ViewportRenderFrame`直接暴露给回调。合同允许任意CPU准备、GPU对象创建和大payload clone发生在提交关键路径；每帧还新建external-binding Vec与owned `RenderPluginRendererOutputs`，多个collector的VG/GI payload通过大量`Vec::extend`合并。empty collector已有O(1)早退，但非空工作没有generation快路、并行调度、预算或背压。

Plugins01联动Render03/12/18把插件合同拆成generation-owned CPU prepare artifact与render-thread record/apply两个阶段：重工作进入有界plugin/compute lane，single-flight发布immutable handles；render线程只消费ready delta、持久external binding identity和预分配output owner，未ready时使用明确neutral或last-good状态。Bevy的pipelined rendering以容量1的双向channel隔离render app线程，并把Extract/Prepare/Queue阶段显式排序；Zircon不需复制ECS形态，但应采用同类线程边界和有界交接。

## PERF-MVP-380：环境上传仍在帧提交链同步转换与碎片写入

`write_scene_uniform`每帧检查environment upload key；稳定key会早退，但generation变化时`ensure_uploaded`仍在提交线程遍历全部source、PMREM和irradiance texels做f32→f16转换，并以六面×mip逐次`queue.write_texture`。本轮只把编码临时分配从face×mip降为每chain一份单面峰值scratch并复用irradiance scratch；转换总量和GPU提交碎片数未改变。

Runtime04与Render11/13/17应让asset/bake worker发布版本化、row-aligned的预编码GPU upload artifact，render owner用持久staging arena在单个upload command batch中按generation提交；稳定帧只比较key。Bevy把image CPU→GPU转换/上传放在`RenderAsset::prepare_asset`阶段，而不是camera uniform写入函数；Zircon还需联动PERF-MVP-352/354复用resident IBL artifact，避免生成、缓存、上传形成多份owner。

## 验收

按collectors 0/1/16/64、plugin payload 0/1/64 MiB、cubemap face 64/128/512/1024、mip 1/all、stable/1% changed记录render-thread callback/conversion wall、queue depth/age/drop、owned/copy/temp bytes、Vec growth、GPU object create、write/copy calls、staging growth、upload bytes与CPU/GPU p50/p95/p99。最终stable plugin heavy prepare/copy/binding rebuild和environment转换/上传均为0；changed prepare/artifact build≤1/generation；render callback有界；cubemap upload batch≤1且staging按capacity复用。focused Cargo、F2行为/像素、timestamp和DX12 RenderDoc通过前，整个目录留在`pending.md`，不进入`review.md`。
