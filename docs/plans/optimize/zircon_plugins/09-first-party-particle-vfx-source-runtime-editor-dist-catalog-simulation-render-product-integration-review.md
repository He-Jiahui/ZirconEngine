---
related_code:
  - zircon_plugins/particles/plugin.toml
  - zircon_plugins/particles/runtime
  - zircon_plugins/particles/editor
  - zircon_plugins/particles/dist
  - zircon_plugins/particles/templates/cpu_sprite_system.toml
  - zircon_plugins/first_party_runtime_catalog/Cargo.toml
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/first_party_editor_catalog/Cargo.toml
  - zircon_plugins/first_party_editor_catalog/src/lib.rs
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/first_party_editor_plugins.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/particles_features/rows.rs
  - zircon_runtime/src/scene/world/render_particles.rs
  - zircon_runtime/src/script/vm/gameplay_host.rs
tests:
  - zircon_plugins/particles/runtime/src/tests
  - zircon_plugins/particles/editor/src/tests.rs
  - zircon_plugins/particles/dist/src/lib.rs
  - tests/acceptance/particles-gpu-readback-mailbox.md
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
  - docs/plans/optimize/zircon_plugins/08-first-party-editor-authoring-extension-document-operation-toolkit-runtime-contract-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md
  - docs/plans/optimize/zircon_runtime/26-particle-vfx-system-emitter-cpu-gpu-simulation-rendering-scalability-determinism-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/42-builtin-runtime-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-review.md
  - docs/plans/optimize/zircon_editor/15-material-shader-graph-instance-vfx-particle-preview-compiler-diagnostics-authoring-review.md
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Public/NiagaraComponent.h
  - dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Private/NiagaraWorldManager.cpp
  - dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Private/NiagaraGpuComputeDispatch.cpp
  - dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Private/NiagaraRendererSprites.cpp
  - dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Private/NiagaraRendererMeshes.cpp
  - dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Private/NiagaraRendererRibbons.cpp
  - dev/Fyrox/fyrox-impl/src/scene/particle_system/mod.rs
  - dev/Fyrox/fyrox-impl/src/scene/particle_system/draw.rs
  - dev/Fyrox/fyrox-impl/src/scene/particle_system/emitter/base.rs
  - dev/godot/scene/3d/gpu_particles_3d.cpp
  - dev/godot/scene/3d/cpu_particles_3d.cpp
  - dev/godot/servers/rendering/renderer_rd/storage_rd/particles_storage.cpp
  - dev/godot/scene/resources/particle_process_material.cpp
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor/Data/VFXDataParticle.cs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor/Compiler/VFXGraphCompiledData.cs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor/Compiler/VFXCodeGenerator.cs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Runtime/Types/VFXTypes.cs
  - dev/bevy/crates/bevy_render/src/extract_component.rs
  - dev/bevy/crates/bevy_render/src/render_asset.rs
  - dev/bevy/crates/bevy_render/src/gpu_readback.rs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 09 · First-Party Particle/VFX Source、Runtime、Editor、Dist、Catalog、Simulation、Render 与 Product Integration 工程化差距

## 1. 结论

`zircon_plugins/particles`不是只有名称和空trait的假包。57个tracked文件中已有可保留的CPU SoA pool、free list、局部确定性RNG、rate/burst、四类发射形状、lifetime/color/size曲线、local/world space、sprite extract、GPU attribute layout、双buffer compute、alive compact、indirect draw、异步readback与可选offscreen GPU测试。包也诚实声明为`experimental`、主能力为`Partial`且默认不启用，这一点比用`Complete`掩盖缺口更健康。

但从第一方插件交付视角看，它仍没有形成“source package可编辑、可保存、可编译、可装配、可运行、可卸载、可证明”的产品闭环。runtime catalog实际链接了Particles，editor catalog却不链接editor provider；App因此可以拿到runtime descriptor，却不能在默认Editor中挂载本包的authoring实现。即使孤立注册editor provider，12个领域operation也只有descriptor，没有event或operation factory，12个菜单全部显式disabled；三个真实存在的ZUI资源只是`Space`占位布局，asset toolkit没有document/save/compiler owner，preview没有Runtime session、viewport render source或时间控制器。

NativeDynamic `dist`更加薄弱：它只返回package/registration metadata，command/event清单为空，bridge、invoke、state save/restore、host-ready和unload行为均为空。source runtime里真实存在的manager、render feature、runtime-prepare collector和四个executor无法由dist恢复，source/native parity因此没有成立。manifest存在、ABI version为3、descriptor symbol可导出，只能证明包装格式，不证明插件能力可执行。

运行时内部也存在三条相互竞争的事实源：manager对GPU资产仍创建并推进CPU fallback；renderer-owned GPU owner又克隆同一实例列表独立执行GPU simulation；产品Scene和脚本则继续通过dynamic JSON直接写最终sprite数组。全仓生产调用搜索没有找到普通world/scheduler调用`ParticlesManager::tick`、从typed component实例化/销毁manager实例或把GPU feedback回写manager。Render Graph声明的三个GPU compute pass executor只校验metadata与资源合同，真正compute在runtime-prepare collector中提前录制，图本身并不拥有dispatch、依赖或queue调度。

上述模拟、Scene、render和Editor最高优先级问题已分别由Runtime26、Runtime05/09A/09B/09C、Editor15/50与Plugins01/06登记canonical P0。本篇不重复计数，登记 **0项新增P0、48项P1和12项P2**。本篇拥有的是Particles单包从manifest/source/editor/runtime/dist到first-party catalog和产品consumer的纵向交付合同；算法与系统本体继续由相邻专项实现。

## 2. 审查边界、规模与currentness

### 2.1 物理冻结

| 范围 | 文件 / 行 / bytes | 冻结事实 |
|---|---:|---|
| `zircon_plugins/particles`全包 | 57 / 8,733 / 306,329 | package manifest、runtime、editor、dist、3个ZUI与1个TOML模板全部逐文件扫描 |
| 测试属性 | 41 | runtime 38、editor 1、dist 2；0 ignored |
| 包内manifest fingerprint | `04a0024e1d515eb2e721f8ddd7f8f717c6283c15a96f76383912933c3434ffb6` | 路径不区分大小写排序，以`path|file_sha256`的LF串再计算SHA-256 |
| Editor命令 | 14 | 2个通用OpenView命令可打开surface；12个领域operation无event/factory且菜单disabled |
| Editor资源 | 4 | 3个ZUI与CPU sprite TOML均实际存在，但ZUI只有占位控件，TOML没有真实create/import/save consumer |
| Catalog产品装配 | runtime 1 / editor 0 | runtime source provider已链接；editor source provider未进入first-party editor catalog |

源revision为`25e09a23178000f2e783ce2143cf70a8b118d404`。冻结时Particles包本身没有tracked working-tree差异；共享catalog、App、Runtime、Editor与总账存在其他会话或用户改动，本文只读这些范围并保留`source_recheck_required: true`。实施前必须重新绑定包、catalog、profile与host generation，不能把本报告fingerprint外推到后续源码。

### 2.2 本轮逐层追踪的链路

1. `plugin.toml`的source/library/native包装、target modes、maturity、capability与optional feature。
2. runtime package manifest、module/service、manager、asset/component、CPU simulation、GPU planner/backend/owner、Render Graph feature/executor与readback。
3. editor provider、surface/template/drawer、asset type/toolkit/template、14个command和测试。
4. dist descriptor、registration manifest、callback/event/state/bridge/host-ready/unload能力。
5. first-party runtime/editor catalog与App entry的实际链接，不用generated metadata行替代provider caller。
6. Scene动态JSON粒子、script host、renderer extract与plugin runtime之间的真实产品可达性。
7. Unreal Niagara、Godot、Fyrox、Unity Graphics VFX Graph和Bevy render extraction/asset生命周期的本地源码边界。

本轮是E3静态源码审查，没有运行Cargo、GPU、Editor窗口、NativeDynamic加载、save/reopen或产品场景。41个测试是源码库存，不等于本轮动态通过；无adapter时返回`None`的offscreen测试也不能作为GPU资格。静态证据足以判定provider缺链、operation无执行体、ZUI占位、dist空行为、生产tick无caller和graph executor不dispatch。

## 3. 当前真实产品链与断点

```text
plugin.toml / generated builtin metadata
  -> first_party_runtime_catalog links runtime_plugin()
       -> module + private ParticlesManager
       -> render feature + runtime-prepare collector + four executors
       -> no world scheduler/component lifecycle drives instantiate/tick/remove
  -> first_party_editor_catalog does not link editor_plugin()
       -> 2 surfaces + 12 disabled domain operations are product-unreachable
  -> dist exports descriptor/registration metadata only
       -> cannot reconstruct manager/render/editor operations/resources

Scene/script product path
  -> dynamic JSON render.particle_sprites
  -> World::collect_render_particles
  -> core colored billboard renderer
  -> bypasses ParticleSystemAsset, ParticlesManager and plugin lifecycle

GPU package path
  -> manager advances CPU fallback for GPU asset
  -> runtime-prepare owner aggregates playing GPU instances and dispatches compute
  -> Render Graph compute executors validate declarations but record no compute
  -> manager.apply_gpu_feedback has no production caller
```

这不是一个可通过“再补几个按钮”修复的问题。需要先选定canonical asset/document/runtime instance/render packet，并让catalog、source provider、native provider、Editor、Scene与renderer全部消费同一代合同。

## 4. 可保留基础

| 基础 | 当前价值 | 重构时必须保留的约束 |
|---|---|---|
| Experimental/Partial声明 | 没有把当前状态包装为产品完成 | 在所有产品门完成前继续fail-close；catalog不得擅自升级maturity |
| 包目录分层 | manifest/runtime/editor/dist/template边界清楚 | 保留单包审计入口，同时让dist真正投影source行为 |
| Typed asset词汇 | emitter、range、shape、burst、curve、physics/animation binding已有结构 | 迁移到versioned source schema和compiled artifact，不把当前Rust struct直接当持久格式 |
| CPU SoA/free list | 已有确定seed和slot复用的可运行baseline | 接入per-world schedule、fixed step、job/chunk和generation identity |
| GPU compute底座 | 有真实WGSL、buffer layout、compact、indirect与readback | 迁入真实Render Graph pass和persistent per-instance allocation |
| Renderer-owned device访问 | runtime prepare能在正确设备上下文创建/执行资源 | 补device generation、retirement、last-good与graph dependency，不回退为manager直接持有device |
| Editor contribution schema | surface、drawer、asset type、toolkit、template和operation均可枚举 | descriptor必须绑定document、factory、transaction、compiler和preview owner |
| 真实包内资源文件 | URI对应ZUI/TOML实际存在 | admission需编译资源并验证controller/binding/schema，而非只检查字符串 |
| 测试层次雏形 | CPU、extract、graph metadata、GPU readback、registration均有局部测试 | 增加default host、native parity、product scene、fault、visual和scale lane |

## 5. 参考实现给出的插件交付边界

### 5.1 Unreal Niagara

`UNiagaraComponent`持有asset切换、warmup tick count/delta、desired age、seek delta、单帧最大seek时间、pooling和scalability注册；`NiagaraWorldManager`与system simulation按world组织执行；GPU compute dispatch和Sprite/Mesh/Ribbon renderer各有明确owner。Zircon不需要复制UObject或Niagara VM，但必须有world-scoped instance、bounded seek/warmup、scalability、compiled program和renderer family。一个全局`Mutex` manager、无限rewind循环和单colored billboard不是等价替代。

### 5.2 Godot

`GPUParticles3D`/`CPUParticles3D`将one-shot、preprocess、fixed FPS、fractional delta、interpolation、amount ratio、visibility AABB、trail、sub-emitter和draw order作为序列化与运行时共同消费的合同，rendering storage真实拥有GPU资源。它说明preview字段必须与runtime语义一致，bounds和simulation cadence不能只停留在Editor文字或测试fixture。

### 5.3 Fyrox

Fyrox particle system是Scene原生节点，并通过`Reflect`、`Visit`、`InheritableVariable`持久化playing、emitters、material、coordinate system、visible distance、particles、free list和RNG；draw代码实际消费particle/material。可借鉴的是Scene生命周期、反射持久化和小而完整的CPU产品链，而不是把其CPU实现当作最终性能目标。

### 5.4 Unity Graphics VFX Graph

本地Unity Graphics包中的`VFXDataParticle`维护capacity/aligned capacity、particle/strip、stored source/current attributes、bounds和context flow；compiled data/code generator形成system、buffer、shader、expression与event artifact。Zircon的TOML模板和Rust asset struct不能同时充当source document、compiler IR和runtime program，固定pass descriptor也不能替代attribute liveness与target artifact。

### 5.5 Bevy

当前Bevy checkout没有第一方Particle/VFX runtime，本篇不虚构功能对标。只采用`ExtractComponent`的MainWorld/RenderWorld分界、`RenderAsset`的added/modified/unused事件、device recovery re-extract、prepare retry和bytes-per-frame limiter，以及GPU readback生命周期作为render架构参考。粒子语义仍由Zircon、Unreal、Godot、Fyrox与Unity VFX证据裁决。

## 6. P0归属：本文不新增最高优先级finding

| 已证实现象 | Canonical owner | 本篇责任 |
|---|---|---|
| 默认产品无typed particle world lifecycle，脚本JSON旁路manager | Runtime26、Runtime05 | 规定插件provider必须接入其world/runtime authority，不重复登记P0 |
| CPU fallback、GPU owner与Scene JSON三事实源 | Runtime26、Runtime09A/09B | 约束package只能发布一个runtime/render generation |
| Render Graph pass声明与真实dispatch分离 | Runtime26、Runtime09A | 要求dist/source registration不能宣称未被graph拥有的执行能力 |
| Particle/VFX workbench假编译、无operation factory、无durable document | Editor15、Editor50 | 记录Particles包的12个具体operation和3个占位ZUI |
| first-party catalog/profile/provider closure缺失 | Plugins06、Runtime42 | 记录runtime 1/editor 0的本包纵向事实 |
| NativeDynamic只导出metadata、无法恢复source行为 | Plugins01 | 定义Particles source/native parity gate |

只要Particles继续保持experimental/partial且默认不启用，本篇不把“尚未达到Niagara能力”单独升级为P0。任何profile、UI或发布元数据若将其提升为Complete、required、默认enabled或用户可完成工作流，必须先关闭canonical P0并通过本篇资格门，否则应拒绝发布。

## 7. P1：Package、Catalog、Capability 与Distribution闭环

| ID | 当前差距 | 需要重构 |
|---|---|---|
| PVFX-P1-001 | runtime catalog实际链接Particles，但没有与产品world driver绑定的activation receipt | catalog activation必须返回provider generation、module/service/system/render registrations和product consumer状态 |
| PVFX-P1-002 | editor catalog不链接`zircon_plugin_particles_editor`，默认Editor看不到source authoring provider | 由产品profile解析runtime+editor closure，缺任一required provider时fail-close并给出原因 |
| PVFX-P1-003 | builtin metadata、Cargo feature、first-party provider调用和运行时selection是多份选择事实 | 收敛为BuildSet生成的单一selection manifest，记录requested/resolved/linked/admitted/activated |
| PVFX-P1-004 | `particles.physics`、`animation_control`、`gpu_simulation`主要停留在manifest/catalog行 | optional feature必须绑定实际provider、manager capability generation、降级语义和撤销路径 |
| PVFX-P1-005 | manager由plugin内部默认构造，产品capability selection没有明确注入已解析feature集合 | activation transaction把effective capabilities传入world runtime；禁止测试专用`with_capabilities`成为唯一配置入口 |
| PVFX-P1-006 | source registration含module/render/component/options/event catalog，dist registration只有metadata | 定义versioned `ParticlesProviderContract`，source/native必须投影同一功能集合或明确拒绝不支持包装形态 |
| PVFX-P1-007 | dist的command/event manifest为空，bridge/invoke/save/restore/host-ready/unload均无行为 | 实现可卸载native bridge、state handoff与registration callbacks，或从manifest删除Native声明 |
| PVFX-P1-008 | ABI version和descriptor symbol测试只证明符号/字段，不证明行为能挂载 | 增加真实DLL load、registration materialization、invoke、quiesce、unload/reload和generation隔离测试 |
| PVFX-P1-009 | dynamic event catalog被声明，但没有找到产品publisher/subscriber或schema演进证据 | 每个event绑定producer、consumer、payload schema/version、ordering、budget和unknown-version策略 |
| PVFX-P1-010 | package资源没有hash/media/schema清单，native包也没有资源装载合同 | build时生成resource manifest并纳入签名、artifact key、size budget与URI owner校验 |
| PVFX-P1-011 | package capability可由descriptor存在推导，未绑定world tick、GPU execution或Editor可操作性 | capability readiness必须消费运行时health/feature receipts，禁止metadata自认证 |
| PVFX-P1-012 | 没有source/native、Client/Editor、CPU/GPU、optional feature组合矩阵 | 建立受控组合矩阵并记录source/build/package/provider generation；unsupported组合必须显式拒绝 |

## 8. P1：Editor Authoring、Document、Operation 与Preview闭环

| ID | 当前差距 | 需要重构 |
|---|---|---|
| PVFX-P1-013 | 三个ZUI文件只有命名`Space`占位，没有可交互列表、曲线、viewport、stats或drawer控件 | 建真实workbench布局与typed controller/binding，资源编译失败阻止extension admission |
| PVFX-P1-014 | 12个领域operation只有descriptor，没有event或operation factory | 每个operation绑定typed payload decoder、factory、permission、transaction outcome与错误诊断 |
| PVFX-P1-015 | 12个领域菜单全部`.with_enabled(false)`，却仍可作为已贡献功能被枚举 | enabled状态由document/runtime capability和selection计算；长期不可用操作不得冒充产品入口 |
| PVFX-P1-016 | editor测试在空registry中孤立注册，不能发现default host冲突、资源root或缺factory | 在完整默认Editor registry中执行mount/admission/invoke/revoke测试，并验证失败原子性 |
| PVFX-P1-017 | asset type只声明`particles.system`展示信息，没有source serializer/importer/migration | 以versioned ParticleSourceDocument接入Asset Registry、import/reimport、dependency和unknown-field保留 |
| PVFX-P1-018 | toolkit只保存view id和open operation，不拥有document identity、dirty、save或close | 接Editor共享Document/Toolkit owner，实现open-once、save/save-as、close veto、external conflict和recovery |
| PVFX-P1-019 | CPU sprite TOML只由测试做`.contains()`形状检查，没有解析、schema或创建事务 | 模板必须经同一parser/compiler验证，创建后能save/reopen/cook并产生stable asset identity |
| PVFX-P1-020 | Add Component没有Scene selection、undo/redo、prefab override或runtime handoff | 通过Editor operation transaction写typed component，验证目标world/entity generation并支持revert |
| PVFX-P1-021 | Add Emitter/Add Module/Edit Curve没有canonical graph/module schema | Editor15的VFX/particle schema registry成为唯一authoring authority，操作只产生versioned semantic edits |
| PVFX-P1-022 | Validate Asset没有调用runtime compiler，也没有source location/fix-it | validation输出结构化diagnostic、node/property location、target/backend、generation与可撤销fix-it |
| PVFX-P1-023 | preview五操作没有真实preview session、bounded clock、viewport render或runtime parity | 建隔离preview world，复用runtime compiled artifact；play/pause/stop/rewind/warmup返回typed receipt并受预算限制 |
| PVFX-P1-024 | inspector drawer只声明surface URI，未读写typed component或展示runtime diagnostics | drawer绑定selection/component generation、transaction和只读runtime telemetry，断线时明确Unavailable |

## 9. P1：Runtime Asset、Instance、Simulation 与Interop闭环

| ID | 当前差距 | 需要重构 |
|---|---|---|
| PVFX-P1-025 | `ParticleSystemComponent`内嵌完整Rust asset，asset引用、版本、reload与依赖身份缺失 | component只持typed asset handle和instance overrides；source经compiler产生immutable runtime artifact |
| PVFX-P1-026 | asset字段没有schema version/migration，ID唯一性、非负范围、curve domain和依赖完整性校验不足 | 建source schema/version与semantic compiler，所有分配前做items/bytes/time/dependency admission |
| PVFX-P1-027 | `looped`字段无任何执行consumer，burst和system lifetime没有循环状态机 | 定义loop/one-shot/completion/restart/sub-emitter语义并在CPU/GPU/preview统一执行 |
| PVFX-P1-028 | 全仓生产代码没有普通scheduler调用`ParticlesManager::tick` | 由Runtime26建立per-world `ParticleWorldRuntime`，注册明确stage、clock domain、pause和shutdown顺序 |
| PVFX-P1-029 | typed component没有进入Scene load/instantiate/change/remove/unload链 | 建增量component observer和generation-qualified instance lease，world unload必须释放CPU/GPU资源 |
| PVFX-P1-030 | manager以单一`Arc<Mutex<_>>`串行tick、rewind、snapshot和控制 | 分离command queue、world simulation state、render snapshot与diagnostic channel；重活不持全局锁 |
| PVFX-P1-031 | handle用饱和递增，达到`u64::MAX`后可复用并覆盖live实例 | 使用slot+generation或checked monotonic identity；exhaustion必须typed失败且不可别名 |
| PVFX-P1-032 | mutex poison直接`expect` panic，单实例tick失败还可留下前序实例已推进 | 定义per-instance fault isolation与tick prepare/commit；poison、panic和partial failure可观测、可恢复 |
| PVFX-P1-033 | diagnostics持续append且snapshot全量clone sprites/diagnostics | 使用有界ring、sequence/page/ack和dirty snapshot；记录drop/staleness并限制每帧复制预算 |
| PVFX-P1-034 | preview rewind按caller给定duration/fixed_dt无步数或CPU时间预算 | 采用checkpoint+bounded seek，返回progress/cancel/deadline/terminal receipt |
| PVFX-P1-035 | physics“collision”只施加damping，`bounce`不消费；animation binding不求值 | 通过Runtime08A/08C typed provider查询真实collision/event parameter，缺provider时显式degraded或拒绝 |
| PVFX-P1-036 | capability只能运行时单向enable，没有revoke、generation或现有实例一致切换 | effective capability随plugin generation原子替换；disable/reload按prepare/drain/commit迁移或终止实例 |

## 10. P1：GPU、Render Graph、Rendering 与Qualification闭环

| ID | 当前差距 | 需要重构 |
|---|---|---|
| PVFX-P1-037 | 三个GPU compute executor只验证pass metadata，真实dispatch在runtime prepare中提前发生 | 把spawn/update、compact、indirect dispatch移入真实Render Graph executor，graph拥有resource/barrier/queue/submission |
| PVFX-P1-038 | manager仍推进GPU资产的CPU fallback，GPU owner又独立模拟，`fallback_to_cpu`不随成功清除 | 每实例只有一个active backend authority；切换需checkpoint/generation/explicit reset或可验证迁移 |
| PVFX-P1-039 | GPU owner把所有playing实例聚成一个asset/backend，增删、暂停或asset变化可重建并重置全体 | 使用per-system compiled program与persistent allocation，拓扑变化只影响对应instance/generation |
| PVFX-P1-040 | 聚合执行取实例中的`max_dt/max_age`，不同time scale实例不能保持独立时间语义 | frame packet携带per-instance/per-emitter dt、age和clock epoch；dispatch不得用全局最大值覆盖 |
| PVFX-P1-041 | GPU随机seed不混入instance identity，相同asset/seed实例可能相关或完全相同 | 定义versioned RandomStream key，包含asset/system/emitter/instance/loop/spawn序列并建立CPU/GPU容差合同 |
| PVFX-P1-042 | 1,048,576 slot按遍历顺序先到先得，后续emitter可得到0且无产品scalability策略 | 建world/device预算、priority/significance/LOD、fair admission、pressure eviction和用户可解释降级 |
| PVFX-P1-043 | GPU只消费曲线首尾，忽略initial rotation/angular velocity、physics、animation等CPU字段 | compiler输出backend support matrix；语义不等价时拒绝GPU或生成明确fallback artifact |
| PVFX-P1-044 | CPU/GPU透明shader都未消费material/texture，GPU还缺rotation、sort、soft particle和velocity | 接Runtime09B/09C/09H1的material、PSO、history和transparency合同，输出真实renderer packet |
| PVFX-P1-045 | runtime prepare在无新执行时可复用last bindings，paused/removed/reset后的staleness边界不清楚 | external binding带owner/generation/frame validity；无有效producer时清除或发布typed last-good状态 |
| PVFX-P1-046 | `apply_gpu_feedback`没有生产caller，manager snapshot中的GPU反馈不是产品执行事实 | renderer提交/完成后通过有界feedback channel回写instance generation，过期或丢弃必须计量 |
| PVFX-P1-047 | GPU测试在无adapter时静默返回，graph测试只证明metadata接受；没有device-loss、OOM或pixel oracle | required GPU矩阵区分Passed/Skipped/Unavailable/Failed，并覆盖真实dispatch、pixels、reset、recovery和retirement |
| PVFX-P1-048 | 没有asset create/save/cook/load、Scene instantiate/tick/render、Editor preview与native reload的端到端lane | 建source/native、CPU/GPU、Client/Editor最小产品场景和1/10/100/1000 system规模基准，绑定BuildSet与证据指纹 |

## 11. P2：P1闭环后的生态与高级能力

| ID | 能力 | 前置条件 |
|---|---|---|
| PVFX-P2-001 | 第三方VFX module/data-interface SDK与受控GPU code extension | versioned IR、sandbox、resource access、ABI、budget和签名门完成 |
| PVFX-P2-002 | Sprite/Mesh/Ribbon/Trail/Beam/Light/Decal/Volume renderer插件族 | canonical compiled program、material/PSO、bounds、sort和history完成 |
| PVFX-P2-003 | Vector field、curl noise、SDF、fluid/volume与scene query模块 | physics/render data interface、GPU residency和fault isolation完成 |
| PVFX-P2-004 | Simulation cache、offline bake、cinematic scrub与deterministic replay | fixed step、checkpoint、artifact version、seek budget和Editor preview完成 |
| PVFX-P2-005 | 跨平台预编译kernel、subgroup优化与压缩attribute layout | canonical IR、fallback backend、numerical tolerance和target qualification完成 |
| PVFX-P2-006 | GPU radix sort、depth bins和受控OIT | renderer/material、GPU budget、visual oracle和overdraw corpus完成 |
| PVFX-P2-007 | Effect-type scalability、自动LOD与perceptual quality tier | significance、budget、telemetry、同画质corpus和用户策略完成 |
| PVFX-P2-008 | 远程capture、逐module CPU/GPU cost、state inspection与debug draw | 安全debug channel、有界readback、generation和redaction完成 |
| PVFX-P2-009 | 多用户VFX graph协作、semantic merge和review diff | versioned document、transaction、stable node identity与conflict authority完成 |
| PVFX-P2-010 | Marketplace particle package、依赖解析、签名安装和回滚 | Plugins生态trust、entitlement、atomic install与BuildSet closure完成 |
| PVFX-P2-011 | Niagara/Unity VFX等外部格式的受限导入与转换诊断 | canonical source schema、license/provenance、loss report和migration corpus完成 |
| PVFX-P2-012 | 基于qualified workload的自动性能回归二分与质量建议 | source/build-bound benchmark、硬件分层和长期telemetry证据完成 |

## 12. 目标架构

### 12.1 单一source到runtime链

```text
ParticleSourceDocument vN
  -> semantic validation + dependency resolution
  -> CompiledParticleProgram
       - simulation IR / backend support matrix
       - attribute liveness + layout
       - renderer programs + material dependencies
       - bounds/scalability/budget metadata
       - source map + diagnostics
  -> immutable ArtifactId(BuildSet, target, compiler, dependency closure)
  -> per-world ParticleSystemInstance(slot, generation)
  -> per-frame SimulationPacket
  -> Render Graph passes
  -> RenderPacket + bounded feedback/telemetry
```

TOML或graph只是source document的编码之一；Rust struct不再兼任持久schema和运行时程序。CPU与GPU消费同一compiled semantics，不支持的backend在compile/admission阶段明确拒绝或产生带原因的degraded artifact。

### 12.2 Provider与产品装配

`ParticlesProviderContract`同时描述runtime module/system、render feature、Editor toolkit/compiler/preview、resources、optional feature dependencies和native parity。first-party catalog解析完整closure，activation transaction发布同一个provider generation。source和native provider要么得到相同receipt，要么明确声明某包装形态不支持，不能以空dist维持表面兼容。

### 12.3 World、render和feedback ownership

Scene只持asset handle与instance overrides；`ParticleWorldRuntime`拥有实例生命周期、clock和commands；Render World拥有GPU allocation、dispatch、draw与completion；feedback以instance generation回到world telemetry。script只能发送spawn/stop/set-parameter/event command，不能直接写最终sprites或伪造GPU frame统计。

### 12.4 Editor产品

Editor toolkit持有versioned document、dirty/save/close、transaction、compiler job和preview session。preview创建隔离world并加载与产品相同的compiled artifact；warmup/seek有预算、取消和进度。所有按钮、菜单和capability状态来自当前document/provider/runtime generation，不使用固定disabled或固定成功文本。

## 13. 分层实施计划

### M0 · Truth Freeze与Catalog Closure

- 保持Particles为experimental/partial/default-off，禁止profile提前提升；
- 生成runtime/editor/source/native capability parity表；
- first-party profile必须同时解析runtime与editor provider closure；
- 删除或禁用无法由source/native真实执行的声明。

### M1 · Source Schema、Document与Artifact

- 建`ParticleSourceDocument v1`、migration、dependency和bounded parser；
- 统一TOML、Editor graph/module/curve与Runtime compiled schema；
- 产出带source map、backend support、layout、renderer和budget的artifact；
- 完成create/open/edit/undo/save/reopen/cook/load闭环。

### M2 · Per-World Runtime与CPU基线

- 将typed component接入Scene生命周期和明确schedule；
- 使用slot+generation、command queue、有界diagnostic和fixed-step budget；
- 实现loop/one-shot/completion、physics/animation bridge与failure isolation；
- 删除dynamic JSON最终sprite作为shipping particle authority。

### M3 · Render Graph与Persistent GPU Runtime

- 将真实compute迁入graph executor并由graph拥有barrier/queue/submission；
- 建per-instance persistent allocation、backend switch和device recovery；
- 统一CPU/GPU语义、RandomStream、bounds、readback和feedback；
- 完成budget/scalability、OOM与large-scene qualification。

### M4 · Renderer Family与Editor Preview

- 完成material/texture/rotation/sort/soft particle/velocity的sprite baseline；
- 再引入mesh/ribbon等renderer family；
- 建真实particle toolkit、module stack、curve editor、diagnostics和viewport；
- preview与runtime使用相同artifact、clock和render path。

### M5 · Native Distribution与Release Qualification

- 实现Particles native provider materialization、state/quiesce/unload/reload；
- 通过source/native、Client/Editor、CPU/GPU和optional feature矩阵；
- 建产品场景、pixel、fault、device-loss、soak、memory和performance证据；
- 只有所有required gate通过后才允许提升maturity或默认启用。

## 14. 验收门

| Gate | 验收内容 |
|---|---|
| PVFX-G01 | package manifest、generated metadata、runtime/editor catalog和App selection由同一BuildSet生成且无漂移 |
| PVFX-G02 | 默认Editor能够真实mount Particles editor provider；缺runtime dependency时给出typed rejection |
| PVFX-G03 | 14个command中所有可见领域operation都有factory/event、payload schema、permission和terminal receipt |
| PVFX-G04 | 三个ZUI经真实模板编译并绑定controller/data；不存在纯`Space`冒充workbench |
| PVFX-G05 | CPU sprite模板由canonical parser/compiler消费，create后可save/reopen/cook/load |
| PVFX-G06 | toolkit支持document identity、dirty、undo/redo、save/save-as、close veto、external conflict和crash recovery |
| PVFX-G07 | preview play/pause/stop/rewind/warmup实际驱动隔离runtime world，且seek/warmup受预算和取消控制 |
| PVFX-G08 | typed particle component完成Scene load/change/remove/world unload，不保留stale instance或GPU resource |
| PVFX-G09 | 产品scheduler每tick驱动per-world runtime，clock/pause/time scale/fixed step语义可复现 |
| PVFX-G10 | script不能写最终sprite或GPU统计，只能调用受限typed particle commands |
| PVFX-G11 | loop/one-shot/burst/completion/restart在CPU、GPU和preview上通过同一语义corpus |
| PVFX-G12 | asset parser在任何分配前限制items/bytes/time/depth，并校验ID、range、curve和dependency |
| PVFX-G13 | handle exhaustion、stale generation、mutex poison、tick failure和world teardown均typed且不别名/不partial commit |
| PVFX-G14 | diagnostics/readback/snapshot均有有界容量、sequence、staleness/drop和分页语义 |
| PVFX-G15 | optional capability启用、禁用、provider reload对现有实例原子且有明确migration/reset结果 |
| PVFX-G16 | physics collision真正查询physics provider并消费bounce；animation binding真正求值或明确Unsupported |
| PVFX-G17 | CPU/GPU每实例只有一个active simulation authority，backend切换不会双推进或静默重置其他实例 |
| PVFX-G18 | Render Graph executor真实记录compute，resource/barrier/async queue与submission均可由capture证明 |
| PVFX-G19 | 每实例dt/age/seed独立；暂停、增删或asset reload不会重建并重置无关系统 |
| PVFX-G20 | GPU budget按world/device/priority公平准入，0-capacity、OOM和pressure有可解释降级 |
| PVFX-G21 | compiler发布CPU/GPU support matrix，不支持字段不能被GPU静默忽略 |
| PVFX-G22 | material、texture、rotation、sort、soft particle、bounds和velocity/history通过GPU像素与时序oracle |
| PVFX-G23 | renderer feedback绑定instance/provider/device generation；迟到、丢失与stale反馈不可污染当前snapshot |
| PVFX-G24 | 无adapter测试记录Unavailable/Skipped而非通过；required GPU lane必须实际执行case并保存adapter/driver信息 |
| PVFX-G25 | source与NativeDynamic provider贡献、行为、resource、state和capability receipt达到parity |
| PVFX-G26 | native enable/disable/reload等待callback、GPU work和allocation quiesce，不泄漏旧generation |
| PVFX-G27 | clean product scene完成create、save、cook、load、instantiate、tick、visible render、stop、reload和reopen |
| PVFX-G28 | 1/10/100/1000 systems及million-particle corpus记录CPU/GPU/frame/RSS/VRAM/upload/readback p50/p95/p99 |
| PVFX-G29 | device loss、OOM、invalid asset、missing dependency、plugin crash和unload fault injection具有last-good/recovery |
| PVFX-G30 | capability只有在provider、artifact、runtime consumer、render output和evidence同代时才能升级Complete |
| PVFX-G31 | `git diff --check`、Markdown链接/frontmatter、finding唯一性和portfolio计数稳定通过 |
| PVFX-G32 | 与Runtime26、Editor15、Plugins01/06的owner边界保持单一，不新增第二套schema/compiler/host/runtime authority |

## 15. 风险、依赖与迁移约束

1. 不应直接扩充当前`ParticleSystemAsset`字段来追赶Niagara。先建立source schema、compiled program和backend support matrix，否则每个新字段都会扩大CPU/GPU/Editor三方漂移。
2. 不应在现有runtime-prepare之外再补一个“真正graph”并长期双跑。迁移必须有单一dispatch owner和明确切换点。
3. 不应让Editor直接锁manager内部状态。preview通过隔离world、commands和snapshot交互，避免Editor生命周期污染产品runtime。
4. 不应因CPU fallback可见而把GPU成功标为已验证。fallback、degraded、GPU-executed必须是不同状态并进入receipt。
5. 不应为NativeDynamic维持空壳兼容。短期无法实现行为parity时，应删除该distribution form或明确Unsupported。
6. 不应以测试数量衡量产品完成度。当前41项测试主要证明局部算法与descriptor；default host、product world、pixels、fault和scale才是提升maturity的证据。
7. 性能目标必须在同场景、同画质、同硬件和正确性门通过后比较；不能靠删除材质、碰撞、排序、history或失败处理获得表面优势。

## 16. 状态与产出记录

| 项目 | 状态 | 日期 | 证据 |
|---|---|---|---|
| Particles 57文件逐文件静态审查 | review_complete | 2026-08-19 | 8,733行、306,329 bytes、fingerprint `04a0024e...ffb6` |
| Catalog/App/source/native产品链 | review_complete | 2026-08-19 | runtime provider已链接、editor provider未链接、dist为空行为投影 |
| 五套参考源码边界 | review_complete | 2026-08-19 | Unreal/Fyrox/Godot/Unity Graphics/Bevy本地源码 |
| Finding登记 | review_complete | 2026-08-19 | 0 P0 / 48 P1 / 12 P2；32项资格门 |
| Production与tests修改 | pending | - | 本篇仅review与重构计划，没有修改或验证实现 |

本报告完成的是Particles第一方插件包纵向审查，不表示Runtime26、Editor15或Plugins01/06已实施，也不表示当前粒子能力达到工程级产品资格。
