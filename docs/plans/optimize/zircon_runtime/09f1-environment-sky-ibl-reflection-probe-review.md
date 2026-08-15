---
related_code:
  - zircon_runtime/src/core/framework/render/environment
  - zircon_runtime/src/asset/importer/environment_ibl.rs
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_cache.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/graphics/runtime/render_framework/render_framework_state/environment_ibl_hydration_cache.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/environment_ibl_compile_options.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/environment_brdf_lut.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/environment_cubemap.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment
  - zircon_runtime/src/graphics/shader/wgsl/zr_environment_core.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_environment.wgsl
  - zircon_runtime/src/scene/world/render.rs
  - zircon_plugins/rendering/features/reflection_probes
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_runtime/09e-direct-lighting-clustered-shadow-review.md
  - docs/plans/performance/01/2026-07-18-runtime-core-framework-render-environment-static-review.md
  - docs/plans/performance/01/2026-07-18-graphics-environment-ibl-bake-static-review.md
  - docs/plans/performance/01/2026-07-18-graphics-environment-probe-buffer-static-review.md
  - docs/plans/performance/01/2026-07-18-graphics-realtime-ibl-static-review.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/SkyLightComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/ReflectionCaptureComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/SkyAtmosphereComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/VolumetricCloudComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ReflectionEnvironment.cpp
  - dev/UnrealEngine/Engine/Shaders/Private/ReflectionEnvironmentShared.ush
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/PostProcessing.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ReflectionEnvironmentCapture.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ReflectionEnvironmentRealTimeCapture.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/SkyAtmosphereRendering.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/VolumetricCloudRendering.cpp
  - dev/bevy/crates/bevy_light/src/atmosphere.rs
  - dev/bevy/crates/bevy_pbr/src/atmosphere
  - dev/bevy/crates/bevy_pbr/src/light_probe/environment_map.rs
  - dev/bevy/crates/bevy_pbr/src/light_probe/generate.rs
  - dev/bevy/crates/bevy_pbr/src/light_probe/light_probe.wgsl
  - dev/bevy/crates/bevy_core_pipeline/src/skybox/mod.rs
  - dev/godot/scene/resources/environment.h
  - dev/godot/scene/3d/reflection_probe.h
  - dev/godot/servers/rendering/renderer_rd/environment/sky.cpp
  - dev/godot/servers/rendering/renderer_rd/shaders/environment/sky.glsl
  - dev/Fyrox/fyrox-impl/src/scene/skybox.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Sky/SkyManager.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Sky/PhysicallyBasedSky/PhysicallyBasedSky.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Sky/PhysicallyBasedSky/SkyLUTGenerator.compute
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Sky/AmbientProbeConvolution.compute
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/Reflection/HDProbe.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/Reflection/HDProbeSystem.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/Reflection/ReflectionProbeTextureCache.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Editor/Lighting/Reflection/HDBakedReflectionSystem.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 09F1 · Environment / Sky / IBL / Reflection Probe 工程化差距

## 1. 结论

Zircon当前的环境光并非空壳。代码已经具备版本化的canonical IBL recipe、source cubemap/PMREM/SH9/IEM数据模型、HDR/EXR导入、CPU并行投影与预过滤、GPU PMREM/SH9/IEM compute、pipeline cache、有界runtime bake/readback/writeback、原子cache写入、prepared RGBA16F upload artifact、可复用staging arena、程序天空realtime IBL双缓冲和分帧状态机。Reflection Probe也已有box/sphere influence、box projection、priority、blend distance、layer、top-two blend、固定slot LRU、revision upload与真实WGPU product test。这些是重构输入，不应被另起一套系统抛弃。

但是普通项目无法创作或持久化这些能力。scene component、scene asset、prefab、property path和World没有Sky、Environment、SkyAtmosphere、Cloud或ReflectionProbe实体；`build_environment_extract`只把viewport的`preview_skybox`布尔值转成默认gradient。现有PBR/Probe产品测试在World生成snapshot之后手工覆盖`snapshot.environment`，所以它们证明了手工注入数据能到达像素，却没有证明项目资产能够使用该功能。Reflection Probe editor插件导出一个Rust trigger，但仓内没有Editor consumer；Workbench中相似的“烘焙”交互也没有调用它。捕获后只把PMREM临时注册到内存asset manager，既不修改scene，也不建立catalog/cook依赖。

程序天空本质仍是三色gradient加sun disc。没有Rayleigh/Mie/ozone、transmittance/multi-scattering/sky-view/aerial-perspective LUT，没有太阳与DirectionalLight的共同authoring，没有雾、云、天气、时间或environment volume。更严重的是realtime IBL把`CaptureSky`和`CaptureCloud`映射到完全相同的gradient capture函数并写入相同cubemap faces；首次更新还在一帧记录六面Sky、六面“Cloud”、全部source mips、全部PMREM mips与SH9。“Cloud”不是简化云，而是重复覆盖和重复GPU工作，不能算作已有云系统。

资源和线程边界也未工程化。`EnvironmentExtract`把包含Arc texel链、SH9、可选IEM和prepared upload bytes的`SourceCubemapEnvironment`直接放入逐帧snapshot，而不是传稳定resource handle。runtime cache miss会在frame submission构建路径同步`fs::read`和decode `.zribl`；普通项目导入在判断staged bundle是否可复用之前先把HDR完整解码成RGBA32F，之后还读取并解码`.zcube`和`.zribl`来判定“Current”。Reflection Probe prepare会在pre-draw路径同步`load_texture_asset`，其`ensure_resident`可继续做artifact I/O/decode，再按face/mip执行48次`queue.write_texture`。

Probe着色扩展性远低于工程目标。CPU每帧分配candidate列表并排序/截断到64，fragment shader随后对每个像素再次线性扫描最多64个probe选top two；已有CPU `select_reflection_probe_blend`并不是产品consumer。layer mask虽打包进`GpuReflectionProbe.misc.w`，shader完全不读取它，环境着色入口也没有当前物体layer，因此实际只按camera layer做粗过滤。64个128x128 RGBA16F、8 mip probe slot约占64 MiB，另加固定1024 planar reflection约11.18 MiB；full renderer在没有probe时也可预留约75 MiB，且没有resolution tier、compression、streaming或VRAM budget owner。

离线与实时bake的局部算法已经比旧审查时完整，但调度仍有明显热路径成本。每个IBL graph pass都会重新生成整份command plan后线性查找本pass，并创建params buffer/bind group；程序天空capture/downsample同样逐operation创建小资源。Reflection Probe capture则顺序克隆六份完整scene snapshot，执行六次HDR render/readback，保留全部float face texel，再在CPU生成mip、PMREM和SH9并同步落盘。它没有后台job、取消、进度、预算、增量失效、自动重捕获、headless cook worker或失败恢复，唯一真实capture product test还被标成manual ignored。

本轮登记10项P0、20项P1、5项P2。P0先建立scene/editor/cook闭环、物理天空与真实云语义、统一environment generation、异步资源/烘焙边界、可扩展probe assignment和有效layer contract；P1再消除热路径allocation/I/O、固定格式与常驻内存、shader能量/降级错误、失效与诊断缺口；P2才进入光谱大气、分布式烘焙、ray/path-traced reference、超大世界和多GPU。完成相同天空、相同反射探针密度、相同画质和相同硬件的CPU/GPU/RAM/VRAM基准前，不能声称性能或表现优于当前Unreal。

## 2. 审查边界与覆盖

### 2.1 已读范围

| 子域 | 文件 / 物理行 | 本轮判定 |
|---|---:|---|
| core environment ABI / CPU IBL / artifact | 28 Rust / 8,661 | E3：skybox、probe、recipe、projection、mip、PMREM、SH9/IEM、blob与upload artifact |
| asset import / cache / project scan | 12 Rust / 5,197 | E3：HDR decode、reuse判断、并行builder、source/derived/runtime cache与project scan入口 |
| submission hydration / compile | 3 Rust / 1,462 | E3：逐帧cache resolution、bounded hydration/pending bake与graph compile option |
| GPU environment / probe / realtime IBL | 50 Rust + 10 WGSL / 15,154 | E3：cubemap upload、BRDF LUT、probe资源、GPU bake、readback/writeback、realtime scheduler与最终shading |
| scene / editor / reflection-probe plugin | 17 Rust + 2 Cargo manifest / 2,465 | E3：World extract断点、viewport preview override、capture trigger、六面捕获与注册 |
| 合计 | 122 / 32,939 | 273个inline test属性；focused fingerprint `2f6b75b65a6da938ee3c27843ef0752f1d0e77a2b2fc62d835b810527a29d9ba` |

另抽查20个独立environment/IBL/probe产品或integration test文件、8,301行、116个test属性。它们未计入focused fingerprint；其中多项通过直接构造`EnvironmentExtract`或覆盖World snapshot绕过authoring，另有大量WGSL source-string断言，不能等同scene到pixel的产品闭环。“E3”表示读到实现、调用链、线程/资源和失败语义，不表示真实GPU动态验收完成。

### 2.2 本轮归属与后续边界

- 09A拥有device/queue/submission、render graph version/access、async queue truth、GPU completion与device loss；09F1的bake/capture/readback不能私建第二套lifetime或把graph的`AsyncCompute`标签当作真实硬件并发。
- 09B拥有persistent RenderScene、view family、visibility和GPU Scene；environment/probe generation从它取得scene/view generation，不能继续clone完整snapshot或每像素扫描全局probe表。
- 09C拥有shader module/PSO generation、公共include和artifact version；09F1负责environment BRDF与sky/probe算法，不允许procedural sky在多个WGSL中继续独立漂移。
- 09D拥有asset handle、residency、upload、budget、streaming和cook artifact；09F1定义environment专用内容与调度策略，不继续在frame extract中传整份resource payload或同步`ensure_resident`。
- 09E拥有DirectionalLight和物理光度contract；09F1必须与其共享太阳身份、光度、shadow/cloud/fog耦合，不再维护一个只存在于`ProceduralSkyParams`的独立sun。
- 09F2单独审查baked lighting、lightmap、irradiance volume和offline bake。本文只登记`AmbientLight.affects_lightmapped_meshes`在最终环境着色未消费，以及sky/probe artifact必须进入共同cook manifest。
- 09F3单独审查Hybrid GI。本文只定义sky/IBL/probe作为indirect source的generation和资源接口，不提前判定HGI算法。
- 09G单独审查froxel、volumetric、cookie、planar/screen-space reflection、SSS与advanced material lobes。本文的reflection只覆盖global IBL和local cubemap probe；固定planar资源约11.18 MiB只作为共享内存压力记录。

### 2.3 参考引擎边界

- Unreal把SkyLight、ReflectionCapture、SkyAtmosphere和VolumetricCloud建成可序列化组件。SkyLight公开captured/specified source、realtime capture、resolution、lower hemisphere、occlusion和cubemap blend；ReflectionCapture有capture queue、MapBuildData identity与encoded HDR；SkyAtmosphere公开Rayleigh/Mie/absorption/multi-scattering/aerial perspective；render端使用RDG、排序capture、irradiance buffer和独立实时捕获管线。Zircon的gradient和手工snapshot注入不能映射为同级功能。
- Bevy提供ECS `Atmosphere`介质/phase/density模型、transmittance/multi-scattering/sky-view/aerial LUT，EnvironmentMapLight、生成/过滤compute、LightProbe与layer/lightmapped flags。它不是Zircon最终性能上限，但证明现代Rust引擎也能把authoring、extract、GPU resource和shader消费接成闭环。
- Godot `Environment`把background、sky、ambient source、reflection source、fog和tonemap统一成资源；`ReflectionProbe`公开update mode、ambient mode、cull/reflection mask、box projection、LOD与blend；RD sky owner负责radiance和processing。其固定/传统限制只能作为工程完整性下限。
- Fyrox SkyBox虽算法较简单，但节点具备Reflect/Visit、资源引用、构建器、序列化和纹理一致性验证。Zircon当前连这一最低限度的scene asset roundtrip都未达到。
- Unity HDRP `SkyManager`维护update context/hash、reflection cubemap、ambient probe compute、static lighting sky和RenderGraph流程；Physically Based Sky有独立LUT/precompute；HDProbeSystem、texture cache和baked reflection editor把注册、culling、Baked/Custom/Realtime模式、preview与持久化连通。仓内Graphics不包含Unity native core，本文只以可见SRP contract作对照。

### 2.4 明确未做

本轮没有修改production code，没有运行Cargo、Editor、cook、真实GPU、PIX/RenderDoc、WPR、device loss、VRAM pressure、跨平台、视觉golden或同场景Unreal benchmark。当前environment/importer/probe范围有大量其他Session修改和未跟踪拆分文件，因此标记`source_recheck_required`；后续实施前必须重新生成fingerprint并复核所有行级结论。

## 3. 当前必须保留并迁移的基础

### 3.1 canonical IBL recipe与artifact identity必须保留

当前recipe统一PMREM 128/8、Normal 32/64/128 samples、diffuse 64、可选IEM 32、内容位和版本；request/hash/descriptor能区分source、layout、quality与artifact内容。重构应把它升级为平台/quality profile和cook manifest，不应退回隐式常量或仅按URI缓存。

### 3.2 immutable Arc数据与prepared upload可迁到09D resource owner

`SourceCubemapMipChain`用Arc持有source/PMREM texels，upload mip bytes也用Arc，clone不再深拷贝大数组。`SceneEnvironmentCubemap`可只上传变化部分，并用复用staging arena批量`copy_buffer_to_texture`。这些数据结构可成为异步resource build的payload，但必须退出逐帧scene contract。

### 3.3 project TaskPool并行构建和timing是有效离线基础

HDR投影、source mip、PMREM和IEM已能使用manager拥有的TaskPool/parallel slice executor，并记录decode、projection、mip、prefilter与IEM工作量。下一步是迁入可取消cook job和GPU/offline worker，而不是删除已有确定性CPU oracle。

### 3.4 GPU IBL pipeline cache与有界readback/writeback值得保留

shader module、bind layout、pipeline和sampler已有cache；runtime bake pending、hydration LRU与writeback队列均以4为上限，readback使用非阻塞poll，cache写入采用临时文件/rename。需要修的是graph pass重建、I/O线程、generation/fence和错误终态，不是回退到同步全CPU路径。

### 3.5 realtime IBL双缓冲、发布语义和compiled graph cache可保留

sliced update写work slot，在完整结果就绪前继续采样ready slot；graph topology有有界variant cache，timestamp/readback只在显式启用时创建。这是正确的stale-while-revalidate方向。必须删除伪Cloud pass，并给首次更新、静态天空和动态天空不同预算。

### 3.6 ReflectionProbe几何、validation、slot retention与top-two基础可迁移

box/sphere influence、rotation、projection extents、priority、blend distance、intensity、layer和revision均已有验证/ABI；slot allocator能按revision复用并在64槽内LRU。目标系统可复用这些authoring语义和cache identity，但probe assignment必须从per-fragment全扫改成GPU-driven spatial list。

### 3.7 parity/unit/product fixtures可转成新contract回归

现有cubemap seam、CPU/GPU PMREM/irradiance parity、artifact corruption、runtime cache、probe blend/upload、realtime slice与WGPU capture fixture覆盖广。实施时先把它们迁移到scene asset、generation、async job和resource handle，再删除只锁定源码字符串或手工inject snapshot的测试表面。

## 4. P0 差距清单

### P0-1：scene asset与World没有Environment/Sky/Probe authoring闭环

`build_environment_extract`只读取viewport `preview_skybox`并生成默认gradient；scene schema、component registry、property path、prefab、save/load和script API均没有Sky/Environment/ReflectionProbe。必须增加versioned scene contract和migration，并以真实project roundtrip到pixel为验收，禁止再用测试覆盖snapshot伪装产品入口。

### P0-2：Editor capture trigger没有接入scene、undo、catalog或cook

`ReflectionProbeCaptureEditorTrigger`只在插件内部导出和调用，仓内没有Workbench/Inspector/command consumer。`register_captured_reflection_probe`同步读取`.zribl`并只向内存asset manager插入TextureAsset，未创建/更新scene component、asset metadata、dependency、dirty state、undo transaction或package manifest。该插件目前是library API样例，不是Editor功能。

### P0-3：程序天空不是物理天空，`CaptureCloud`还是重复覆盖

三色gradient、sun disc和独立rotation不能提供大气透射、多重散射、aerial perspective、地表反照率、行星尺度或稳定曝光。`CaptureSky`与`CaptureCloud`共用同一次`record_capture`并写相同faces，首次更新把相同gradient渲染两遍。必须删除伪Cloud语义，建立物理SkyAtmosphere/Cloud组件、LUT和明确的静态/动态更新图。

### P0-4：太阳、天空、雾、云、曝光和DirectionalLight没有共同真值

`ProceduralSkyParams.sun_direction`与scene DirectionalLight互不关联，environment revision也不知道direct light、fog/cloud或exposure的变化。需要唯一`EnvironmentGeneration`与显式dependency graph：太阳/大气变化只invalidate受影响LUT/IBL，云只更新其capture/ambient/shadow数据，exposure不应无条件重烘radiance。

### P0-5：environment资源以逐帧payload传递，且submission路径同步读盘

`SourceCubemapEnvironment`携带Arc source/PMREM texel、IEM、SH9和prepared upload artifact进入`EnvironmentExtract`。runtime hydration miss在`build_frame_submission_context`调用cache store同步`fs::read`/decode。必须硬切换为09D拥有的typed handle、residency ticket和last-good GPU generation；render/submit线程只查询ready状态，I/O/decode/cook不得进入其锁域。

### P0-6：Reflection Probe prepare可同步ensure-resident并以48次texture write上传

probe candidate选中后，pre-draw `prepare`调用`load_texture_asset`；资产未驻留时可能同步读取/解码artifact。每个新PMREM再对8 mip x 6 face执行`queue.write_texture`。需要提前residency admission、batched staging copy、upload byte/time budget和deferred readiness，未就绪probe使用last-good/sky fallback而不是阻塞帧。

### P0-7：每像素线性扫描64个probe，layer contract实际失效

CPU先按camera layer和距离选择最多64个probe，WGSL又对每个fragment线性扫描全部probe选top two。`misc.w`保存layer mask却无任何shader读取，入口也没有object layer。必须由persistent scene/visibility生成object/cluster/tile probe list，shader只遍历局部短列表，并把object/camera/capture/reflection mask统一成可测试ABI。

2026-08-16 source quantification: `MAX_REFLECTION_PROBES` is 64 and the full
array reserves 64 `128x128`, eight-mip, six-face RGBA16F cubemaps. Its exact
payload capacity is 67,107,840 bytes (about 64 MiB); the fixed `1024x1024`,
eleven-mip RGBA16F planar chain adds 11,184,808 bytes (about 10.7 MiB). The
full-path shader's `zr_environment_select_probes` executes one loop iteration
per admitted probe for every shaded fragment: 1920x1080 with 64 probes is
132,710,400 iterations per frame (7,962,624,000 at 60 fps), and 3840x2160 is
530,841,600 iterations per frame. Those values exclude cubemap sampling,
box-projection and shading work; they prove the fixed-array algorithm cannot
become the 1k-probe product path by tuning branches or lowering a hidden cap.

The M9 replacement must derive one visibility-owned
`ReflectionProbeSpatialAssignment` from the persistent scene generation. It
contains a view/cluster transform, object reflection-mask input, packed probe
indices, offset/count table, bounded per-cluster capacity from the quality
profile, and explicit overflow metadata. Forward and deferred material paths
call the same WGSL lookup to obtain a local list before ranking the existing
top-two candidates. Probe authoring (shape, priority, blend and box
projection) remains in `GpuReflectionProbe`; only candidate assignment moves
out of the fragment-wide scan. The builder runs on probe/scene/view-generation
change, not once per material or fragment, and reports list-length distribution,
overflow, fragment visits, upload bytes and residency misses through the single
environment telemetry snapshot.

### P0-8：Reflection Probe捕获是同步六次render/readback加CPU bake

捕获为每个face clone完整snapshot，顺序调用六次`render_scene_color_hdr`，保留全部RGBA32F结果，随后CPU建mip/PMREM/SH9并同步持久化。需要可取消后台capture job、一次graph中的六view/batched submission、GPU filter、异步readback/写入、scene revision失效、progress和失败恢复；Editor manual、runtime on-demand与cook worker必须共享同一job contract。

### P0-9：正常导入与warm-cache判定仍做完整decode/read

普通`stage_environment_ibl_source`在reuse判断前先解码整张HDR为RGBA32F；`staged_bundle_state`读取并解码`.zcube`，再读取derived artifact才判定Current。必须先用source fingerprint、manifest header、schema/recipe/platform hash做轻量命中，内容块按需验证；完整decode/filter只能发生在明确miss或审计模式，并由cook scheduler承担。

### P0-10：没有统一budget/readiness/失败终态，无法证明优于Unreal

当前存在import timing、runtime bake queue和部分probe错误，但没有统一记录environment generation、cache hit原因、CPU/GPU bake时间、capture latency、upload bytes、probe list occupancy、VRAM、fallback、stale age和device loss。必须建立单一readiness/telemetry truth和匹配画质benchmark协议；关闭大气、减少probe或使用低分辨率取得的帧时不能算“优于”。

## 5. P1 差距清单

### P1-1：每个IBL graph pass重复构造整份command plan

`record_ibl_bake_wgpu_pass_for_request`每次调用`ibl_bake_wgpu_command_plan_for_request`，分配全部PMREM/SH9/IEM command后再线性查找当前pass。compiled graph应直接持有immutable encoded command/parameter index，不在record热路径重建字符串和Vec。

### P1-2：IBL与realtime recorder逐dispatch创建params buffer/bind group

pipeline虽已cache，小uniform和bind group仍按mip/face/operation创建。应使用persistent parameter arena、dynamic offset或push-constant等后端能力，并按pipeline/layout批量record；稳定generation不得有heap/RHI对象增长。

### P1-3：首次realtime IBL把全部工作塞进一帧

首次发布前的full update包含两次六面capture、全部downsample、PMREM和SH9，容易形成明显首帧hitch。需要预烘fallback、分阶段ready policy、GPU time budget和可选低分辨率bootstrap，再渐进替换为目标质量。

### P1-4：静态gradient也自动进入realtime IBL

只要程序sky intensity大于0，renderer就为其启用realtime resources；没有Static/OnChange/TimeSliced/EveryFrame策略、minimum interval、change threshold或importance。静态天空应优先cook/cache，动态天空按revision和预算更新。

### P1-5：procedural sky存在多份WGSL owner

skybox draw与realtime capture各自实现gradient/sun公式，环境fallback又从公共shader采样同类参数。必须由09C的单一sky module生成display、capture和reference variant，source hash进入environment generation，避免同场景天空与反射不一致。

### P1-6：无PMREM的procedural fallback在粗糙度上错误

fallback specular直接按reflection direction取gradient，与roughness无关；diffuse也直接按normal方向取sky颜色而非半球卷积。该路径只能作为明确标识的临时last-good/preview降级，不能无提示进入产品画质。

### P1-7：BRDF LUT仍在首个renderer构造时做CPU积分

128 x 32 x 128共524,288个sample iteration通过`OnceLock`每进程执行一次，再上传RG16F。应作为versioned builtin/cook artifact或可信预生成常量随shader recipe发布；若支持运行时重建，也必须后台化并有last-good LUT。

### P1-8：基础IBL只有single-scatter split-sum质量合同

当前2-channel GGX LUT、PMREM和specular occlusion可作为standard PBR baseline，但没有明确multiple-scattering energy compensation、LUT domain/version与advanced lobe共享约束。clearcoat/sheen/anisotropy等归09G；09F1必须先固定base lobe参考和能量误差门禁。

2026-08-16 current-source recheck confirms that the immediate base-lobe AO
formula is not a Zircon-specific visual patch: `zr_environment_specular_occlusion`
uses `saturate(pow(NoV + AO, roughness^2) - 1 + AO)`, exactly matching Unreal
`GetSpecularOcclusion` in `ReflectionEnvironmentShared.ush`. Zircon applies
that scalar to the split-sum specular term, applies AO to indirect diffuse,
and its advanced clearcoat path calls the same helper. Therefore the next
hard cut must not replace that formula in isolation. It must introduce a
versioned `EnvironmentPbrRecipe` that fixes PMREM roughness mapping, LUT
domain/precision, base-lobe and clearcoat energy ownership, and the reference
scene/error gate as one contract. Multiple-scattering compensation remains
explicitly disabled until every supported indirect lobe consumes the same
versioned recipe and the error gate accepts it.

### P1-9：CPU IEM构建复杂度过高

direct cosine IEM的估算是source texels乘output texels，虽然已并行仍会随输入/输出分辨率快速放大。应以GPU compute或SH重建作为生产路径，CPU direct integration保留为离线oracle/高质量选项，并记录samples、误差和耗时。

### P1-10：artifact为raw RGBA16F整体blob，缺少流式/平台格式

`.zcube/.zribl`没有BC6H/RGB9E5等平台profile、chunk table、independent mip/face read、compression budget或transcode generation。不能为了固定128 probe而把所有HDR数据永久保持同一raw格式；格式变化需recipe/schema硬版本化。

### P1-11：prepared upload会同时保留float texel与RGBA16F bytes

SourceCubemapEnvironment在CPU侧可同时持有source/PMREM RGBA32F Arc与预编码RGBA16F upload bytes，upload arena又把当帧内容拼接到复用Vec并写GPU staging。需要明确CPU retention policy，在GPU ready/cook完成后释放可重建数据，且全部内存进入09D budget会计。

### P1-12：probe与planar资源固定预留约75 MiB

64 probe slot约64 MiB，固定1024 RGBA16F planar mip chain约11.18 MiB；full renderer可能在零probe/零planar时仍创建。应按feature demand惰性创建、按quality/device预算选择resolution/capacity、在压力下可回收并保留last-good，而不是只有environment-only preview特例。

### P1-13：probe格式被硬编码为128、8 mip、RGBA16F

consumer拒绝其他resolution/mip layout，所有slot同质。需要per-probe quality class、platform format、atlas/array page identity和清晰的sampling metadata；默认值可以保留，但不能成为ABI唯一合法值。

### P1-14：probe CPU admission每帧分配、partial sort和再次sort

`prepare`每帧构造candidate Vec，超限时`select_nth_unstable`，再排序selected和overflow，并做registry lookup。应消费09B change set和spatial index，仅在probe/view generation变化时更新packed list，稳定相机/场景的CPU工作接近零。

### P1-15：ProbeBakeTiming只有标签，没有调度语义

`EditorManual`和`RuntimeManual`不能表达Baked/Custom/OnLoad/OnChange/EveryFrame、priority、interval、resolution、capture mask、time budget或staleness。新增模式必须驱动同一个capture scheduler，不能只增加enum而无consumer。

### P1-16：probe capture缺少可见性、排除与自反射contract

当前placement包含influence/projection，但没有独立capture cull mask、reflection mask、near/far、LOD、sky inclusion、transparent/emissive policy、self exclusion和exposure。Godot/Unreal/HDRP已有这些基础authoring；Zircon需要与09B visibility和09E lighting共享view policy。

### P1-17：environment切换没有时域crossfade与history identity

source revision变化后资源可以替换，但没有global sky/probe的radiance crossfade、exposure-aware transition、camera cut/history reject或stale age显示。动态昼夜和streaming场景会产生明显跳变；transition必须携带old/new generation和内存上限。

### P1-18：AmbientLight的lightmap语义被最终着色忽略

`affects_lightmapped_meshes`存在于scene asset/component，却未进入`SceneUniform`或environment shader；ambient始终与IBL相加。具体lightmap能量归09F2，但09F1必须提供可消费的ambient/environment source policy，防止静态间接光重复。

### P1-19：readiness与diagnostics没有共同产品入口

cache/import/probe/realtime各有局部report或test统计，却没有Editor可查询的环境资源状态、pending reason、artifact provenance、fallback generation、probe overflow和GPU bake timing。公共诊断必须来自runtime owner，Editor只展示，不复制判断逻辑。

### P1-20：测试数量掩盖了产品入口和动态证据缺失

现有测试大量直接构造小cubemap、手工注入EnvironmentExtract或匹配WGSL字符串；真实probe capture test是ignored manual。需要project asset roundtrip、Editor command、cook/reload、real WGPU image golden、规模、故障和soak测试，并记录设备/driver/recipe/artifact。

## 6. P2 差距清单

### P2-1：光谱/多行星大气与高阶天气系统

在RGB物理大气闭环后，再进入spectral rendering、多层介质、任意行星、月/星、天气前沿和艺术指导体积；不能以这些高级目标推迟P0的基本大气正确性。

### P2-2：ray/path-traced reflection与probe reference

建立hardware RT/path-traced ground truth、ray traced skylight/reflection、denoise和hybrid fallback，用于验证probe/IBL误差。其资源和PSO仍必须通过09A-09D公共owner。

### P2-3：分布式、增量和远程烘焙

在单机可取消cook job完成后，增加content-addressed分布式worker、scene chunk增量、跨机器artifact复用和签名验证；不能先把同步六面capture包装成远程命令。

### P2-4：超大世界environment streaming与多环境层级

支持world partition、分层environment volume、probe page streaming、远近层次和跨cell连续blend。基础generation、空间assignment和budget完成前不进入该阶段。

### P2-5：multi-GPU、foveated与云端渲染调度

为多adapter复制/分片LUT、probe与capture job，并对注视点/云渲染设置quality budget；所有artifact仍保持device-independent source与device-specific derived分层。

## 7. 目标架构

### 7.1 单一generation链

```text
Versioned Scene Assets / Environment Profiles / Volumes / Probes
                              |
                              v
                    EnvironmentChangeSet
                              |
          +-------------------+-------------------+
          |                   |                   |
          v                   v                   v
 Atmosphere/Sky Generation  IBL Bake Request  Probe Capture Requests
          |                   |                   |
          +--------- async cook/job scheduler ---+
                              |
                              v
              Content-addressed environment artifacts
                              |
                              v
       09D residency handles + device/resource generation tickets
                              |
          +-------------------+-------------------+
          |                                       |
          v                                       v
 Global Environment GPU Set             Spatial Probe Assignment
          |                                       |
          +-------------------+-------------------+
                              v
                    Unified PBR Environment ABI
                              |
                              v
               readiness / telemetry / debug views
```

### 7.2 owner职责

| Owner | 唯一职责 | 禁止承担 |
|---|---|---|
| scene environment authoring | profile/component/volume/probe序列化、property、migration | GPU对象、同步烘焙、临时snapshot patch |
| environment generation builder | change set、dependency、content key、old/new generation | 直接I/O、WGPU handle、Editor状态 |
| environment cook/capture scheduler | 可取消job、CPU/GPU预算、progress、artifact commit | per-frame shader selection、scene mutation绕过undo |
| 09D environment residency adapter | handle、mip/page、upload、VRAM、last-good、eviction | sky算法、probe选择策略 |
| atmosphere/sky renderer | LUT、sky display、cloud/fog耦合、capture radiance | 独立太阳真值、私有asset cache |
| reflection probe system | spatial index、admission、capture state、GPU local list | per-fragment全局扫描、同步ensure resident |
| environment shading module | base PBR IBL、energy、layer/list消费、fallback | resource loading、probe排序、重复sky公式 |
| diagnostics service | generation/readiness/cache/job/VRAM/perf统一快照 | 再推导另一套运行时真值 |

### 7.3 关键数据合同

- `EnvironmentGenerationId`至少包含scene/profile revision、sun/atmosphere/cloud dependency、shader/recipe version和device generation；不相关的exposure/UI变化不能误触发bake。
- `EnvironmentResourceHandle`只暴露ready/last-good/failure/provenance和09D lease；frame extract不再暴露texel Vec/Arc或filesystem path。
- `ProbeAssignmentGeneration`包含view、object/cluster layer、probe spatial revision、resident slot/page generation与overflow状态；shader只消费局部offset/count。
- `EnvironmentArtifactManifest`分离device-independent source、platform-derived PMREM/IEM/LUT、chunk/mip table、compression、content hash、recipe/schema/tool version和依赖。
- `EnvironmentJobTicket`具有queued/running/readback/commit/ready/cancelled/failed终态、CPU/GPU/time/byte预算和可恢复错误；任何pending都不得永久静默。

## 8. Hard Cutover 规则

1. scene schema上线后删除普通项目使用`preview_skybox`生成产品环境的路径；preview只能明确使用preview profile。
2. 删除测试/工具在World snapshot之后覆盖`environment`作为产品验收方式；fixture仅可用于底层算法unit test。
3. `EnvironmentExtract`硬切换为typed handle/generation；旧payload字段不保留双写兼容层。
4. submission/pre-draw路径禁止`fs::read`、HDR/artifact decode、`ensure_resident`和CPU PMREM/IEM构建。
5. 删除`CaptureCloud -> record_capture(gradient)`；真实Cloud未ready时显式Disabled，不保留误导pass名。
6. probe shader删除全局64线性扫描和未消费layer字段；新local-list ABI一次切换forward/deferred/generated PBR。
7. Reflection Probe capture统一进入job scheduler；旧同步六面API只可作为test oracle，不能由Editor产品调用。
8. artifact schema/recipe/platform profile一次硬版本升级，旧cache按miss处理或显式迁移，不做启发式兼容。
9. `AmbientLight.affects_lightmapped_meshes`、probe masks和capture modes没有consumer时不得宣称ready。
10. 旧environment WGSL owner在公共module接管后删除，禁止display/capture/fallback三份公式长期并存。

## 9. 分层实施里程碑

### M0：冻结失败证据、目标ABI与benchmark协议

补scene roundtrip失败测试、orphan trigger测试、warm-cache I/O计数、伪Cloud GPU工作、probe layer、64-probe scan、首帧bake、同步capture和VRAM基线；固定目标Unreal版本、硬件、场景、画质和采样统计。

#### 2026-08-16 current-source recheck

- `World::build_environment_extract`仍只把`SceneViewportExtractRequest.settings.preview_skybox`映射为`EnvironmentExtract`；它没有读取World组件、scene asset或profile，因此不能作为产品environment入口。
- `RealtimeIblTimeSliceScheduler`首次更新仍提交`CaptureSky(ALL)`和`CaptureCloud(ALL)`；`RealtimeIblWgpuRecorder`把两者落到相同的`record_capture`和相同source mip 0目标。这是重复的六面capture，不是cloud实现。
- 现有scene基础已经有serde节点记录、固定component clone/serde projection、`ZrReflect`注册、property path和transaction；M1必须复用这条持久化链路，不得以仅runtime的临时struct替代。

这些结论只完成M0的当前源码复核；未产生GPU timing、VRAM、WPR、RenderDoc或视觉验收，M0和全计划的implementation status仍为pending。

#### Cross-pipeline ViewFamily hard gate (2026-08-16)

09F1 does not own camera sizing, temporal history, display mapping, or the
post-process graph. It must consume the one resolved
`RenderViewFamilyPipeline` contract from Render01/Render06/Render07. That
contract already distinguishes the presenter-facing display rectangle from the
primary and secondary logical rectangles, their padded allocations, and the
enabled graph phases. An environment feature may not recreate any of those
extents from a scalar render scale, window size, or camera preview flag.

- Sky, global IBL, local-probe shading, depth/velocity and scene-linear
  environment contributions are written in the `SceneLinear` target. A temporal
  path reconstructs the primary target into secondary space; only effects
  explicitly assigned to the post-reconstruction slot may sample that result.
  Display mapping, display-space effects, the optional secondary spatial
  upscale, output transfer, and present must never re-evaluate environment
  radiance at a different extent.
- Reflection-probe and SkyLight capture are independent offscreen products.
  Their capture extent/profile is part of `EnvironmentGeneration`, not a copy
  of the camera's primary or secondary rectangle. Display exposure, temporal
  jitter, output transfer, editor overlays, and dynamic-resolution decisions
  are camera/product concerns and must not invalidate an environment bake.
- `RenderViewFamilyPipeline::phase_targets(...)` is the only geometry source
  for allocation, viewport/scissor, graph-resource routing, history and
  presentation. The current optional `FrameSubmissionContext` handoff is a
  migration state, not a supported fallback: Render01 must make it required in
  one constructor cut before 09F1 can claim a product environment path.
- Render07 must atomically replace the ambiguous legacy
  `TaaResolve -> DepthOfField -> MotionBlur` chain with separate
  pre-reconstruction and post-reconstruction scene-linear slots, resources and
  dependencies. This follows Unreal's primary/secondary separation: DOF can
  consume primary scene color/depth before temporal reconstruction, while
  motion blur consumes the reconstructed result. Renaming one enum or adding
  an environment-specific pass is forbidden because it leaves target geometry
  and dependencies inconsistent.
- Render17 owns per-viewport GPU timing and the bounded dynamic-resolution
  state. It publishes a prior-frame, scope-matched immutable decision before
  Render01 resolves the next ViewFamily. Environment scheduling may observe the
  decision for telemetry but cannot drive it from bake, import, or probe work.

This gate is deliberately ahead of M1 product extraction: a persisted
EnvironmentProfile must survive any device resolution and post-process mode,
while its scene-linear radiance is consumed once by the resolved view family.
Acceptance requires real 1920x1080 and odd-size 1919x1079 temporal/spatial
captures showing identical display ownership, correct history reset behavior,
and no environment resize/bake caused solely by dynamic resolution. This is an
architecture requirement, not current visual evidence.

#### Current-source cutover prerequisites (2026-08-16)

The current submission builder resolves `RenderViewFamilyPipeline` and passes
it to post-process validation, but it returns `FrameSubmissionContext::new(...)`
without calling `with_view_family_pipeline(...)`; the context still stores an
`Option` and its accessor can only `expect`. This is an incomplete migration,
not a supported compatibility state. Render01 must make the pipeline a
required constructor argument, delete the optional field and fluent setter,
and update all direct test constructors in the same compiler-led cut.

The same builder selects `RenderUpscalerKind::Temporal` for effective TAA, yet
always constructs its policy with `with_spatial_primary_fraction`. It therefore
forces `secondary_fraction = 1.0` even for a temporal path and cannot express
Unreal's independent primary-to-secondary reconstruction followed by optional
secondary-to-display spatial upscale. Render06/Render07 must carry an explicit
secondary fraction in the resolved camera/quality policy, choose
`with_temporal_fractions` for temporal reconstruction, and hard-split
`PrimaryUpscale` from `SecondaryUpscale`. A temporal primary-to-display path
may remain a valid quality profile, but it must be represented as an explicit
`secondary_fraction = 1.0` decision rather than as the only ABI.

The environment side has the analogous temporary boundary:
`SceneWorld::build_environment_extract` still maps
`preview_skybox` directly to `EnvironmentExtract::procedural_default()`, while
`EnvironmentExtract` itself can carry full `SourceCubemapEnvironment` texels
and prepared upload bytes. M1a must introduce the persisted profile/change-set
before M4 replaces that payload with a typed ready handle. Neither rendering
resolution nor scene environment identity may be inferred from a preview flag
after the respective cutover.

#### Target pass and target-space contract

The new phase model must preserve Unreal's target-space behavior instead of
only copying its pass names. For a temporal path, the ordered contract is:

`SceneLinear(primary) -> PreReconstructionSceneLinear(primary) -> TemporalReconstruction(primary to secondary) -> PostReconstructionSceneLinear(secondary) -> DisplayMapping(secondary) -> DisplayPostProcess(secondary) -> SecondaryUpscale(secondary to display, when selected) -> OutputTransform -> Present`.

Depth of field and any explicitly pre-reconstruction translucency belong in
the primary scene-linear slot. Motion blur, its dependent velocity products,
bloom/exposure input preparation, and post-reconstruction material hooks
consume the reconstructed secondary result. This follows the actual Unreal
ordering of `DiaphragmDOF::AddPasses`, temporal upscaler execution, then
`AddMotionBlurPass`.

For a spatial-only path, scene-linear and display work remain at primary
resolution until the late display-space conversion:

`SceneLinear(primary) -> SceneLinearPostProcess(primary) -> DisplayMapping(primary) -> DisplayPostProcess(primary) -> PrimaryUpscale(primary to secondary, when selected) -> SecondaryUpscale(secondary to display, when selected) -> OutputTransform -> Present`.

This mirrors Unreal's late `PrimaryToSecondary` and `SecondaryToOutput`
upscale stages. A direct primary-to-display profile is represented by omitting
the secondary stage, not by making a single `SpatialUpscale` enum serve both
transitions. Render06/Render07 must replace the current single
`SceneLinearPostProcess` and `SpatialUpscale` phase meanings with these
resource-specific phases, and every graph cache key must contain the enabled
phase mask plus the primary, secondary, display logical viewports and padded
allocation extents. Render features declare one legal phase and color space;
they cannot infer a target from the window or select a hidden fallback target.

Render17 owns the timestamp scope for this graph. A completed sample records
the source frame generation, ViewFamily id, viewport generation, upscaler
kind, all three logical extents, and separate elapsed values for scene linear,
pre-reconstruction, temporal reconstruction, post-reconstruction, display
mapping, display post-process, primary upscale, secondary upscale, and output
work. The dynamic-resolution controller consumes only the presented
ViewFamily total from a matching prior-frame sample. Sky/probe capture,
PMREM, IBL readback and artifact I/O publish separate budget/timing telemetry;
they must not lower the camera's primary fraction. If such work shares a queue
with presentation, its scheduler must enforce a frame budget and report its
debt rather than hiding it inside the resolution feedback loop. An unavailable
or scope-mismatched sample holds the prior decision. This is also the required
current-machine basis for future CPU/GPU/power comparisons; no historical
viewer startup duration can substitute for it.

### M1：建立versioned scene/environment authoring

加入EnvironmentProfile、Sky/SkyAtmosphere/Cloud/SkyLight/ReflectionProbe/EnvironmentVolume schema、component、property/reflection、prefab/save/load/script migration；World只从这些资产生成environment change set。

#### M1 target ABI and hard-cut boundary

M1不是向`ProceduralSkyParams`追加字段，也不是为现有`preview_skybox`增加另一条分支。它一次性建立以下单向数据流：

`SceneAsset -> NodeRecord/typed component -> reflection/property/undo -> World render extract -> EnvironmentGeneration -> job/residency admission -> render graph -> shared display and capture shading`。

- `EnvironmentProfile`是versioned root contract，只保存质量profile、更新策略、资源引用和下列component之间的明确关系；它不持有Arc texel、prepared upload bytes、filesystem path或WGPU object。
- `SkyAtmosphere`保存planet/ground albedo、RGB Rayleigh、Mie、absorption/ozone、multi-scattering和quality inputs；`VolumetricCloud`保存enabled/density/radiance/ambient/shadow authoring。cloud在真实density/radiance path上线前必须显式Disabled，不能保留`CaptureCloud` pseudo-pass。
- `SkyLight`保存source mode、realtime/update policy、resolution/quality、occlusion和lower-hemisphere policy；`ReflectionProbe`保存influence、projection、capture/reflection masks、priority、blend和update mode。两者只引用artifact/resource identity，不直接承载derived texture payload。
- `DirectionalLight`是太阳光度和方向的唯一上游身份；`EnvironmentGenerationId`由scene/profile revision、sun/atmosphere/cloud dependency、shader/recipe version和device generation组成。曝光、UI、camera preview等不在该identity内，不能触发radiance bake。
- `EnvironmentGeneration`只发布immutable change set、ready/last-good/failure/provenance和job ticket。frame submission只消费ready generation，绝不读盘、decode、cook或`ensure_resident`。

落地M1时必须同时覆盖component registration、World clone/serde projection、scene asset roundtrip、property path、script reflection和render extract；只增加Rust struct、只写Editor控件或只在测试后覆盖snapshot均不构成M1完成。旧`preview_skybox -> product environment`映射在M3删除，preview只可解析独立preview profile。

#### M1a first atomic source scope

M1a is the first shippable part of M1: a persisted profile may select only the
existing `Disabled`, procedural, or source-cubemap producer, while atmosphere
and cloud remain explicitly Disabled. The source owner must claim the complete
set below as one migration, then extend it only where the current compiler
identifies a direct ABI consumer:

- Scene asset and artifact-cache DTO: `asset/assets/scene/entity.rs`, a new
  scene environment asset module and its exports,
  `asset/assets/scene/management.rs`, and
  `asset/artifact/cache_payload/scene.rs`. The new optional field must retain
  old scenes unchanged through serde defaults and must remain visible to
  management/direct-reference accounting.
- Typed scene state: a new `scene/components/scene/environment.rs`, both
  component export modules, `scene/components/scene/node.rs`,
  `scene/world/typed_api/fixed_components.rs`, `scene/world/typed_api.rs`,
  `scene/world/world.rs`, and `scene/world/derived_state.rs`.
- Public authoring: `scene/reflect/builtin_reflection/registration.rs`,
  `scene/world/component_access.rs`, property entry/read/write and compiled
  binding modules. Every persisted profile field must have one canonical
  property path and one script-visible type identity.
- Project conversion and frame boundary:
  `scene/world/project_io/scene_asset.rs`, `scene/world/render.rs`, and the
  framework environment extract contract. `World` chooses the active profile
  deterministically by active hierarchy, render layers and priority; viewport
  preview can influence only `PreviewEnvironmentExtract`.

M1a acceptance is a real scene save/reopen/extract roundtrip containing an
active profile, a disabled profile and a legacy scene with no profile. It must
prove profile mutation changes `EnvironmentGenerationId` while camera exposure
or preview flags do not. It does not claim physical sky/cloud pixels, GPU
resource residency, realtime bake throughput or editor capture; those remain
M2--M7 work and must not be simulated by `CaptureCloud`.

#### M1a compiler-led migration map

The current scene implementation already provides the correct persistence
shape: `SceneEntityAsset` stores optional asset DTOs, project I/O converts
them into `NodeRecord` plus generic ECS components, and a
`Persistent*ComponentSnapshot` rebuilds the canonical store through
clone/serde. M1a must add one `SceneEnvironmentProfileAsset` in a dedicated
scene environment asset module and one optional `environment_profile` field on
`SceneEntityAsset`, both `serde(default)` so legacy scenes remain unchanged.
Its runtime peer is `EnvironmentProfileComponent`, registered beside the
other scene components and carried by `NodeRecord` only as the insertion
transport. It is then included in a persistent environment snapshot; it must
not use the existing `RuntimeOnlyPostProcessComponentSnapshot` pattern.

The M1a profile has a deliberately small, versioned authoring surface:
`enabled`, `priority`, an authoring/render layer mask, update policy, quality
profile identity, and `Disabled | Procedural | SourceCubemap(AssetReference)`
source selection. Resource references are converted through the standard
project resolver and included in `SceneAsset::direct_references` and
management records. HDR bytes, `.zcube`/`.zribl` locations, decoded texels,
prepared upload artifacts, and WGPU handles are invalid fields for both the
asset DTO and component.

`World::build_environment_extract` becomes a deterministic selector rather
than a preview-flag adapter. It considers only hierarchy-active profiles whose
layer mask intersects the selected camera's volume mask, ranks by descending
priority and then stable entity id, and emits an `EnvironmentGenerationRequest`
containing the chosen profile identity and revision. It must not resolve a
source cubemap, read an artifact, or construct `SourceCubemapEnvironment`.
An absent or explicitly disabled profile produces the typed disabled request;
preview becomes a separately labelled `PreviewEnvironmentExtract` override.
The next resource/residency milestone may turn a ready request into an
`EnvironmentResourceHandle`, but cannot alter selection rules or recompute a
private generation id.

This one compiler-led cut requires roundtrip tests for a legacy no-field scene,
two equal-priority profiles with deterministic entity-id selection, inactive
and layer-mismatched rejection, direct-reference accounting, and the invariant
that exposure/preview edits leave `EnvironmentGenerationId` unchanged. It is
the minimum product loop for authoring identity, not a visual substitute for
M4 residency or M5 sky/cloud shading.

### M2：接通Editor、undo、preview与cook dependency

Inspector/gizmo/profile editor/capture command进入统一operation和undo transaction；捕获结果更新scene资产并写catalog/cook manifest。preview profile与product environment明确隔离。

### M3：建立唯一Environment Generation与太阳真值

统一DirectionalLight/Sun、atmosphere、cloud、fog、exposure dependency和revision；实现增量invalidations、old/new generation、last-good和状态快照，删除布尔preview推导产品环境。

### M4：硬切换handle/residency与异步I/O

将source/PMREM/IEM/LUT/probe texture迁入09D typed handle、lease、streaming和budget；cache/artifact读取在worker完成，render thread只做ready admission与batched upload。

### M5：完成物理大气与真实Cloud基线

实现RGB Rayleigh/Mie/ozone、transmittance/multi-scattering/sky-view/aerial LUT、地表反照率和太阳耦合；Cloud要么有真实density/radiance/ambient/shadow capture，要么保持Disabled。display与IBL capture共享shader source。

### M6：重构IBL cook/artifact/cache

manifest-first warm hit、chunked source/derived、平台format、GPU生产路径、CPU oracle、atomic commit和错误恢复；正常warm import不得decode HDR或完整`.zcube/.zribl`。记录命中原因和各阶段性能。

#### M6 manifest-first atomic contract

M6 must not add a third independently-written cache file. M8 already publishes
the source `.zcube` and importer-derived `.zribl` as one recoverable durable
bundle; a manifest committed outside that generation can point at one old and
one new target after interruption. The replacement contract is a versioned
`EnvironmentArtifactManifest` owned by 09D's asset/residency boundary and
committed in the same generation transaction as every referenced source and
derived target (or as one atomically replaced generation pointer to immutable
targets).

- The manifest stores the exact source content identity supplied by asset
  import, source/recipe/schema/tool/platform-profile identities, producer,
  byte lengths, bounded header checks, content hashes, and a chunk table for
  source mips, PMREM mips, SH9/IEM and optional platform-transcoded payloads.
  File time, URI, equirectangular dimensions, and an HDR header are discovery
  hints only; none may stand in for the content identity.
- A normal warm decision reads only the bounded manifest plus the fixed
  `.zcube`/`.zribl` headers needed to reject a stale or corrupt generation. It
  returns typed residency requests; it does not allocate RGBA32F pixels,
  decode complete containers, or hydrate all chunks. Full source decoding,
  filtering, checksum audits and repair run only on a declared miss, audit, or
  worker-owned residency request.
- `AssetImportContext` must split exact `SourceIdentity` from the lazy decode
  context. Project metadata/cook records carry the former across reopen so a
  warm import does not retain or rehash a large HDR byte buffer just to decide
  reuse. A newly observed external source may compute that identity once in
  the importer worker before it can participate in a cache decision.
- 09D exposes one `EnvironmentArtifactReader`/residency facade to importer,
  renderer and cook. It owns manifest/header reads, chunk admission,
  corruption quarantine and current-generation selection. Importer and frame
  submission must not parse container headers themselves, and render/submit
  threads may only observe a ready handle or last-good generation.

The first M6 migration must preserve the M8 pair-recovery tests and add
generation-interruption coverage for the manifest target, a bounded-read
counter test, a source-identity mismatch miss, and a worker-only full-hydrate
test. It is a hard schema/version cut: old cache entries become explicit
rebuildable misses or undergo a declared migration, never heuristic fallback.

### M7：重构realtime IBL scheduler

Static/OnChange/TimeSliced/EveryFrame策略、bootstrap quality、GPU time budget、changed dependency、crossfade和history identity；compiled command直接进入graph，移除per-pass plan/params/bind分配。

#### M7 current-source hard cutover contract (2026-08-16)

`RealtimeIblTimeSliceScheduler` currently makes `published_key == None` a
special full-update branch. At the default `128x8` recipe that branch records
both six-face captures, all source mips, all PMREM mips and SH9 before it
publishes. `CaptureCloud` is not a second radiance producer: the current WGPU
recorder sends it through the same gradient-capture routine and into the same
source mip-zero faces as `CaptureSky`. The second pass therefore overwrites
the first and consumes GPU work without adding cloud radiance. M7 must replace
this branch, not tune its batch size.

The replacement is one recipe-derived `EnvironmentGenerationTicket` state
machine used by bootstrap and subsequent updates. It owns a generation id,
declared producer set, dependency cursor, work-slot identity, immutable recipe
key, last-good generation and cancellation result. Its only normal
publication transition is after the final dependent operation succeeds. Until
then the renderer samples a declared fallback or the prior last-good
generation; it never exposes a partially filled cubemap. A revision change
cancels the old ticket without replacing that ready generation.

- The default producer set contains sky radiance only. There is no
  `CaptureCloud` operation, allocation or timing category until the M5 cloud
  producer supplies physically meaningful radiance through the same
  generation contract. A disabled cloud producer must schedule zero work.
- Operations are explicit dependency nodes: producer capture faces, source
  downsample mips, PMREM mip/face ranges and diffuse SH9. A budget policy
  selects only dependency-ready nodes whose predicted workgroup and measured
  GPU-time cost fit the ticket's per-frame allowance. It must retain a resume
  cursor rather than treating first use as permission to bypass the budget.
- Quality is an immutable recipe input, including source size, PMREM mip
  count, face quota, format and producer set. Static environments choose
  cook/cache; OnChange and TimeSliced tickets are revision-gated; EveryFrame
  is permitted only for declared dynamic producers with an explicit update
  interval and importance policy.
- `RealtimeIblGraphKey` must include that full recipe layout, scheduler
  topology and resource generation. Replace the fixed 34-entry topology
  assumption and linear lookup with a bounded key-indexed cache whose eviction
  cannot discard an in-flight graph. The current `Vec::position` plus hard
  assertion is unsafe because the scheduler accepts arbitrary mip counts and
  one through six capture faces per frame, while 34 only describes the former
  eight-mip, two-face sequence. Device loss, texture reallocation, recipe
  change or bind-layout change invalidates graph templates and resource-bound
  command data together.
- Each graph variant owns immutable dispatch templates. Recording may select a
  template and its slot-local bindings, but must not rebuild the complete IBL
  command plan or create a params buffer and bind group for every dispatch.
  In particular, the current PMREM-slice and SH9 paths reconstruct the whole
  request plan and search it per recorded pass. These bindings are recreated
  only for their declared device/resource generation invalidation.
- Timestamp telemetry is per producer/downsample/PMREM/SH9 operation and
  includes scheduled versus completed work, ticket age, fallback age and
  publish/cancel reason. The current two-query batch bracket is insufficient:
  it cannot attribute a capture, source-mip, PMREM or SH9 cost. Presentation
  dynamic resolution consumes only the matching view-family presentation total;
  bake/capture queue debt stays in environment-job telemetry and cannot change
  render resolution.
- The measured viewer evidence must keep these two timing domains separate.
  The existing Ready-frame GPU sidecar may retain its aggregate
  `direct_realtime_ibl` presentation-pass value, but a versioned
  `EnvironmentGenerationTrace` records every IBL operation with its ticket,
  recipe fingerprint, device/resource generation, source and work slots,
  scheduled/completed workgroups, GPU duration, and terminal reason. The
  profile manifest binds that trace to the same binary/source fingerprints,
  screenshot generation, WPR interval, and Energy Meter interval. A profile
  verifier must reject an aggregate-only IBL result when an IBL ticket ran;
  adding arbitrary `pass.*` names to the display-frame schema is forbidden.
  Today `render_scene` times the complete batch as `direct_realtime_ibl` but
  calls `record_prepared_frame(..., false, ...)`, so the internal two-query
  recorder is never enabled by ordinary viewer GPU timing. Merely exposing
  `RealtimeIblGpuTimingReport` through the framework is not evidence until the
  trace request is propagated, read back, drained and written by the profiling
  boundary.

Required regression coverage: bootstrap emits the same bounded first slice as
an OnChange request and does not publish it; no-cloud schedules no cloud pass;
an unchanged scene submits no graph/bind allocations; `capture_faces_per_frame
== 1` and non-default mip counts cannot exhaust cache topology capacity;
device loss rebuilds templates and bindings; a complete ticket equals the
reference recipe output; and every executed operation has a timing category.
The implementation must first wire this ticket through the common environment
generation owner. It must not add another per-frame renderer-local scheduler.

### M8：重构Reflection Probe capture job

batched six-view graph、visibility/capture mask、GPU prefilter、异步readback/commit、cancel/progress/retry、scene revision和自动失效；Editor/runtime/cook共享job API和artifact。

### M9：GPU-driven spatial probe assignment

由09B scene index生成object/cluster/tile probe list，支持layer/reflection mask、priority、blend、box/sphere、resident page和overflow。稳定场景不重建candidate，fragment成本不再与全局probe数线性相关。

M9 hard-cuts the fixed `array<ZrGpuReflectionProbe>` scan as the assignment
interface. The graphics ABI becomes: persistent probe records and resident-page
metadata, a per-view `ReflectionProbeSpatialAssignment` header, cluster
offset/count entries, packed candidate indices, and an overflow record. The
quality profile defines tile/cluster dimensions and the local-list capacity;
the shader never silently falls back to scanning all resident probes when a
cluster overflows. Forward, deferred and generated-material variants share one
WGSL lookup/include, receive the object's reflection mask through the existing
scene/material ABI, and rank only the returned candidates. A list miss has an
explicit global-environment/last-good fallback and a telemetry reason.

The implementation order is: (1) version the framework DTO and shader ABI;
(2) build deterministic CPU/reference lists from the 09B persistent scene;
(3) upload and validate list ranges/overflow without changing sampling;
(4) enable the common shader lookup in every PBR path; (5) delete the global
fragment loop and fixed-64 admission policy; (6) move capacity/resolution and
residency into 09D quality/budget ownership. Required product coverage includes
different object masks in one camera view, boundary crossfades, dense overflow,
1/64/1k probe movement, stable-frame zero rebuilds, and identical source inputs
through forward/deferred paths.

### M10：统一environment shading与降级质量

公共WGSL module覆盖forward/deferred/generated PBR，固定BRDF LUT/PMREM/SH9/IEM reference、energy与specular occlusion；无PMREM fallback显式标记并有视觉门禁，lightmap/advanced lobe通过09F2/09G接口扩展。

### M11：资源预算、device loss与诊断闭环

零probe惰性资源、quality tier、VRAM/CPU cache budget、eviction、device recreate、job终态、probe overflow和Editor debug view进入统一telemetry；错误可见且有界，不panic、不永久pending。

### M12：产品、规模、故障与soak验收

真实project import/save/reopen/cook/package、昼夜变化、1/64/1k probes、layer、capture invalidation、camera cut、streaming、cache corruption、device loss、VRAM pressure和24h soak全部产生可复现artifact。

### M13：同画质竞争性验收

在相同HDR sky/大气参数、probe布局、分辨率、材质、曝光和输出误差约束下，对照选定Unreal版本统计p50/p95/p99 CPU/GPU、RAM/VRAM、首帧、更新hitch和相机运动稳定性；只根据数据判断是否达到“优于”。

## 10. 验收矩阵

| 场景 | 必须证明 | 失败信号 |
|---|---|---|
| scene/profile roundtrip | sky/atmosphere/cloud/probe字段从磁盘到像素保持版本语义 | snapshot手工覆盖、字段丢失、preview代替product |
| Editor capture | command/undo/progress/cancel/dirty/catalog/cook完整 | orphan Rust trigger、只注册内存texture |
| physical sky | LUT/reference误差、太阳/地表/高度响应正确 | 三色gradient冒充大气、display与IBL不同 |
| cloud disabled/enabled | disabled为0工作；enabled捕获真实cloud radiance | CaptureCloud重复写gradient |
| warm HDR import | manifest命中不decode HDR、不完整读artifact | warm路径仍RGBA32F decode/全blob decode |
| runtime cache | I/O后台化、last-good连续、终态明确 | submission线程读盘、永久pending、帧hitch |
| unchanged frame | environment/probe CPU allocation和upload接近0 | 每帧Vec/sort/full buffer write |
| first realtime update | 在预算内渐进ready，无单帧峰值 | 一帧双六面capture+全mip/PMREM/SH9 |
| probe layer/mask | 同相机不同object layer得到正确probe | 只按camera layer过滤、misc.w无消费 |
| 1/64/1k probes | spatial list正确、overflow有界、fragment成本局部化 | 每像素扫64、silent drop、排序抖动 |
| probe capture invalidation | scene/source/settings变化精确触发，静态帧不触发 | stale artifact或无条件重捕获 |
| quality/platform | resolution/format/compression可配置且shader ABI正确 | 唯一128/8/RGBA16F、跨平台失效 |
| device loss/cache corruption | generation重建、坏artifact隔离、可恢复 | panic、stale WGPU handle、错误cache命中 |
| VRAM pressure/soak | memory/job/cache/page有上限且平稳降级 | 零probe仍75 MiB、资源/queue单调增长 |
| Unreal parity | 同画质同输入原始capture可复现 | 降功能/降精度换帧时、无误差约束 |

### 10.1 性能门禁

- warm stable frame：render/framework state锁域内filesystem/decode/cook次数为0；无变化environment/probe generation的CPU heap allocation、GPU upload bytes和graph bake pass均为0或有记录的维护理由。
- import/cache：记录source bytes、header/full read bytes、decode/filter CPU time、worker occupancy和hit reason；warm hit只读有界manifest/header，不随HDR像素数线性增长。
- realtime sky：记录每帧capture faces、downsample/PMREM mips、dispatch、GPU time、ready/stale age和crossfade memory；任何单帧不得无预算执行完整高质量更新。
- probe：记录global/resident/visible/assigned/overflow probe、list长度分布、fragment visits、upload bytes和VRAM；目标复杂度随可见局部列表增长，不随`pixels x global 64`增长。
- capture：记录queue wait、six-view cull/draw/GPU/readback/filter/commit、peak CPU/GPU bytes和cancel latency；Editor主线程不得等待完整capture完成。
- memory：source float、prepared bytes、staging、GPU global/probe/planar、old/new transition和cache分别会计；零feature资源按需释放，压力降级有确定priority。
- 竞争目标：相同输入、输出误差、分辨率、probe密度和更新频率下，Zircon的p50/p95/p99 render thread、GPU environment/reflection、首帧/update hitch、RAM/VRAM峰值和运动稳定性均不得显著劣于目标Unreal版本；“优于”必须有预注册场景和统计显著性。

## 11. 参考实现映射

| Zircon目标 | Unreal主参考 | 次级/下限参考 | 不能直接照搬的部分 |
|---|---|---|---|
| scene/environment authoring | SkyLight/ReflectionCapture/SkyAtmosphere/Cloud components | Godot Environment/Probe、Fyrox SkyBox | UE UObject/World不能越过Zircon scene schema/operation |
| physical atmosphere | SkyAtmosphereRendering | Bevy atmosphere LUT、HDRP Physically Based Sky | 参数/坐标/曝光需Zircon统一单位与reference |
| sky/ambient update | SkyLight realtime capture/irradiance | HDRP SkyManager、Godot sky owner | 单队列WGPU不应伪称UE async overlap |
| IBL artifact/cook | reflection environment capture/build data | Bevy filter compute、HDRP ambient convolution | UE DerivedData/texture format需映射09D artifact体系 |
| local reflection probe | sorted reflection captures | HDRP HDProbeSystem、Godot ReflectionProbe | 固定64数组不是最终空间assignment |
| probe capture/editor | capture queue + MapBuildData | HDRP baked reflection editor | Editor transaction必须使用Zircon operation/undo |
| resource residency | reflection cubemap arrays/streaming | HDRP texture cache、Bevy RenderAsset | 所有GPU lifetime归09A/09D，不在09F1私建 |
| diagnostics/benchmark | RDG stats/cvars/debug | HDRP debug、Godot debug | debug开关不能替代产品readiness truth |

## 12. 证据缺口与风险

### 12.1 已有证据

- IBL recipe、artifact encode/decode/corruption、source identity、seam、CPU/GPU parity、runtime cache/writeback有较多unit/integration tests。
- cubemap prepared upload、selective update、probe slot/upload/blend、realtime slice/double buffer/compiled graph cache有局部行为测试。
- WGPU产品测试证明手工EnvironmentExtract与ReflectionProbeData可以生成真实画面差异。
- import timing、bake workload、probe capacity和部分GPU timestamp提供重构挂点。

### 12.2 仍缺失的决定性证据

- 没有普通scene asset创作Sky/Environment/ReflectionProbe并save/reopen/cook到像素的测试。
- 没有Editor实际调用capture trigger、undo/redo、进度/取消、scene dirty和package reload证据。
- 没有物理大气、真实云、太阳/方向光/雾耦合或昼夜动态golden。
- 没有证明warm normal project import跳过HDR decode和完整artifact读取。
- 没有证明render/submission线程零filesystem/decode/ensure-resident。
- 没有object layer probe测试；现有GPU mask字段只验证编码，不验证消费。
- 没有1k probe spatial assignment、overflow、相机运动稳定或fragment visit profile。
- 没有capture job并行/取消/失败/device loss/VRAM压力/24h soak。
- 没有同画质Unreal/Fyrox/Bevy/Godot/HDRP图像误差与性能原始artifact。

### 12.3 实施风险

- environment schema与光度/曝光迁移会改变旧preview外观；必须version转换并保留before/after capture，不能以“更物理”掩盖破坏。
- 09F1强依赖09A-09E；若先在旧snapshot/raw WGPU上快速补组件，会形成第三套generation、cache和resource owner。
- 物理大气与Cloud容易扩张范围；必须先实现可验证的RGB baseline和Disabled真值，再分层增加质量。
- artifact/format硬切换会使旧cache失效；project源资产必须可重建，错误必须显式，不应保留无版本兼容猜测。
- probe spatial list和shader ABI切换会影响forward/deferred/material permutation；必须由09C统一编译并一次cutover。
- old/new environment crossfade会短期增加内存，必须由预算owner决定是否降级，不能无界双份驻留。

## 13. 完成定义

09F1实现只能在以下条件全部满足后标记complete：

1. 普通versioned scene/project可author、保存、重开、prefab、script和cook Environment/Sky/Atmosphere/Cloud/ReflectionProbe，并从World真实extract到像素。
2. Editor capture由实际command/Inspector触发，支持undo、progress、cancel、scene dirty、catalog/dependency和package reload；不存在orphan trigger。
3. gradient仅作为明确preview/fallback；物理大气和Cloud具有真实算法/Disabled语义，太阳与DirectionalLight共享真值。
4. frame extract只携带typed handle/generation；submission/pre-draw路径没有filesystem、decode、CPU bake或同步ensure-resident。
5. normal warm import不decode HDR/完整artifact；artifact具备manifest/chunk/platform profile、版本和可恢复corruption处理。
6. realtime IBL具有静态/动态策略、分帧GPU预算、last-good/crossfade；不存在伪Cloud pass或首次无预算全更新。
7. Reflection Probe capture进入可取消异步job并共享Editor/runtime/cook contract；六view/filter/readback/commit有预算与终态。
8. probe使用object/cluster/tile spatial assignment，layer/mask有真实consumer；fragment不再线性扫描全局64 probe。
9. probe/global environment资源按需创建、可配置质量/格式并进入CPU/VRAM预算；零feature不固定预留约75 MiB。
10. forward/deferred/generated PBR共享versioned environment module，BRDF/PMREM/SH9/IEM/fallback通过CPU reference、GPU parity和视觉golden。
11. cache/job/resource/probe/readiness与diagnostics只有一个runtime truth，device loss、corruption、overflow、pressure和24h soak均有产品证据。
12. 与选定Unreal版本的同硬件同画质基准完成；报告只根据可复现数据陈述是否达到“优于”，未达到时继续保留差距。

在此之前，`docs/plans/zircon_runtime/render/11-environment-lighting.md`和shader/IBL output records中的局部milestone完成，只能表示算法片段或手工fixture已存在，不能表示工程级Environment/IBL/Reflection产品完成。
