---
related_code:
  - zircon_plugins/particles/runtime/src/service.rs
  - zircon_plugins/particles/runtime/src/component.rs
  - zircon_plugins/particles/runtime/src/asset.rs
  - zircon_plugins/particles/runtime/src/simulation/cpu.rs
  - zircon_plugins/particles/runtime/src/simulation/pool.rs
  - zircon_plugins/particles/runtime/src/render/extract.rs
  - zircon_plugins/particles/runtime/src/render/feature.rs
  - zircon_plugins/particles/runtime/src/render/runtime_prepare.rs
  - zircon_plugins/particles/runtime/src/render/gpu/backend.rs
  - zircon_plugins/particles/runtime/src/render/gpu/planner.rs
  - zircon_plugins/particles/runtime/src/render/gpu/runtime_owner.rs
  - zircon_plugins/rendering/features/vfx_graph/runtime/src/lib.rs
  - zircon_runtime/src/scene/world/render_particles.rs
  - zircon_runtime/src/core/framework/render/frame_extract/particle.rs
  - zircon_runtime/src/graphics/particle_runtime_provider
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_plugin_graph_resources.rs
  - examples/vampire/scripts/vampire_game/main.zr
tests:
  - zircon_plugins/particles/runtime/src/tests
  - zircon_plugins/particles/runtime/src/render/gpu/backend/test_readback.rs
  - zircon_runtime/src/scene/tests/render_extract/particles.rs
  - zircon_runtime/src/graphics/tests/render_product_particle_velocity.rs
  - tests/acceptance/particles-gpu-readback-mailbox.md
plan_sources:
  - docs/plans/optimize/zircon_runtime/99d-runtime-particle-vfx-system-emitter-cpu-gpu-simulation-rendering-scalability-determinism-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md
  - docs/plans/optimize/zircon_editor/15-material-shader-graph-instance-vfx-particle-preview-compiler-diagnostics-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Public/NiagaraComponent.h
  - dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Private/NiagaraWorldManager.cpp
  - dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Private/NiagaraScalabilityManager.cpp
  - dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Private/NiagaraGpuComputeDispatch.cpp
  - dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Private/NiagaraRendererSprites.cpp
  - dev/Fyrox/fyrox-impl/src/scene/particle_system/mod.rs
  - dev/Fyrox/fyrox-impl/src/scene/particle_system/draw.rs
  - dev/godot/scene/3d/gpu_particles_3d.cpp
  - dev/godot/scene/3d/cpu_particles_3d.cpp
  - dev/godot/servers/rendering/renderer_rd/storage_rd/particles_storage.cpp
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor/Data/VFXDataParticle.cs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor/Compiler/VFXGraphCompiledData.cs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor/Compiler/VFXCodeGenerator.cs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Shaders/VFXCommon.hlsl
  - dev/bevy/crates/bevy_render/src/extract_component.rs
  - dev/bevy/crates/bevy_render/src/render_asset.rs
  - dev/bevy/crates/bevy_render/src/gpu_readback.rs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime Particle / VFX 当前源码工程化差距

## 1. 结论

当前 `zircon_plugins/particles` 已经有真实的 CPU SoA/free-list、rate/burst、shape、lifetime、gravity/drag、曲线、seed、sprite extract，以及真实 WGSL ping-pong、alive compaction、counter、indirect draw、透明绘制和有限 readback。它不是空的 descriptor，也不应被重写成另一个临时 demo。

但它还不是工程级 Particle/VFX runtime。`ParticleSystemComponent` 没有接入 Scene/ECS 的存储、序列化、attach/detach 和 scheduler；`ParticlesManager` 以可 clone 的 `Arc<Mutex<_>>` 同时承载所有实例、preview、诊断和模拟。GPU 资产在 manager 中先走 CPU fallback，而 `ParticleGpuRuntimeOwner` 又从同一组件副本重新规划和推进 GPU 状态，形成两条事实源。

GPU 部分虽然会创建 compute pipeline，但所有 playing 实例会被重建为一个按当前帧拼接的 aggregate asset，容量按 emitter 顺序争用；`ParticleGpuFramePlanner` 只将曲线压成首尾两个值，`ParticleGpuFrameParams::expected_frame_extract` 和 `render/extract.rs` 仍用 live/spawn count 合成 alive 与 indirect 参数。Runtime prepare 录制 compute 后才把 buffer 注册为 graph external resource，graph 中的粒子 compute executor 没有成为真实执行所有者。VFX Graph 更直接：编译结果只是 pass 名称和诊断，dispatch 永远 `[1,1,1]`，两个 executor 都是 `Ok(())`。

因此本轮不新增 P0：package/feature 当前仍标记 experimental/Partial 且默认关闭，Editor15 的菜单可见但无真实 operation/preview 的 P0 继续作为既有唯一计数。本轮新增 **24 项 P1、8 项 P2、18 项资格门**。任何把粒子或 VFX 标为 Complete、required、默认启用或声称性能超过 Unreal 的变更，在下述门关闭前必须 fail-close。

## 2. 审查边界与冻结统计

本轮逐段读取 `package -> component/asset -> manager lifecycle -> CPU pool/simulation -> GPU layout/planner/program/backend -> runtime owner -> runtime prepare/readback -> render graph/transparent renderer -> core Scene extract/history -> product script -> tests`，并以参考引擎的 world owner、compiled program、GPU dispatch、renderer family 和 scalability 作为对照。未运行 Cargo、WGPU、Editor、RenderDoc、GPU profiler 或产品场景；这是 review-only。

| 范围 | 文件 | 行数 | bytes | test attributes | ignored | fingerprint |
|---|---:|---:|---:|---:|---:|---|
| `zircon_plugins/particles/runtime`（Rust/TOML/ZUI） | 44 | 7,336 | 280,249 | 45 | 1 | `eb5971f9a2e93d48fccb81aeb4068f399139484e1958a76418613b66ecbb9897` |
| `zircon_plugins/particles/editor` + `dist`（Rust/TOML/ZUI） | 12 | 709 | 31,183 | 3 | 0 | `018a766b230305578548d770066423b7ba3080b33225684207db11c02e37a067` |

指纹按路径排序、逐文件 SHA-256，再对 `path<TAB>hash<LF>` manifest 做 SHA-256。实施前必须对所有重叠代码重取指纹；工作树中的既有修改未回退。

## 3. 可保留底座

| 领域 | 事实 | 保留条件 |
|---|---|---|
| CPU storage | `CpuParticlePool` 使用 alive/free-list/SoA，生命周期和 previous position 有实际数据 | 转为 world-owned pool，避免每帧全局单锁与全 slot 扫描 |
| CPU semantics | rate、burst、seed、四种 shape、local/world、gravity/drag、size/color curve 和有限值验证存在 | 由统一 module IR 驱动，补 fixed-step、substep、collision、sub-emitter 和 deterministic receipt |
| GPU primitives | backend 创建真实 WGSL、双缓冲、compact、counter、indirect args 与 transparent pipeline | 迁入 graph executor，按 program generation 与 device generation 管理资源 |
| render contract | camera sorting、bounds、layer、depth、previous-state history、GPU readback DTO 存在 | 改为 identity/generation keyed 的 simulation packet，不让 DTO 猜测 GPU 状态 |
| diagnostics | runtime diagnostic 队列有 256 上限、sequence、分页和 acknowledge | 增加结构化 code、severity、world/instance identity、drop reason 和 telemetry sink |

## 4. 五引擎对照裁决

### Unreal Niagara

`NiagaraWorldManager`、`NiagaraSystemSimulation` 和 `NiagaraScalabilityManager` 将 world、tick group、并发 batch、pool、significance、distance/instance budget 和 cull 决策作为正式 owner。`NiagaraGpuComputeDispatch` 管理 dispatch、free-ID、sort、readback latency、profiling 和低延迟透明路径；Sprite/Mesh/Ribbon renderer family 各自拥有材质、排序、motion vector、visibility 和资源策略。Zircon 当前把这些职责压在 `ParticlesManager`、一个 aggregate backend 和单一 billboard extract 上。

### Unity Visual Effect Graph

`VFXDataParticle` 保存 capacity/aligned capacity、attribute liveness、strip/bounds 和 context flow；`VFXGraphCompiledData` 与 `VFXCodeGenerator` 产出稳定的 buffer/attribute layout、事件、indirect、shader 和可安装的 compiled data。Zircon `VfxGraphCompileReport` 只有两个字符串和诊断，没有 IR、source map、artifact、event contract 或 renderer output set。

### Godot / Fyrox

Godot CPU/GPU 粒子公开 one-shot、preprocess、fixed FPS、fractional delta、interpolation、amount ratio、visibility AABB、draw order、trail、sub-emitter 和 collision material；Fyrox 将粒子作为 Scene node，使用 Reflect/Visit/InheritableVariable 保存 emitter、material、playing、free-list、RNG、距离和坐标系。即便这些引擎规模较小，也没有把粒子留在动态 JSON 或 preview-only manager 中。

### Bevy

本地 Bevy checkout 没有第一方 Niagara 级粒子 runtime。本报告只采用其 MainWorld/RenderWorld `ExtractComponent`、`RenderAsset` prepare/retry 和 GPU readback pool 的 ownership contract，不把 Bevy 的缺少粒子实现误当作 Zircon 的功能许可。

## 5. P1 差距与重构要求

| ID | 当前证据 | 工程化重构 |
|---|---|---|
| RT-PFX-01 | `ParticleSystemComponent` 只有 entity/asset/transform/playing/time_scale，`particle_component_descriptors` 只是 descriptor；Scene `render_particles.rs` 仍扫描 `render.particle_sprites`/`gameplay.particle_sprites` 动态 JSON | 建 versioned typed `ParticleSystemComponent` carrier、asset reference、reflect/serde schema、attach/detach hooks；shipping 删除 JSON authority，仅保留显式 migration diagnostic |
| RT-PFX-02 | 没有 `ParticlesManager::tick` 的生产 scheduler caller；manager 的 `tick` 在 service.rs:240 只遍历自身 map | 建 `ParticleWorldRuntime`，绑定 WorldId/generation、fixed/update tick group、pause/time dilation、shutdown/retirement receipt，并在 Scene schedule 中注册唯一 caller |
| RT-PFX-03 | service.rs:188 instantiate 把 GPU backend 无条件设为 `fallback_to_cpu`，同时提示没有 renderer executor；runtime owner 又从 `gpu_runtime_instances()` 复制组件 | 让 capability/admission 先产生 `SimulationBackendDecision`，CPU/GPU 只能一个 owner；GPU fallback 必须是带原因和 generation 的迁移，不得两边都推进 |
| RT-PFX-04 | service.rs:240 全局 mutex 锁住所有实例；service.rs:452 `expect` 在 poison 时 panic | 按 world 分区、按 instance slot/command buffer 写入；读快照使用 epoch/Arc swap；poison 转成可恢复 runtime error，禁止 hot path panic |
| RT-PFX-05 | service.rs:249 rewind_preview 同步 reset 后循环到 playback 秒数 | Preview 使用独立 PreviewWorld、可取消 fixed-step job、最大步数/预算和结果 receipt；runtime seek/warmup 复用同一 timeline contract |
| RT-PFX-06 | apply_animation_event 在无 handle 时用 entity 的第一项匹配，多个 emitter 时语义不确定 | 使用 stable instance/emitter identity、event sequence、target generation 和明确 fan-out policy；无 target 必须返回诊断而不是静默成功 |
| RT-PFX-07 | `ParticleSystemAsset` 只有 emitter 参数；material/texture 是可选 handle，无 renderer family、blend/depth、sub-emitter、collision、LOD、trail 或 output schema | 引入 `ParticleSourceDocument -> ParticleSemanticIR -> CompiledParticleProgram`；属性、module、event、renderer output、bounds 和 feature requirements 都进入版本化 artifact |
| RT-PFX-08 | CPU update 在 simulation/cpu.rs:280 附近逐个 slot 更新；没有 fixed-step accumulator、substep、collision query、sub-emitter 或 deterministic replay receipt | pool 保留 SoA，但用 fixed-step scheduler、per-system step budget、SIMD/worker batch、collision adapter、sub-emitter queue 和 seed/state checksum |
| RT-PFX-09 | `render/extract.rs:33` 每次构造 `previous_sprites: Vec::new()`；velocity history 由 core viewport 后置补齐，GPU 粒子没有 per-particle previous state | simulation packet 输出 current/previous state、stable particle id、generation、velocity/reactive flags；GPU path 产生真实 previous buffer 或明确 velocity fallback |
| RT-PFX-10 | `render/extract.rs:40-60` 将 GPU emitter 的 live count 当 `spawned_total`、alive count 和 `[6,count,0,0]` indirect args | readback/indirect 只接受 GPU counter 写入的 authoritative frame，附 frame index/latency/stale/drop；没有 GPU 结果时输出 `Unavailable` 而不是伪造计数 |
| RT-PFX-11 | runtime_owner.rs:111-150 将所有 playing 实例按顺序拼成 `runtime-prepare-aggregate` asset；asset/topology 改变会重建整个 backend | 建 persistent per-system/per-program allocation table、free-ID allocator、capacity admission/eviction、program generation 和 per-instance state；聚合只能是批处理，不得成为 state owner |
| RT-PFX-12 | aggregate_frame_for:235-265 收集每实例 frame 后仍使用单个 max_dt/age packet；每 emitter 只保留独立 dt，但 aggregate 的状态、loop、seed、events 未保留 | 以 instance packet 传递独立 clock/direction/loop occurrence/seed，并在 GPU buffer 中保留 instance/emitter offsets；禁止以 max age/asset concat 代表系统语义 |
| RT-PFX-13 | planner.rs 的 `size_curve`/`color_endpoint` 只取首尾 key；多 key 仅 compile warning | 编译完整曲线为采样纹理/压缩段或可执行 expression，保留 interpolation、precision、source map；超限必须选择明确 CPU fallback 并记录 artifact decision |
| RT-PFX-14 | runtime_prepare.rs 在 collector 中调用 `owner.execute_instances` 直接录制 compute，然后注册 static external buffer；粒子 graph pass 只声明资源 | graph pass executor 负责真实 dispatch、barrier、queue ownership、timestamp 和 resource lifetime；prepare 只完成 upload/admission，禁止图外执行改变 graph 语义 |
| RT-PFX-15 | particles/render/feature.rs 的四个 workload 固定 `[1,1,1]`；VFX Graph lib.rs:23/104-107 同样固定 `[1,1,1]` | dispatch extent 必须绑定编译 program 的 particle capacity/active count/indirect mode，支持 zero work、overflow、device limits、async fence 和 profiling |
| RT-PFX-16 | `compile_vfx_graph` 仅验证是否有 SpawnRate/ShaderGraphMaterial，返回 pass 字符串；两个 executor 在 lib.rs:124-131 都是 `noop_render_executor` | VFX Graph 与 particles 共用唯一 IR/compiler/artifact；实现 attribute liveness、event/context graph、module validation、shader/material compile、renderer outputs、source map、install receipt，去掉第二套 no-op runtime |
| RT-PFX-17 | core binder 对未提供的 particle external buffer 可绑定 neutral/fallback；neutral frame 会把计数 clamp 后继续 | fallback 只可用于无实例/设备初始化，并带 required/optional 资源等级；shipping GPU 缺资源必须 fail-close 或明确 CPU mode，不能让 neutral 伪造可见效果 |
| RT-PFX-18 | `ParticleEmitterAsset` 的 bounds 由 live sprites AABB 推导，GPU aggregate 只使用摘要；没有 conservative authored bounds、visibility AABB、LOD/significance | 将 authored conservative bounds 与 dynamic bounds 分开，接入 visibility/culling、distance/instance budget、significance、LOD、quality tier、hysteresis 和 per-world telemetry |
| RT-PFX-19 | material/texture 只挂在 sprite snapshot；particle render feature 没有 material family、blend mode、soft particle、lighting、ribbon/mesh/trail 输出 | 建 renderer registry（sprite/mesh/ribbon/volume/decal）、material domain contract、depth/scene-color/velocity/reactive inputs 和 PSO/cache key；每种 family 有独立 draw/indirect ABI |
| RT-PFX-20 | GPU backend 的 buffers 没有 device generation、residency、retirement、device-loss recovery；capacity 只在 layout compile 时 clamp | 由 graphics resource owner 管理 device epoch、recreate/retire、budget/residency、readback age/drop、pipeline cache 和 validation receipt；设备丢失不能静默沿用旧 bindings |
| RT-PFX-21 | diagnostics 有界但 capability 缺失时 physics/animation 模块变成 no-op warning；没有 cause/effect/parameter source | 将 optional module 编译为 capability requirement，runtime admission 决定 reject/degrade；每次 degrade 输出结构化 reason、scope、frame 和 recovery action |
| RT-PFX-22 | 旧产品路径仍把 `Vampire` 粒子作为最终 JSON sprite 写入 Scene render extraction | 迁移 gameplay API 为 `spawn_system/stop/set_parameter/send_event` command；脚本只持有 stable handle，render extract 只消费 simulation packet |
| RT-PFX-23 | 现有测试集中在 descriptor、snapshot、neutral readback 和 source guards；未证明多 world、1000+ emitters、GPU/CPU equivalence、device loss、frame budget 或 network/replay | 建 required test matrix：CPU/GPU golden、fixed-step/replay hash、capacity overflow、aggregate isolation、graph execution trace、readback latency/drop、device loss、multi-viewport、stress/benchmark 和 product scene acceptance |
| RT-PFX-24 | package/feature manifest 与 `rendering.vfx_graph` 仍是 optional/Partial，catalog/runtime/editor/dist 不能安装同一 compiled artifact | 建 manifest 的 compiler/schema/device capability fingerprint、artifact closure、runtime install receipt、rollback/retirement 和 profile gate；未闭合时保持 experimental |

## 6. P2 性能与质量差距

| ID | 当前差距 | 需要重构 |
|---|---|---|
| RT-PFX-25 | CPU pool update 对 alive 数组做全 slot 扫描 | 维护 dense alive list + sparse slot map，允许 worker chunk 和 SIMD 更新 |
| RT-PFX-26 | snapshot 遍历所有实例并复制/排序 sprite；没有 per-camera packet cache 或 dirty range | 按 world/viewport/camera generation 缓存、增量上传和分层排序 |
| RT-PFX-27 | sprite sorting 以 CPU distance sort 为主，GPU path 没有 depth/key sort contract | 为透明 renderer 提供 GPU radix/bitonic sort、stable key、sort budget 和 fallback telemetry |
| RT-PFX-28 | emitter `id` 通过 aggregate 字符串拼接生成，长度/分配随 frame topology 变化 | 使用 interned stable IDs 与 numeric instance/emitter index，字符串仅用于诊断 |
| RT-PFX-29 | readback 只请求 counters/indirect，反馈没有 cadence、frame age、drop 或 consumer | 定义 readback policy、latency budget、stale watermark、consumer acknowledgment 和 telemetry |
| RT-PFX-30 | shader/pipeline cache 是 backend 实例级，aggregate 变化可能重编译 | 按 compiled program fingerprint + device profile 做共享 PSO/cache，异步编译并可回退 |
| RT-PFX-31 | bounds 只以 sprite size 建球，未区分 billboard orientation、trail、mesh 和 conservative authored bound | 每种 renderer 输出 bounds contract，并在 visibility 中使用 generation-aware conservative volume |
| RT-PFX-32 | CPU/GPU curve、physics、coordinate semantics 不是同一执行 IR | 统一 scalar/vector/color expression bytecode 或生成式 IR，并有 CPU/GPU conformance corpus |

## 7. 资格门

当前裁决为 **16 Fail / 2 Partial / 0 Pass**：G14（readback mailbox 局部存在）与 G15（局部 CPU/GPU 测试存在）为 Partial，其余均 Fail；没有任何门可以记为 Pass。

| Gate | 必须证明 |
|---|---|
| RT-PFX-G01 | Scene/ECS typed component 可保存、加载、attach、detach、clone 并保留 asset/reference generation |
| RT-PFX-G02 | 每个 world 只有一个 ParticleWorldRuntime owner，scheduler、pause、shutdown 与 handle stale 行为可追踪 |
| RT-PFX-G03 | CPU/GPU backend decision 是单一 authority，fallback 不会双重推进或双重提交 |
| RT-PFX-G04 | fixed-step、substep、loop、seek、warmup、rewind、time dilation 和 replay hash 在 CPU/GPU 有 golden 一致性 |
| RT-PFX-G05 | compiled program 有 schema/compiler/device fingerprint、attribute layout、module diagnostics、source map 和 install receipt |
| RT-PFX-G06 | VFX Graph 与 Particle asset 使用同一个 IR/artifact，不存在字符串-only 或第二套 no-op compiler |
| RT-PFX-G07 | Render Graph 真正拥有 compute/transparent executor、resource lifetime、barrier、queue sync、timestamp 和 dispatch extent |
| RT-PFX-G08 | GPU dispatch 按 active/capacity 动态计算，正确处理 zero/overflow/device limit，而非固定 `[1,1,1]` |
| RT-PFX-G09 | GPU counters、alive IDs、indirect args、previous state 和 readback frame identity 可证明为 authoritative |
| RT-PFX-G10 | persistent per-system allocation、free-ID、capacity admission、eviction、retirement 与 device loss 已覆盖压力测试 |
| RT-PFX-G11 | renderer registry 至少覆盖 sprite/mesh/ribbon 或明确产品边界，每个 family 有 material/depth/blend/velocity contract |
| RT-PFX-G12 | visibility bounds、culling、significance、distance/instance budget、quality tier、hysteresis 与 telemetry 有生产 caller |
| RT-PFX-G13 | physics/animation/sub-emitter/event module 缺失时采用 reject/degrade policy，不静默 no-op |
| RT-PFX-G14 | GPU readback 有 cadence、age、drop、stale、ack 和 consumer；neutral buffer 不伪造 gameplay/render truth |
| RT-PFX-G15 | CPU pool、GPU buffer、snapshot、sorting 和 upload 在目标规模下有 benchmark 与 frame budget receipt |
| RT-PFX-G16 | device loss、shader compile failure、asset missing、capacity overflow 和 stale handle 都能恢复或 fail-close |
| RT-PFX-G17 | 产品脚本不再写 `particle_sprites` 最终 JSON；只发送 typed command/event 并获得 handle/receipt |
| RT-PFX-G18 | 必需 test matrix 与真实产品场景运行；静态 source guard、旧 acceptance Markdown 和 neutral test 不得单独记通过 |

## 8. 推荐实施顺序

1. 先冻结 `ParticleSourceDocument`、typed Scene component、`ParticleSemanticIR`、compiled artifact schema 和 world/instance identity；同时硬切 JSON 产品旁路。
2. 建 `ParticleWorldRuntime` 与 scheduler/fixed-step/preview job，迁移 CPU pool 和 event/animation/physics command，加入 generation、budget、replay receipt。
3. 让 GPU program 由同一 compiler 生成，建立 persistent allocation 和 backend decision；把真实 compute/transparent recording 移入 Render Graph executor，补动态 dispatch、barrier、device-loss 和 readback identity。
4. 建 renderer registry、material/depth/velocity/visibility/scalability contract；再实现 mesh/ribbon/trail/sub-emitter 等扩展，而不是继续增加单一 sprite 字段。
5. 最后接 Editor authoring/preview/telemetry 和 catalog/install receipt；所有 profile gate 通过后才从 experimental/Partial 改状态。
