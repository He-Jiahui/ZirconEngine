---
related_code:
  - zircon_plugins/particles
  - zircon_plugins/rendering/features/vfx_graph
  - zircon_runtime/src/graphics/particle_runtime_provider
  - zircon_runtime/src/core/framework/render/frame_extract/particle_extract_policy.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/particle.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/update_particle_previous_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/particle_previous_sprites.rs
  - zircon_runtime/src/graphics/visibility/declarations/visibility_particle_upload_plan.rs
  - zircon_runtime/src/graphics/visibility/planning/build_particle_upload_plan.rs
  - zircon_runtime/src/scene/world/render_particles.rs
  - zircon_runtime/src/script/vm/gameplay_host.rs
  - zircon_runtime/src/script/vm/gameplay_host/lifecycle.rs
  - examples/vampire/scripts/vampire_game/main.zr
tests:
  - zircon_plugins/particles/runtime/src/tests
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
  - docs/plans/optimize/zircon_editor/15-material-shader-graph-instance-vfx-particle-preview-compiler-diagnostics-authoring-review.md
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
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

# 26 · Particle/VFX System、Emitter、CPU/GPU Simulation、Rendering、Scalability、Determinism 与 Product Integration 工程化差距

## 1. 结论

Zircon已经有一组值得保留的粒子基础，而不是完全空白。`zircon_plugins/particles`明确把package标为experimental、主能力标为`Partial`且默认不启用；CPU侧有SoA particle pool、free list、确定seed的局部RNG、有限值校验、point/sphere/box/cone emitter、burst/rate、曲线、local/world space和sprite extract。GPU侧已经实际生成WGSL，建立attribute layout、双buffer simulation、alive index、counter、indirect draw、renderer-owned prepare/external binding、异步readback admission以及可在有adapter时执行的offscreen测试。Core renderer又保留camera layer、depth/overlay分路、previous sprite state和velocity pass。Scene extract最近已从“扫描全部entity”改为扫描dynamic-component owner。这些都比descriptor-only功能更接近真实实现，重构时不能丢失。

但当前没有一条产品权威链把这些基础连起来。全仓生产调用搜索没有找到`ParticlesManager::tick`的scheduler/driver，`ParticleSystemComponent`也没有进入Scene/ECS加载、序列化、实例化和卸载。Vampire实际通过`gameplay.set_particle_sprites`把JSON写入`render.particle_sprites`，`World::render_particles`再直接解析成colored quads；它完全绕过particle asset、manager、CPU/GPU simulation、material和plugin lifecycle。与此同时，GPU资产在manager内仍建立CPU fallback并随`tick`推进，renderer-owned `ParticleGpuRuntimeOwner`又克隆同一manager实例列表执行独立GPU simulation。产品旁路、CPU fallback和GPU owner由此形成三条互不一致的状态权威。

资产模型仍是未版本化的普通Rust结构，只覆盖sprite emitter。`looped`字段没有执行consumer，GPU只取scalar/color curve首尾端点，physics所谓collision只做固定damping且从不查询physics world，`bounce`未消费；animation binding字段也没有被事件路径求值。CPU每tick扫描全部allocated slot，spawned particle在大`dt`内被更新完整`dt`，没有fixed substep、catch-up budget、checkpoint或replay。Manager在单一`Arc<Mutex<_>>`内串行tick、rewind和全量snapshot，diagnostic无界；handle计数饱和后会复用`u64::MAX`并静默覆盖live实例。

GPU实现的问题不只是“功能较少”。Owner把所有playing GPU system聚合成一个asset/backend，新增、删除、暂停或asset变更都可能重建整个buffer/pipeline并重置其他系统；不同实例的frame delta最终取`max_dt`作为整次dispatch的全局`dt`，快实例可把慢实例中的存活粒子推进过多。1,048,576全局slot按BTree顺序先到先得，后续emitter可以获得0 capacity；当前约26 words/particle，双状态buffer在上限处约208 MiB，尚未计alive/counter/emitter/readback资源，却没有GPU预算、优先级、LOD或pressure策略。暂停会从aggregate移除，stop只重置manager/planner age而未明确清空GPU alive state。pipeline同步创建、device generation/recovery、last-good、readback staleness与shader/material generation也都缺失。

渲染端仍只是colored billboard。CPU每帧把每个sprite扩成6个vertex，并为depth、overlay和velocity批次分别创建新GPU vertex buffer；WGSL只传颜色，`material`与`texture`字段完全不消费。GPU透明shader同样只读position/size/color，没有UV、texture、rotation、soft particle、material、sort、lighting或velocity/history输出。没有mesh/ribbon/trail/light/decal/volume renderer，也没有面向大规模透明粒子的GPU sort/binning和可验证overdraw策略。Scene JSON还允许调用者自报`gpu_frame` count/bounds，解析失败多数静默变成缺省或忽略，telemetry不能证明真实simulation执行。

独立`rendering.vfx_graph` runtime又定义了第二套五节点graph，compiler只检查max particle、SpawnRate和ShaderGraphMaterial是否存在，随后固定声明`[1,1,1]` dispatch；simulation与transparent executor都直接`Ok(())`。它必须被吸收到唯一`CompiledParticleProgram`，不能继续作为另一条可注册但不执行的runtime authority。

本篇没有登记P0。原因不是这些实现已具产品资格，而是particle package当前诚实标为experimental/partial、VFX Graph默认不启用；面向用户的固定“compiled/running”假成功和authoring断链已由Editor15登记P0。任何profile若把本篇功能提升为Complete、required或默认enabled，必须先通过本篇产品门，否则应fail-close。本轮登记 **0项P0、60项P1和12项P2**，均未实施。

## 2. 审查边界、方法与 currentness

### 2.1 物理扫描

本篇冻结140个输入、43,229行、2,010,138 bytes：113个Zircon source/test/product输入与27个参考实现输入。Zircon输入覆盖particle plugin全部57个文件、rendering VFX Graph八个文件、runtime provider、core particle renderer、visibility/upload、previous-state/velocity、Scene JSON extract、script gameplay host、Vampire调用与acceptance记录。输入清单按relative path和单文件SHA-256排序后组合指纹为`6554789ade30fab8443e1af93688475c3587b4200bee26b17b9f1367bed5d31f`。

物理规模中，`zircon_plugins/particles`为57文件、8,733行、306,329 bytes；`zircon_plugins/rendering/features/vfx_graph`为8文件、295行、10,153 bytes；`particle_runtime_provider`为3文件、49行、1,316 bytes；core scene particle renderer为21文件、1,193行、43,626 bytes。另逐函数深读208行extract policy、657行Scene particle extract、manager/service、CPU pool/simulation、GPU planner/program/backend/owner/readback/transparent shader及产品script host。

### 2.2 本轮追踪的生产链

1. package/plugin registration -> private `ParticlesManager`与`ParticleGpuRuntimeOwner` -> render feature/executor/runtime prepare。
2. `ParticleSystemAsset`/component -> manager instantiate/play/pause/stop/tick -> CPU `ParticleSystemInstance` -> snapshot/extract。
3. manager `gpu_runtime_instances` -> aggregate asset/layout/backend -> planner -> compute -> indirect transparent draw/readback -> manager feedback。
4. Core `RenderFrameExtract.particles` -> particle extract policy -> CPU billboard/velocity与plugin GPU external buffers -> frame stats/previous state。
5. Script `set_particle_sprites` -> dynamic JSON component -> `World::render_particles` -> sorted sprite snapshot或caller-authored `gpu_frame`。
6. `rendering.vfx_graph` graph/compile descriptor -> fixed workload -> two no-op executors。
7. unit/static GPU tests -> optional adapter test -> acceptance Markdown；没有普通product world从particle asset创建、tick、render、reload、save/reopen的lane。

### 2.3 证据等级与 currentness

本轮为E3 source-level review：从声明、实现、调用者、产品入口、资源状态、shader和测试失败/跳过分支逐层闭环；参考实现只用于确定边界，不以类名数量判定Zircon差距。源revision为`25e09a23178000f2e783ce2143cf70a8b118d404`，冻结的particle/VFX/core输入在扫描时没有工作区差异。

没有重跑已知无变化的plugin locked metadata失败lane，也没有用缺adapter时返回`None`的GPU测试证明GPU通过。现有Editor、Hub、WOC和lockfile动态阻断仍由各自报告拥有；本篇不把未执行命令计为成功。仓内其他区域与plan处于在途状态，因此保留`source_recheck_required: true`。

## 3. 当前可保留的工程基础

| 基础 | 当前证据 | 保留与提升条件 |
|---|---|---|
| Capability truth | particle package为experimental，主能力为Partial且默认不启用 | 在产品lane完成前继续fail-close，不把pass descriptor或测试存在投影为Complete |
| CPU storage | SoA arrays、free list、bounded per-emitter capacity | 增加active set/chunk/job与world budget，不退回per-particle heap object |
| Local deterministic seed | 局部LCG、seed可重复单测 | 纳入versioned RandomStream/checkpoint/replay，不把单进程重复等同跨平台确定性 |
| Validation | asset/component对finite、range、capacity有明确错误 | 提升到versioned schema/compiler diagnostic与source location |
| GPU data path | typed layout、ping-pong state、counter、alive indices、indirect draw | 改为per-generation persistent allocation与compiled program，不在拓扑变化时全量重建 |
| Renderer ownership | GPU execution由renderer prepare获取device/queue并提供external bindings | 绑定device generation、retirement、last-good和render graph真实resource usage |
| Readback admission | readback经异步请求和通用admission | 增加particle-specific cadence、age/drop/staleness与无readback常态路径 |
| Previous state | CPU particle有previous sprite与velocity pass | 统一identity/history generation，并为GPU path提供真实velocity或明确reactive mask |
| Scene scan | dynamic-component owner驱动而非探测全部entity | 迁移为typed component query与dirty frontier，删除JSON第二authority |
| GPU tests | shader parse、layout/readback/indirect与可选offscreen execution | adapter不可用必须形成typed skip receipt；required GPU matrix不能静默通过 |

## 4. 参考实现给出的工程边界

### 4.1 Unreal Niagara：world simulation、scalability与多renderer分层

`UNiagaraComponent`明确持有pooling、desired age/seek、reset/reinitialize和warmup语义；`NiagaraWorldManager`与`NiagaraSystemSimulation`按world和system组织simulation，而不是进程全局manager全量锁。`NiagaraScalabilityManager`把effect type、significance、distance/visibility culling和budget纳入统一决策；GPU dispatch与Sprite/Mesh/Ribbon renderer分开。Zircon应吸收system instance、world owner、scalability manager、compiled data与renderer family边界，不照搬UObject、Niagara VM或其所有模块复杂度。

### 4.2 Godot：生命周期、fixed FPS、bounds、trail与CPU/GPU合同

Godot `GPUParticles3D`/`CPUParticles3D`显式提供one-shot、preprocess、fixed FPS、fractional delta、interpolation、amount ratio、visibility AABB、draw order、trail与sub-emitter等运行时合同；rendering storage维护GPU particle resources，process material表达collision等模块。它证明这些不是Editor装饰字段，而是simulation、culling、render和serialization共同消费的合同。Zircon还应在此基础上补BuildSet、generation、budget与更严格的determinism/evidence。

### 4.3 Fyrox：Scene原生、反射/持久化与可运行CPU系统

Fyrox particle system是Scene node，使用`Reflect`、`Visit`、`InheritableVariable`持久化emitters、material、playing、particles、free list、RNG、visible distance和coordinate system；draw与material是实际消费者。该参考规模小于Niagara，但直接揭示Zircon当前component/asset没有进入Scene、serializer和product schedule的断层。可借鉴scene-native lifetime和可运行CPU baseline，不把其CPU实现当作百万粒子性能目标。

### 4.4 Unity Graphics VFX Graph：compiled particle data与attribute program

仓内Unity VFX Graph以`VFXDataParticle`管理capacity、aligned capacity、particle/strip、stored current/source attributes、bounds mode及context flow；compiler/code generator输出compiled system data、attribute buffers、indirect/bounds和shader资源。Zircon五节点graph加固定pass name不是同等级compiler artifact。需要建立canonical graph/IR、attribute liveness/layout、context program、resource binding、source map与target-specific artifact；Editor15继续拥有authoring document和UI。

### 4.5 Bevy：只作render architecture参考

当前Bevy checkout没有第一方particle/VFX runtime，本篇不虚构功能对标。仅采用其`ExtractComponent`的MainWorld/RenderWorld边界、`RenderAsset`的changed asset提取与GPU prepare/device recovery re-extract、upload bytes-per-frame节流、GPU component buffer和readback生命周期作为架构参考。粒子语义与产品完整性必须由Zircon自身及Unreal/Godot/Fyrox/Unity VFX证据裁决。

## 5. Owner裁决与非重复边界

| Owner | 本篇拥有 | 邻接报告继续拥有 |
|---|---|---|
| Particle runtime authority | world-scoped system instance、schedule、lifecycle、CPU/GPU parity、event与product reachability | Runtime05拥有通用Scene/ECS/world lifecycle；Plugins06拥有package/profile/catalog closure |
| Particle asset/artifact | runtime schema、compiled simulation/render program、generation与migration消费合同 | Runtime04拥有通用asset/import/cook/residency；Editor15拥有source document/compiler UX |
| Simulation time | particle fixed-step、substep、warmup、checkpoint、RNG stream消费 | Runtime22拥有全局Clock/Random/Replay schema |
| Physics/Animation bridge | 粒子collision/event binding语义与failure policy | Runtime08A/08C拥有physics/animation provider本体 |
| GPU execution | particle allocation/state/readback/reset/device recovery与CPU/GPU parity | Runtime09A拥有RHI/render graph/fence/device总合同 |
| Rendering | sprite/mesh/ribbon等particle renderer、material/texture/sort/velocity | Runtime09B/09C/09H1拥有RenderScene、PSO/material与通用history |
| Authoring/Preview | runtime gateway、snapshot、step/rewind真实性 | Editor15拥有document、graph、curve、compiler UI和preview session产品流 |
| Qualification | particle product scene、correctness、scale、fault、visual与performance corpus | Tooling通用测试/evidence报告拥有runner与receipt schema；本轮暂停新增tooling专题 |

## 6. P1：产品权威、资产、生命周期与系统接入

| ID | 当前差距 | 需要重构 |
|---|---|---|
| PARTICLE-P1-001 | `ParticlesManager::tick`在生产代码没有scheduler/driver调用者 | 建`ParticleWorldRuntime`并注册明确Update/FixedUpdate stage、world generation、pause和shutdown顺序 |
| PARTICLE-P1-002 | 产品可见粒子由dynamic JSON `render.particle_sprites`直接生成，与plugin runtime并列 | 硬切到typed `ParticleSystemInstanceRef`；JSON只保留迁移reader且不得进入shipping authoring |
| PARTICLE-P1-003 | Script host可直接设置最终sprite数组，绕过asset、预算、simulation和权限 | 暴露spawn/stop/set-parameter/event等受限command，返回typed handle/outcome并做per-principal budget |
| PARTICLE-P1-004 | `ParticleSystemComponent`没有Scene/ECS serializer、loader、instantiate或remove consumer | 建versioned scene component、asset handle resolution、world attach/detach和save/reopen roundtrip |
| PARTICLE-P1-005 | Manager是可克隆进程内`Arc<Mutex<_>>`，没有world/session scope或owner generation | 由World持有唯一runtime owner；handle绑定world/owner epoch，卸载时drain/retire |
| PARTICLE-P1-006 | GPU asset在manager中持续CPU fallback，GPU owner又执行独立simulation | 每实例只选择一个authoritative backend；fallback必须是显式迁移/重建并发布backend generation |
| PARTICLE-P1-007 | `rendering.vfx_graph`是第二套graph/runtime authority，executor全部no-op | 删除独立执行权威，compile到唯一`CompiledParticleProgram`并由同一world/runtime执行 |
| PARTICLE-P1-008 | VFX descriptor宣称buffer读写和固定compute workload，但真实executor不访问资源 | resource usage与dispatch必须来自compiled program；无program时拒绝feature activation |
| PARTICLE-P1-009 | GPU请求在manager中无条件记录BackendUnavailable CPU fallback，即使renderer owner可执行 | 建BackendSelection/Reason/Generation状态机，UI/telemetry展示实际active backend而非两套状态 |
| PARTICLE-P1-010 | plugin registration没有实例teardown、manager schedule removal和GPU resource retirement证明 | owner lease覆盖registry、world systems、callbacks、buffers、readback；unload前等待fence与callback drain |
| PARTICLE-P1-011 | `next_handle.saturating_add`在MAX后复用MAX，BTreeMap insert可覆盖live实例 | 使用checked generational slot/owner epoch；exhaustion返回typed error且永不替换live slot |
| PARTICLE-P1-012 | sprite stable key只有emitter index与slot，多个system挂同entity时可碰撞 | identity包含world/instance generation/emitter/slot generation，并定义recycle与history retirement |

## 7. P1：Asset、CPU Simulation、Determinism 与 Interop

| ID | 当前差距 | 需要重构 |
|---|---|---|
| PARTICLE-P1-013 | Asset是普通Clone/PartialEq Rust struct，无serde、schema version、source identity或migration | 定义`ParticleSourceAsset`版本、migration、unknown-module保留及immutable compiled artifact |
| PARTICLE-P1-014 | `looped`只有声明/default/builder，没有simulation consumer；无duration/one-shot完成事件 | 明确system/emitter duration、loop delay、completion、auto destroy与restart语义并双backend一致 |
| PARTICLE-P1-015 | 只支持sprite与四种基础shape，缺module/context/data-interface与renderer schema | 先建可扩展typed module registry、pin/property schema、compatibility和unsupported fail-close |
| PARTICLE-P1-016 | GPU curve只取首尾key，CPU求值完整key序列，同asset跨backend表现不同 | compiler统一resample/LUT/analytic representation，输出误差界、memory cost与CPU/GPU parity test |
| PARTICLE-P1-017 | 大`dt`下本帧spawn全部被推进完整`dt`，产生时间聚团和寿命偏差 | 按spawn timestamp/substep分布积分；定义hitch clamp、catch-up与discard policy |
| PARTICLE-P1-018 | simulation只有variable `dt`，没有fixed step、max substeps或determinism class | 消费Runtime22 clock snapshot，按asset/profile声明RealTime/Fixed/Replayable并记录tick identity |
| PARTICLE-P1-019 | 局部LCG没有algorithm/version/stream ID、state codec、checkpoint或divergence evidence | 接入versioned RandomStream，保存seed+state+draw count并支持checkpoint/replay/hash compare |
| PARTICLE-P1-020 | `live_count`与tick/snapshot扫描全部allocated slot，capacity增长后成本与live数脱钩 | 维护dense active indices/chunks、generation free list和SIMD/job-friendly SoA；给出复杂度合同 |
| PARTICLE-P1-021 | 只有per-emitter max，缺world CPU/GPU memory、spawn、update、draw、overdraw预算 | 建ParticleScalabilityManager，按effect class/priority/distance/significance/visibility实施LOD/睡眠/剔除 |
| PARTICLE-P1-022 | world-space initial velocity未按emitter transform变换，local粒子又跟随完整当前transform | 冻结position/vector/normal/local/world语义，处理non-uniform scale、rebase与current/previous transform |
| PARTICLE-P1-023 | physics capability仅施加force；collision启用后每帧damping，`bounce`从未消费 | 通过Physics query/batch/contact owner实现collision/friction/restitution/kill/event，缺provider时fail-close或明确degraded |
| PARTICLE-P1-024 | animation binding字段不求值；无handle时选同entity第一个实例，找不到时静默Ok | binding编译为stable target/event route，支持一对多策略、generation、missing-target diagnostic与deterministic order |

## 8. P1：Manager、Extract、Bounds、Diagnostics 与 Scheduling

| ID | 当前差距 | 需要重构 |
|---|---|---|
| PARTICLE-P1-025 | manager持单一mutex完成全部实例tick，任何慢系统阻塞控制、extract和feedback | world schedule生成immutable work list，实例/chunk并行，mutation通过command buffer在明确barrier提交 |
| PARTICLE-P1-026 | preview rewind在mutex内按任意seconds/fixed_dt无界循环 | 建有deadline/cancel/max-step的async seek，使用checkpoint和progress receipt，不阻塞runtime owner |
| PARTICLE-P1-027 | snapshot克隆全部diagnostic、sprite、component与GPU feedback，extract再次分配/排序 | 建generation-qualified incremental snapshot、shared immutable buffers和reader-gated debug detail |
| PARTICLE-P1-028 | diagnostics为无界`Vec<String>`语义，缺code/source/generation/dedup/retention | 接入typed diagnostic store，限制items/bytes/age并区分asset compile、instance、GPU和degraded状态 |
| PARTICLE-P1-029 | manager mutex poison用`expect`终止进程，GPU owner另有typed Poisoned错误 | 统一failure domain与supervisor decision；隔离坏instance并保留world可诊断终态 |
| PARTICLE-P1-030 | capability只能追加启用，没有revoke、provider generation或实例重新资格 | capability snapshot绑定provider lease/generation；变化触发prepare/commit迁移或明确停止不兼容实例 |
| PARTICLE-P1-031 | 没有spawn/update/event/bounds/extract stage依赖图和工作量估计 | compiler生成stage DAG、resource access、work units和queue lane，scheduler按budget/admission执行 |
| PARTICLE-P1-032 | 没有prewarm、warmup budget、delayed start、completion、auto deactivate或pool policy | 建完整system lifecycle并让gameplay、Editor preview和pool使用同一状态机 |
| PARTICLE-P1-033 | 没有sub-emitter、death/collision/output event或bounded event queue | 建typed event stream、producer admission、generation target和overflow/drop/reconciliation策略 |
| PARTICLE-P1-034 | bounds没有manual/recorded/automatic/compiled policy，CPU/GPU culling无法证明保守 | artifact携analytic/recorded bounds；runtime更新有budget并验证NaN、teleport、local/world与trail扩张 |
| PARTICLE-P1-035 | Scene extract对全部sprite做CPU距离排序；sort order只是次键且GPU path不排序 | 定义None/age/distance/custom/order语义，按renderer/material分桶并提供GPU sort或受控近似策略 |
| PARTICLE-P1-036 | snapshot携material/texture option，但core renderer与Scene JSON路径全部丢弃 | resolve到generation-qualified material/texture render asset；missing/not-ready使用显式fallback disposition |

## 9. P1：GPU Simulation、Resource Lifetime 与 Backend Parity

| ID | 当前差距 | 需要重构 |
|---|---|---|
| PARTICLE-P1-037 | 所有playing GPU实例被合并为单一asset/backend，拓扑变化会重建全局状态 | 使用persistent heap/page allocator与per-system descriptor/program，局部增删不重置无关实例 |
| PARTICLE-P1-038 | pause通过从aggregate移除实例改变拓扑，已有粒子可能立即消失并重建他人 | pause/emission-pause/simulation-pause/render-disable分别建状态；保留slot与alive state直到明确retire |
| PARTICLE-P1-039 | stop只重置CPU/planner age，没有明确清零GPU alive/counter/indirect state | stop/reset/reinitialize发generation-tagged clear/init command并等待正确barrier后发布完成 |
| PARTICLE-P1-040 | 1,048,576 slot按实例/BTree顺序分配，晚到emitter可获0容量且无公平/优先级 | budget allocator支持reservation、priority、minimum guarantee、rejection与scalability downgrade receipt |
| PARTICLE-P1-041 | 双state buffer上限约208 MiB且无memory telemetry/pressure/reclaim | 按live attributes做liveness-packed layout，page residency/compaction/pressure回收并纳入GPU memory budget |
| PARTICLE-P1-042 | aggregate变化时同步创建shader/module/buffer/pipeline，可能在render prepare造成stall | compiled shader/PSO异步cache，key含program/layout/material/target/device；保留last-good并预算compile |
| PARTICLE-P1-043 | GPU owner没有device generation、loss/recreate、buffer retirement或readback cancellation合同 | 接RHI device lifecycle，失效旧bindings/readback，recreate后从authoritative state恢复或明确reset |
| PARTICLE-P1-044 | GPU frame delta来自manager CPU instance的`age_seconds`，而manager本身无产品tick | world clock直接驱动唯一backend；render thread只消费immutable tick packet，不从fallback实例推导时间 |
| PARTICLE-P1-045 | 多实例各算dt后取`max_dt`作为全局dispatch dt，较慢实例的存活粒子会被过推进 | per-system/emitter参数携自己的step count/dt，或按同clock cohort分dispatch并做parity vector |
| PARTICLE-P1-046 | GPU program虽存rotation字段却spawn为0且不更新；无texture/UV/material/soft particle/collision | attribute liveness由compiled modules生成，renderer contract消费rotation/UV/material并验证CPU/GPU语义 |
| PARTICLE-P1-047 | admitted frame持续排counter/indirect readback，缺particle cadence、queue age和drop policy | 默认无回读；profiling/debug按采样预算请求，返回request/frame/generation/staleness并限pending bytes |
| PARTICLE-P1-048 | GPU work未admit时可复用旧bindings/output，调用者看不到Skipped/Stale/Degraded | frame/extract携execution disposition、last simulated tick、data age和reason；超龄后停止render或明确fallback |

## 10. P1：Rendering、History、测试与产品资格

| ID | 当前差距 | 需要重构 |
|---|---|---|
| PARTICLE-P1-049 | CPU renderer每帧每sprite扩6 vertex并分配Vec，带宽与CPU成本线性放大 | persistent instance buffer/structured buffer + unit quad，dirty range upload与batch indirect draw |
| PARTICLE-P1-050 | depth、overlay、velocity每批每帧`create_buffer_init`新GPU buffer | 使用frame allocator/ring/staging belt和明确fence retirement，记录upload bytes与allocation count |
| PARTICLE-P1-051 | CPU/GPU shader都只输出顶点色，asset material/texture没有视觉作用 | 接Runtime09C material/PSO，支持texture/flipbook/UV sheet、blend/depth/soft-particle与color-space合同 |
| PARTICLE-P1-052 | 只有camera-facing sprite，没有mesh、ribbon/trail、beam或screen/aligned variants | 建RendererSet和per-renderer compiled data，先完成sprite/mesh/ribbon三类及fallback/unsupported策略 |
| PARTICLE-P1-053 | 所有粒子共享单alpha blend/depth pipeline，没有material batching或permutation budget | material/renderer state形成稳定PSO key，batch按pipeline/resource排序并限制permutation/cardinality |
| PARTICLE-P1-054 | GPU透明粒子没有velocity/history输出，TAA/motion blur只能看到CPU path的previous state | GPU保存previous position/transform并输出velocity或reactive/transparency mask，处理reset/teleport/generation |
| PARTICLE-P1-055 | CPU history以`(entity, stable_sprite_key)`匹配，key碰撞会串用previous position | 使用完整ParticleInstanceId/slot generation；消失、重生、backend switch和world rebase显式invalidate |
| PARTICLE-P1-056 | Dynamic JSON解析大量使用`Option`静默忽略malformed/non-finite/unknown字段 | 迁移期使用versioned bounded parser与typed diagnostic；shipping profile拒绝旁路组件 |
| PARTICLE-P1-057 | JSON调用者可自报`gpu_frame` alive/count/indirect/bounds，telemetry可与真实GPU无关 | 只有renderer-owned signed/internal frame packet可发布GPU状态；script/public DTO不得写执行证据 |
| PARTICLE-P1-058 | offscreen GPU helper在无adapter或request失败时`.ok()?`返回None，测试可无receipt跳过 | required matrix把Unavailable/Skipped/Failed分开，记录adapter/backend/driver/features和执行case数量 |
| PARTICLE-P1-059 | 没有clean product scene完成asset load、instantiate、tick、render、stop、reload、save/reopen | 建最小产品lane并校验可见像素、particle counts、backend、diagnostics、resource retirement和artifact identity |
| PARTICLE-P1-060 | 没有1/10/100/1000 systems、million-particle、overdraw、hitch、device-loss、soak或CPU/GPU parity证据 | 建versioned workload corpus与correctness/perf/fault gates；同硬件同画质统计p50/p95/p99及memory峰值 |

## 11. P2：P1正确性闭合后的高级能力

| ID | 能力 | 前置条件 |
|---|---|---|
| PARTICLE-P2-001 | 大规模透明粒子的GPU radix sort、depth bins与受控OIT策略 | P1 sort语义、renderer/material和GPU budget已稳定 |
| PARTICLE-P2-002 | Niagara/VFX级可插件化Data Interface目录与用户自定义module | compiled schema、ABI/security、resource access和budget gate完成 |
| PARTICLE-P2-003 | Vector Field、Curl Noise、SDF、流体/体积场交互 | physics/render data interface及GPU memory/compute预算完成 |
| PARTICLE-P2-004 | 静态/蒙皮mesh surface/volume采样与骨骼事件发射 | mesh/skeleton generation、alias table artifact和animation bridge完成 |
| PARTICLE-P2-005 | Particle light、decal、volume/fog injection与audio/gameplay output | recipient system admission、lifetime、budget和feedback loop防护完成 |
| PARTICLE-P2-006 | Cinematic simulation cache、offline bake与scrub | deterministic tick、checkpoint、artifact version和Editor preview完成 |
| PARTICLE-P2-007 | 自适应simulation/render frequency、temporal reprojection与perceptual LOD | 基础scalability、history和quality error metric完成 |
| PARTICLE-P2-008 | Async-compute overlap、multi-queue与可选multi-GPU partition | RHI ownership、barrier、device group和profile evidence完成 |
| PARTICLE-P2-009 | 平台专用kernel、subgroup优化与压缩attribute layout | canonical IR、fallback backend、numerical tolerance和target artifact完成 |
| PARTICLE-P2-010 | 可选cross-vendor deterministic GPU profile | fixed-point/ordered algorithm、driver matrix和性能成本合同完成 |
| PARTICLE-P2-011 | 远程运行时particle capture、逐module cost与GPU state inspection | 安全debug channel、bounded readback、snapshot generation与redaction完成 |
| PARTICLE-P2-012 | 基于qualified corpus的自动quality tier建议与回归二分 | 多硬件长期telemetry、同画质oracle和不可自认证evidence完成 |

## 12. 目标架构

### 12.1 Canonical ownership

```text
ParticleSourceAsset + schema/plugin revisions
  -> ParticleSemanticCompiler
  -> CompiledParticleProgram
       contexts / module DAG / attributes / renderer set
       bounds / budgets / CPU kernels / GPU kernels / debug map
  -> ParticleArtifactStore + last-good generation
  -> ParticleWorldRuntime(world_id, owner_epoch)
       instance registry / clock / command buffer / event stream
       scalability / budget / diagnostics / checkpoints
  -> authoritative backend per instance
       CpuParticleExecutor | GpuParticleExecutor
  -> immutable ParticleRenderPacket(tick, generation, disposition)
  -> RenderWorld extract / material batches / history / submit
  -> bounded telemetry + qualification receipt
```

Source、compiled program、live instance、GPU allocation和render packet必须使用不同typed identity。任何backend switch、hot reload、device recovery或world transfer都通过prepare/commit generation完成；不能通过克隆component、比较整个asset或复用裸`u64`推断同一实例。

### 12.2 Compiled program

`CompiledParticleProgram`至少包含source/dependency/schema/compiler/target key、context DAG、attribute liveness与packed layout、spawn/update/event/output kernels、renderer recipes、resource/data-interface access、bounds policy、fixed-step/determinism class、CPU/GPU compatibility、memory/work estimate、diagnostics/source map和migration version。VFX Graph与template particle asset只作为两种source frontend，不能生成两套runtime。

### 12.3 Runtime与render packet

World runtime在simulation barrier消费typed commands，按clock cohort、visibility/scalability和budget产生work；CPU/GPU executor只写自己的authoritative state。Render packet只暴露当前tick可消费的sprite/mesh/ribbon batches、material/resource handles、bounds、history identity和execution disposition。Render thread不得重新解释source asset，gameplay/script也不得直接提交最终GPU count或vertex数组。

### 12.4 Failure与degraded语义

至少区分`Ready`、`Compiling`、`UsingLastGood`、`CpuFallback`、`BudgetDegraded`、`SimulationSkipped`、`DeviceRecovering`、`Unsupported`、`Failed`。每个状态携source/artifact/runtime/device generation、reason、first/last tick和恢复动作。fallback只有在视觉/时序容差有资格证据时才可自动执行，否则停止实例并返回typed outcome。

## 13. 分层实施计划

### M0 · Truth Freeze与删除第二authority

- 将VFX no-op feature与JSON shipping authoring标为Unavailable/legacy migration；
- 建立product reachability test，冻结manager无tick、component无Scene consumer与Vampire旁路现状；
- 修复handle exhaustion、diagnostic bound和GPU/CPU双authority等立即错误合同。

### M1 · Schema、Artifact与Scene接入

- 建versioned `ParticleSourceAsset`、scene component、migration和unknown-module preservation；
- VFX Graph与particle template编译到`CompiledParticleProgram`；
- 完成create/load/instantiate/save/reopen/cook/install/last-good generation链。

### M2 · World Runtime、Clock与CPU基线

- 建per-world owner、typed commands、system lifecycle、fixed/substep、event stream和checkpoint；
- CPU executor改dense active chunks、job batches、spawn timestamp积分与bounded snapshot；
- 接Physics/Animation真实provider，缺失时fail-close或明确degraded。

### M3 · GPU persistent runtime

- 用page/heap allocator替换aggregate rebuild，建立per-system program/descriptor与fair budget；
- 实现clear/reset/pause/backend switch、device recovery、pipeline cache和readback policy；
- 建CPU/GPU golden vector与per-instance dt/parity门。

### M4 · Renderer family与history

- CPU/GPU统一instance buffer、material/texture/flipbook/soft particle和PSO batching；
- 完成sprite、mesh、ribbon/trail renderer以及sort/bounds/visibility；
- GPU velocity/reactive mask、history identity、teleport/reset/rebase invalidation闭合。

### M5 · Scalability、Product与Competitive Qualification

- 建effect type、priority/significance、distance/visibility、memory/compute/overdraw budget与quality tier；
- Vampire迁移为typed particle command并加入普通product scene；
- 执行correctness、visual、CPU/GPU、memory、fault、device-loss、soak和同场景竞争性能矩阵。

## 14. 验收门

| Gate | 验收内容 |
|---|---|
| PARTICLE-G01 | shipping产品不再读取无版本`render.particle_sprites`/`gameplay.particle_sprites`作为粒子authoring authority |
| PARTICLE-G02 | 普通Scene可保存/加载typed particle component并解析同代artifact |
| PARTICLE-G03 | 每个live instance只有一个authoritative simulation backend |
| PARTICLE-G04 | manager/world runtime由产品schedule推进，pause/scale/fixed tick合同可追踪 |
| PARTICLE-G05 | handle exhaustion返回错误且压力测试不覆盖live实例 |
| PARTICLE-G06 | source、artifact、instance、allocation、render packet identity类型分离并带generation |
| PARTICLE-G07 | `looped`、duration、one-shot、warmup、stop/reset/restart语义有CPU/GPU一致vector |
| PARTICLE-G08 | 大dt spawn按时间分布，fixed-step catch-up受max steps/time预算约束 |
| PARTICLE-G09 | RNG algorithm/version/stream/state进入checkpoint/replay并能定位首次divergence |
| PARTICLE-G10 | CPU hot path成本随live/dirty粒子而非allocated high-water无条件增长 |
| PARTICLE-G11 | world CPU/GPU memory、spawn、update、draw、overdraw均有producer-side budget |
| PARTICLE-G12 | local/world position/vector/normal和non-uniform scale/rebase语义通过测试 |
| PARTICLE-G13 | collision真实查询Physics provider；bounce/friction/kill/event均非no-op |
| PARTICLE-G14 | animation/event binding使用stable target/generation且missing target不静默成功 |
| PARTICLE-G15 | preview seek可取消、有步数/time budget并利用checkpoint |
| PARTICLE-G16 | diagnostics有stable code/source/generation/dedup和items/bytes/age上限 |
| PARTICLE-G17 | VFX Graph executor执行compiled work；无program时feature activation失败 |
| PARTICLE-G18 | resource usage/dispatch extent由compiled program生成并与实际encoder访问一致 |
| PARTICLE-G19 | GPU实例增删/暂停/asset变更不重建或重置无关实例 |
| PARTICLE-G20 | stop/reset明确清除GPU alive/counter/indirect并发布完成generation |
| PARTICLE-G21 | capacity分配有公平/priority/minimum/rejection receipt，无静默0-slot emitter |
| PARTICLE-G22 | GPU allocation进入memory budget，pressure可回收/降级且不OOM崩溃 |
| PARTICLE-G23 | pipeline/program异步准备，render thread无未预算同步compile/create尖峰 |
| PARTICLE-G24 | device loss取消旧readback/binding并在新device generation恢复或明确reset |
| PARTICLE-G25 | 多实例不同dt不会因全局max dt互相过推进 |
| PARTICLE-G26 | curve/module CPU/GPU结果在定义容差与seed/tick corpus内一致 |
| PARTICLE-G27 | 常规render不依赖每帧GPU readback；debug采样受bytes/rate/age限制 |
| PARTICLE-G28 | skipped/stale/degraded frame在packet和telemetry中可见，不复用旧数据伪装新执行 |
| PARTICLE-G29 | CPU sprite走persistent instance/ring buffer，无每批每帧独立GPU buffer创建 |
| PARTICLE-G30 | material、texture、UV/flipbook、blend、depth和soft particle进入实际shader/PSO |
| PARTICLE-G31 | sprite、mesh、ribbon至少三类renderer有source->artifact->runtime->visual产品lane |
| PARTICLE-G32 | sort policy在CPU/GPU、透明批次与camera间语义一致并受预算约束 |
| PARTICLE-G33 | GPU particles输出正确velocity/reactive mask，reset/teleport不产生历史拖影 |
| PARTICLE-G34 | bounds在manual/compiled/runtime模式下保守、finite且覆盖trail/teleport |
| PARTICLE-G35 | public script不能伪造alive count、indirect args、bounds或GPU完成证据 |
| PARTICLE-G36 | GPU test无adapter时生成明确skip receipt，required硬件矩阵不把0执行计pass |
| PARTICLE-G37 | clean product lane完成load/instantiate/tick/render/stop/reload/save/reopen/teardown |
| PARTICLE-G38 | 1至1000 systems及百万粒子corpus报告CPU/GPU frame p50/p95/p99和memory峰值 |
| PARTICLE-G39 | device loss、OOM pressure、shader failure、plugin unload、world teardown和24h soak均有终态receipt |
| PARTICLE-G40 | “优于Unreal”只在同硬件、同场景、同画质、同可见正确性与公开统计协议下表述 |

## 15. 风险、依赖与迁移约束

1. 先删除第二authority，再扩功能。若继续同时维护JSON sprites、CPU fallback、GPU owner和VFX no-op graph，新增module只会扩大parity矩阵。
2. 先修per-instance dt、stop/reset和handle exhaustion等正确性，再做百万粒子优化；错误更快不是性能成果。
3. 不能把GPU实现等同“比CPU高级”。无material/history/device recovery/budget的GPU path不能替代qualified CPU fallback。
4. Particle artifact依赖Runtime04/09C，world owner依赖Runtime05/22/24，GPU lifetime依赖09A，visibility/history依赖09B/09H1；实施顺序必须遵守这些底层合同。
5. Editor15只能通过runtime gateway预览，不能再实现私有clock、simulation或固定成功字符串。
6. Bevy没有第一方particle runtime，只能支持render architecture判断；不得用其缺席降低Zircon功能完成标准。
7. Unreal/Fyrox/Godot/Unity参考提供边界和失败预防，不自动证明Zircon采用相同结构即可获得相同性能。
8. 本轮暂停新增tooling优化审查；测试/evidence实现仍消费既有通用控制面，不新开tooling专题抢占runtime修复owner。

## 16. 状态与产出记录

| 里程碑 | 状态 | 日期 | 证据 |
|---|---|---|---|
| Zircon source/test/product inventory | review_complete | 2026-08-16 | 113输入；plugin、VFX Graph、core renderer、Scene/script/Vampire链 |
| 五套参考边界核对 | review_complete | 2026-08-16 | 27输入；Bevy明确仅作render architecture参考 |
| Currentness fingerprint | review_complete | 2026-08-16 | 140输入、43,229行、2,010,138 bytes；SHA-256 `6554789a...d31f` |
| Product authority与CPU/GPU/render深读 | review_complete | 2026-08-16 | 无manager产品tick；JSON旁路；CPU fallback与GPU owner双simulation；VFX no-op |
| Finding与owner裁决 | review_complete | 2026-08-16 | 0 P0 / 60 P1 / 12 P2；与Editor15、Runtime04/05/09/22-24不重复计数 |
| Production重构与动态资格 | pending | - | 本篇只新增review；未修改production、tests、Cargo、manifest或workflow |

完成标准不是“让demo里出现更多粒子”，而是唯一source/artifact/runtime/backend/render authority能在普通产品Scene中被加载、调度、预算、恢复和验证；CPU/GPU、Editor/Product、save/reload与device generation对同一asset给出可解释且有证据的行为。达到这些门之前，Particle/VFX必须继续以experimental/partial呈现。
