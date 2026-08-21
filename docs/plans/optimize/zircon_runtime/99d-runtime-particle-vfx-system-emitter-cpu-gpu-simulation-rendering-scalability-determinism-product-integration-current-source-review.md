---
related_code:
  - zircon_plugins/particles
  - zircon_plugins/rendering/features/vfx_graph
  - zircon_plugins/rendering/runtime/src/lib.rs
  - zircon_runtime/src/core/framework/render
  - zircon_runtime/src/graphics/particle_runtime_provider
  - zircon_runtime/src/graphics/runtime_prepare_collector
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/particle_previous_sprites.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/particle.rs
  - zircon_runtime/src/graphics/visibility/declarations/visibility_particle_upload_plan.rs
  - zircon_runtime/src/graphics/visibility/planning/build_particle_upload_plan.rs
  - zircon_runtime/src/scene/world/render_particles.rs
  - zircon_runtime/src/script/vm/gameplay_host.rs
  - zircon_runtime/src/script/vm/gameplay_host/lifecycle.rs
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/rendering/workbench_vfx_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/rendering/workbench_extension_particle_library_workspace.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/render_asset_vfx.rs
  - examples/vampire/scripts/vampire_game/main.zr
tests:
  - zircon_plugins/particles/runtime/src/tests
  - zircon_plugins/particles/runtime/src/render/gpu/backend/test_readback.rs
  - zircon_plugins/particles/editor/src/tests.rs
  - zircon_plugins/rendering/features/vfx_graph/runtime/src/lib.rs
  - zircon_runtime/src/graphics/tests/m4_behavior_layers/particles.rs
  - zircon_runtime/src/graphics/tests/plugin_feature_compile/particle.rs
  - zircon_runtime/src/graphics/tests/render_product_anti_alias/particle.rs
  - zircon_runtime/src/graphics/tests/render_product_particle_velocity.rs
  - zircon_runtime/src/scene/tests/render_extract/particles.rs
  - tests/acceptance/particles-gpu-readback-mailbox.md
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/08a-physics-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/09h1-temporal-aa-velocity-history-upscaling-review.md
  - docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md
  - docs/plans/optimize/zircon_runtime/23-coordinate-space-unit-precision-transform-numeric-robustness-large-world-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/26-particle-vfx-system-emitter-cpu-gpu-simulation-rendering-scalability-determinism-product-integration-review.md
  - docs/plans/optimize/zircon_editor/15-material-shader-graph-instance-vfx-particle-preview-compiler-diagnostics-authoring-review.md
  - docs/plans/optimize/zircon_plugins/09-first-party-particle-vfx-source-runtime-editor-dist-catalog-simulation-render-product-integration-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Public/NiagaraComponent.h
  - dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Private/NiagaraWorldManager.cpp
  - dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Private/NiagaraScalabilityManager.cpp
  - dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Private/NiagaraGpuComputeDispatch.cpp
  - dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Private/NiagaraRendererSprites.cpp
  - dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Private/NiagaraRendererMeshes.cpp
  - dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Private/NiagaraRendererRibbons.cpp
  - dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Private/NiagaraSystemSimulation.cpp
  - dev/Fyrox/fyrox-impl/src/scene/particle_system/mod.rs
  - dev/Fyrox/fyrox-impl/src/scene/particle_system/draw.rs
  - dev/Fyrox/fyrox-impl/src/scene/particle_system/particle.rs
  - dev/Fyrox/fyrox-impl/src/scene/particle_system/emitter/base.rs
  - dev/godot/scene/3d/gpu_particles_3d.cpp
  - dev/godot/scene/3d/cpu_particles_3d.cpp
  - dev/godot/servers/rendering/renderer_rd/storage_rd/particles_storage.cpp
  - dev/godot/scene/resources/particle_process_material.cpp
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor/Data/VFXDataParticle.cs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor/Compiler/VFXGraphCompiledData.cs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor/Compiler/VFXCodeGenerator.cs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Shaders/VFXCommon.hlsl
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Runtime/Types/VFXTypes.cs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Runtime/Utilities/VFXRuntimeResources.cs
  - dev/bevy/crates/bevy_render/src/extract_component.rs
  - dev/bevy/crates/bevy_render/src/render_asset.rs
  - dev/bevy/crates/bevy_render/src/gpu_component_array_buffer.rs
  - dev/bevy/crates/bevy_render/src/gpu_readback.rs
  - dev/bevy/crates/bevy_render/src/batching/gpu_preprocessing.rs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime Particle / VFX System、Emitter、CPU-GPU Simulation、Rendering、Scalability、Determinism 与 Product Integration 当前源码工程化差距

## 1. 结论

当前粒子实现已经越过“只有descriptor”的阶段。CPU侧有SoA/free-list、有限值校验、局部确定seed、rate/burst、四种shape、lifetime/velocity/gravity/drag、size/color curve、local/world space和sprite extract；GPU侧有真实WGSL、ping-pong state、alive compaction、counter、indirect draw、offscreen透明绘制和readback；Core renderer有camera layer、depth/overlay、CPU previous-state与velocity输出。这些是应保留的工程基础。

当前工作树还新增了两项实质进展：`ParticleRuntimeSnapshot`的sprite/diagnostic payload改为`Arc<[T]>`共享，未变化snapshot不再深拷贝；diagnostic改为256条有界队列、64条分页、单调sequence、stale cursor与acknowledge。旧Runtime26的“diagnostic无界”和“每次snapshot全量复制payload”结论已经过时，不能继续作为当前缺口。

但产品权威链仍未成立。全仓生产搜索仍没有`ParticlesManager::tick`的scheduler调用，`ParticleSystemComponent`没有进入Scene/ECS load/save/attach/detach；Vampire仍通过`gameplay.set_particle_sprites`写dynamic JSON最终sprite。GPU资产又在manager中推进CPU fallback，而renderer的`ParticleGpuRuntimeOwner`另行推进GPU state。产品旁路、CPU fallback和GPU owner仍是三条不同事实源。

GPU与Render Graph的边界比旧报告揭示得更严重：spawn/update、compact、indirect args在`RuntimePrepareCollector`中通过owner直接录制并执行，之后才把已执行buffer登记为static external resource；Render Graph内三个compute executor只验证队列和resource contract，不录制compute。图声明的async compute、barrier、profiling和资源读写并不拥有真实工作。独立`rendering.vfx_graph`又固定声明`[1,1,1]` workload，simulation/transparent两个executor仍直接`Ok(())`。

旧Runtime26关于“aggregate用全局`max_dt`推进所有emitter”的判断应正式撤回。`ParticleGpuFrameParams.dt`确实保存最大值，但每个`ParticleGpuEmitterFrameParams`编码自己的`dt`，WGSL读取`emitter.sim.y`；顶层`max_dt`没有成为emitter simulation输入。仍开放的是：所有playing实例共用一个aggregate asset/backend，拓扑、暂停或asset变化可重建整个backend并丢失其他系统状态；1,048,576 slot按顺序争用；没有world budget、persistent allocation、device generation或retirement。

本轮不新增P0。`particles`仍诚实标记`experimental/Partial`，GPU/physics/animation optional feature默认关闭；VFX Graph也默认关闭。Editor15拥有菜单可见但compile/simulate/preview假成功的P0，本篇不重复计数。任何profile在以下资格门关闭前把粒子/VFX提升为Complete、required、默认启用或“优于Unreal”，都必须fail-close。本篇用当前源码重新归并为 **0项新增P0、48项P1、12项P2和44项资格门**；它取代Runtime26的currentness，数量不与旧60/12简单相加。

## 2. 审查边界、currentness与证据

### 2.1 冻结语料

| 范围 | 文件 / 行 / 非空行 / bytes / test attributes / ignored | 证据等级 | fingerprint |
|---|---:|---|---|
| Zircon production / contract / product slice | 106 / 18,409 / 16,916 / 675,558 / 78 / 0 | E3主链逐段读取；E2 owner与调用扫描 | `266455362cdf75cc98005f6dd68c21c89b9620ff024606e61fe4714165eae9ec` |
| dedicated tests / acceptance | 20 / 3,373 / 3,105 / 125,276 / 53 / 1 | E2/E3断言与skip分支分类；未执行 | `2497f972e051556a44893b723d44c7b56b58d17bcc33b067bce52e4bf467c7c9` |
| Unreal Niagara | 8 / 16,308 / 13,928 / 705,713 | E3 | `e2115ce08eb2020fd66b3657f04677fd784e89abe8d06ea38b51422546bd24e8` |
| Unity Graphics VFX Graph | 6 / 5,349 / 4,630 / 234,949 | E3 | `bc74a3ee287f91f695b880fe9577cee29813e34f71f43f1ff703faf001bc53b8` |
| Godot particles | 4 / 7,759 / 6,583 / 339,325 | E3 | `ce118c2c28b243c58abe349fa402da1b11ca5781f48737f76cd88f76eb730cfd` |
| Fyrox particle | 4 / 1,478 / 1,313 / 52,623 | E3 | `674bc7d6cf6dafb07325f11287694e5665304d04a0c93bede3e93d820f0054de` |
| Bevy render architecture | 5 / 4,115 / 3,745 / 161,924 | E3/E2 | `d7029124106fc7630ed3507bcb7eb44a58fb623dacda7ca5aeb0c4aab5c73fee` |
| combined reference slice | 27 / 35,009 / 30,199 / 1,494,534 | E3/E2 | `c7a925264ef06f0ab80494a709b8611952e28ec90a9abcd2ee6a1f783730ea2a` |

fingerprint算法为：路径排序后，对每个working-tree文件计算SHA-256，再对UTF-8 `path<TAB>hash<LF>` manifest计算SHA-256。冻结对象是2026-08-22共享working tree，基线HEAD为`be5a281c96b6dc9d33b5c9d0f2699a8bf75afcf1`，coordinator epoch为336；不是只读HEAD快照。

冻结时`particles/runtime/src/{lib.rs,service.rs,simulation/cpu.rs,render/extract.rs,tests/mod.rs}`有在途修改，`tests/snapshot.rs`为untracked新增文件；本报告按磁盘现状读取并纳入指纹，不修改或回退这些代码。进入实现前必须重新取指纹并对受影响结论做source recheck。

Bevy、Fyrox、Godot与Unity Graphics revision分别为`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`与`a7e4c051d256a781ab362c64316b125a1e104694`。Unreal目录不是独立Git checkout，只使用上述8文件及manifest fingerprint，不伪造revision。

### 2.2 数据链读取深度

本轮沿`package/profile -> component/asset -> manager lifecycle -> CPU simulation/pool -> GPU planner/program/backend/runtime owner -> runtime prepare -> graph executor -> transparent renderer/readback -> core particle renderer/history -> Scene JSON -> Script/Vampire -> Editor contribution/template -> tests/acceptance`闭环读取。对生产slice做全量owner/caller搜索，对asset、service、CPU、GPU、graph、Scene旁路、Editor入口和VFX Graph逐函数E3读取。

### 2.3 明确未做

本轮是review-only，没有修改Rust、Cargo、WGSL、asset、Editor或tooling；没有运行Cargo、WGPU、Editor、RenderDoc、参考引擎、GPU profiler或产品场景。没有用可选adapter测试、旧acceptance Markdown或类型注册证明当前GPU/产品通过。tooling按用户要求排除，后续迁移Rust。

## 3. 当前可保留基础与旧结论校正

| 项目 | 当前源码裁决 | 后续处理 |
|---|---|---|
| package truth | `particles`为experimental/Partial，GPU/physics/animation默认关闭 | 保留fail-close；产品门关闭后才能升级 |
| CPU storage/simulation | SoA、free-list、有限值验证、局部seed和基础module真实存在 | 保留数据布局，替换全slot扫描和单锁world owner |
| GPU execution | compute、alive compaction、indirect draw和透明像素写入是真实执行 | 迁入Render Graph真正的pass executor，不保留图外执行 |
| CPU render history | per viewport-camera previous sprites与velocity可用 | 扩展generation identity，并补GPU velocity/reactive history |
| snapshot/diagnostic | payload共享；diagnostic有界、分页、sequence、ack | 旧“无界/全量clone”结论关闭；继续补结构化code和性能资格 |
| GPU readback mailbox | counter/indirect可转neutral output并存取 | 尚无自动runtime feedback consumer；补cadence/age/drop/stale |
| Scene extraction | 从dynamic-component owner扫描，避免探测所有entity | 只是旁路性能改进，仍应hard cut到typed component query |
| GPU `max_dt` | 每emitter编码独立dt，shader读`emitter.sim.y` | 撤回旧“全局最大dt推进所有emitter”结论 |
| GPU aggregate | 单backend聚合所有playing实例，aggregate变化重建 | 当前仍开放，改为persistent per-system/per-generation allocation |
| VFX Graph | 五节点检查、固定pass与no-op executor | 不作为第二runtime；吸收到canonical particle compiler |

## 4. 五引擎参考对照

### 4.1 Unreal：world owner、simulation batch与scalability是主边界

`UNiagaraComponent`提供pooling、desired age/seek、warmup、reset/reinitialize和scalability cull语义；`NiagaraWorldManager`按world、tick group、component pool和system simulation组织运行；`NiagaraSystemSimulation`明确拆分game-thread、concurrent batch和finalize task；`NiagaraScalabilityManager`按effect type、更新频率、significance、instance count和budget决策。Zircon不能继续让一个可克隆全局mutex同时承担world、preview、diagnostic和simulation owner。

### 4.2 Unreal：GPU dispatch和renderer family不是一个colored billboard pass

`NiagaraGpuComputeDispatch`有dispatch、sort、free-ID、readback latency、GPU profiling与low-latency translucency路径；Sprite renderer消费material、alignment/facing、sub-image、sort、visibility、distance cull和accurate motion vector，Mesh与Ribbon renderer拥有独立资源和策略。Zircon需要可编译renderer set与共享simulation packet，而不是把所有表现锁死在单一RGBA billboard。

### 4.3 Unity Graphics VFX Graph：graph必须产出可复用compiled program

`VFXDataParticle`维护capacity/aligned capacity、current/source attributes、strip、bounds与context flow；compiled data/code generator生成system、buffer、attribute layout、expression、event、indirect和shader资源。Zircon当前五节点结构只返回两个pass字符串，没有IR、attribute liveness、source map、target artifact或generation，不能称为VFX compiler。

### 4.4 Godot与Fyrox：较小实现也有完整生命周期和持久化

Godot CPU/GPU particles公开one-shot、preprocess、fixed FPS、fractional delta、interpolation、amount ratio、visibility AABB、draw order、trail、sub-emitter与collision material。Fyrox粒子是Scene node，使用`Reflect`/`Visit`/`InheritableVariable`保存emitter、material、playing、particles、free list、RNG、visible distance和coordinate system。它们的规模都小于Niagara，但仍证明Scene持久化、fixed step、bounds、material和lifecycle不是高级可选项。

### 4.5 Bevy：只采用render-world contract，不作粒子功能降级许可

当前Bevy checkout没有第一方particle/VFX runtime。只采用其`ExtractComponent`的MainWorld/RenderWorld同步、`RenderAsset`的changed extraction/prepare/retry/bytes-per-frame以及GPU readback pool/event生命周期。粒子语义、renderer family和产品完整性仍由Zircon自身与Unreal/Unity/Godot/Fyrox裁决。

## 5. P0裁决

本轮 **0项新增P0**。原因是功能当前仍为experimental/Partial且默认关闭，而不是实现已达标。Editor15中“可见authoring/compile/simulate/preview产生固定成功结果”的P0继续开放且唯一计数；Plugins09继续拥有包级source/editor/runtime/dist/catalog纵向交付。以下任一变化都触发P0复核：默认启用VFX Graph或GPU simulation、profile标Complete/required、shipping产品依赖dynamic JSON旁路、菜单从disabled变为可操作但无真实handler、或对外宣称production-ready/超过Unreal。

## 6. P1工程差距

### 6.1 产品权威、资产与生命周期

| ID | 当前差距 | 需要重构 |
|---|---|---|
| Runtime103-P1-01 | `ParticlesManager::tick`没有生产scheduler调用者 | 建`ParticleWorldRuntime`并注册明确fixed/update stage、world generation与shutdown顺序 |
| Runtime103-P1-02 | Vampire/Script直接写最终sprite JSON | hard cut为spawn/stop/set-parameter/event command，产品只持typed instance handle |
| Runtime103-P1-03 | Scene同时接受`render.particle_sprites`与`gameplay.particle_sprites` | 删除shipping JSON authority；迁移reader只输出显式conversion diagnostic |
| Runtime103-P1-04 | `ParticleSystemComponent`不进入Scene storage、load/save、attach/detach | 建versioned typed component与asset reference roundtrip |
| Runtime103-P1-05 | manager是可克隆`Arc<Mutex<_>>`，无world/session owner | World持有唯一owner；所有handle绑定world/owner epoch |
| Runtime103-P1-06 | GPU asset在manager推进CPU fallback，renderer owner另推GPU | 每实例只允许一个authoritative backend；fallback发布generation迁移结果 |
| Runtime103-P1-07 | `rendering.vfx_graph`形成第二graph/runtime authority | 删除独立执行权，统一编译为`CompiledParticleProgram` |
| Runtime103-P1-08 | asset只是`Clone/PartialEq` Rust结构，无serde/schema/version/importer/artifact | 建`ParticleSourceAsset`、schema migration、semantic compiler与derived artifact |
| Runtime103-P1-09 | `looped`可author但没有执行consumer | 定义one-shot/loop/duration/completion/restart状态机并由CPU/GPU共同执行 |
| Runtime103-P1-10 | handle在`u64::MAX`后饱和复用并可覆盖live instance | 使用slot+generation+owner epoch；耗尽时拒绝或retire，不覆盖 |
| Runtime103-P1-11 | play/pause/stop缺completion、warmup、seek、pool和event receipt | 建显式instance lifecycle与terminal outcome |
| Runtime103-P1-12 | tick、rewind、snapshot、control共用一把锁；rewind可在锁内无界循环 | 分离command queue/simulation snapshot；warmup/seek受step与time budget约束 |

### 6.2 Simulation、determinism与interop

| ID | 当前差距 | 需要重构 |
|---|---|---|
| Runtime103-P1-13 | CPU update/live count扫描全部allocated slot | active set/chunk/job化，dead slot不进入每帧主循环 |
| Runtime103-P1-14 | 本tick新spawn粒子被推进完整`dt` | 按spawn time/fractional step积分，定义burst与rate的frame partition不变性 |
| Runtime103-P1-15 | 无fixed step、substep、catch-up cap与interpolation | 接入Runtime22 clock domain并生成step receipt/degradation |
| Runtime103-P1-16 | 自定义RNG只有局部可重复，无algorithm ID/stream key/draw counter | 使用versioned random stream并可snapshot/replay/migrate |
| Runtime103-P1-17 | GPU多key size/color curve只降为首尾端点 | compiler生成curve LUT或piecewise program，CPU/GPU共享oracle |
| Runtime103-P1-18 | CPU/GPU字段与module parity未定义 | 编译时生成backend support matrix；不支持时拒绝或显式qualified fallback |
| Runtime103-P1-19 | world-space初速度没有按component transform旋转 | 建typed local/world/emitter/vector space转换并覆盖非均匀缩放 |
| Runtime103-P1-20 | physics collision只做damping，未查询physics world，`bounce`未消费 | 建batched query/collision event/response contract，CPU/GPU有可声明差异 |
| Runtime103-P1-21 | animation binding字段未求值；无handle时取entity首个instance | 绑定compiled parameter handle与instance identity，缺失/歧义fail-close |
| Runtime103-P1-22 | 无event handler、sub-emitter、parameter namespace或data interface | compiler生成spawn/update/event context和typed external binding |
| Runtime103-P1-23 | 无checkpoint、replay、network/save与backend migration state | 定义serializable simulation checkpoint和determinism tier |
| Runtime103-P1-24 | sprite stable key只有emitter index+slot，跨system可碰撞且复用无generation | 使用world/system/emitter/slot/generation组合identity |

### 6.3 GPU、Render Graph、资源与readback

| ID | 当前差距 | 需要重构 |
|---|---|---|
| Runtime103-P1-25 | 所有playing实例共用一个aggregate asset/backend | persistent per-program pool与per-instance allocation，不以全局重建同步拓扑 |
| Runtime103-P1-26 | pause/remove/asset/order变化可重建backend并重置其他系统 | generation-qualified migration、copy/retire和last-good fallback |
| Runtime103-P1-27 | 1,048,576 slot按遍历顺序先到先得，后emitter可获0 capacity | world budget、priority/significance、reservation和typed admission outcome |
| Runtime103-P1-28 | 上限约26 words/particle，双state约208 MiB且无pressure策略 | attribute liveness/packing、budget telemetry、eviction/LOD与platform tier |
| Runtime103-P1-29 | 实际compute在runtime prepare图外执行 | 把spawn/update、compact、indirect真正录制到graph pass executor |
| Runtime103-P1-30 | 三个compute executor只校验metadata | executor必须消费compiled workload/resource handles并产生execution receipt |
| Runtime103-P1-31 | feature descriptor固定`[1,1,1]`，真实dispatch由backend另算 | graph compile从program/capacity派生dispatch，禁止双重描述 |
| Runtime103-P1-32 | 已执行buffer事后登记为static external resource | graph在执行前取得真实resource owner、version、queue和lifetime |
| Runtime103-P1-33 | GPU work未admit时复用旧active buffers，可能绘制陈旧状态 | 输出明确stale age/backend generation；超阈值停绘或降级 |
| Runtime103-P1-34 | pending readback queue本地无界且每admitted frame请求counter+indirect | 配置cadence、最大in-flight、drop/coalesce和no-readback常态路径 |
| Runtime103-P1-35 | 只检查FIFO队首，卡住的front阻塞后续ready项 | token化completion并按ready/age收割，超时产生typed failure |
| Runtime103-P1-36 | backend/pipeline同步创建，无device generation、retirement和共享PSO owner | 接入graphics device/PSO/shader cache与fence-qualified retirement |

### 6.4 Rendering、culling、Editor与产品闭环

| ID | 当前差距 | 需要重构 |
|---|---|---|
| Runtime103-P1-37 | CPU每sprite扩6 vertex并为depth/overlay/velocity分别新建buffer | instance buffer、ring allocator、batch/indirect与shared geometry |
| Runtime103-P1-38 | CPU/GPU最终只渲染颜色；material/texture句柄无真实消费，GPU rotation也未进入billboard | compiled renderer binding、UV/sub-image/material/rotation/facing/alignment |
| Runtime103-P1-39 | 无mesh/ribbon/trail/light/decal/volume renderer | renderer family registry，共享attribute packet并独立compile/qualify |
| Runtime103-P1-40 | 无GPU sort/binning、soft particle、overdraw预算与OIT policy | 按material/view分类，提供sort key、depth fade、half-res/OIT与budget |
| Runtime103-P1-41 | GPU粒子无velocity/history/reactive输出 | 生成previous transform/position或明确reactive mask与history invalidation |
| Runtime103-P1-42 | visibility upload plan只比较entity membership，且无生产consumer | dirty frontier覆盖simulation/material/transform/generation并接入prepare |
| Runtime103-P1-43 | Scene JSON允许调用者自报`gpu_frame` count/bounds | telemetry只来自executed backend receipt；删除caller-authored GPU truth |
| Runtime103-P1-44 | JSON字段错误多为默认/忽略，material/texture固定None | typed parser/validator；迁移失败带entity/component/source diagnostic |
| Runtime103-P1-45 | VFX Graph compiler只检查三个条件并返回pass字符串，executor no-op | canonical IR、attribute liveness、context program、source map与target artifact |
| Runtime103-P1-46 | Particle Editor菜单全部disabled，ZUI主体是`Space`，operation无handler | Editor15建立transactional document、command handler和runtime preview gateway |
| Runtime103-P1-47 | `particles.system`/template只有注册与字符串测试，无runtime codec/importer | asset catalog、create/open/save/reimport/cook/runtime resolve闭环 |
| Runtime103-P1-48 | 没有首方产品从asset完成load/tick/render/reload/save-reopen；GPU测试可静默return | 建required product scene与typed GPU skip/failure receipt |

## 7. P2产品、诊断与治理差距

| ID | 当前差距 | 需要重构 |
|---|---|---|
| Runtime103-P2-01 | diagnostic虽有界但容量256/page64为硬编码 | 纳入runtime config、memory budget和per-code aggregation |
| Runtime103-P2-02 | diagnostic主要是自由文本，无code/source location/remediation | 结构化compiler/runtime diagnostic schema并可追到node/module/instance |
| Runtime103-P2-03 | snapshot性能门默认`#[ignore = "release performance gate"]` | release lane强制执行并保存机器、样本、percentile receipt |
| Runtime103-P2-04 | offscreen GPU测试在adapter/device失败时返回`None`并成功结束 | required matrix使用typed skip且不得满足GPU资格 |
| Runtime103-P2-05 | GPU parity主要验证count/indirect，不比较逐粒子attribute | 小场景CPU/GPU position/velocity/lifetime/color tolerance oracle |
| Runtime103-P2-06 | 无renderer visual corpus与temporal sequence | 保存source fingerprint、settings、image/sequence和metric |
| Runtime103-P2-07 | 无10k/100k/1M、burst storm、multi-view和overdraw矩阵 | 建quality-tier规模门、frame/GPU ms、VRAM、upload/readback budget |
| Runtime103-P2-08 | 无device loss、OOM、shader failure、readback stall和hot reload矩阵 | fault injection与last-good/recovery/retirement receipt |
| Runtime103-P2-09 | 无world unload、handle exhaustion、pause/seek/restart并发压力测试 | deterministic lifecycle/fuzz/property corpus |
| Runtime103-P2-10 | Editor测试只验证注册/template字符串 | 操作、undo/redo、save/reopen、compile error、preview step和crash recovery测试 |
| Runtime103-P2-11 | acceptance Markdown记录的是旧运行，且明确runtime rerun被邻域改动阻断 | 新证据必须绑定当前source fingerprint和完整dependency state |
| Runtime103-P2-12 | 没有与Unreal/Godot/Fyrox/Unity同场景质量/性能比较 | 先定义同画质、同粒子行为、同硬件的可复现实验，再谈优于 |

## 8. Owner裁决与目标架构

### 8.1 唯一source、artifact与runtime链

目标链固定为：

`ParticleSourceDocument -> ParticleSemanticCompiler -> CompiledParticleProgram -> ParticleWorldRuntime -> ParticleSimulationInstance(CPU or GPU) -> immutable ParticleRenderPacket -> Render Graph -> renderer/history/readback receipt`

`rendering.vfx_graph`不再拥有第二套runtime asset/executor；它只是一种Particle source authoring frontend。`particles.system`、VFX Graph和未来module library都编译到同一artifact schema。

### 8.2 `ParticleWorldRuntime`

由World唯一持有，负责component attach/detach、fixed/update schedule、command queue、instance lifecycle、backend选择、checkpoint与shutdown。Manager API降为该owner的内部服务，禁止跨world克隆全局锁。

### 8.3 `CompiledParticleProgram`

artifact包含schema/compiler版本、source hash、attribute liveness/layout、spawn/update/event context、curve LUT、renderer set、resource bindings、backend support matrix、bounds/scalability metadata、shader/PSO keys和source map。运行时不重新解释authoring node。

### 8.4 `ParticleGpuExecutionService`

Graphics拥有真实GPU资源、device generation、allocation、pipeline、queue/barrier、retirement和readback。Particle runtime只提交immutable work packet并接收generation-qualified outcome；compute必须由Render Graph pass真实执行。

### 8.5 `ParticleRendererRegistry`

Sprite、Mesh、Ribbon/Trail、Light/Decal/Volume各自编译renderer packet；material、texture、UV、sort、depth fade、velocity/reactive、OIT和ray/path support都有明确capability与degraded reason。

### 8.6 `ParticleScalabilityService`

按world/view/effect type维护significance、distance/visibility、instance/particle/GPU-time/VRAM/overdraw budget，输出stable admission/LOD decision。容量不足不能由BTree遍历顺序决定。

### 8.7 `ParticleAuthoringGateway`

Editor15拥有document、transaction、graph/curve UI、undo/redo和preview workflow；runtime提供compile、instantiate、step、seek、snapshot、diagnostic和capture gateway。Editor不直接修改runtime内部asset struct。

### 8.8 邻接owner

Runtime04/05拥有通用asset与Scene/ECS合同；Runtime22/24拥有clock/RNG/replay与identity exhaustion；Runtime09A/B/C/H1拥有RHI/Render Graph、RenderScene、material/PSO与history；Runtime08A/08C拥有physics/animation provider；Editor15拥有authoring产品流；Plugins09拥有package/dist/catalog。本文只拥有Particle/VFX语义compiler、world instance、backend parity、renderer family、scalability和产品接入，不复制邻域P0。

## 9. 依赖顺序与重构里程碑

### M0：冻结characterization并修正文档事实

保留当前CPU/GPU count、curve、snapshot、readback和transparent draw测试；新增`max_dt`每emitter回归、graph executor实际dispatch断言和JSON旁路inventory。更新所有仍声称diagnostic无界或全局dt污染的索引。

### M1：Source schema与compiler artifact

定义versioned `ParticleSourceAsset`、component reference、canonical IR、compiled program、diagnostic/source map与migration。VFX Graph只作为source frontend。

### M2：World owner与Scene hard cut

接入typed component load/save/attach/detach和明确schedule；建立generation handle。移除普通产品对dynamic JSON最终sprite的写入能力。

### M3：唯一backend与lifecycle

实现play/pause/stop/completion/warmup/seek/restart/pool状态机；每实例选择CPU或GPU唯一authority，fallback是显式generation migration。

### M4：Deterministic simulation core

fixed/substep/catch-up、fractional spawn time、versioned RNG、event/sub-emitter/parameter context、checkpoint/replay与CPU oracle。

### M5：CPU性能结构

active chunks、job schedule、instance upload/ring allocator、bounds增量更新，关闭全slot扫描和每pass新buffer。

### M6：GPU Render Graph authority

建立persistent allocation，把spawn/update、compact、indirect、bounds/sort真正迁入graph executor；加入device generation、retirement和last-good。

### M7：Renderer family与material

先完成Sprite material/texture/UV/rotation/facing/sort/soft particle/velocity，再按产品需要加入Mesh和Ribbon/Trail；其他renderer按独立capability进入。

### M8：Scalability、bounds与visibility

实现world/view budget、significance、LOD、culling、GPU bounds和pressure策略；dirty frontier接入真实prepare。

### M9：Physics、animation与data interface

以compiled binding接入batched collision、event和parameter provider；unsupported GPU module必须显式拒绝或降级。

### M10：产品迁移

Vampire改用particle asset与typed command；建立load/tick/render/reload/save-reopen首方场景，删除JSON GPU telemetry authority。

### M11：Editor authoring与preview

关闭disabled/Space-only shell，完成create/open/edit/compile/preview/diagnostic/undo/save/reopen；preview使用同一compiled artifact/runtime。

### M12：资格、证据与竞争性优化

通过正确性、规模、故障、视觉、temporal、device/platform和产品门后才升级capability。最后才做与Unreal的同画质benchmark和针对性性能优化。

## 10. 验收资格门

| Gate | 必须证明 |
|---|---|
| G01 | `ParticleSystemComponent`可Scene save/reopen并保持asset identity与参数 |
| G02 | World load/unload自动attach/detach且无跨world实例泄漏 |
| G03 | 正常产品schedule实际推进particle runtime，不依赖测试手动tick |
| G04 | Script只能发typed command，不能写最终sprite或GPU counter truth |
| G05 | 同一instance同一时刻只有CPU或GPU一个authoritative backend |
| G06 | handle含owner epoch/generation，耗尽不会覆盖live instance |
| G07 | one-shot/loop/completion/restart/pause/stop状态机CPU/GPU一致 |
| G08 | warmup/seek受step/time budget约束并返回receipt |
| G09 | source schema/version/migration可round-trip并拒绝未知破坏性字段 |
| G10 | compiler artifact绑定source hash、compiler version和target/device contract |
| G11 | VFX Graph编译到canonical program，不注册第二套no-op runtime |
| G12 | attribute liveness/layout与curve program有CPU oracle |
| G13 | 新spawn粒子在不同frame partition下结果满足定义的容差 |
| G14 | fixed/substep/catch-up在30/60/144Hz与暂停恢复下行为稳定 |
| G15 | RNG有algorithm/stream/draw identity并可checkpoint/replay |
| G16 | local/world/vector space在旋转与非均匀缩放下有oracle |
| G17 | physics collision真实查询provider并消费friction/bounce |
| G18 | animation/event binding按instance identity解析，歧义fail-close |
| G19 | CPU active set不随历史allocated slot线性扫描 |
| G20 | 10k/100k/1M workload有明确CPU/GPU ms、VRAM和quality tier |
| G21 | GPU allocation按budget/significance决策，不依赖遍历顺序 |
| G22 | 单实例pause/remove/asset hot reload不重置无关系统 |
| G23 | spawn/update、compact、indirect在Render Graph executor内真实录制 |
| G24 | graph resource version/barrier/queue/profile覆盖真实compute |
| G25 | dispatch由compiled capacity派生，不存在固定`[1,1,1]`双重描述 |
| G26 | device loss/OOM/shader failure有last-good、retire和恢复receipt |
| G27 | readback有cadence、max in-flight、age/drop/stale和超时策略 |
| G28 | readback卡住一项不会阻塞后续ready completion |
| G29 | stale GPU output超过阈值不会继续伪装当前帧绘制 |
| G30 | Sprite renderer消费material、texture、UV、rotation/facing与sort |
| G31 | soft particle/depth interaction有reverse-Z与MSAA测试 |
| G32 | GPU particle输出velocity或明确reactive/history policy |
| G33 | CPU upload使用instance/ring/batch，不为三个pass重复创建临时buffer |
| G34 | Mesh与Ribbon/Trail至少各有一条真实compiled/render产品lane |
| G35 | bounds由simulation/renderer产生，caller不能自报GPU activity |
| G36 | visibility dirty frontier覆盖simulation、transform、asset/material generation |
| G37 | diagnostic具code/source/remediation，容量和分页可配置 |
| G38 | snapshot共享性能门在release lane非ignored执行并保存receipt |
| G39 | adapter缺失形成typed skip，required GPU matrix不会假通过 |
| G40 | CPU/GPU逐attribute parity覆盖position/velocity/lifetime/color/count |
| G41 | Editor完成create/edit/compile/preview/undo/save/reopen闭环 |
| G42 | Vampire首方场景从asset完成load/tick/render/reload/save-reopen |
| G43 | visual/temporal artifact绑定source fingerprint、settings、GPU/driver与metric |
| G44 | 同硬件同画质同语义benchmark后才允许“优于Unreal”声明 |

## 11. 测试与artifact判定

专用测试有53个test attributes和1个ignored性能门。它们覆盖CPU确定性/lifetime/control、extract排序与metadata、GPU layout/planner/readback/count parity、可选offscreen真实compute/transparent pixel、graph resource contract、optional capability、registration、validation、snapshot sharing和diagnostic paging。这比纯descriptor测试强，但GPU helper在adapter/device获取失败时返回`None`，因此相关测试可在没有GPU证据时成功结束。

`tests/acceptance/particles-gpu-readback-mailbox.md`是有价值的历史记录，但其自身明确写明当前runtime rerun被邻域physics/animation churn阻断，并承认内置renderer尚未自动从alive/indirect输出执行GPU透明渲染、没有runtime feedback consumer。它不绑定本轮source fingerprint，不能作为当前产品资格证据。

本轮没有发现绑定当前源码的普通产品particle scene、save/reopen receipt、GPU capture、视觉序列、规模曲线、VRAM/GPU ms、device-loss恢复或与参考引擎的同场景benchmark。现有单元测试证明局部机制存在，不能证明系统已工程化完成。

## 12. 完成定义与退出条件

只有当M0-M12按依赖顺序实施、44项gate均有当前source-bound证据、Scene/Script/Vampire旁路被hard cut、VFX Graph并入唯一compiler/runtime、CPU/GPU只有一个authoritative backend、真实GPU compute归Render Graph所有、renderer/material/history/scalability/Editor/product闭环完成后，本报告才能把`implementation_status`改为complete。

在此之前，`particles`必须保持experimental/Partial，VFX Graph与GPU simulation必须保持默认关闭；不允许用注册成功、pass名称、固定workload、no-op executor、caller-authored counter、可选GPU测试或旧acceptance记录替代产品资格。
