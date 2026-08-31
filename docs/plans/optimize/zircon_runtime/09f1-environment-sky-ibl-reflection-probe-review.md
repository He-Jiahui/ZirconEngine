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
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Components/ReflectionCaptureComponent.cpp
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

程序天空本质仍是三色gradient加sun disc。没有Rayleigh/Mie/ozone、transmittance/multi-scattering/sky-view/aerial-perspective LUT，没有太阳与DirectionalLight的共同authoring，没有雾、云、天气、时间或environment volume。历史上realtime IBL曾把`CaptureSky`和`CaptureCloud`映射到相同gradient capture并重复覆盖；current source 已硬删除该伪operation，production中的命中为0，graph contract也禁止重新引入。这个关闭只消除了重复GPU工作，不构成Cloud或物理天空能力。

资源和线程边界也未工程化。`EnvironmentExtract`把包含Arc texel链、SH9、可选IEM和prepared upload bytes的`SourceCubemapEnvironment`直接放入逐帧snapshot，而不是传稳定resource handle。runtime cache miss会在frame submission构建路径同步`fs::read`和decode `.zribl`；普通项目导入在判断staged bundle是否可复用之前先把HDR完整解码成RGBA32F，之后还读取并解码`.zcube`和`.zribl`来判定“Current”。Reflection Probe prepare会在pre-draw路径同步`load_texture_asset`，其`ensure_resident`可继续做artifact I/O/decode；每个PMREM实际按mip执行8次、每次覆盖6个array layer的`queue.write_texture`，但64个冷probe连同header/params上传仍会超过默认64 MiB staging预算。

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

### P0-3：程序天空仍不是物理天空；重复 `CaptureCloud` 子项已关闭

三色gradient、sun disc和独立rotation不能提供大气透射、多重散射、aerial perspective、地表反照率、行星尺度或稳定曝光。current source 已删除`CaptureCloud` enum/graph/recorder分支，并以source contract要求production零命中；禁用cloud现在调度零工作。剩余P0是物理SkyAtmosphere/Cloud组件、LUT、共同sun truth与明确静态/动态更新图，不得因删除伪pass而关闭。

### P0-4：太阳、天空、雾、云、曝光和DirectionalLight没有共同真值

`ProceduralSkyParams.sun_direction`与scene DirectionalLight互不关联，environment revision也不知道direct light、fog/cloud或exposure的变化。需要唯一`EnvironmentGeneration`与显式dependency graph：太阳/大气变化只invalidate受影响LUT/IBL，云只更新其capture/ambient/shadow数据，exposure不应无条件重烘radiance。

### P0-5：environment资源以逐帧payload传递，且submission路径同步读盘

`SourceCubemapEnvironment`携带Arc source/PMREM texel、IEM、SH9和prepared upload artifact进入`EnvironmentExtract`。runtime hydration miss在`build_frame_submission_context`调用cache store同步`fs::read`/decode。必须硬切换为09D拥有的typed handle、residency ticket和last-good GPU generation；render/submit线程只查询ready状态，I/O/decode/cook不得进入其锁域。

### P0-6：Reflection Probe prepare可同步ensure-resident且缺少上传准入/回滚

probe candidate选中后，pre-draw `prepare`调用`load_texture_asset`；资产未驻留时可能同步读取/解码artifact。每个新PMREM实际生成8个mip upload，每个upload一次覆盖6个array layer，并非48个独立write；结构性问题是prepare在确认共享RHI staging headroom之前同步驻留、克隆payload并排入整组upload。64个冷probe的texture bytes为`67,107,840 B`，加probe/header/planar参数buffer后为`67,114,176 B`，必然超过默认`67,108,864 B` staging预算。拒绝后现有discard只清pending列表，下一frame会重新load/clone/schedule。需要09D拥有的异步residency ticket、准入前byte/count预算、transactional slot generation和deferred readiness；未就绪probe使用last-good/sky fallback而不是阻塞或重复失败整帧。

### P0-7：每像素线性扫描64个probe，layer contract只有camera级门禁

CPU先按camera layer和距离选择最多64个probe，WGSL又对每个fragment线性扫描全部active probe选top two。current source已经读取`misc.w`，但只与同一camera layer mask相交，实质上重复CPU camera级门禁；Forward对象和Deferred GBuffer都没有提供object reflection mask。必须由persistent scene/visibility生成object/cluster/tile probe list，shader只遍历局部短列表，并把object/camera/capture/reflection mask统一成可测试ABI。

2026-08-16 source quantification: `MAX_REFLECTION_PROBES` is 64 and the full
array reserves 64 `128x128`, eight-mip, six-face RGBA16F cubemaps. Its exact
payload capacity is 67,107,840 bytes (about 64 MiB); the fixed `1024x1024`,
eleven-mip RGBA16F planar chain adds 11,184,808 bytes (about 10.7 MiB). The
full-path shader's `zr_environment_select_probes` executes one loop iteration
per admitted probe for every selector invocation: 1920x1080 with 64 probes is
132,710,400 iterations per frame (7,962,624,000 at 60 fps), and 3840x2160 is
530,841,600 iterations per frame. Standard PBR invokes the selector once, while
the active clearcoat path invokes it again after a planar miss, so its static
worst case is twice those values. Those values exclude cubemap sampling,
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

### P0-8：Reflection Probe捕获产品/性能验收仍开放（早期同步描述已失效）

2026-08-31 current-source 复核确认早期“六次同步 `render_scene_color_hdr`/readback + CPU bake”描述
已失效。当前显式 capture path 移动一个 `SceneViewportRenderPacket` 进入可复用 batch，在同一
graphics encoder 记录六个 RGBA16F raster face、source mip、8 个 PMREM dispatch 与 1 个 SH9
dispatch；direct-light payload 只打包一次。request/poll/cancel、revision、progress、terminal
failure/last-good 与可选 persistence 都走同一非阻塞 job contract，旧同步入口有 source guard。
因此本项不再要求重写一条已经不存在的 CPU bake 路径。剩余 P0 是 current graph 的产品 consumer、
view-neutral scene/LOD、capture shadow、publication/physical residency transaction 以及同源 GPU/
memory/power 验收，详见 §12.20A--§12.29。状态为
`source_gpu_graph_and_nonblocking_job_implemented_pending_product_correctness_and_managed_performance_power_validation`。

### P0-9：正常导入与warm-cache判定仍做完整decode/read

普通`stage_environment_ibl_source`在reuse判断前先解码整张HDR为RGBA32F；`staged_bundle_state`读取并解码`.zcube`，再读取derived artifact才判定Current。必须先用source fingerprint、manifest header、schema/recipe/platform hash做轻量命中，内容块按需验证；完整decode/filter只能发生在明确miss或审计模式，并由cook scheduler承担。

### P0-10：没有统一budget/readiness/失败终态，无法证明优于Unreal

当前存在import timing、runtime bake queue和部分probe错误，但没有统一记录environment generation、cache hit原因、CPU/GPU bake时间、capture latency、upload bytes、probe list occupancy、VRAM、fallback、stale age和device loss。必须建立单一readiness/telemetry truth和匹配画质benchmark协议；关闭大气、减少probe或使用低分辨率取得的帧时不能算“优于”。

## 5. P1 差距清单

### P1-1：每个IBL graph pass重复构造整份command plan（implemented，managed validation pending，见12.32）

`record_ibl_bake_wgpu_pass_for_request`每次调用`ibl_bake_wgpu_command_plan_for_request`，分配全部PMREM/SH9/IEM command后再线性查找当前pass。MVP先硬切为只构建当前pass的exact command，将一次bake从`O(P^2)`降为`O(P)`；长期compiled graph应直接持有immutable encoded command/parameter index，不在record热路径重建字符串和Vec。

### P1-2：IBL与realtime capture/source-mip仍逐dispatch创建params buffer/bind group

2026-08-31 current source 复审修正了原描述：realtime PMREM/SH9 recorder已有按`(slot, command key)`的持久binding cache；默认ticket包含10个PMREM和1个SH9 cacheable pass，stable work slot重放是11 hits、0 params buffer和0 bind-group创建。未收敛的是3个capture和7个source-mip pass，每个generation仍固定创建10个小uniform buffer与10个bind group；首个slot的完整ticket另有11组PMREM/SH9冷创建。通用artifact bake也仍由每个command创建独立params/bind group。现有ignored WGPU profiler已分别输出capture/source-mip/PMREM-SH9创建数、creation micros和adapter identity，但本轮没有current-source执行数据，不能证明这10组动态对象是CPU/GPU/功耗主瓶颈。后续先采cold/warm p50/p95/p99与WPR/WPA，再在persistent parameter arena、dynamic offsets或immutable per-slot templates间选择；禁止在没有profile时扩大cache或改变scheduler。状态：`realtime_pmrem_sh9_source_closed_dynamic_capture_source_mip_profile_pending`。

### P1-3：首次realtime IBL把全部工作塞进一帧（MVP source closed，见12.31）

首次发布前的full update包含两次六面capture、全部downsample、PMREM和SH9，容易形成明显首帧hitch。需要预烘fallback、分阶段ready policy、GPU time budget和可选低分辨率bootstrap，再渐进替换为目标质量。

### P1-4：静态gradient也自动进入realtime IBL（MVP reclassified，见12.31）

只要程序sky intensity大于0，renderer就为其启用realtime resources；没有Static/OnChange/TimeSliced/EveryFrame策略、minimum interval、change threshold或importance。静态天空应优先cook/cache，动态天空按revision和预算更新。

### P1-5：procedural sky存在多份WGSL owner（MVP source closed，见12.31）

skybox draw与realtime capture各自实现gradient/sun公式，环境fallback又从公共shader采样同类参数。必须由09C的单一sky module生成display、capture和reference variant，source hash进入environment generation，避免同场景天空与反射不一致。

### P1-6：无PMREM的procedural fallback在粗糙度上错误

fallback specular直接按reflection direction取gradient，与roughness无关；diffuse也直接按normal方向取sky颜色而非半球卷积。当前六态realtime IBL status已能把它明确标为`Fallback`/`FailedFallback`，但冷启动画质缺陷仍存在。2026-08-31结构复审确认默认首代是21个accepted frame batch：3个双面capture、7个source mip、3个PMREM mip0双面batch、7个其余PMREM mip和1个SH9；`RealtimeIblFrameBatch.operations`、topology key、attempt token、retry与timing均以单operation为原子合同。把冷启动机械合并成一次submission仍约需17个GPU pass（all-face capture、7 source mip、8 PMREM mip、SH9），不会降低滤波工作量，且会制造未测量的启动尖峰。故本轮不做局部合批或降低采样数；必须先用同一冷启动场景记录per-pass GPU timestamp、submission/command build、publication latency、RSS/VRAM与WPR/WPA能耗，再原子修改scheduler、topology cache、retry/completion和timing ABI。状态：`algorithm_defect_confirmed_structural_bootstrap_cut_pending_current_source_gpu_and_power_profile`。

### P1-7：BRDF LUT runtime CPU积分已硬切为versioned builtin

优化前 128 x 32 x 128 共 524,288 个 sample iteration 会在首个 renderer lease 发布前同步执行。
2026-08-31 production 已改为只物化/上传 16,384-byte RG16F builtin；动态 integrator 仅用于测试期
byte certification，不保留 runtime fallback。31 个冷进程的 current-source materialization p50
为 `31.1 us`（优化前 integrator p50 `22.086 ms`，isolated ratio `710.16x`），固定 1 allocation/
`16,400 B`。ready-frame evidence 同步 hard cut 到 v17 的 builtin-materialization 语义；详细范围、
哈希与限制见 §12.30。状态为
`implementation_complete_static_and_tooling_green_pending_managed_rust_wgpu_product_timing_power_validation`。

### P1-8：基础IBL只有single-scatter split-sum质量合同

当前2-channel GGX LUT、PMREM和specular occlusion可作为standard PBR baseline。2026-08-31 source复审确认`IblBakeRecipe`已经版本化并统一拥有PMREM sample tiers、roughness/mip双向映射、FIS texel solid-angle scale、full-roughness cosine threshold、diffuse source mip、output format，以及asset CPU/runtime GPU diffuse integrator identity；CPU/GPU producer不是未区分的共同hash。剩余缺口是BRDF LUT的128x32x128 domain/integrator尚未进入该recipe identity，且base-lobe multiple-scattering energy compensation没有明确mode、artifact version或误差门。current source没有multiple-scattering/energy-compensation consumer。MVP继续以当前Unreal-equivalent joint-Smith single-scatter为唯一baseline，先完成managed image/error/performance验收；不得为关闭P1条目提前加入高级补偿。clearcoat/sheen/anisotropy仍归09G。状态：`base_single_scatter_mvp_recipe_present_brdf_lut_identity_and_multiscatter_contract_pending`。

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

direct cosine IEM的估算是source texels乘output texels，虽然已并行仍会随输入/输出分辨率快速放大。2026-08-31调用链重审确认它不是renderer热路径：默认asset import只请求`PMREM_SH9`，只有显式`environment_ibl_irradiance_cube=true`才在asset-import/cook阶段生成IEM；current bundle cache hit在任何cubemap/IEM构建前立即返回。renderer缺失/失效artifact时使用既有GPU PMREM/SH9/IEM graph，运行时hydration只解码artifact并准备upload。因此MVP生产分层已经达到“GPU runtime、cached opt-in CPU cook、CPU oracle/test”的目标，不实施renderer侧替换。P1-9状态改为`runtime_path_closed_opt_in_cook_profile_retained`；后续只在真实opt-in cook语料显示该阶段主导导入耗时或功耗时，才比较SH重建/GPU cook，并同时记录samples、误差、耗时、RSS和能耗。

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

2026-08-31 current-source全链重审确认这不是缺少几个枚举值，而是公开了无行为合同：`ProbeBakeTiming`只被`ReflectionProbeData`默认值、getter/builder、offline bake对默认值的重复覆盖、reflection-probes插件placement JSON/runtime projection和roundtrip测试消费；renderer、World、Editor和capture scheduler均不读取它。插件已经提供非阻塞request/poll/cancel包装，但仓内没有Workbench、Inspector、World或其它产品consumer调用该包装。`RenderFramework::request_environment_capture`已经拥有generation去重/拒旧、单pending/单active有界队列、进度、取消、terminal status、last-good publication和可选persistence payload。因此向snapshot继续增加`Baked/Custom/OnLoad/OnChange/EveryFrame`只会产生第二套没有执行权的策略表。

Unreal主参考也不把这些名字作为render snapshot标签：静态reflection capture通过dirty/on-load队列进入`UpdateReflectionCaptureContents`；runtime capture以显式`RefreshCapture`请求、一次性fast/smooth标志、帧预算和分帧状态进入同一分配器；every-frame只作为诊断CVar或独立SceneCapture能力存在，SceneCapture还会警告在every-frame启用时手动capture是重大重复开销。Zircon MVP据此硬切为：从`ReflectionProbeData`及公共导出删除`ProbeBakeTiming`，不保留serde alias/re-export/compat shim；authoring/lifecycle owner未来只保存可执行的baked或runtime能力、quality/capture policy和artifact identity，并把change/manual/on-load事件转换为带revision/generation的显式`RenderEnvironmentCaptureRequest`；RenderFramework scheduler继续是唯一调度、预算、取消、失败和publication权威。`EveryFrame`不进入reflection-probe MVP，只有取得动态capture GPU/功耗profile且共享budget owner能稳定限流时才可另行评估。

2026-08-31 coordinator epoch 584 已完成六文件 blob reconciliation，随后原子实施 hard cut：删除
`ProbeBakeTiming`、`ReflectionProbeData` 字段/getter/builder、environment/root 两级导出、offline 默认覆写与
插件 placement JSON/runtime projection，不保留 serde alias 或 compatibility shim。core probe JSON 和插件
placement JSON 均不再输出 `bake_timing`；placement 保留 `deny_unknown_fields` 并由回归明确拒绝携带旧字段
的 payload。production Rust 的 `ProbeBakeTiming` 为零命中，`bake_timing` 只剩两处 negative assertion；
offline bake 不再写伪策略。TDD RED snapshot 为 `2475`，focused rustfmt、静态合同与 scoped diff integrity
通过。剩余 P0 产品缺口仍是 scene/Editor lifecycle owner 尚未把 authored manual/change/load 事件转换为
revisioned request 并展示同一 scheduler 的 terminal status。状态更新为
`fake_snapshot_policy_hard_cut_implemented_static_green_product_scene_consumer_pending`；managed Cargo/产品验证
待执行，M5-M8 不推进。

### P1-16：probe capture缺少可见性、排除与自反射contract

2026-08-31 已实施layer-culling request/key与packet-local基础合同：neutral `RenderEnvironmentCaptureRequest`
持有独立`RenderLayerSet`，默认覆盖scene-schema-v1的32层，空集合显式表示sky-only；该集合进入
capture bake key，在唯一一次`O(M)` mesh draw census之前安装到初始camera descriptor，并由全部六个
capture camera的`culling_mask`继续持有。reflection-probe JSON
合同硬切到schema v2并新增`capture_layer_mask`，与placement的receiver `layer_mask`保持正交，
旧v1不静默迁移。mask还进入terminal output identity；bake key以固定`u64`数量和`u32`层号编码，
不受host pointer width影响。但当前`RenderFramework::request_environment_capture`接收的是
`SceneViewportRenderPacket`：上游`World::build_viewport_render_packet`已按viewport camera layer过滤并按
viewport位置选择LOD，后置capture mask只能进一步收窄，不能恢复被viewport排除的对象。因此独立capture
extract owner仍未闭合；剩余reflection mask、LOD、sky/transparent/emissive policy、self exclusion、
exposure和direct-light receiver channel仍开放，必须与09B visibility和09E lighting共享view policy。
当前状态为`capture_layer_request_key_and_packet_local_culling_implemented_static_green_full_capture_extract_pending`。

### P1-17：environment切换没有时域crossfade与history identity

2026-08-31 current-source与Unreal主参考复审后，本项从笼统缺口收敛为advanced dynamic-transition能力，而不是静态/Editor-baked MVP的正确性阻塞。核心capture output已经携带独立scene/environment/output revision、capture mask、probe target、persistence/runtime-cache identity；scheduler只发布当前generation。local probe allocator额外保留一个physical spare，capture刷新期间继续采样旧ready slot，成功提交时原子轮换，取消/失败释放candidate，因此已有last-good但提交后立即释放旧slot。普通global environment目前仍是单套resource/bind-group，且rebind失败回滚缺口已交给Render11；在该事务修复前增加old/new双资源会扩大半提交状态空间。

Unreal runtime reflection capture的smooth blend是显式一次性请求，仅在capture以前有内容且有受CVar限制的blend slot时启用；slot不足直接pop，并与runtime capture帧预算、timeslice和fade state共同拥有，而不是每次source revision变化都无条件双驻留。Zircon MVP据此保持“last-good到新generation的原子切换”，不增加第二次PMREM/probe sample、双bind group、通用temporal history或camera-cut依赖。后续transition owner必须先取得动态昼夜/streaming场景的GPU p50/p95/p99、额外resident bytes、slot pressure/fallback count和WPR/WPA功耗before数据，再定义bounded transition slots、old/new generation、pre-exposure blend domain、duration、cancel/supersede和内存回收；camera cut只在设计真正使用屏幕时域history时才参与，不能借名强行绑定。当前状态：`mvp_atomic_last_good_identity_present_advanced_crossfade_deferred_pending_render11_transaction_and_dynamic_profile`。

### P1-18：AmbientLight的lightmap语义被最终着色忽略

2026-08-31 current source 已完成09F1 consumer policy。`RenderAmbientLightSnapshot`保留scene component的`affects_lightmapped_meshes`，`SceneUniform::from_frame`在同一个O(A) fold内分别累积全部ambient和允许作用于lightmapped mesh的ambient；empty/preview fallback保持两者一致。Forward/generated/fallback按GPU Scene已有的instance lightmap bit选择ambient。Deferred GBuffer把同一bit写入既有`Rgba16Float` emissive alpha，full与environment-only lighting用原有一次emissive load解码，未新增MRT、binding或texture sample。SceneUniform ABI只增加一个16 B `vec4`，没有shader feature、permutation或PSO identity变化。09F2仍拥有baked-lighting能量与lightmap生产，09F1只负责ambient source eligibility，避免重复间接光。源码合同和完整ABI镜像静态检查为GREEN；E盘隔离`naga 29.0.3`验证full Deferred、environment-only Deferred、fallback、两个skybox variant与六个normalizer delegate全部通过，probe source/binary SHA-256为`1738DA460B1CF5535F45297C5E70EFAD0A8FD6AA561ACE34A971E29413DB37AC`/`D8E7D5E589B67B875A1F36405D00B11090751B2D536B713A97CDB10E5AC64EAC`。managed Cargo/WGPU、当前源码PNG、RenderDoc、GPU timing与功耗仍待执行。状态：`implementation_complete_static_and_isolated_naga_green_pending_managed_wgpu_and_product_validation`。

2026-08-31 current-source follow-up：上述“完整ABI镜像GREEN”只适用于当时被隔离Naga probe覆盖的Shader06集合。Render07随后新增的`ssao_spatial_denoise.wgsl`与`ssao_bilateral_upsample.wgsl`由builtin descriptor直接装配，并读取`ambient_color`之后的camera字段，却遗漏`lightmapped_ambient_color`，导致后续字段相对496-byte CPU合同少16 bytes。两文件属于foreign in-flight half-resolution AO dependency set，09F1不拆开覆盖；已发布[`Render07 SSAO SceneUniform lightmapped ambient offset drift`](../../zircon_runtime/render/07/failure-2026-08-31-ssao-scene-uniform-lightmapped-ambient-offset.md)。P1-18的ambient eligibility功能仍为implemented，但广域current-source ABI门重新标记为`implementation_complete_core_naga_green_cross_plan_ssao_mirror_failure_open`，修复回传并完成managed Naga/WGPU前不得称accepted。

### P1-19：realtime IBL已有共同status入口，但environment全域仍未统一

realtime IBL现由runtime owner统一输出`Fallback/Baking/Ready/RefreshingLastGood/FailedFallback/FailedLastGood`、published/pending/queued identity、generation frame、coalesced count和optional typed failure；framework query先完成pending submission，且不保留failure-only并行入口。cache/import/probe/capture仍是局部report或test统计，尚未形成Editor可查询的artifact provenance、probe overflow、capture latency与全域readiness产品。公共诊断必须继续来自runtime owner，Editor只展示，不复制判断逻辑；本项因此为Partial而非关闭。

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
- 该2026-08-16基线曾让`RealtimeIblTimeSliceScheduler`首次更新同时提交`CaptureSky(ALL)`和`CaptureCloud(ALL)`，并由recorder写入相同source mip 0。current source 已按M7 hard cut删除后者；本条保留为历史失败证据，不再描述现状。
- 现有scene基础已经有serde节点记录、固定component clone/serde projection、`ZrReflect`注册、property path和transaction；M1必须复用这条持久化链路，不得以仅runtime的临时struct替代。

这些结论只完成M0的当前源码复核；未产生GPU timing、VRAM、WPR、RenderDoc或视觉验收，M0和全计划的implementation status仍为pending。

#### P0-7 structure and measurement recheck (2026-08-24)

本次重审以`zircon_runtime`当前WGSL、probe resource owner、09B/09D计划边界为事实源，并以Unreal为主参考、HDRP为渲染管线辅助参考。审查时的SHA-256分别为：`zr_environment.wgsl` `ac6195265b2b437d3c3b0d764e69fdac3f6d61e3d1abfbfa7798a57d79f291cf`、`probe_buffer/resources.rs` `b7f47a28fc7dbc68bf22eeb36af4799d34abe4ced4e5ff00f850ccf4d7c061f2`、`gpu_layout.rs` `264da38339eaf0f4f8cb46f111bd55edb661d8e2cd1bb4b746dbc19f745eaec8`。

- `zr_environment_select_probes`仍以`min(header.probe_count, arrayLength(...))`遍历全局`array<ZrGpuReflectionProbe>`，只消费`misc.x`强度；`GpuReflectionProbe::from_probe`虽已把layer mask写入`misc.w`，入口没有object reflection mask，故不能在shader正确过滤。CPU `prepare`又逐帧创建candidate `Vec`、按camera layer/距离排序、截断64、查询registry，并可能在提交前调用`load_texture_asset`和逐mip/face upload。此为每片元O(global probes)与每帧CPU筛选/同步资产路径的同一结构问题，不能靠降低常量、加局部early-out或只改shader修复。
- 并行的post-process reflection路径是独立的8-probe buffer，`encode_reflection_probes`当前固定返回零数组和零数量；它既不是PBR probe的consumer，也不能作为M9 ABI的临时替代。M9必须先收束这些并行真值，不能让其中之一继续隐藏回退扫描。
- Unreal的`ReflectionEnvironmentPixelShader.usf`先以`ComputeLightGridCellIndex`查询`FCulledReflectionCapturesGridHeader`，取得`NumReflectionCaptures`与`DataStartIndex`后才聚合radiance；HDRP `HDProbeSystem`同样先建立带state hash的cull state，再查询有界索引结果。可迁移的结论是“由视图/空间可见性生产offset/count局部列表”，不是复制任何引擎的类型名或固定容量。

因此M9保持既定顺序：先由09B persistent render scene发布版本化`ReflectionProbeSpatialAssignment`（view/cluster变换、object reflection mask、offset/count、packed index、resident generation、overflow）；再有确定性CPU reference list和范围/overflow上传验证；随后让forward、deferred、generated PBR共用lookup，最后才删除全局扫描、固定64 admission和资源常量的本地所有权。09D随后接管slot/page容量、分辨率及VRAM预算。当前09B尚未形成该persistent scene authority，09D也未形成probe residency owner；在此前直接改probe shader会制造第二套分配器，违反本计划的owner边界。

性能数据尚未产生，不能推断功耗、GPU耗时或相对Unreal的结果。后续M9实现前必须在固定二进制、驱动、画质、HDR source、相机轨迹和1920x1080/3840x2160下，分别记录1/64/1k probe的CPU frame、GPU frame、probe assignment build/rebuild、每片元候选访问、overflow、upload bytes、resident/peak VRAM、fallback和Energy Meter区间；RenderDoc从`D:\\Tools\\renderdoc`捕获PBR forward/deferred shader实际循环与buffer range，WPR只覆盖同一运行区间。稳定场景必须证明assignment零重建，密集场景必须证明overflow有显式fallback而非全局扫描。当前受M0的coordinator-managed验证阻断与未清理的外部D/E/F artifacts影响，尚未运行这些采集；`implementation_status: pending`和`source_recheck_required: true`不变。

#### PBR call-contract and probe-cost recheck (2026-08-26)

本次在修改优化算法前重新沿`zr_shading_standard_pbr(.basic).wgsl -> zr_environment.wgsl -> zr_environment_core.wgsl`核对forward/deferred/generated调用合同，并反查`SceneUniform::from_frame`、SH9 bake/upload与probe CPU/GPU排序规则。当前静态结论如下；它们只说明现有数学合同未发现重复应用，不等于GPU画面或性能验收完成。

- PBR调用方传入未乘`(1 - metallic)`的base/diffuse color，direct lighting与environment owner分别且仅分别应用一次`zr_surface_metallic_diffuse_energy_scale`；baked diffuse也只在自己的合成owner应用一次。AO只缩放ambient/indirect diffuse与specular occlusion，不缩放direct light，未发现重复metallic或AO能量衰减。
- SH9生产端与WGSL消费端使用相同Y-up基函数顺序，系数包含irradiance-over-pi卷积因子`1, 2/3, 1/4`；consumer直接乘albedo，因此这里不存在额外补乘或除以pi的依据。BRDF LUT坐标、roughness到PMREM mip映射和specular-occlusion公式也保持单一owner。
- forward路径消费材质`dielectric_f0`，deferred路径固定`0.04`不是可局部修补的GBuffer错误：当前非默认IOR由既定材质能力路由到forward。在扩展deferred GBuffer ABI和所有pass identity之前，不应偷占现有通道。
- 正交相机传给环境反射的是world-space surface-to-camera方向，也就是相机backward轴`rotation * +Z`；透视相机仍由world position重建逐像素view vector。现有符号正确，本轮只补充旋转相机契约测试与ABI注释，防止后续误用`Transform::forward()`的`-Z`反号。
- 真正已确认的结构性热点仍是`zr_environment_select_probes`：每个PBR fragment对全部resident probes做`O(P)`扫描，包含influence、距离、旋转与top-two排序，当前上限64；clustered light grid没有承担probe assignment。复杂度是`O(shaded fragments x resident probes)`，不能用某个WGSL微优化证明问题消失。

在M9结构修改前增加一组可归因基线，而不是直接实施空间索引：固定同一场景、材质、HDR、曝光、相机轨迹和shader generation，在1920x1080与3840x2160下分别测0/1/8/32/64个可见probe，forward/deferred各采集GPU p50/p95/p99、probe fragment visits、environment pass/draw耗时、CPU prepare/sort/upload、resident/peak VRAM和Energy Meter区间；另以1/64/1k全局probe验证CPU assignment规模。RenderDoc必须记录实际shader、loop bound、buffer range和draw绑定，时间戳/WPR/功耗区间绑定同一binary/source fingerprint。只有线性斜率被复现后，才比较09B cluster/tile list的有界候选K及其build/rebuild成本；优化后用相同矩阵证明稳定帧零重建、fragment成本不再随全局P增长且overflow不回退全表扫描。当前未执行GPU/WPR/功耗采集，M0、M9、M10与`implementation_status`均保持pending。

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

The 2026-08-16 baseline made `published_key == None` a full-update branch that
recorded two six-face captures before source mips, PMREM and SH9. The second
capture was the same gradient producer writing the same source mip-zero faces.
Current source has completed the required hard cut: `CaptureCloud` is absent
from the scheduler, graph and WGPU recorder, and a source contract rejects its
return. The remaining M7 work concerns budget/profile evidence and physical
producer integration, not tuning or restoring that duplicate operation.

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
WGSL lookup/include, receive the object's reflection mask through a versioned
scene/material/GBuffer ABI, and rank only the returned candidates. A list miss has an
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

## 12.4 2026-08-30 camera/probe layer ABI repair

The existing 16-byte `GpuReflectionProbeHeader` now uses its second word for
the selected camera's legacy scene-schema mask. `GpuReflectionProbe.misc.w`
continues to carry the probe mask, and the shared WGSL selection helper rejects
non-intersecting candidates before influence weighting. This closes the
previously unconsumed camera/probe mask contract without adding a binding,
buffer, permutation, or per-fragment data fetch. It is deliberately not the
M9 spatial assignment: object reflection-mask input is still absent from the
GPU scene/material ABI, so the fixed global scan remains the explicit fallback
until 09B/09C publish the typed local-list contract.

### 12.5 2026-08-30 runtime-cache consumer boundary

The reflection-probe runtime now has an explicit consumer for a completed
`RendererGpuRuntime` cache artifact. It requires the capture request's source
hash, validates the current cache descriptor through the existing cache store,
and registers only the PMREM TextureAsset in memory. Missing identity,
cache-miss, stale, and rejected artifacts fail closed. The path does not write
an asset-derived `.zcube` or claim editor scene/catalog/undo publication; those
remain a separate authoring owner.

### 12.6 2026-08-30 shader/IBL cost model and M9 measurement gate

The current capture path keeps the source cubemap mip chain, PMREM, and SH9
on one GPU command encoder. The canonical 128-face PMREM has `P=8` output
mips with 32/64/128 importance samples by roughness band; the runtime diffuse
SH9 path evaluates a 32x32x6 domain (6,144 samples) with a 64-thread
workgroup reduction. These are fixed capture-time costs, not evidence that a
different sample count or a per-fragment filter would be faster. CMFT and
Unreal likewise schedule face/mip filtering during capture or preprocessing
and reuse the resulting resident artifact.

The more material steady-frame upper bound is probe selection. The current
WGSL fallback scans at most 64 active probes per selector invocation, so the
base-lobe bound is `pixels * min(active_probes, 64)`: 132,710,400 candidate
visits at 1080p and 530,841,600 at 4K. An active clearcoat lobe performs a
second selection after a planar miss, so the corresponding worst cases are
265,420,800 and 1,061,683,200 visits. The current workload report multiplies
pixels by active probes and therefore describes one selector invocation, not
the clearcoat-inclusive total. The CPU preparation path is `O(N)` candidate construction,
`O(N)` selection, `O(64 log 64)` final ordering, and up to 64 synchronous
texture loads on cold slots; warm resident slots avoid the load but do not
remove the fragment scan. These are complexity bounds, not measured timings.

M9 remains a measurement-gated structural change. Before selecting tile or
cluster dimensions, list capacity `K`, or changing PMREM/SH9 sample budgets,
the same scene and hardware must provide 1/8/32/64-probe warm and cold runs
for forward and deferred paths: GPU timestamps, a fragment-visit proxy or
instrumented visit count, list length/overflow, CPU p50/p95/p99, RSS, VRAM,
and WPR/WPA power. Only those data can justify replacing the fixed global scan
with the typed `ReflectionProbeSpatialAssignment` ABI already specified in
this plan. Until then, no guessed shader micro-optimization is authorized.

Status remains `review_complete`, `implementation_status: pending`, and
`source_recheck_required: true`; this section records the architecture and
acceptance gate only.

### 12.7 2026-08-30 diffuse-energy owner re-alignment

Source recheck found the standard/environment diffuse paths had reintroduced a
view-Fresnel helper despite the canonical `1 - metallic` decomposition. The
Forward, Deferred, environment-only, fallback, and baked-lightmap consumers now
share the scalar metallic owner. Fresnel remains explicit only in the
transmission energy helper; ordinary diffuse therefore performs no Schlick
evaluation and adds no texture sample, binding, permutation, or PSO key. This
is a source-contract and bounded ALU reduction only. Current-source managed
GPU timestamps, visit proxies, CPU latency, RSS/VRAM, WPR/WPA power, PNG, and
RenderDoc evidence are still required before any M9 performance status changes.

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

### 12.8 fullscreen uniform ABI 端序审计

源码审计发现 fullscreen pass 参数打包曾使用主机端序，导致同一 WGSL uniform 合同在不同 CPU 端序上可能产生不同字节。现已在唯一上传前 owner `FullscreenPassPlan::write_parameter_bytes()` 中固定 `u32::to_le_bytes()`，并加入 source-contract 断言；这是 ABI 正确性修复，不引入新的 buffer、binding、采样、分支、循环、permutation、Naga/WGPU module 或 PSO identity。

当前仅有 scoped rustfmt/static 证据，受管编译、GPU capture、产品截图、CPU/GPU timing、RSS/VRAM 与 WPR 功耗数据仍待完成；因此本项保持 `source_closed_pending_managed_validation`，不将端序修复误报为性能收益。

compute indirect 的 12-byte test fixture 也已改为 `u32::to_le_bytes()`，避免 host-endian 测试数据掩盖 dispatch ABI 问题；该调整仅影响测试构造，不增加 runtime dispatch 成本。

### 12.9 capture persistence identity 分离

复审发现 C4 曾把 asset/editor 提供的 `IblBakeArtifactRequest` 直接交给 renderer runtime-cache writeback。该 request 的 source hash 是外部资产身份，不拥有捕获位置、clip range、Fast/Normal/High quality 或 scene/environment/output generation；直接复用会让不同 GPU capture recipe 在同一外部 key 下互相覆盖，且把 `AssetImporterCpu` 语义边界与 `RendererGpuRuntime` 缓存边界混在一起。

当前硬切为两条身份：asset/editor request 只作为未来 `.zcube`/catalog/undo 发布意图随 terminal output 保留；renderer writeback 使用 `RenderEnvironmentCaptureRequest::ibl_bake_key()` 派生的 `runtime_cache_artifact_request`。reflection-probe runtime consumer 同样重建 renderer-owned key，不再用外部 source hash 直接查 runtime cache；source hash 仍作为显式持久化 admission，缺失时 fail closed。该修复只改变 CPU identity construction，不增加 GPU dispatch/readback、binding、sample、PSO、permutation 或 allocation。当前 readback 没有 source cubemap，故不得把 PMREM/SH9 runtime cache 反向包装为资产 `.zcube`；editor asset-derived staging、catalog 与 undo 继续保持 pending。源码 identity contract、`rustfmt --check`、scoped diff integrity 与环境/RenderDoc Python `18/18` 通过，managed Cargo/WGPU 与产品证据仍开放。

### 12.10 source cubemap readback 与 `.zcube` 边界

进一步源码复审确认 `.zcube` 只拥有 capture source mip pyramid，PMREM/SH9/IEM 是不同 producer/recipe 的派生数据；原编码 API 却要求完整 `SourceCubemapMipChain`，会迫使 GPU capture 路径构造并不属于 source container 的过滤结果。资产层现在同时提供 validated source-texel 与 canonical RGBA16F-byte 两个入口，后者可直接封装 GPU readback，不经过整链 `f32x4` 展开或二次 half-float 量化。

第一版 backend 草案按 `face -> mip` 建立 66 个 callback。静态预算复审证明它在 1024 尺寸必然失败：RHI 默认每帧最多 64 个请求，且完整链 `67,108,848 B` 也无法放进单个 `16 MiB` 请求或 `32 MiB` 帧。当前结构改为每个 face 打包全部 mips，并按 live device profile 的 request/frame/pending budget 排批；上一批未全部 terminal 时拒绝下一批。1024 时每面 padded `11,190,016 B`，默认每批两面 `22,380,032 B`，共六请求/三帧。callback 到达后立即移除 row padding并写入唯一的 `67,108,848 B` canonical buffer，随后释放 face delivery；canonical 加最大当前 delivery 为 `89,488,880 B`，不再同时驻留完整 padded 链和 `134,217,696 B` f32 链。该数值不是实测 RSS/VRAM，native staging 和驱动分配仍必须由后续 profiling 量化。

该 API 随后的实现已接到 capture scheduler，但不是塞入普通 viewport 帧的一次性读回。framework 以 `Capturing -> Persisting` typed owner 保留 source target、probe reservation 和 submission ticket；只有带 persistence output URI 的请求进入 persistence，每次成功 render pump 最多提交一个 budget-derived face batch。scheduler 对 persistence 请求的 plain success fail closed，只有携带一次性 `RenderEnvironmentCaptureSourcePayload` 才能进入 terminal success。cancel、supersede、diagnostic admission、ticket、payload validation 或 mailbox 失败均取消 reservation，且不覆盖 last-good。该路径没有 `device.poll`、ticket wait、新 binding、sample、PSO 或 permutation；但生产增量现在真实包含六个 source copy/readback request、默认三批 diagnostic submission 和 `67,108,848 B` canonical CPU payload，因此旧有 `+0` 结论作废，必须以后续 current-source profile 为准。

reflection runtime 将 owned canonical payload 编码为 `.zcube`，editor adapter 再交给 `ProjectAssetManager` 现有 durable transaction，使 source、sidecar、derived artifact、registry/resource mutation 与 watcher echo 同批发布，不直接 `fs::write`。第二轮内存审计发现 `source_bytes_for_import(&source)` 会克隆 generated snapshot；当前 targeted generated-source 路径改为 `take_source_bytes_for_import(&mut source)`，把同一 vector 移入 importer context 并最终移入 prepared source write，消除一个持久的 `67,108,864 B` 1024 `.zcube` clone。source readback 阶段的逻辑 live upper bound 仍是 canonical `67,108,848 B` 加最大 delivery `22,380,032 B`，即 `89,488,880 B`；header insertion 的 allocator reallocation、importer 内部临时量、native staging、RSS/VRAM 仍没有实测，因此不得把该逻辑数值当作进程峰值或性能收益。

editor undo 复审不允许把 project publication 硬塞进 UI trigger。当前 editor transaction 通过同步 `EditCommand::apply/revert` 和 `EditContext` 回放，而 project source/catalog/resource transaction 由 `ProjectAssetManager` 独立拥有；receipt 的 previous hash 不足以恢复旧内容，在 history 中复制整份旧 `.zcube` 又会造成每条命令约 64 MiB 的常驻成本。结构性后续应是后台 capture/publication job，加 project-owned content-addressed reversible source-swap token，再由 operation factory 把 token 与 scene placement/dirty state 作为一个 editor command 提交。该 owner、post-take failure retry、current-source PNG/RenderDoc、CPU/GPU timing、RSS/VRAM 和 WPR/WPA 尚未完成，所以 09F1 与 M6-M9 保持 pending。

### 12.11 captured `.zcube` importer 与 bulk-data profile gate

端到端 import routing 复审发现一个功能级扩展冲突：`texture_importer.cubemap` 是项目 `.zcube` 的唯一 importer，但原实现只接受 TOML manifest；GPU capture 输出的 `ZRZCUBE1` binary container 会在 `source_text()/toml::from_str` 失败，因而上一版 project publication 实际无法形成闭环。当前不新建竞争 importer，而在原 owner 内先检查八字节 magic：binary 路径只验证 header、完整 mip pyramid 与 payload 长度，直接保留 RGBA16F container；非 binary 路径继续处理 six-files、horizontal/vertical cross 与 equirectangular manifest。只有 cubemap recipe 从 version 2 升为 3，避免无关 image/container/PSD/array artifact 失效。该路径不进行 `decode_rgba16f_texels`、`f32x4` 展开、offline mipgen、BC5 transcode 或 direct GPU upload。

对 N x N、完整 mip pyramid 的 source container，payload 规模为 `8 * 6 * sum(max(1, N >> mip)^2)`，1024 时 container 含 32-byte header 共 `67,108,864 B`。当前发布链至少包含以下 `O(B)` 工作：readback row-unpadding 写 canonical payload；header insertion 的 `copy_within`；source hash；binary importer 把 source `Vec` clone 到 `TextureAsset`; `ArtifactCacheAsset::from_imported` 再 clone 一次并由 bincode/zstd 流式编码；durable transaction 把 source 写入 staging 并同时 hash。覆盖已有 target 时还会流式 backup/hash 旧文件并再次读取验证 original digest。准备 cache artifact 时 source、TextureAsset、ArtifactCacheAsset 三份 payload 可同时存在，1024 的逻辑 payload 为 `3 * 67,108,864 = 201,326,592 B`；cache clone 释放后，prepared source write 与 ready TextureAsset 仍为 `134,217,728 B`。这些数值不含 allocator reallocation、zstd state、chunk residency、native staging，也不是 RSS 实测。

Unreal 本地参考 `Runtime/CoreUObject/Public/Serialization/EditorBulkData.h` 的 `FEditorBulkData` 使用 content-derived payload id、`FSharedBuffer` ownership、future/callback retrieval 与 unload/reload；`BulkData.h` 同样把 bulk payload serialization/lifetime 与对象元数据分离。Zircon 的结构性候选因此不是继续微调 `Vec::clone`：应建立 immutable shared bulk payload/content identity，使 import context、source-only TextureAsset 与 artifact serializer 借用同一 backing；让 durable transaction 接受 segmented/shared/already-staged input；默认不把 source-only `.zcube` 常驻为可上传 runtime texture；并由同一 content-addressed owner签发 undo source-swap token。只有 E 盘 current-source ETW/WPA profile 同时采集各阶段 CPU p50/p95/p99、allocation/RSS high-water、read/write bytes、compression ratio、wall time 与 WPR energy 后，才允许选择其中一个迁移切片。当前仅完成 importer correctness 与成本模型，未声明 CPU、内存、I/O 或功耗收益。

这里的 publication 闭环必须进一步限界：binary `.zcube` importer 只形成 source-only `TexturePayload::Container`，既有 external cubemap staging 只识别 DDS/KTX，因此该事务当前没有自动生成 asset-derived PMREM/SH9/IEM。live renderer 的 PMREM/SH9 仍按独立 runtime-cache identity 发布与读取，这足以维持当前会话的 last-good，却不能证明 cold restart、cook/package、cache eviction recovery 或 source-only rebuild。下一结构切片必须先指定唯一 source-to-derived recipe owner、缓存失效键和 cook/runtime residency 契约，再决定它属于 texture importer、IBL baker 还是 project pipeline；在此之前不把 binary import success 记作完整 PBR persistence acceptance。

### 12.12 cold/cook source-to-derived owner 与 miss-path memory gate

上述 owner 选择已通过现有架构收敛，不新增第二套 baker：`asset/importer/environment_ibl/source_cubemap_texture.rs` 统一分类 captured `ZRZCUBE1` 与 DDS/KTX，project scan 在普通 texture import 后调用 `prepare_source_cubemap_texture`，而 `.zcube`/`.zribl` identity、CPU recipe、algorithm version、paired restore 和 durable publication 继续由既有 IBL staging owner 管理。captured 分类只读取/验证 32-byte header、payload 长度和 texture metadata，不展开 texel；current pair 命中后不做 `decode_rgba16f_texels` 或 PMREM/SH9 rebuild。miss 才解码 source mips并用现有 Normal-quality asset-importer recipe 生成 asset-derived `.zribl`。兼容入口 `stage_external_source_cubemap_texture` 保留，但 production module 从误导性的 `external_source` hard-cut 为 `source_cubemap_texture`。

这个功能闭环暴露了更高的结构性下界。1024 captured source container 为 `67,108,864 B`，解码后的 `f32x4` source 为 `134,217,696 B`；miss output 准备期间 project context source、imported source-only `TextureAsset`、重新编码的 cache `.zcube` 与 decoded source 同时存在时，仅四项就是 `3 * 67,108,864 + 134,217,696 = 335,544,288 B`。default 128-face PMREM 的 f32 chain 另为 `2,097,120 B`，还未包括 encoded `.zribl`、artifact-cache clone/bincode/zstd、allocator peak 或 native staging。warm hit 仍需对 source container 做 content hash，但不会进入 f32 decode/PMREM compute。该结果是 source-derived ownership model，不是 RSS 实测。

因此下一次优化不能直接把同步 CPU bake或某个 clone 视为唯一瓶颈。E 盘 ETW/WPA 必须分别标记 classify/hash、source decode、PMREM/SH9、paired cache encode、artifact-cache serialization 与 durable commit，并采集 CPU p50/p95/p99、allocation/RSS high-water、read/write bytes、wall time 与 WPR energy。profile 后只允许优先实现证据支持的结构切片：共享 immutable bulk backing；让 asset-derived cache 引用 durable project `.zcube` 而非再写同体积副本；或把 cache-miss source-to-derived bake移入 editor background job并保留 last-good。当前只声明 cold/cook 功能 owner 已接线，不声明性能验收。

### 12.13 cold/cook phase instrumentation 与 editor job admission gate

为执行 §12.12 的 profile-first 要求，source IBL staging 已新增八个 production phase owner：`source_classify`、`source_identity`、`cache_probe`、`source_decode`、`cubemap_build`、`irradiance_cube_build`、`bundle_encode` 与 `bundle_commit`。这些 phase 只包围对应工作块；`EnvironmentIblSourceStagingTiming` 继续保留 equirect projection、source mip、PMREM、SH9 子阶段归因，但 `total()` 只累计外层 `cubemap_build` 一次，避免把内部子阶段重复计时。原 `bundle_write()` 保留为 encode 与 commit 的兼容汇总，便于现有调用方迁移。

增量 review 发现旧 `decode_external_source_cubemap` 不是纯 decode：它在 DDS/KTX texel 展开后立即调用完整 `build_source_cubemap_from_source_mips`，把 PMREM/SH9 算法成本隐藏在 `source_decode`，会使 `cubemap_build=0` 并误导后续优化。当前保留该公开 API 的兼容行为，同时新增 crate-private decode-only source-mip API；cold/cook staging 将 DDS/KTX 与 captured `.zcube` 都送入同一个 `CubemapBuild` phase 和 Normal-quality rebuild owner，没有增加第二次 bake。

开启 `profiling` 时，每次 terminal staging report 会在 `asset` stream 输出 face/mip layout、source/derived bytes、各阶段微秒、parallel work-item 数与 IEM source-sample visit 上界。项目事务路径在 prepared IBL writes 交给 project durable transaction 前发布 prepare 观测；独立 staging 路径在 bundle durable commit 后发布完整观测。项目整体 artifact serialization、prepared/committed bytes 与 file commit 继续由现有 `ProjectGenerationPhase`/`ProjectGenerationObservation` 记录，后续 WPA 分析必须用 source URI/generation 与同一 capture window 对齐两组 owner，不得把 counter 时间当作 ETW allocation、I/O 或 energy 实测。

Editor background job 也暴露了容量级结构矛盾。1024 `.zcube` container 是 `67,108,864 B`，恰好等于 Editor09 默认 pending byte limit；payload 本体已占用 `100%`，给 URI、manager access、channel、job object 和 `Vec` capacity 留出的 headroom 为 `0 B`。因此按 payload length 提交会低报真实 retained bytes，按真实 footprint 提交则在默认空队列也应拒绝。当前不提高全局 limit、不伪造 `estimated_pending_bytes`、不建立无界 content-addressed undo 目录。Unreal `FEditorBulkData` 的复核结论仍是目标结构：content-derived payload id、shared immutable buffer、future/callback retrieval、可持久化 backing 与 unregister/unload 生命周期；Zircon 必须先由 profile 决定 shared bulk、already-staged durable input 或 Editor09 ownership-transfer admission 中哪一项成为首个结构切片。

当前证据为 focused static contracts `40/40`、Rust formatting 与 scoped diff integrity。尚未运行 Cargo、current-source ETW/WPA、WGPU、RenderDoc、PNG、RSS/VRAM 或 WPR energy；本节状态为 `measurement_infrastructure_complete_pending_managed_profile`，不授权算法、sample budget、全局 job quota 或 bulk ownership 优化。

### 12.14 procedural sky 单一源码与 generation identity

P1-5 的 current-source 复审确认 GPU radiance 真值已经收敛：skybox display、realtime capture、deferred full-scene、fallback mesh 与 generated material module registry 都直接拼装同一 `zr_procedural_sky.wgsl`。唯一残留的 `environment/procedural_environment.rs` 没有生产 caller，只在自身测试中做 horizon-to-zenith CPU lerp，并忽略 ground 与 directional sun；它不能作为 display/capture reference。对它和 root wiring 的硬删除因两条路径不在 Shader06 immutable write scope 而从本切片撤回，后续必须通过 audited owner transfer 完成，不能以越界修改伪造收敛。

统一源码此前仍缺 producer identity：`runtime_bake_key` 只返回参数 key，而 capture/downsample WGSL 改动不会自动形成新 runtime-cache generation。现在 capture-local `shader_source_identity` owner 在编译期分别计算完整 capture WGSL（已包含公共 sky module）与 downsample WGSL 的 64-bit FNV-1a content hash，并组合为 128-bit source identity；runtime key 将其与参数 `source_hash` 做四次 `u32` XOR。Frameworks01 当前拥有 `ibl_bake_shader_plan.rs`，所以本切片不跨会话替换该文件原有的 content-hash helper；待其 owner 完成 hard cut 后再评估去重。当前路径没有运行时字符串扫描、heap allocation、binding、dispatch、texture sample、PSO 或 shader permutation 增量。

本次没有把 `intensity` 或 `rotation_radians` 塞进 capture uniform/key。两者是有意的 final-sampling 参数：sky display 与 procedural fallback乘总强度，PMREM/SH9/IEM consumer同样在采样后乘强度；rotation则在 lookup/SH basis方向上统一应用。这样同一 radiance artifact可用于不同曝光强度与朝向，避免无意义重烘。identity/liveness 静态矩阵 `14/14`、Rust formatting 与 scoped diff integrity通过；dead CPU helper 的 hard cut 已明确延期，不登记 zero-hit。尚无 Cargo、WGPU、PNG、RenderDoc、GPU timing、RSS/VRAM 或 WPR energy，本节状态为 `source_closed_pending_managed_validation`。

### 12.15 realtime IBL 连续变化代际饥饿

在采样常量之前复审 scheduler/runtime 调用链发现一个结构性 liveness 缺陷：`prepare_frame` 每帧把最新 sky key 交给 `request_rebake`，而该 API 对不同 key 会丢弃当前 ticket 并从 `CaptureSky { first_face: 0 }` 重启。默认 8 个 PMREM mip、每帧 2 个 capture face 的完整代际需要 `3 capture + 7 source mip + 3 mip0 prefilter + 7 remaining prefilter + 1 SH9 = 21` 个成功 submission；太阳或天空每帧变化时，旧算法完成率严格为 `0/21`，无论运行多久都不会发布新 ready cubemap。

Unreal `ReflectionEnvironmentRealTimeCapture.cpp` 的 `FRealTimeSlicedReflectionCapture` 保持一个 work cubemap 走完 capture/mip/convolution/diffuse-irradiance 状态，终态才交换 `ConvolvedSkyRenderTargetReadyIndex`；变化输入不会破坏正在构建的 cubemap 内部一致性。Zircon 现在同样固定一份 active `ProceduralSkyParams` snapshot，并把期间所有输入覆盖到单个 latest-wins `queued_sky`。当前代际在最多 21 个 accepted slice 后发布，然后立即以最后一次请求启动下一代。请求处理、队列空间和每帧状态更新均为 `O(1)`，没有无界 revision queue、allocation、binding、dispatch、sample、PSO、permutation 或 GPU resource 增量。

回归在每个 slice 前都递增 `source_revision`，要求首代在 64 次上限内发布、published key 仍是首代、pending/active 只保留最后一次 revision。源码 RED 证明 active/queued/transition owner 原先不存在，GREEN 静态矩阵报告 `14/14`，focused `rustfmt --check` 与 scoped diff integrity通过。`21` 是算法完成上界而非实测延迟；仍需在 current-source 可编译基线上记录静态天空与连续太阳动画的 publish interval、stale/retry count、GPU pass p50/p95/p99、frame time、RSS/VRAM 与 WPR/WPA energy，并通过产品 PNG 与 `D:\Tools\renderdoc` 确认 ready/work cubemap 不混面。

### 12.16 realtime IBL 代际进度与合并更新测量合同

§12.15 修复了饥饿，但现有 timing sidecar 只记录每个 slice 的 `frame_number/generation/logical_state/operation`。它可以证明 21 个 operation 被执行，却不能直接区分“默认 21 帧发布”“GPU retry 后晚发布”或“active 期间输入持续变化但被 latest-wins 合并”；如果先凭 wall time 调整 faces-per-frame 或 sample budget，仍会把调度延迟、GPU pass 成本和输入 churn 混成一个瓶颈。

在任何进一步调参前，CPU recording report 必须增加四个同代际字段：`generation_start_frame_number`、`generation_elapsed_frame_count`、`coalesced_source_change_count`、`queued_generation_pending`。active generation 建立时记录下一次提交帧并把合并计数清零；active 期间只有 latest slot 实际换成不同 bake key 才递增计数，同 key 重复请求不计；请求回到 active key 时撤销 queued 标志但保留已观察到的 churn 计数；Published 后若存在 queued snapshot，则下一代从下一 frame 重新计时。默认无 retry 的 terminal SH9 应报告 elapsed=`21`；retry/stale 注入后的 elapsed 可大于 `21`，但同一 generation 的 start 和计数必须单调稳定。

该测量只增加两个 `u64` runtime 字段、一个 `Option<u64>` 与既有 queued-state 的布尔投影；每帧仍为 `O(1)`，不读取额外时钟，不新增 allocation、GPU query、dispatch、sample、binding、PSO、permutation 或 GPU resource。sidecar 同时输出逐 sample 字段，后续 current-source 连续太阳 profile 才能把 publication interval、coalescing rate、retry count与既有 CPU/GPU pass timing 对齐。测试先以缺少 10 个生产/sidecar owner 的 RED 建立合同；当前源码行为矩阵 `18/18`，并锁定连续输入下首代在第 `21` 个 accepted slice 发布、下一代从 frame `22` 重新计时、相同 queued key 不重复累计。focused `rustfmt --check` 与 scoped diff integrity通过。此节状态为 `measurement_source_wired_pending_managed_profile`；不得据此宣称延迟、吞吐或功耗改善。

### 12.17 环境旋转与方向域一致性复审

在继续调整 PMREM、SH9 或程序天空算法前，current-source 方向链已按 producer、scene uniform、skybox display、Forward、Deferred 与 fallback 重新展开。统一约定是 GPU cubemap/PMREM/SH9 lookup 对查询方向应用正角 Y 轴旋转 `R(+theta)`；程序天空不旋转查询方向，而 `ResolvedProceduralSun::direction_for_sampling_rotation` 在 CPU 侧对太阳方向应用逆矩阵 `R(-theta)`。因此 `dot(R(+theta) * direction, sun) == dot(direction, R(-theta) * sun)`，程序太阳与同源 cubemap 的亮斑方向一致。horizon/zenith/ground 只依赖 Y 分量，Y 轴旋转不会改变渐变。realtime capture 使用未旋转的 resolved sun 烘出可复用 radiance artifact，最终 PMREM/SH9 consumer 才应用同一 lookup rotation；`rotation_radians` 不进入 bake identity 是避免朝向变化触发重烘的正确设计，不是 identity 漏项。

Forward Standard PBR、environment-only Forward、Full/Standard-preview/environment-only Deferred 与 fallback mesh 最终都装配 `zr_procedural_sky.wgsl + zr_environment_core.wgsl`，并从核心的 diffuse/specular owner 进入 SH9、irradiance cube、PMREM 或 procedural fallback。skybox source-cubemap display 使用相同正角 lookup matrix；procedural display 使用相同 inverse-rotated scene sun。零旋转通过 `environment_rotation_sin_cos.z < 0.5` 直接返回，非零旋转只读取 CPU 预计算的 sine/cosine，没有 per-pixel `sin`/`cos`、额外 normalization、texture sample、dispatch、PSO 或 permutation。方向复审未发现符号反转、双旋转或 Forward/Deferred/fallback 分叉，因此不授权生产 shader 修改。

仓库已有的 source contract 明确断言 GPU 正角两行和 CPU 逆角两行，但其 `SKYBOX_SETTINGS_SOURCE` 仍指向正在由外部 session hard-cut 的旧单文件 `environment/skybox.rs`，而 current worktree 已删除该文件并拆为 `skybox/` 目录。Shader06 不越界修补该 foreign path，也不把未执行的合同写成通过证据。本节只登记源码代数、装配路径和复杂度复审，状态为 `review_complete_pending_foreign_hard_cut_and_managed_validation`；仍需 hard-cut owner 完成 include 接线后执行合同，并在 current-source 产品 PNG/RenderDoc 中用非对称太阳 HDR/程序太阳分别验证 `0/90/180` 度朝向。

### 12.18 realtime IBL GPU 生命周期与 P1-2 剩余成本

current-source 生命周期盘点确认 GPU texel residency 不随 generation 增长。runtime 首个 procedural frame 才创建两个固定 slot；每个 slot 含一份 128x128、8 mip、6 face、RGBA16F source cube，一份同布局 PMREM cube和 144-byte SH9。单 cube 为 `8 * 6 * sum(128^2..1^2) = 1,048,560 B`，双 slot 的 source+PMREM+SH9 合计 `4,194,528 B`，不含 view/driver metadata。generation 只交换 ready/work slot，不重新分配 texture。HDRI-only renderer 不创建这些资源。

compiled-graph cache 已按 closed scheduler topology 实施 release-mode FIFO capacity；默认 `2 faces/frame` 与 `8 mips` 的容量为 `(3 capture + 7 source-mip + 3 mip0-prefilter + 7 remaining-prefilter + 1 SH9) * 2 slots = 42`。execution-resource cache 使用完全相同的 `(ready slot, work slot, operation)` key，layout变化时清空，因此当前 key space 同样最多42项，但超限保护仍只有 `debug_assert`；只有将来给 key 增加动态字段或改变 scheduler topology 时才需要先补 release eviction。IBL compute pipeline cache 的shader集合为静态 PMREM/SH9/IEM owner，PMREM各 mip共享 pipeline key；realtime PMREM/SH9 binding cache按 slot与 dispatch slice限界，默认上限是 `2 * (8 mips * 3 face batches + 1 SH9) = 50` 个entry，稳定 generation 命中时不再创建params buffer或bind group。

P1-2 尚未完全关闭：capture owner 每代3个 face batch、source-downsample owner每代7个 mip仍各创建一个短生命周期 uniform buffer与bind group，即固定 `10 buffer + 10 bind group/generation`。现有 `RealtimeIblWgpuRecordReport` 已分别输出 capture/source-mip creation count与creation micros，§12.16也能把这些样本与generation start/elapsed/churn对齐。下一步先在E盘current-source profile中分别采集静态首代、稳定无更新和连续太阳动画的 creation micros p50/p95/p99、native allocation、RSS/VRAM high-water、GPU pass、frame time与WPR energy；只有其占比或在途native对象水位证明为瓶颈，才允许在 capture pipeline owner 内选择 persistent per-slot/per-face-batch uniform+bind-group cache、dynamic-offset arena或后端 immediate/push-constant方案。当前不凭对象计数直接实现，状态为 `bounded_lifecycle_review_complete_pending_managed_profile`。

### 12.19 P0-7 probe assignment 结构与复杂度重审

current-source probe prepare 已先按camera layer、baked cubemap与正强度筛选，再按相机到influence的距离选择/排序最多64个active probe；WGSL仍逐fragment遍历全部active probe，重复camera layer gate并执行sphere/box精确权重与top-two排序。Forward的`ZrShadingContext`已有`frag_coord`、`position_ws`和`instance_index`，Deferred也有fullscreen position与重建world position，但当前GBuffer `material.a`只有8-bit flags，其中低7位为shading model、最高位为receive-shadows，没有object reflection mask。因此M9必须版本化scene/material/GBuffer查询合同，不能把camera mask重复读取描述成object级过滤已经完成。

同一片元的provider选择还没有成为可复用结果。基础Standard PBR先经`zr_environment_pbr_components_with_dielectric_f0_and_specular_normal_normalized`选择一次probe；启用clearcoat且planar miss时，`zr_pbr_advanced_environment_normalized`会再次进入`zr_environment_reflection_color_after_planar`并重新扫描同一组probe。selection只依赖world position、layer/mask、shape、priority和blend，不依赖base/coat reflection direction；两个lobe只需以各自direction对同一selection做box projection与cubemap sample。现有workload report的`pixels * active_probe_count`只覆盖一次selector调用，因此64 probes的clearcoat静态上界不是1080p `132,710,400` / 4K `530,841,600`，而是planar miss下的`265,420,800` / `1,061,683,200` candidate visits。该数值仍是上界，不是GPU耗时。

Unreal `ReflectionEnvironmentPixelShader.usf`先用`ComputeLightGridCellIndex`取得`FCulledReflectionCapturesGridHeader`的`NumReflectionCaptures/DataStartIndex`，`ReflectionEnvironmentComposite.ush`再通过`GetCulledLightDataGrid`遍历该cell的capture indices。Zircon现有light grid已经提供perspective/orthographic view映射、保守sphere投影与tile/z-bin相交，可抽取的是influence rasterization算法，不是light专用bitmask ABI或每帧构建生命周期。当前light grid每次调用仍新建`zbins`、`tile_masks`、`zbin_min_max`，随后再生成一份packed upload payload；probe集合通常更稳定，直接复制会把GPU线性扫描换成CPU重复分配、构建和上传。

bitmask预算也否决了直接复用其存储作为1k-probe目标。64 probes需要2 words/tile；在现有`8192` tile-word预算下，1080p会从8 px退化到32 px tile，4K退化到64 px，二者均为`60 * 34 * 2 = 4,080` tile words。`4096` z-bin-word预算在stride=`2 header + 2 mask`时给出1024 bins；加128-byte params的逻辑上传为`32,832 B`。全屏全深度重叠的静态builder上界为`64 * (2,040 tiles + 1,024 bins) = 196,096`次mask OR，尚不含stats与upload copy。1k probes需要32 words/tile，同预算会把1080p/4K分别推到约128/256 px tile，并把z-bin降到`floor(4096 / 34) = 120`；这种粗化会重新扩大局部候选，不能成为M9结构。

M9目标因此保持稀疏、版本化、单一owner：09B先发布probe scene generation与persistent world-space index；per-view assignment按probe/view/residency generation产生cell `offset/count`、packed candidate indices与显式spill pages，稳定scene+view命中时零重建。共享的spatial influence rasterizer可服务light与probe builder，但两者保留独立payload index域和生命周期。group1的16..30范围当前只有binding 19未占用，故候选ABI应优先扩展binding 17的versioned probe header，并以单个binding 19 storage buffer打包cell headers、indices和spill metadata，而不是再要求三条仿light-grid binding。Forward/Deferred/fallback统一接收`ZrEnvironmentProbeQuery { frag_coord, world_position, object_reflection_mask }`，一次取得selection并供base/clearcoat复用；box probe只以保守bounding sphere进入broad phase，精确box/sphere weight、priority、blend、top-two和sky remainder继续由同一narrow phase决定。overflow必须走cell spill或明确last-good/global-environment原因，禁止静默丢probe或回退扫描全部resident probes。

目标fragment复杂度为`O(1 + C_cell)`，不再是`O(P_global)`；在全部probe真实重叠的极端场景，精确top-two至少需要检查未被证明支配的每个候选，因此`O(C_cell)`已经是output-sensitive下界。进一步降低密集场景只能依赖可证明的dominance pruning、authoring/quality overlap budget或分层spill traversal，不能以固定K截断改变视觉语义。per-view build复杂度和存储为`O(P_visible + total_cell_overlaps)`，后续GPU-driven实现必须与确定性CPU reference逐cell等价。

实现前的E盘profile矩阵固定同一binary/source fingerprint、driver、HDR、材质、曝光与相机轨迹，在Forward/Deferred分别测0/1/8/32/64 active probes及clearcoat off/on、planar hit/miss、1080p/4K、稀疏/全重叠；记录selector invocation与candidate visits、environment GPU p50/p95/p99、CPU prepare/sort/build/rebuild、allocation、upload bytes、RSS/VRAM、frame time与WPR/WPA energy。1/64/1k global probe场景验证world index与assignment规模，RenderDoc从`D:\Tools\renderdoc`确认实际loop、cell range、binding 17/19和base/clearcoat selection复用。优化后使用同一矩阵与同机同画质Unreal capture作经验对照，必须证明稳定scene+view零重建、稀疏场景fragment visits不随global P线性增长、dense spill不丢失、Forward/Deferred像素等价；在这些current-source数据前，状态为`architecture_review_complete_pending_managed_profile`，M9与`implementation_status`继续`pending`。

### 12.20 P0-6 cold-probe residency、upload admission与失败闭环重审

current-source的真实上传粒度已经纠正：`append_probe_pmrem_texture_uploads`先把完整PMREM字节复制到一个`Arc<[u8]>`，随后为8个mip各生成一个upload；每个upload的`depth_or_array_layers=6`，所以一个probe是8次texture write而不是旧计划所述48次。单个128x128、8 mip、6 face、RGBA16F payload为`1,048,560 B`。64个冷probe仅texture即为`67,107,840 B`；再加64个96-byte `GpuReflectionProbe`、16-byte header和176-byte planar params，frame resource packet至少为`67,114,176 B`，比RHI默认`64 MiB = 67,108,864 B` staging预算多`5,312 B`，尚未计入该frame任何其他buffer/texture upload。即使只冷启63个probe，总包也为`66,065,520 B`，只给其余frame资源留下`1,043,344 B`。

CPU路径在GPU准入前已经付出成本。`load_texture_asset`经`load_typed`先调用同步`ensure_resident`；cold miss在调用线程执行prepared artifact `.read()`并发布runtime payload，resident hit仍把完整`TextureAsset`克隆一次。upload helper随后再复制到长期留在batch中的`Arc<[u8]>`。64探针准备结束附近，asset manager resident payload、累计upload payload和当前单probe临时clone的源推导逻辑载荷可达`2 * 67,107,840 + 1,048,560 = 135,264,240 B`，未计I/O/decode临时对象、allocator capacity、metadata、其他frame uploads或native staging；该数值不是实测RSS。现有`load_texture_asset_snapshot`虽然能保留exact revision且不克隆整个`TextureAsset`，但仍同步`ensure_resident`，并且`TextureAsset`内部`Vec<u8>`不能直接成为当前`WgpuTextureUpload`要求的`Arc<[u8]>` range owner，因此单改调用点不能闭合问题。

失败路径形成确定性的重复准备循环。resource-batch admission在总payload超过剩余staging预算时拒绝，graphics submission和`commit_pending_uploads`不会发生；下一frame开头的`discard_pending_uploads`只清`Vec<PendingReflectionProbeUpload>`。slot allocator中的`Pending(previous_epoch)`在新epoch不再available，`acquire`又把它改成当前epoch并重新排upload。现有`render_probe_resources_retry_an_uncommitted_cubemap_upload`正面断言discard后下一次prepare再次`asset_load_call_count=1`并再次安排8个mip write。因此64冷probe不是“偶尔超预算后自然分帧”，而是会重复同步load/clone/prepare并继续拒绝整帧；同cubemap的新revision还会在upload接受前覆盖allocator记录的ready revision，现有单revision entry不足以表达last-good与pending-new并存。

结构性目标由09D/RHI owner闭合，而不是提高全局预算或把8次write继续微调：

1. `ResourceStreamer`在render prepare之外异步prefetch并发布exact-revision residency ticket；render线程只查询ready/failed/pending snapshot，不执行artifact I/O/decode。
2. texture bulk payload改为immutable shared/range-addressable owner，residency、validation与upload batch共享同一内容身份，消除`TextureAsset clone -> Arc copy`链；具体bulk实现必须服从09D的通用asset contract。
3. probe scheduler在materialize payload和改写slot状态之前，从统一RHI owner取得带保留headroom的byte/count admission；每frame只stage预算内的K个probe，其余保持有界优先队列，并报告deferred count/bytes、oldest age、retry和rejection reason。
4. slot使用ready generation与pending generation的transactional reservation。只有resource batch和graphics submission被接受后才原子publish新revision；reject/cancel/supersede恢复last-good，不把未ready probe写入active header。没有last-good时显式使用sky fallback。
5. frame不得因probe cold upload超限而整体失败；admission前便截断本frameprobe upload工作，提交ready subset并让剩余工作跨frame推进。dense cold start的完成时间可以受预算限制，但每frameCPU/I/O、staging bytes和pending count必须有硬上界。

Unreal参考只用于确认owner/lifetime形状，不照搬其face/mip锁纹理细节：`UpdateReflectionCaptureContents`把update/load保持在显式队列中并等待shader/texture compile前置条件，`CaptureOrUploadReflectionCapture`再通过render command上传；`OnDataUploadedToGPUFinal`只在最终上传之后释放source map-build data以维持单份CPU owner。其cubemap array还按估算GPU memory上限约束capture size/count。Zircon应采用相同的queue/readiness/final-commit思想，同时服从自身RHI staging budget和异步09D residency合同。

实施前profile必须写入E盘，固定source/binary fingerprint、driver与场景，测1/8/32/63/64 cold probes、warm reuse、同资源revision更新、staging headroom充足/不足和提交失败注入；记录prepare、artifact read/decode、clone/copy allocation、upload admission、8-write flush、deferred age、retry、frame failure、CPU p50/p95/p99、RSS/VRAM high-water、frame time与WPR/WPA energy。优化后同矩阵必须证明render线程无同步artifact read、预算不足仍提交ready frame、每个revision只materialize/排队一次、稳定warm帧零asset load/texture write，且CPU/内存/功耗与同机同规模Unreal经验值接近。当前只有源码与静态字节模型，状态为`architecture_review_complete_pending_residency_owner_and_managed_profile`；P0-6、M4和性能验收均未关闭。

### 12.20A P0-8 explicit scene-capture source and measurement recheck

The current explicit reflection-probe capture path supersedes the original
six-snapshot/CPU-filter description in P0-8. It moves one
`SceneViewportRenderPacket` into a reusable batch, builds one opaque command set,
records six RGBA16F raster faces, generates the HDR source mip chain, and records
the canonical eight PMREM dispatches plus one SH9 dispatch in one graphics
encoder. Direct lights are packed once; six view-specific light grids share one
CPU payload and 18 upload ranges. The current source-scale model is therefore
`O(M + 6K)` for one mesh census plus six command replays, rather than six scene
extractions or CPU RGBA32F face retention. At face 128 the planned GPU work is
six raster passes plus 16 compute dispatches; at face 1024 it is six raster
passes plus 19 compute dispatches. These are source counts, not timings.

The optimization-before-profile contract now exposes 17 successful-capture
profiler counters from values that the recorder/filter already computed: face
passes, command builds, commands per face, opaque/alpha-mask/advanced-PBR
commands, draw/state/bind-skip counts, source-mip/PMREM/SH9 dispatches,
source-mip params-buffer/bind-group creation and its existing binding-creation
microseconds, plus IBL params-buffer/bind-group creation. The publication adds
no new clock, query set, GPU readback, allocation, traversal, or `device.poll`.
An explicit probe target also fails before upload/graphics submission when its
resource revision or `CapturePending` reservation is unavailable; silent
success without a probe-array copy is no longer a valid sample.

The rejection-path order is now explicit. Resource revision resolution is a
read-only admission step and runs before full-provider expansion or capture
target allocation. On an environment-only renderer, this avoids creating the
full provider's static texture-layout floor of `78,292,648 B` for an invalid
target (`67,107,840 B` PMREM cube array plus `11,184,808 B` planar mip chain),
as well as the capture target's `2,162,800 B` at face 128 or `72,351,856 B` at
1024. These numbers exclude buffers, views, alignment, allocator overhead, and
driver residency and are not RSS/VRAM measurements. The mutable slot reservation
remains after the fallible command-building stages until a single rollback guard
can own every intervening error path; moving only that mutation earlier would
weaken the transaction rather than optimize it.

The E-drive profile must correlate those counters with the exact request/cache
identity, face size 128/256/512/1024, empty/1/8/32 direct lights, 0/1K/10K
opaque draws, Fast/Normal/High filtering, warm/cold pipeline state, successful
target publication and injected target-admission failure. Record capture CPU
p50/p95/p99, per-pass GPU timestamps, native buffer/bind-group allocations,
upload bytes, peak RSS/VRAM, RenderDoc event/dispatch ranges and WPR/WPA energy.
Only after a same-source/same-binary run identifies a dominant phase may P0-8
change command replay, grid construction, transient bindings, filter samples or
submission topology. No current-source Cargo/WGPU/RenderDoc/timing/power data
exists yet, so this is `measurement_infrastructure_complete_pending_managed_profile`
and P0-8 performance acceptance remains open.

### 12.20B P0-8 reflection-probe capture last-good physical transaction review

Current source review invalidates the metadata-only rollback assumption in
12.20. `ProbeCubemapSlotAllocator::reserve_for_capture` currently overwrites
the existing entry revision and returns the same physical cube-array slot.
`copy_environment_capture_probe` then records every PMREM mip directly into
that slot before the submission ticket settles. A changed-revision cancel can
mark the entry pending again, but it cannot restore texels already overwritten
by an accepted GPU submission. Therefore a single-slot `CapturePending` state
cannot implement the documented last-good promise, irrespective of Rust-side
revision bookkeeping.

The primary Unreal reference confirms that last-good refresh needs a distinct
physical owner. `ReflectionEnvironmentCapture.cpp` keeps stable regular probe
indices for normal capture/upload, while runtime refresh blending allocates a
reserved cubemap from the array tail and copies the old regular cubemap into it.
`ScenePrivate.h` separates regular and `NumReservedCubemaps` capacity and keeps
the reserved source in `FSmoothBlendEntry`; `ReflectionEnvironment.cpp` copies
and remaps both regular and reserved slots during array resize. Unity HDRP's
`ReflectionProbeTextureCache` is a useful counterexample: its hash-driven cache
updates atlas storage in place and cannot satisfy Zircon's stronger rollback
contract. Fyrox gives each probe an independent render target, which avoids a
shared-array slot transaction but does not provide Zircon's bounded dense-array
layout. The selected design therefore follows Unreal's physical separation,
adapted to Zircon's single active capture transaction rather than its blending
queue.

The MVP hard cut is `64` logical probe entries backed by `65` physical PMREM
cubemap slots. The one extra slot is excluded from normal logical capacity and
rotates with the committed entry: capture reserves the free physical slot,
keeps any ready entry/revision untouched, records all PMREM copies into the
reservation, then atomically publishes the new slot only after the submission
owner commits. Commit returns the former ready slot to the free set; cancel,
supersede, persistence failure, or submission failure returns the reserved slot
and leaves the old entry and texels unchanged. A new target fails closed when
all 64 logical entries are occupied, while an existing target can still refresh
because the physical spare remains available. A second concurrent capture
reservation fails closed. This is sufficient because
`RenderFrameworkState::pending_environment_capture_submission` is one
`Option<EnvironmentCaptureSubmission>` spanning capture and persistence; no
second transaction can legally need another physical landing slot.

At the fixed 128-face RGBA16F/8-mip layout, the physical transaction costs one
additional cubemap chain, exactly `1,048,560 B` of source-derived texture
payload (`6 * 8 B * sum(128^2 .. 1^2)`). Probe metadata remains capped at 64;
there is no extra shader loop, descriptor record, binding, sample, dispatch,
PSO, permutation, CPU bulk clone, or per-capture allocation. This is a fixed
correctness cost, not measured VRAM, RSS, latency, throughput, or power. The
implementation must split logical entry capacity from physical slot count,
track the reserved slot outside the ready LRU, and regression-test changed- and
same-revision cancel, atomic commit/slot rotation, full logical capacity, and
single-reservation admission before any optimization claim.

The managed E-drive acceptance matrix remains unchanged: WGPU/DX12 must inject
success, cancel, supersede, persistence failure, graphics-submit failure, and
changed-revision refresh while sampling the prior probe; RenderDoc from
`D:\Tools\renderdoc` must confirm capture copies target the non-visible spare
until commit and that the next refresh rotates to the former slot. Record actual
VRAM, capture GPU/CPU p50/p95/p99, RSS, frame time, and WPR/WPA energy against
the same source/binary fingerprint. Until those runs exist, the status is
`implementation_complete_pending_managed_gpu_profile`. The allocator now keeps
capture ownership outside the ready LRU, rotates one physical spare on commit,
keeps changed-revision last-good visible during refresh, and protects that entry
from ordinary allocation pressure. Focused production-source `rustc --test`
reports `12/12`; static texture descriptor/view, stale-state, formatting, diff,
and owner-size gates pass, with `resources.rs` reduced from 941 to 791 lines by
separating capacity, selection, and report owners. This evidence authorizes only
the transaction correction; it is not WGPU/DX12, measured VRAM, timing, power,
or a performance-gain claim.

### 12.21 P0-5 environment extract 与 runtime-cache hydration 提交边界重审

当前 `build_frame_submission_context_from_source` 在拿到 `RenderFrameworkState` 后仍直接调用 `resolve_and_rehydrate_environment_ibl_cache`。cache hit 可以从容量为4的 `EnvironmentIblHydrationCache` 复用已准备的 `SourceCubemapEnvironment`，其大块 texel/row payload 通过 `Arc` clone 保持低复制；但首个 miss 仍在提交构建线程内执行 `IblBakeArtifactCacheStore::read_runtime_cache` 的 `fs::read`、checksum/header/payload decode，再把 PMREM RGBA16F 重新解码成 `Vec<[Real;4]>`，随后构造 source/PMREM/irradiance upload rows。cache 中的 pending bake 只抑制重复 graph reservation，并不抑制每帧对磁盘 cache 的探测：pending request 在进入 `begin_runtime_bake` 之前仍经过完整 `read_runtime_cache`。

以常用 1024 source face、11 source mips、128 PMREM face、8 PMREM mips为例，单个 hydration entry 的核心逻辑载荷约为：source f32 texels `134,217,696 B`、PMREM f32 texels `2,097,120 B`、padded source upload rows `67,140,096 B`、padded PMREM rows `1,079,808 B`和默认 1x1x6 irradiance upload `1,536 B`，合计 `204,536,256 B`，另有固定 SH9/metadata。仅按4条目数截断即可保留约 `818,145,024 B`（约780.24 MiB）的这部分CPU logical payload；这不是RSS、VRAM或驱动分配测量，且不含 cache blob、decode scratch、allocator capacity、bind resources和其它资源类别。换言之，“bounded hydration cache”目前只有 entry-count bound，没有统一 byte/residency/fence budget。

runtime cache miss 还存在短时多份载荷：`.zribl` 文件的 PMREM RGBA16F + SH9 + 120-byte header + 32-byte checksum约 `1,048,856 B`；`fs::read`返回的编码 Vec 在校验和 `IblBakeArtifactPayload::decode` 的 `bytes.to_vec()` 期间重叠，之后 `decode_pmrem_texels` 又创建约 `2,097,120 B` 的 f32 PMREM，直到 hydrated environment 完成才释放中间 blob。source environment本身的 source f32链以及 `with_prepared_upload_artifact` 新建的 padded rows仍同时存活。所有这些工作都发生在 frame submission context，而不是后台 I/O/decode owner。

环境 cubemap GPU侧已有 `committed`/`pending` upload key，但 `SceneEnvironmentCubemap::ensure_uploaded` 在 staging encode成功后即替换 source/specular/irradiance texture和view，再等待外层 graphics submission；`discard_pending_upload`只清 key，不恢复旧物理资源。提交失败或后续 upload admission拒绝时，逻辑上可能只剩尚未写入的 replacement texture，无法保证 last-good GPU generation继续可采样。正确的 owner边界应让新 texture、bind views、staging reservation和upload key组成同一 submission ticket，只有 queue/graphics submission accepted 后 publish，失败/cancel/supersede则保留旧 generation并将新对象交给09A fence retirement。

P0-5的结构性修复必须落在09D/09A共同owner，不是把 hydration cache容量调大或简单把 `fs::read` 包进另一个函数：

1. `RenderAssetResidency`/`ResourceStreamer`以 typed asset handle、request/demand/device generation和异步 I/O/decode ticket发布 exact-revision immutable bulk；render submission只读取 ready snapshot或明确 pending/failed状态。
2. cache resolution拆成非阻塞 `probe/request/publish` 三阶段。cache miss只提交一次后台 read/decode，pending request不得重复磁盘读；命中结果发布到按 bytes、priority、last-use和fence受控的 shared hydration store。
3. runtime artifact payload、source f32 texels与padded upload rows应以可切片共享bulk或range owner连接，避免 encoded Vec、payload Vec、decoded Vec和upload Vec无界重叠；保留header/descriptor常驻，按实际 subresource demand读取。
4. environment cubemap replacement使用双代 `last_good + pending` resource ticket；提交接受前不替换绑定可见对象，接受后原子切换，旧对象延迟到fence retirement。cache eviction只驱逐未被ticket/scene/view pin的CPU/GPU代。
5. 预算由统一RHI UploadQueue提供 compressed/read/decode/staging/destination/in-flight峰值 reservation；当容量不足时提交旧代/sky fallback并推进有限重试，而不是让整个 frame 因 cache hydration 或 environment upload 失败。

E盘 profile必须固定同一 source/binary fingerprint、driver、HDR和相机，覆盖 warm hit、cold miss、stale/rejected cache、同一 pending request连续60帧、4条目LRU、1/2/4/5个1024环境、submission failure、device generation变化和 eviction/fence；记录 `fs::read`/decode/rehydrate micros、bytes in each owner、cache hit/miss/pending probe count、frame prepare/submit p50/p95/p99、allocation/RSS/VRAM high-water、GPU upload/first-visible latency、frame drops与WPR/WPA energy。修复后必须证明 pending miss不重复I/O、render线程I/O为0、cache按byte和fence受控、失败不丢last-good且稳定warm帧不重建/upload。当前只有源码审查和静态模型；精确物理资源/bind-group rollback 缺陷与验收合同已交给 [`Render11 environment cubemap rebind last-good transaction`](../../zircon_runtime/render/11/failure-2026-08-31-environment-cubemap-rebind-last-good-transaction.md)。状态为`architecture_review_complete_render11_rebind_failure_open_pending_residency_owner_and_managed_profile`；P0-5/M4以及性能验收仍未关闭。

### 12.22 P0-8 capture terminal publication transaction review

当前成功路径的 scheduler terminal status、source payload、probe physical slot 与
filtered residency 曾属于两个分离临界区。控制面可在 `Succeeded` 后、物理发布前读取
状态；把物理发布简单前移又会允许 concurrent cancel/supersede 发布 stale generation。
结构修复把 scheduler 设为唯一 publication gate：在持有 scheduler mutex 时先验证
active generation、进度、terminal intent 与 persistence mailbox，再以固定
`scheduler -> renderer state` 锁序执行一次 `Publish/Discard`，最后才暴露 terminal
status/source payload。取消/替代只执行 `Discard` 并返还 spare，绝不覆盖 last-good。
transient source/depth scratch 的 move/drop 在取得 scheduler 锁前完成，临界区只保留
常数级 probe commit/cancel 与有界 residency map publication。

该 hard cut 不新增 draw、pass、dispatch、sample、PSO、permutation、queue、readback、
allocation 或 `device.poll`；仍只有一个 pending 与一个 active capture。它同时拆分了
过大的 scheduler 文件，使控制面与 completion 协议分别可审查。容量模型也按 canonical
SH9 `9 * vec4<f32> = 144 B` 修正：face-128 peak 为 `2,162,800 B`，face-1024
为 `72,351,856 B`，resident PMREM+SH9 为 `1,048,704 B`，64 项上界
`67,117,056 B`。这些仍是 source-derived bytes，不是 VRAM/RSS 实测。状态为
`atomic_publication_implemented_static_contract_passed_pending_managed_gpu_profile`；E 盘
最小 `rustc --test` harness 对真实 scheduler/control-plane/completion 报告 `16/16`，
其中包含 runtime-only success 保留未消费 persistence mailbox 的回归，但不构成
crate/WGPU compile ticket。
受管 WGPU/DX12、RenderDoc、timing、memory 与 WPR/WPA energy gate 保持开放。

### 12.23 P0-8 capture view-neutral scene source and LOD review

当前 capture 的 scene source 不是 authoritative render scene，而是已经按 caller viewport
相机完成 LOD 投影的 `SceneViewportRenderPacket`。其中 `RenderMeshSnapshot` 只保留最终
model/mesh/material/primitive handles；capture batch 随后仅替换六个 face camera，故
viewport 与 probe 距离不同时会渲染错误 LOD，且不同 LOD 的 primitive/material 绑定也可能
一起错误。移除 `mesh_lod` 标记不改变资源句柄，不能作为修复。

UE `CaptureSceneIntoScratchCubemap` 为 capture position 创建 reflection-capture view，
再以该 view 的 `LODDistanceFactor` 进入 visibility/LOD。Zircon 已有同类基础 owner：
world-owned `RenderComponentChangeArtifact` 保存 base/all-LOD immutable payload，
`RenderSceneComponentProjector` 将其投影为持久 `RenderSceneMeshSource`；但它尚未调度进
`SceneRenderer`。因此本项不允许在旧 packet 上增加 all-LOD 副本或新建 capture-only
cache，而是依赖 Render03 接入唯一 persistent scene/residency generation。

接入后 capture admission 对 `M` 个候选 primitive 在 probe origin 各做一次有序 threshold
选择，资源准备/command build 一次，六面只 replay；目标复杂度
`O(M log L + K + 6)`，不接受 `O(6M)` 全提取或六次 asset/material resolution。旧
viewport-packet API 必须同提交硬删除，generation/residency mismatch 在录制前 typed
fail-closed。稳定 scene generation 要证明 source projection=0、all-LOD clone bytes=0、
redundant admission=0。

动态验收固定 E 盘 source/binary/driver 指纹，使用 viewport/probe 距离分离、至少三档
LOD 且每档不同 primitive/material 的场景；记录 candidate visits、LOD selections、selected
bindings、command builds、CPU allocation/bytes 和 p50/p95/p99、GPU timestamps、VRAM/RSS、
WPR/WPA energy，并以 `D:\Tools\renderdoc` 确认六面全部消费 probe-origin selection。
当前只有源码与 Unreal 对照审阅，未实施且无耗时/功耗结论；状态为
`architecture_review_complete_blocked_on_render03_persistent_scene_wiring`，不阻塞其它
shader/IBL MVP 非验收任务。

### 12.24 P0-8 cold procedural capture specular fail-closed

首次 realtime IBL 发布前，capture full-roughness policy 仍会进入 raw procedural 的
perfect-reflection fallback；该路径没有 PMREM，roughness=1 不产生卷积，可能把清晰太阳
specular 烘进 probe。唯一 environment core 已硬切为：ready source/realtime PMREM 正常
采样；无 PMREM 且 capture policy 开启时 specular 返回 0；普通 viewport 继续保留原来的
reflected-direction fallback。diffuse sky、ambient、direct、emissive 与 sky background 不变。

该修复不增加 ABI、binding、feature/permutation、PSO、texture、upload 或 CPU work；
capture-only cold path 源码层少做一次 procedural radiance，并由既有 zero-reflection gate
跳过 environment BRDF lookup，但尚无 GPU timing/功耗结论。assembled Forward contract
锁定三段顺序，静态 RED/GREEN、rustfmt、diff check 通过。状态为
`implemented_static_contract_passed_pending_managed_gpu_profile`。

### 12.25 P0-8 capture shadow product ownership review

capture 当前对 direct lights 构建六个 light grid，却禁止 shadow command 并给 receiver
传入 `None` shadow atlas，实际得到 unshadowed direct lighting。UE capture 没有关闭 Game
shadow show flag。复用 caller viewport atlas 会错误继承 camera-dependent directional
cascades；按六 face 重建 generic shadow plan 又会把 caster/command 工作扩张为 `O(6M)`。

目标结构是 generation-qualified capture lighting product：point/spot/static light-owned
shadow 可在 scene/light/caster generation 一致时复用；directional 采用一次有界 probe-volume
coverage；六个 face receiver 绑定同一已验证 product。missing/stale 状态 typed fail-closed，
不得把声明有阴影的 light 静默变成 unshadowed。验收记录 caster visits、shadow views、
commands、atlas bytes、reuse hits、CPU/GPU p50/p95/p99、VRAM/RSS、WPR/WPA，并由 RenderDoc
确认六面绑定相同 capture generation。当前仅架构审阅，状态为
`architecture_review_complete_pending_capture_shadow_product_owner`。

### 12.26 Standard-PBR direct diffuse owner convergence

优化前复审覆盖 Forward Basic/Advanced、Deferred、fallback、environment、lightmap、
transmission、clearcoat 及共享 isotropic/anisotropic GGX。现状不是单纯 ALU 细节，而是
owner 分裂：间接光使用 material-owned `base_color * (1 - metallic)`，四条 direct path
却再逐灯施加 `1 - F(VoH)`。这既破坏 source-independent material decomposition，也让
light-grid 边界预计算的 diffuse BRDF 在每个 light consumer 内重新取得另一套语义。

按 Unreal Default Lit 主标准完成硬切：prepared metallic Lambert 项由所有 direct/indirect
consumer 复用；Fresnel 留在 GGX specular，以及 transmission/clearcoat 各自的 layer
complement。共享 GGX 直接返回 `vec3` specular BRDF，旧 component struct 与两层 wrapper
删除，不保留兼容入口。Bevy direct-light composition 作为交叉参考，不覆盖 Unreal 主裁决。

算法规模仍为 `O(L)`，其中 `L` 是访问的 light 数；metallic clamp、base-color scale 与
`/PI` 仍在循环外一次准备。本次没有新增 sample、binding、branch、loop、ABI、PSO 或
permutation。删除 wrapper/额外 Fresnel result 只能登记为 source/IR simplification，不能
登记耗时收益。静态合同已证明 production WGSL 中 legacy component API 为零且六条新合同
存在；受管 Naga/WGPU 后还必须在同一场景/二进制/驱动下记录 GPU p50/p95/p99、shader
instruction/occupancy、RenderDoc composition、RSS/VRAM 与 WPR/WPA energy，确认没有新的
bottleneck 后才能关闭性能门。当前状态为
`implementation_complete_static_green_pending_managed_gpu_profile`。

### 12.27 Realtime IBL bounded failure and backoff

结构复审覆盖 snapshot freeze、time-slice scheduler、graph recorder、submission completion、
ready/work publication 与 framework diagnostics。旧实现把 recording/submission 失败压成同一个
布尔 `Retry`，既无原因、次数、退避和 terminal receipt，也不会释放同一 generation；永久失败
会以无界次数持续占用 job slot。这是整体生命周期缺陷，不是 shader 指令级瓶颈。

当前 scheduler 成为唯一失败策略 owner：每个 logical stage 最多三次连续失败，第一次后跳过
一个完整帧、第二次后跳过两个完整帧、第三次终止；任何成功 slice 都重置该 stage 预算。
terminal key 在 source identity 改变前禁止重启，published key/ready slot 保持 last-good；runtime
同时清空 active 与 latest-wins queued snapshot。typed report 区分 `Recording` / `Submission`，并
携带 key、generation、state/substep、operation、attempt、next eligible frame、terminal 与
last-good availability，通过现有 renderer/framework facade 查询。frame eligibility 使用半区间
wrapping sequence 比较，`u64::MAX` 失败后正确跳过 frame 0 并在 frame 1 恢复。

成功热路径仍为 `O(1)` scheduler 状态且不改变 21-slice 默认工作量；永久失败从无界重试收敛
为每 stage `O(1)` 的三次尝试。新增工作只有常量计数、比较和报告 copy，不增加 GPU dispatch、
binding、sample、allocation、PSO、permutation 或资源容量。E 盘独立 `rustc -O` harness 直接
编译 production scheduler 并验证六态 readiness、publication、1/2-frame backoff、terminal
last-good、failed-key 抑制、published-key 恢复、新 key 恢复、基于每次尝试 frame identity 的
delayed-completion rejection 和 frame-counter rollover；
source/binary SHA-256 为
`C0BF937A1F9A51FDB031834515B46821DABBA9E0F830683281BB3764015A6EA3` /
`1A015F79BC5731A716E82291687415F1001C88F3280A90FC9B592FC691E22718`。这不是耗时、功耗或
GPU throughput 数据；managed Cargo/WGPU fault injection、公开诊断产品证据、PNG、RenderDoc、
RSS/VRAM 与 WPR/WPA 仍开放。当前状态为
`implementation_complete_static_and_isolated_behavior_green_pending_managed_fault_product_validation`。
独立 follow-up review 在 attempt-frame 修复后报告 `Critical 0 / Important 0`，确认旧 completion
只能返回 `Stale`，不能清除或推进同 logical stage/substep 的新 retry。

### 12.28 Realtime IBL readiness product snapshot

failure-only facade 在产品边界复审中被拒绝：它只能回答“最近是否失败”，无法区分尚未请求、
首次 bake、ready、last-good refresh、无 last-good terminal 和保留 last-good terminal。最终硬切
由 `RealtimeIblRuntime` 一次性投影无分配 `RealtimeIblStatusReport`，包含六态 readiness、
current/start frame、published/pending/queued key、coalesced source-change count 与 optional typed
failure。`SceneRenderer` 只转发该 runtime truth；`WgpuRenderFramework` 先完成 pending submission，
再在既有 operation/state lock 顺序内读取。旧 failure-only runtime/renderer/framework getter 全部
删除，不保留两个可能分叉的诊断入口。

snapshot 构造是固定字段复制和常量分支，时间/空间均为 `O(1)`，不读取 GPU、不分配、不新增
lock、binding、sample、dispatch、PSO 或 permutation。源码行为合同覆盖 fallback、baking、ready、
refreshing-last-good、failed-fallback、failed-last-good 与新 key recovery；framework source contract
锁定 finish-submission 先于 renderer access。managed Cargo/WGPU 尚未执行，因此这些行为测试仍待
受管运行；cache/import/probe/capture 也尚未合并到同一全域 environment status，P1-19 只标为
Partial。当前状态为
`implementation_complete_static_pending_managed_status_product_validation`。

### 12.29 Realtime IBL last-good freshness owner

对六态 status、runtime publication、frame counter 与 Unreal `USkyLightComponent` 的
`CaptureStatus + SecondsSinceLastCapture` 复审后，剩余产品缺口不是另一个状态枚举，而是
freshness 没有 owner。Zircon 的 `RefreshingLastGood` / `FailedLastGood` 只返回 published key；
Editor、telemetry 或自动恢复策略若各自记时，会形成互相漂移的第二套状态机。active start frame
也已经公开，但 inclusive elapsed 仍需每个 consumer 自行处理 generation 尚未进入首帧和 `u64`
rollover 两个边界。

目标 hard cut 由 `RealtimeIblRuntime` 在成功 publication 时记录唯一 published frame，并在现有
`RealtimeIblStatusReport` 中投影 published frame、last-good age 与 active inclusive elapsed。
所有距离使用 scheduler 同一半区间 wrapping frame-sequence 语义；尚未开始首个 active frame 时
elapsed 为 `0`，publication 当帧 age 为 `0`，rollover 不产生巨大伪年龄。terminal failure、retry
与 published-key restoration 必须保留 last-good publication frame，新 generation 成功才替换。
该基础设施只增加固定 `Option<u64>` 状态与常量整数运算，不读取 wall clock/GPU、不分配、不新增
lock、binding、sample、dispatch、PSO 或 permutation。production 行为回归覆盖六态、terminal
last-good、published-key restoration、rollover 与 successor publication replacement；受管 Cargo 尚未
执行。E 盘 harness 已直接验证共享 half-range age helper 与 rollover，source/binary hashes 见 12.27。
状态为 `implementation_complete_static_and_isolated_sequence_green_pending_managed_status_product_validation`；
这不是性能、功耗或延迟收益声明。
独立 follow-up review 报告 `Critical 0 / Important 0 / Minor 0`，确认 pre-first-slice、
publication-frame、rollover、terminal/restoration 与 successor replacement 合同均成立。

### 12.30 P1-7 BRDF LUT cold-start production profile and versioned builtin design

本轮在优化前重新审阅 `environment_brdf_lut.rs -> SystemTextureGenerationOwner::acquire ->
SystemTextureResources::prepare -> SceneEnvironmentBrdfLut::from_system_textures` 的完整路径。
GPU ownership 已集中为每个 WGPU device generation 一个 immutable system-texture lease，scene
侧只 clone texture/view handle；但首次 `acquire` 持有 publication mutex，同步进入 process-wide
`OnceLock::get_or_init`，完成 `128 * 32 * 128 = 524,288` 次 GGX/Hammersley sample、
RG16F encode、resource prepare 和 10-texture upload submission 后才发布 lease。`OnceLock` 只让
同一进程后续 generation 复用 CPU bytes，不能消除首个 renderer 的同步停顿。

E 盘 `production_brdf_lut_profile.rs` 通过 `include!` 直接编译当前 production integrator，使用
counting global allocator，并由 31 个独立进程各执行一次首次 build。环境为 Windows x64、
AMD Ryzen 7 5800H (8C/16T)、`rustc 1.94.1 -O -C codegen-units=1 -C target-cpu=native`；未调用
Cargo。production integrator/source-owner SHA-256 分别为
`0DBD5D51E783D60D74C1FCE3E50588E02C1B6061A26551BBC3A75CE8D9351C5F` 与
`761BF0117B0E775BE012F2A966586CE5C3E3E56A97B5ED191D9F727624C80349`。结果如下：

- wall time min/p50/p95/p99/max：`20.967/22.086/36.618/40.505/40.505 ms`，mean
  `23.888 ms`；
- 每进程固定 `524,288` sample iterations、4,096 texel、1 次 `32,768 B` allocation，checksum
  `3171.452236790`；
- harness/source、binary、raw runs、summary SHA-256 分别为
  `B0D79C5B7449BC4AEA00F595ACA878D84C7CF207718457B240B82B4BDA9E7068`、
  `2FF4A14E5FA746C32A8E8E74D44CAA558A9F21188976ABA087F7E735A206C3DB`、
  `FB4D20B609BD588F5BC3B126D8FC21381223F103E36C3D4ABCF1EFF5616EF1F2`、
  `B008577E73DB230890D30721FD632B3EAE4DC9B42228EA2072A3841F956492F8`，artifact 位于
  `E:/zircon-profiles/shader06-brdf-lut-startup-20260831/`。

该基线只测 production CPU integrator；未包含 half encode、WGPU texture creation/upload、完整
renderer startup、GPU、RSS/VRAM 或功耗，因此不能用来宣称总启动耗时或 energy 改善。它已经
足以证明 20--40 ms 的同步 CPU 积分是首 lease publication 前的结构性热点。WPR/WPA 可用，但
必须留到相同 product binary、startup interval 和 DX12 workload 的 before/after capture，避免用
20 ms isolated harness 的采样噪声伪造功耗结论。

主参考 Unreal `SystemTextures.cpp` 使用同一 128x32 domain、128 samples、RG16-ish filterable
format 和 system-texture lifetime，`BRDF.ush` 的 `EnvBRDF`/clearcoat consumers 固定 RG scale/bias
合同。它同样在 system texture initialization 中积分，因此只裁决算法、extent、consumer 与唯一
owner，不代表其同步循环满足 Zircon MVP startup budget。Unreal 的 `EnvBRDFApproxLazarov` 是
不同数值近似；在 base/clearcoat/SSR 未做统一误差与图像门前拒绝替换。cmft/cmftStudio 搜索未
发现 PreIntegratedGF/split-sum LUT publication；它们仍只作为 cubemap/filter tool reference，
不得引入第二套 runtime owner。

选定 hard cut 是 versioned builtin artifact，而不是微优化积分循环或后台 race：

1. neutral environment contract 声明唯一 BRDF LUT recipe identity，至少包含 algorithm version、
   `128x32` extent、128 samples、RG16F format、16,384-byte payload length 与 content hash；recipe
   变更与 artifact bytes 必须同提交。
2. build/certification 工具继续调用当前高精度 production integrator 和 canonical half encoder，
   输出 checked-in immutable RG16F payload。测试重新生成并 byte-for-byte 比较，同时保留 Unreal
   anchors、finite/nonnegative 与 high-sample error gate。
3. product `SystemTextureGenerationOwner` 只 materialize builtin bytes 并沿现有单一 native texture
   upload ticket 发布；不得在 release path 调用 integrator，也不得在 artifact mismatch 时退回同步
   rebuild。编译期长度和测试期 hash/bytes mismatch 直接 fail closed。
4. device generation、drop order、10-upload batch、16,768-byte aggregate upload 与 scene lease
   consumer 保持不变。builtin CPU payload 与设备无关，可跨 device recreation 复用；GPU resource
   仍按 generation 重建，不创建全局 WGPU owner。
5. startup diagnostics 同步删除含混的 runtime `cache_built/payload_build` 语义，明确报告 builtin
   materialized、cache wait、materialization 时间与 upload submission。旧动态 build compatibility
   path 和旧 ready-frame 字段不保留 alias；v16 只允许显式 legacy 读取。

source hard cut 已按该设计落地。`SystemTextureGenerationOwner` 的 production 段只调用
`builtin_environment_brdf_lut_rg16float_bytes`，不再引用 integrator、half encoder 或 runtime
sample count；测试段保留 canonical generator/encoder 并逐 byte 比较。checked-in artifact 的
algorithm version 为 `2026_08_31_0001`，extent `128x32`、128 samples、`Rg16Float`、长度
`16,384 B`，SHA-256
`406956356B136BD079CDCCE8DCB86F9E20D596681F7457AD38D91A7EE472674D`；两次独立生成与当前
仓内文件一致。GPU device-generation owner、10-upload batch、16,768-byte aggregate upload 与
single-ticket publication 没有改变。

runtime report、viewer startup、ready-frame writer/validator、profile summarizer 与 visual oracle
已原子 hard cut 到 `builtin_payload_materialized` / `builtin_payload_cache_wait` /
`builtin_payload_materialization`。ready-frame schema 从 v16 升到 v17，default validation/profile
只接受 v17；v16 保留旧字段且仅允许显式 legacy read。全工作区 Rust 旧 build/built 符号检索为
0；精确 rustfmt/check、Python compile 和 evidence/summary/visual-oracle `63/63` 回归通过
（`220.033 s`，工作区 E 盘临时根）。

对已经接入 production artifact 的 exact `Arc::from(&BUILTIN[..])` 路径重新执行 31 个独立冷进程，
min/p50/p95/p99/max 为 `26.0/31.1/63.9/188.0/188.0 us`，mean `37.852 us`；每次固定
1 allocation、allocator charge `16,400 B`、payload `16,384 B`、checksum
`6197230336633806570`。相对 before integrator p50 `22.086 ms` 的 p50 ratio 为 `710.16x`。
算法规模已从 `O(W * H * S)` 的 524,288 次 GGX/Hammersley iteration 收敛为
`O(16,384 B)` materialization，加既有固定上传；shader ABI、texture sample、binding、pass、draw、
dispatch、PSO、permutation 与 LUT 内容均不变。

该 after 数据仍是 current production payload 路径的 isolated CPU 证据，不包含 full renderer、
WGPU texture create/upload、GPU、RSS/VRAM 或功耗。状态因此推进为
`implementation_complete_static_and_tooling_green_pending_managed_rust_wgpu_product_timing_power_validation`，
而不是性能终验完成。仍需 coordinator managed Rust tests；product startup report 证明同一 ticket/
upload count；current-source HDRI screenshot 与 `D:\Tools\renderdoc` replay；同一 binary/interval 的
WPR/WPA before/after。只有这些证明 GPU/upload 瓶颈、画质与 energy 预算同时成立后，P1-7 才能
关闭并进入 milestone commit。

### 12.31 P1-3/P1-4/P1-5 current-source disposition

三条早期 P1 finding 已不能原样指导实现。P1-3 所述“首次 update 全塞一帧”已被默认
`capture_faces_per_frame=2` 的 scheduler 替代；当前无 retry generation 按 3 个 capture batch、
7 个 source-mip slice、8 个 PMREM face slice 和 3 个 SH9 slice，在第 21 个 accepted frame
publication。这个计数是状态机 work bound，不是 GPU elapsed 数据；首个资源/PSO 初始化仍需
managed timing 独立确认。

P1-4 所述“静态 gradient 自动 realtime”也需区分必要首代与重复更新。runtime key 包含 source
revision、horizon/zenith/ground、有效 directional sun 与 capture shader content identity，但有意
排除只在最终 sampling 生效的 intensity/rotation。published key 相同且无 ticket 时
`request_rebake` 返回 false；active 期间输入变化 latest-wins coalesce，同 key 重复不累计。因此
MVP 已是确定性 OnChange，不存在静态 sky EveryFrame rebake。一次首代 generation 是当前
procedural source 得到 roughness PMREM/SH9 的正确性成本；在 versioned cooked procedural artifact
owner 出现前，不应为消除它而恢复 raw fallback。Static/threshold/minimum interval/importance/
EveryFrame policy 属于 cook/editor 产品层后续，不扩张当前 MVP ABI。

P1-5 的多公式 owner 已硬切为一个 WGSL primitive：viewport display/fallback 从
`zr_environment_core.wgsl` 调用 `zr_procedural_sky_radiance`，realtime capture 通过 `concat!`
拼接同一个 `zr_procedural_sky.wgsl`，wrapper 只负责 cube direction 与 storage write；capture
shader identity 同时进入 bake key。当前检索只发现该函数的一份定义。

据此 P1-3、P1-4、P1-5 的 MVP source findings 标为 closed/reclassified；不新增 source 变更，也
不宣称 GPU timing、startup、画质或功耗通过。managed Naga/WGPU、current-source PNG、
`D:\Tools\renderdoc` 与同区间 timing/WPR gates 仍开放。

### 12.32 P1-1 general IBL command-plan production-source profile

本轮先完成整体 owner 和调用规模复审，再做优化决定。通用 runtime artifact path 的 executor 从
current frame environment 与 compiled graph resource metadata 重建 `IblBakeArtifactRequest`，随后每个
pass 都调用 `record_ibl_bake_wgpu_pass_for_request`。后者每次生成完整
`IblBakeWgpuCommandPlanSet`，再按 pass name/kernel kind 线性查找一条 command。完整
`PMREM_SH9_IEM` 请求有 8 个 PMREM、1 个 SH9、1 个 IEM pass；每份 plan 有 10 个 command 和
55 个 readback copy，所以一次 bake 的 command construction 是 `P * P = 100` 个 command build，
不是 10 个。realtime path 已按 topology 缓存 compiled graph variant，并为 slice 构建 exact runtime
kernel；它没有复用这条 full-plan path，也不应成为第二个通用 cache owner。

隔离基准完全位于
`E:/zircon-profiles/shader06-ibl-command-plan-20260831/`，没有 Cargo 和 C 盘产物。harness 直接
`#[path]` 编译当前 production invocation `compiler.rs`、`ibl_bake_shader_plan.rs` 和
`ibl_bake_wgpu_command_plan.rs`；profile copy 与原文件只差一个 `realtime_slice` absolute path，原始
command-plan SHA-256 为
`A0F6FF5AA9F6FEA8E7065D63DC156923CAC6528DBEA29DAF55D4AE0CC6F507F0`。固定 512/10 source、
128/8 PMREM、`PMREM_SH9_IEM` 请求下，单 full-plan 固定 396 allocations/41,570 B；当前十 pass
执行为 3,960 allocations/415,700 B；一次 build 加十次 lookup 为 396 allocations/41,570 B。

31 个独立进程各执行 128 次 scenario。current ten-pass min/p50/p95/p99/max 为
1,370,903.1/1,716,197.7/2,848,416.4/3,053,585.2/3,053,585.2 ns，mean 1,801,517.9 ns；
one-build/ten-lookup candidate 为
98,631.2/141,324.2/472,246.9/481,593.8/481,593.8 ns，mean 165,907.5 ns；p50 ratio
为 12.14x。harness source/exe/raw/summary SHA-256 分别为
`26C20761BC3193103BF84D7730CD55C837ECCAA5A9C2A5B741D1D762CD3D336F`、
`89D3623B2725CC6C27EAAFA9FA03D479DBF30E816DFD8BAD2AF71B6A7567D473`、
`8927A58BE988D52090194E7B8819B255EABBAAFD1989312E7FC9304E5BBC4A43`、
`A52D523851EC55A6D479658D82B45E3E627585F1E18AF4413700565A648258A0`。依赖 rlib 是现有 debug
artifact，时间只作为 isolated production-source CPU 结构证据；allocation/bytes 和 exact call count
才是当前算法规模的硬证据，不能推导产品帧、GPU 或功耗收益。

Unreal 主参考确认 pass payload owner 位于 graph authoring：`RenderGraphUtils.h:540-564` 的
`FComputeShaderUtils::AddPass` 接收当前 shader/parameters/group count 并把它们捕获进单个 RDG pass；
`ReflectionEnvironmentCapture.cpp:598-613` 按 mip/face 分配当前 pass parameters。Zircon 的长期结构也应
让 compiled pass 持有 immutable payload。但当前 `RenderGraphComputePassMetadata` 只覆盖 shader source、
entry point 和 binding schema，不能无损表达 IBL uniform words、storage-view mip/layer 和 readback copy；
直接把 IBL 塞入 generic compute executor 会丢失 ABI，而扩展全 render-graph payload 是跨模块高级改造。

因此 MVP hard cut 确定为 exact-pass construction：dispatcher 先解析当前 pass kind，只构建对应
kernel/command（必须包含当前 artifact readback），不再生成其它九条；总复杂度从 `O(P^2)` 收敛到
`O(P)`，不新增 cache、hash lookup、PSO/permutation、GPU dispatch 或 compatibility fallback。实现后的
RED/GREEN 必须固定 10-pass 只构建 10 个 command、readback offset/length 与旧 full plan 完全一致、
PMREM mip0..7/SH9/IEM 各一次、missing/ambiguous pass fail closed，并重复上述 31-process allocation/time
profile。2026-08-31 已完成 exact-pass hard cut。dispatcher 先严格解析 canonical `(executor_id, pass_name)`，再调用
`ibl_bake_wgpu_command_plan_for_kind` 只构建当前 requested kernel；未请求、越界、非规范或 executor/pass 不匹配均
fail closed。完整 `PMREM_SH9_IEM` 的 10 个 pass 仍产生 10 个 command 和 55 个 readback copy，逐项
readback offset/length 与 full plan 相同；realtime slice/runtime-kernel 路径未复用或改变。四文件 atomic
dependency set 已取得 Shader06 coordinator lease，最终源码哈希为：command plan
`22B8D474D3F97B960292D935A861AFCA35C438C815C53BA08C6B4F1D77C5ACBF`、command-plan tests
`858103365E8CC0B352EB3A3F7555801DE60A4EDB0C38C98E9E25F344E7D3D053`、dispatcher
`0A588EC75A1B7E20AAACBAC55E7078F58E5A81EBCFD035A56F414F069D83E64B`、dispatcher tests
`EB85A431242C7F1D20ACE43476288A1CE389C3BE04F536C364BEF8943EB4B556`。

最终源码绑定的隔离 profile 位于 `E:/zircon-profiles/shader06-ibl-command-plan-20260831/`，31 个独立进程各执行
128 次 legacy/exact scenario。legacy allocation/bytes 为 `3960/415700`，exact 为 `393/27498`，即下降
`90.08%/93.39%`；command/readback 固定为 `10/55`，checksum 全部为 `0`。按每 scenario 归一化，legacy
min/p50/p95/p99/max/mean 为 `1539145.3/1831436.7/3150285.2/3354645.3/3354645.3/2006622.7 ns`，
exact 为 `123973.4/175880.5/298189.1/318781.3/318781.3/189672.3 ns`，p50 ratio `10.41x`。最终
harness source/exe/raw/summary SHA-256 为 `49535C6A1FF16F274B939515B08B18ECE172FBE494452FED17D0DCF3E335AAB3` /
`98BD1C8F9BC620B47FE3A37F086241119613AA4DC0B11E2D34AB2405E57B5460` /
`EB58658A8A40D5490369EBC04A5EA83A27B4F3C0AC97CCD3609A662122E2D5FD` /
`B7E69FC4FF1225AD51625D1DC4BA748E7BB46122BBFF0D04B15284479D9D508C`。这是 isolated production-source CPU
结构证据；尚不能推导产品帧、GPU、RSS/VRAM 或功耗收益。

当前状态为 `implementation_complete_isolated_profile_green_pending_managed_rust_wgpu_product_validation`。
managed Cargo/WGPU、产品 CPU scope、DX12 PNG/RenderDoc 与 matched WPR/WPA 仍未执行，因此不推进
M5/M6/M7/M8 或性能/功耗验收。

### 12.33 P1-8 versioned environment PBR recipe boundary

2026-08-31 的完整模块复审对照 Unreal `PreintegratedGF` 初始化/`EnvBRDF` 消费与 cmft/cmftStudio 的
offline cubemap filtering owner，确认现有 128x32 RG16F joint-Smith split-sum LUT 属于 device-global
system texture，而 `.zribl` recipe identity 属于 producer-specific PMREM/diffuse artifact。直接把 LUT
字段加入 `IblBakeRecipeIdentity` 会让仅改变设备级 LUT 的版本无条件淘汰所有环境 blob，违反两种生命周期。

本轮新增 `EnvironmentPbrRecipe` composite contract：它组合现有 `IblBakeRecipe`、独立的
`EnvironmentBrdfLutRecipe`（algorithm `2026_08_31_0001`、`128x32`、128 samples、
`GgxJointSmithSplitSum`、`Rg16Float`）和显式 `SingleScatterSplitSum` energy mode。asset/runtime
composite identity 仍只在 IBL diffuse producer 上不同；共享的 LUT identity 不写入 IBL artifact header。
系统纹理的 extent、row stride、WGPU format 和 expected payload bytes 均从 canonical recipe 派生，payload
只保留不可由 recipe 推导的 builtin SHA/bytes。生产路径继续使用 versioned builtin materialization，不恢复
runtime integration，不增加 binding/sample/permutation/PSO，也没有 multiple-scattering consumer。

TDD RED snapshot 为 `2460`；实现后的 focused rustfmt、静态 source guard 与 scoped diff integrity 通过。
该切片是零动态帧成本的身份/基础设施闭环，不声称 GPU、RSS/VRAM、画质、功耗或 elapsed-time 收益。受管
Cargo/Naga/WGPU、current-source PNG、`D:\Tools\renderdoc` replay、错误注入和 WPR/WPA 仍待执行，故
P1-8 状态更新为 `implementation_complete_static_recipe_green_pending_managed_runtime_validation`，M5-M8
保持 `in_progress`。

### 12.34 P0-9 warm-cache structural before profile and implementation gate

2026-08-31 current-source review confirms that the normal importer performs
`decode_texture_source_image_rgba32f` before it can derive the request, then
`staged_bundle_state` reads and decodes the complete `.zcube` and reads/decodes
the `.zribl` before reporting `Current`. The durable bundle reader also repeats
source/derived observations and checks the publication journal so a cache probe
cannot observe a cross-generation pair. This is a real ordering defect, not a
missing `exists` check.

The before profile is bound to the repository's 2K Poly Haven Lakes HDRI and
its published 256-face bundle. It runs 31 independent E-drive processes with
8 iterations per process and records source decode wall time, allocation count,
allocated bytes, full staged-content read bytes, and p50/p95/p99/min/max. The
profile does not claim product-frame, GPU, RSS/VRAM, or power results; those
remain separate managed viewer and WPR/WPA gates. The baseline harness source,
raw output, summary, and executable must remain under
`E:/zircon-profiles/shader06-p0-9-warm-cache-before-20260831/`.

The completed baseline contains 31 independent processes, 8 iterations each,
and 248 checksum-stable samples for a 2048x1024 / 5,918,432 B HDR source. Source
decode min/p50/p95/p99/max/mean is
`91.23/139.50/278.55/347.74/553.45/151.27 ms`; the full staged-content probe is
`8.58/15.11/23.94/38.20/59.85/15.81 ms`; combined warm-hit cost is
`99.81/156.21/294.35/364.41/568.04/167.08 ms`. Every sample performs 21 source
decode allocations / 58,728,995 B plus 6 cache-probe allocations / 21,070,832 B
and reads 8,438,056 B from `.zcube/.zribl`. The current-wire profile fixture
rewraps the repository's real 256-face legacy payload in the current v4 header
and BLAKE3 checksum solely to measure current decode scale; it is not a visual or
correctness artifact. Harness/exe/raw/summary/current-wire SHA-256 values are
`BA40BF7ADC943B45D2433DB3D78A46B8DA9390D14989A09A8E52713F10E46DAE` /
`32E2389D26A4B062F790EE96C24D13AD38EF3D02805B7645E525385B9E218D5F` /
`E912A76D78FCB6D577FC28723601B82945531539F12F8BA6FD5BF647432B9D53` /
`0AA2B58FF55329FBC3E6B1A2D099DDE4FEBF940F16CCE5C7F3CA8B3BEF4F3252` /
`5DB9D074C7341F55AB5F851ED9C21D3B2C19D8EAB3F8BCD52EB884EAB84F465A`.

The measured call graph and reference review are now complete. Unreal's texture
DDC keeps record metadata separate from payload values: availability checks use
`ECachePolicy::Query | ECachePolicy::SkipData`
(`TextureDerivedData.cpp:2823-2825`), payload hydration addresses stable value
IDs and fetches only requested chunks (`TextureDerivedData.cpp:2064-2088`,
`3455-3458`), and `FBuildOutputInternal` exposes metadata independently from its
value array (`DerivedDataBuildOutput.cpp:122-180`, `222-235`). cmft/cmftStudio
remain offline filtering and authoring references; neither is an engine cache
publication authority and therefore neither justifies an `exists`-only fast
path.

The Zircon hard-cut design is one request-keyed, logically immutable bundle
generation, not a second cache:

1. `image_decode` adds a borrowed-source metadata path backed by
   `ImageReader::into_dimensions`. It reuses the existing extension/explicit/
   guessed-format selection and returns width, height, and a stable resolved
   decoder-format identity without creating `DynamicImage` or RGBA32F texels.
   The resolved format identity participates in `derive_source_identity`; mode
   aliases that resolve to the same decoder share an identity, while a semantic
   decoder change cannot overwrite the same generation.
2. The full `IblBakeArtifactRequest` remains the generation key. A fixed-size,
   little-endian binary manifest lives under a versioned
   `render/ibl-source-bundle` root keyed by the existing full request digest.
   It binds magic/schema, staging and bake algorithm versions, source
   fingerprint, equirectangular dimensions and resolved decoder format,
   producer/wire-platform ABI, source/PMREM layouts, requested contents, and
   the encoded length plus BLAKE3 digest of both `.zcube` and `.zribl`.
   `wire-platform` describes the portable little-endian/RGBA16F contract, not
   the host OS, so device recreation and Windows/Linux do not fork identical
   CPU artifacts.
3. A warm probe recovers the existing IBL journal, decodes only that small
   manifest, matches every request field, requires both target paths to be real
   regular files with the recorded lengths, re-observes the manifest, and
   rejects any pending publication. It performs no source-pixel decode and no
   `.zcube/.zribl` `fs::read`; therefore its payload I/O and allocations are
   independent of source face size and artifact size. A same-length payload
   corruption is intentionally not rehashed by this availability query.
4. Hydration remains the content authority. Before decoding or applying either
   payload it verifies both recorded BLAKE3 digests, then re-observes the small
   manifest and publication barrier. A digest mismatch rejects the payload and
   invalidates the current marker so the next import rebuilds; payload bytes are
   never consumed merely because the metadata probe succeeded. This replaces
   the current second full source/derived read with a second manifest read.
5. Publication remains owned by the existing durable transaction. A cold build
   prepares source, derived, then manifest as three `PreparedFileWrite` values;
   the source-reuse branch prepares derived then manifest. Manifest-last is a
   defense-in-depth current marker, while the journal commit point remains the
   atomic authority. Both the standalone IBL recovery policy and project
   generation recovery policy must accept the manifest target. Missing or
   legacy two-file bundles are cache misses and are rebuilt into the three-file
   generation; no compatibility-hit path survives.
6. The request identity is deterministic, so republishing a valid request may
   only reproduce identical payload digests. Manifest/path existence without a
   matching request, accepting one payload, overwriting through an independent
   writer, or using a mutable host-global `current` locator is forbidden.

Required RED/GREEN coverage is: header-only dimensions equal full decode;
decoder-setting identity convergence/divergence; manifest round trip and
magic/schema/request/layout/length rejection; old two-file bundle is a miss;
warm hit opens neither payload; source-reuse and cold writes place manifest
last; standalone and project crash recovery never expose a partial generation;
stale requests remain distinct; same-length corruption is rejected at
hydration and clears current state; and a reader racing every target-replace
boundary cannot report a partial bundle as current. The after profile must use
the same 31 processes x 8 samples and fixture, record zero source-pixel decodes
and zero `.zcube/.zribl` bytes on warm probe, keep manifest size at or below
512 B, keep warm-probe allocation bytes below 64 KiB and independent of payload
size, and reduce combined p50/p95 to at most 25% of the before values
(`39.05/73.59 ms`). These are CPU/cache gates only; current-source viewer,
WPR/WPA power, RSS/VRAM, WGPU, PNG, and RenderDoc remain separate acceptance
evidence.

Implementation is not yet authorized by ownership. At `baselineEpoch=584`,
`ibl_source_cubemap_staging.rs`, `environment_ibl.rs`, and `image_decode.rs` are
attributed to cancelled/stale/archived owners and can only enter Shader06 by a
fresh exact transfer; `asset/project/manager/durable_transaction.rs` is still
attributed to active Frameworks01; and
`ibl_source_cubemap_staging/tests.rs` is already foreign-staged in the shared
index with no coordinator attribution. Shader06 will not split behavior,
overwrite that staged blob, or omit project recovery. The complete dependency
set must first be reconciled to one legal integration owner. Status:
`before_profile_and_architecture_review_complete_implementation_pending_ownership_reconciliation`.

### 12.35 P0 shared-model prepared-cache ownership hard cut

2026-08-31 对 Runtime02 handoff 的完整调用链复审分两步关闭了同一 source-asset 生命周期。
第一步确认真正的逐实例结构性复制发生在 `ResourceStreamer::load_model_asset`：
prepared-cache current hit 从 `PreparedModel.asset: Arc<ModelAsset>` 取得 `&ModelAsset` 后深
clone，而 `extend_pending_draws_for_mesh_instance` 按 mesh instance 调用该 helper；相同 model
因此形成 `O(I * model_payload)` 的 vertex/index/virtual-geometry 复制。streamer helper 与 fallback
现已统一返回 `Option<Arc<ModelAsset>>`；current revision 只执行 `Arc::clone`，连续命中保持
`Arc::ptr_eq`，stale revision 只接受 distinct latest `Arc`，fallback 成功后恰好包装一次。

第二步复核发现第一次结论仍漏掉一个结构性旁路：`ensure_model` 虽然只包装一次，却继续直接调用
`AssetManager::load_model_asset`。当 model source revision 未变、仅外部 mesh dependency 变化时，
prepared fast return 会失效，ensure 随即重新构造整份未变 `ModelAsset`。当前 hard cut 令 ensure
消费同一个 `self.load_model_asset(id)` Arc helper；source current 时只增加一次原子引用计数，再用
`model.as_ref()` 完成 geometry resolution，GPU resource 的独立 `Arc::new` 不变。生产依赖集中不再
存在 owned `ModelAsset` deep-copy 或第二 source cache owner。逐实例 source payload 工作量从
`O(I * model_payload)` 收敛为 `O(I)` Arc 操作，dependency-only rebuild 的 source payload 工作量从
`O(model_payload)` 收敛为 `O(1)` Arc 操作；prepared model 仍只持有一份 source payload。

RED/GREEN 同时加入 ensure 单一 helper 路径合同，以及一个 ignored current-source profile：fixture
为 `262,144` vertices、`786,432` indices，按当前 `MeshVertex` ABI 约 `28,311,552 B`，计划对
`16,384` 次 current hit 逐次验证 Arc、vertex storage 与 index storage 指针不变且 fallback 为零。
该 profile 尚未由受管 Cargo 执行，以上字节数是布局推导、次数是测试输入，不是 elapsed、allocation、
RSS、GPU 或功耗实测。

五文件 dependency set 已在 coordinator 下归并到 Shader06；来自 archived Render18 的三文件
transfer-preview fingerprint 为
`3f808bab3f337a59f67e446fd6978cdbde597a8e37733f3131b8ebe90ecdcc9d`，apply request 为
`1b9b625640c8476bb842a7032dfe5d4b`，随后五文件 lease request 为
`2bd4614f0023460d987f0a64cf82cfc6`。第二次五文件 exact-scope lease/attribution request 为
`1038d1ed77ac4f48aee4c647b18011e1` / `068d3a896e294552bbd950bcb1c3e7f6`；ownership matrix
`baselineEpoch=584` 仍显示五个 dependency path 均归 Shader06、无竞争 owner。最终
load/ensure/accessors/extend/prepared SHA-256 分别为
`22B1A68EB2D9189FC12CB01645F42BD0915CA1B46EB3BDD2BC5A10132B944922` /
`74805DD382F15A31CE1CA75C6FAB09724EF569767B5684A8C2B7AE9FD16AD2DE` /
`9D938CD8B168E6B11E5FF302AEB317E23A7A618708D6E4F61EBCBDC6AB9DD92C` /
`961D196F8E5008EC1B60BF7FAA9990F5368420D525998721863F18E496708B32` /
`C4134746BE41B8AA433ACD9C7BA64249E863D99CDEB2C24CF07DE340384E98C7`；focused
`rustfmt --check`、单一路径/指针结构合同与 scoped `git diff --check` 通过。受管 Cargo 的 CPU lane 仍被其它
Session 预留，故当前状态为
`implementation_complete_static_pointer_identity_green_pending_managed_rust_and_current_source_profile`；
不得据此宣称产品 timing、RSS、功耗或里程碑验收完成。

Runtime02 的后续 transfer-preview 仍不能形成合法 ownership transfer：请求的
`01a04c62-01c7-7ea1-8328-87fa4eb2b125` 不是当前 coordinator session，且
`session show` 没有该记录。`baselineEpoch=584` 的矩阵仍将五条依赖路径归属于 Shader06，
无 competing owner，且第二次 exact-scope lease/attribution 已由上述 receipt 刷新；因此本 owner
继续负责完整 Arc 生命周期闭包，
不向不存在的 session 转移，也不接管 `Cargo.toml`、`Cargo.lock` 或
`zircon_runtime/Cargo.toml`。后续 transfer 必须基于服务提交后的 fresh preview；在此之前
只保留静态 pointer/format 证据与未执行的 realistic-payload profile，受管 Cargo、真实分配/
字节观测、当前源 PNG、RenderDoc、GPU/RSS/VRAM 与功耗 gate 均保持 pending。

同日重新生成 viewer production-source closure 作为后续受管图像与性能证据的同源锚点：
闭包共 `18` 个生产 Rust 源文件，manifest 位于
`E:\zircon-profiles\shader06-source-closure-manifest-20260831.json`，SHA-256 为
`6BC104E6BA2C32FF0B9830825CD09F612FB36547FA15C160086A34653E20DDE5`，文件大小为
`3159` bytes。该 manifest 只记录 source path、byte length 与 SHA-256，不执行 Cargo、
WGPU 或产品渲染；后续 current-source DX12/RenderDoc/PNG/profile 必须绑定同一 manifest，
否则不得将结果归因于本 Shader06 源码。

同一 source identity 还生成了完整 Shader PBR critical-source manifest：117 个关键源文件
均通过存在性检查，canonical sorted-JSON SHA-256 为
`97464d01e8f213a1f1e41cefe5161f6033f8caa407f8495a9546fd643d4bd238`，文件位于
`E:\zircon-profiles\shader06-critical-source-manifest-20260831.json`，文件 SHA-256 为
`D5121000D70A0EDCC85C5D16276DCB5B809E22B730CC266BE92F620D87AEB663`
（`17187` bytes）。
它覆盖 viewer 闭包之外的环境、材质、deferred、realtime IBL、resource streamer 与
pipeline owner；同样只提供 source identity，不代表编译、WGPU、PNG、RenderDoc 或性能
验收。后续跨模块 profile 必须同时记录这两个 manifest hash。

同日对 realtime IBL recorder 的生产边界复审发现：graph workload 与缓存后的 WGPU command
此前只由测试比较，生产命中路径不会 fail closed。当前硬切把每个 PMREM/SH9 fixed graph
extent 传入 binding cache，并在 cache hit/miss 的 compute encode、pipeline、params 与
bind-group 工作前比较 command dispatch。成功路径每个 cacheable dispatch 仅增加一次
allocation-free `[u32; 3]` equality，不改变 4,124-workgroup ticket、采样预算、binding、
PSO、permutation 或 GPU 资源；旧 `[4,4,6]`/`[1,1,1]` 漂移会带 key 和两侧 extent
明确失败。静态格式、顺序合同和 diff integrity 已通过；受管 Rust/WGPU、当前源 PNG、
RenderDoc、GPU timing、RSS/VRAM 与功耗仍 pending，因此这不是性能收益或里程碑验收。

2026-08-31 P0-9 architecture-first warm-cache optimization: before editing, the current
importer/call graph and Unreal `TextureDerivedData` metadata-first cache path were reviewed,
and a 248-sample Poly Haven 2K baseline was recorded: combined decode/staging `p50=156.21 ms`,
`p95=294.35 ms`, with 21 decode allocations / `58,728,995 B` and 6 cache-probe allocations /
`21,070,832 B` per sample. The selected structural change is a versioned source-bundle
manifest, not a micro-optimization: a fixed 252-byte little-endian manifest records decoder
identity, source/bake layout and BLAKE3 stamps; warm classification reads only manifest and file
metadata, while hydration validates payload bytes before decoding. Source, derived and manifest
publication share one durable transaction with manifest-last ordering; old schema, missing or
request-mismatch bundles are rebuildable misses. Equirect identities include resolved decoder
format; captured ZCUBE/DDS/KTX keep separate stable container identities. Recovery target
validation moved to `bundle_recovery.rs`, leaving the staging root below the module-size budget.
Static Rust formatting, diff integrity and focused contract review are green. The after-profile
gate is combined `p50/p95 <= 39.05/73.59 ms`, manifest `<=512 B`, warm payload reads `0`, and
warm allocation `<64 KiB`; these are unachieved acceptance targets. Managed Cargo/WGPU,
ETW/WPA, current-source PNG, `D:\Tools\renderdoc`, RSS/VRAM and power remain pending because
the coordinator Cargo lane is occupied by another session. Status:
`implementation_complete_static_bundle_contract_green_pending_managed_rust_and_profile`.

The earlier ownership-reconciliation paragraph in this section is retained as pre-transfer
evidence and is superseded by the subsequently applied exact dependency-set transfers recorded
above. The resulting critical-source contract now covers all production children introduced by
P0-9, not only the root modules: `133` paths, zero duplicate/missing files, with PowerShell
contract regressions green at `30/30 + 4/4`. The non-overwriting manifest is
`E:\zircon-profiles\shader06-critical-source-manifest-p0-9-20260831.json`, canonical SHA-256
`6763f7906021af18edcce68a2c96f4cd1ebe413d8e8a132fec2693ec1b314973`, file SHA-256
`A5C3C154E430E7C86DA55574997B2D26DEC65EF9B1EA1DBA14EA3F12251A4A58` (`19384` bytes).
The viewer closure is separately bound by
`E:\zircon-profiles\shader06-source-closure-manifest-p0-9-20260831.json`, file SHA-256
`83BD5E68A4EF3B71352E3105802853F415EBC00692BFAA55E6CB45A9EF2616AE` (`18` files, `3157`
bytes). These are source-identity results only; managed Rust, current-source latency/allocation,
WGPU, PNG, RenderDoc, RSS/VRAM and power gates remain open.

P0-9 follow-up review found one remaining warm-hit bypass in the caller-decoded API. Normal serial
and parallel staging first called `probe_environment_ibl_warm_cache`, but
`stage_environment_ibl_source_with_parallel_executor_and_decoded_image` entered the shared builder
directly; that builder hydrated and decoded both `.zcube` and `.zribl` before consulting the current
manifest. The hard cut now passes an explicit `EnvironmentIblBundleProbeState`: normal entries pass
`AlreadyMissed` after their metadata probe, while the caller-decoded entry passes `Required`. The
shared builder checks `current_bundle_manifest_matches` before `staged_bundle_state` only when
required, returns the reused report on a current manifest without reading payloads, and hydrates on
miss with unchanged stale/recovery semantics. Cache-probe time uses saturating accumulation so the
first metadata decision is not overwritten by later miss classification. RED reproduced the absent
probe state; static GREEN proves manifest-before-hydration ordering, explicit miss propagation,
focused `rustfmt --check`, and scoped diff integrity. Final orchestrator/warm-probe SHA-256 values are
`D82D6D65DCA2F3A0CBBA52BC28DAE491A95F0879781E87AACF61BEB0A3618099` /
`36BBE79883BB60876F555C9E15D7F350196B309814EAD917DA81D0F0F4FB12A9`. The critical-source
contract also now includes load/ensure/prepared/mesh-instance instead of binding this cross-module
hard cut only through accessors. The non-overwriting P0-9b manifest contains `137` unique/existing
paths at `E:\zircon-profiles\shader06-critical-source-manifest-p0-9b-20260831.json`; canonical/file
SHA-256 values are `94F4D07B4D96F7DBF3E990B475382C7ABF8A88EABE4E31F0356078F9A2B02B0F` /
`3A0149719B04ABC916C2D6F1F533F5AA39A97ACD956E44C4B34DABDB7BA7E211` (`20046 B`). The viewer-only
18-file manifest is unchanged because none of its modules changed. Managed Cargo and the 31-run
after profile remain pending, so the p50/p95, payload-read, allocation and power targets are not
claimed and P0-9/M5-M8 do not advance.

### 12.36 P0-10 environment telemetry cadence and ownership review

2026-08-31 在任何 P0-10 production 修改前完成 current-source owner/cadence 复审。原 P0-10
描述的字段并非全部缺失，而是分属三种不可互换的真值：

| Cadence | Existing owner | Existing identity and payload | Required convergence |
| --- | --- | --- | --- |
| current snapshot | `RenderFrameworkState` / `RealtimeIblRuntime` | `RenderStats.last_generation`、同帧 `RenderFrameProfile`、probe workload，以及 realtime IBL readiness/published/pending/queued key 和 stale age | framework 在完成 pending submission 后、同一 state lock 内投影一个 immutable snapshot；不得新增长期可变副本 |
| asynchronous completion | scene submission journal / realtime IBL CPU/GPU collectors | source frame/generation、submission ticket、GPU status、profile capture epoch、operation label、scheduled/completed work | 继续保留显式 identity；不得把 delayed event 冒充 current-frame measurement，也不得由 snapshot query drain collector |
| asset/cook attempt | environment importer / durable bundle transaction | request、`Reused/Written/Skipped`、分阶段 timing、encoded bytes、bundle publication | 由 asset operation 返回 generation trace；renderer 不缓存 filesystem path、cache reason 或 importer timing |

现有 probe workload 已通过 `SceneReflectionProbeResources::last_workload_report` 进入
`RenderStats.last_reflection_probe_workload`，包含 extracted/candidate/active/capacity dropped、
scheduled cubemap bytes/write count、同步 asset load 次数/CPU 时间与 rejection；它并非 test-only。
`RenderFrameProfile` 已拥有 current-frame CPU submit、异步 GPU profile identity、staging bytes、
persistent texture resident bytes 和 degrade/budget 状态。`RenderSceneSubmissionCompletionReport`
已经把 `DeviceLost` 与 failed/cancelled/observation/tracking failure 分开，并保留 source frame 与
submission ticket。`RealtimeIblStatusReport` 已拥有六态 readiness、generation key、last-good age、
active elapsed 和 typed failure。结构性缺口是这些报告只能被分别查询，调用方可在不同锁时刻拼成
不一致组合；不是再发明一个 environment 状态机。

Unreal `ReflectionEnvironmentCapture.cpp:1094-1302` 的可取之处同样是 owner 形状：budget admission、
hysteresis、fade、fast-render burst 和 eviction 在同一 capture owner 内决策；
`1328-1408` 以 forced/manual、distance或frame age 选择唯一更新对象，并完成当前 timeslice 后再轮换；
`1460-1510` 由同一 owner 推进 face progress。它没有证明把 delayed GPU timing、DDC/import timing 和
current scene state 合成一个无身份的 mutable struct 是正确的。Zircon 应采用相同的单 owner/显式
progress 思路，但 device/submission lifetime 仍归 09A，asset publication 仍归 09D。

P0-10 的 hard-cut 顺序因此固定为：

1. 新增 allocation-free `EnvironmentRuntimeSnapshot`，在 framework 完成 pending submission 后只取
   一次 operation/state lock，组合 current realtime IBL report、同帧 probe workload、共享
   `Arc<RenderFrameProfile>`、scene completion event 和明确的 frame-generation identity。构造时验证
   `last_generation == last_frame_profile.frame_generation`；无 current frame 时不得把 default profile
   冒充有效样本。snapshot 不拥有 queue、timer collector、filesystem path 或第二套 readiness。
2. 为 asset/cook 增加 versioned `EnvironmentGenerationTrace`，明确 cache decision/reason、request/
   artifact provenance、CPU phase timing 和 publish terminal。它由 operation 返回或写入 bounded
   diagnostics service，不进入 renderer-local state。warm hit、legacy/missing/request mismatch、
   corruption/rebuild必须可区分。
3. 把 realtime IBL CPU/GPU operation reports 和 capture completion按 generation/ticket 投影到 bounded
   trace store；query 是非破坏快照，profiling drain 继续作为原始采样接口。延迟 GPU 样本必须保留
   source generation，不能回填到不匹配 current snapshot。
4. M11 再统一 budget debt、probe residency/VRAM、capture latency、device recreate 和 terminal failure；
   Editor 只展示该服务的 snapshot/trace，不重新推导 readiness。M13 的 Unreal 对照必须固定同一 HDR/
   atmosphere/probe/layout/resolution/material/exposure/recipe，并分别报告 presentation 与 bake/capture
   queue；降分辨率、关大气、减少 probe 或吞掉 fallback 的数据无效。

第一切片的 RED/GREEN 合同为：snapshot query 先完成 pending submission；只锁一次 operation/state；
frame/profile identity 匹配时共享现有 `Arc` 而非 deep clone；无 current frame 显式返回 `None` profile；
delayed scene/GPU event保留自身 generation；query 不 drain CPU/GPU timing collector；probe upload/occupancy、
VRAM/staging、fallback/stale age和device-loss均来自既有 owner。复杂度为 `O(1)` 固定字段复制加一次
`Arc` 原子引用，不增加 per-frame allocation、GPU dispatch、binding、PSO、permutation、采样或资源容量。
本节是 implementation gate，不是性能收益、产品验收或 P0-10 关闭；受管 Rust/WGPU、current-source
PNG/RenderDoc、RSS/VRAM、WPR/WPA 以及 matched Unreal benchmark 仍必须在实现后执行。

### 12.37 P0-10 immutable environment runtime snapshot first slice

2026-08-31 完成了 12.36 定义的第一段 hard cut，但没有提前实现 asset/cook trace store。realtime IBL
readiness、failure 与 generation DTO 从 graphics 私有实现移动到
`core::framework::render::environment`，graphics 只 re-export 同一个类型，不保留第二份状态定义或转换
副本。新的 `EnvironmentRuntimeSnapshot` 组合 current frame generation、共享的
`Arc<RenderFrameProfile>`、reflection-probe workload、scene-submission completion 和 realtime IBL
status。无 current frame 时 profile 明确为 `None`；frame/profile generation 不一致时返回 typed
`FrameProfileGenerationMismatch`，不会发布混合时刻的数据；delayed scene completion 仍保留自己的
frame/ticket identity。

`RenderFramework` 现在提供统一 query contract，WGPU implementation 严格按
`finish_submission -> one operation lock -> one state lock -> projection` 执行。投影直接读取现有
`RenderFrameworkState`/renderer truth，不调用会 deep-clone 大型 `RenderStats` 的 `query_stats`，也不
drain realtime IBL CPU/GPU timing collector。匹配 profile 只做一次 `Arc::clone`，probe/scene/realtime
report 为固定大小复制；查询规模为 `O(1)`，不新增长期 mutable state、per-frame Vec clone、GPU
dispatch、binding、PSO、permutation 或资源容量。

该边界遵循 architecture-first/runtime-absorption 约束：纯 DTO/error/trait 归 runtime core facade，WGPU
只实现 backend projection，Editor 以后只能消费 facade，不能直接访问 renderer。Unreal
`ReflectionEnvironmentCapture.cpp:1094-1118,1328-1408,1460-1510` 继续作为单 owner budget/progress
主参考；Unity Graphics `DebugDisplaySettingsStats.cs` 与 Fyrox `fyrox-graphics/src/stats.rs` 只支持
“runtime 产出、UI 展示”的消费方向；cmft `cubemapfilter.cpp:965-1009` 的共享 filter progress 仅用于
确认离线 producer ownership，不被复制为 renderer telemetry store。

TDD RED 首先确认 core contract、trait query 和 WGPU projection 均不存在。实现后的 focused
`rustfmt --check`、single-owner/single-lock/no-drain source contract、scoped `git diff --check` 和 Shader PBR
source-closure Pester 均通过；Pester 为 `34/34`。源码回归定义 `16,384` 次重复 projection，并要求
`Arc::ptr_eq` 及 pass-vector storage pointer 不变，但该 Rust test 尚未由受管 Cargo 执行，因此不是
allocation/timing 实测。关键源码哈希为 status DTO
`6F6843D4BB563C5E6A42EBFD93663F8C9553390A340593332ABFA6E6D52DD4CC`、snapshot
`6840E424351D408AC81993D950812F87E791B67B7A14428CB766B6F5B830C08A`、WGPU projection
`832B25A00E18D3C3E48BD677D4E40405EFBB1280C9885EC06F744928F733412D`。扩充后的
critical-source contract 覆盖 facade、error、trait binding、query wiring 和 graphics status owner，共
`148` 个 unique/existing paths；非覆盖 manifest 位于
`E:\zircon-profiles\shader06-critical-source-manifest-p0-10-20260831.json`，canonical/file SHA-256
同为 `5F233BE3C035E553ECD85B9DC8E5F8C77065E6CD8DA7781FA16ABA66C3FCC125`（`21627 B`）。

当前状态为
`runtime_snapshot_first_slice_complete_static_green_trace_store_and_managed_product_validation_pending`。
P0-10 的 versioned generation trace、bounded delayed-event store、统一 budget debt 与 matched Unreal
profile 仍未实现；受管 Cargo/WGPU、current-source PNG、`D:\Tools\renderdoc`、RSS/VRAM、WPR/WPA 和
功耗仍 pending，因此不宣称性能收益、瓶颈消失、P0-10 关闭或 M5-M8 验收完成。

### 12.38 P0-10 asset generation-trace transaction-boundary review

snapshot 第一切片后继续复审 asset/cook call graph，确认第二切片不能只在 renderer 或 direct importer
旁边增加一个 `VecDeque`。`EnvironmentIblSourceStagingReport` 已持有 request、`Skipped/Reused/Written`、
分阶段 timing、source/derived bytes 和并行 work-item 规模；direct staging 的 standalone commit 完成后才
返回 report，因此这条路径已有正确 terminal 边界。缺口位于 project import：
`prepare_environment_ibl_import` 调用 `PreparedEnvironmentIblSourceStaging::into_file_writes` 后只把
`Vec<PreparedFileWrite>` 合并到 project generation，report 在 project durable transaction commit 前被
丢弃。若在 prepare/encode 时发布 `Written` trace，会把尚未提交、可能冲突或恢复失败的 generation
误报为 terminal success。

现有 manifest-only probe 同样只返回 `bool`，把 missing manifest、decode/schema rejection、request/source
identity mismatch、source length mismatch、derived length mismatch、racing manifest replacement 和 pending
publication 全部折叠成 miss。它足以保证 warm-cache correctness，却不满足 P0-10 要求的 cache-decision
reason。单靠 `EnvironmentIblSourceStagingStatus` 或根据 elapsed time 猜原因都属于禁止的第二 truth。

因此下一 hard cut 的原子 dependency set 固定为：artifact store 把 metadata-only probe 提升为 typed
decision；staging report 以 versioned、无 filesystem path 的 immutable generation trace 暴露该 decision；
`PreparedEnvironmentIblSourceStaging` 保留 report 与 writes；full/targeted project generation 在 durable
commit outcome 确认后才发布 terminal trace，standalone path 仍在自身 commit 后发布；失败/recovery 必须
携带 request/generation 和 terminal reason。bounded store 只保存已经 terminal 或显式 pending 的 typed
entry，非破坏 snapshot 不复用 profiling drain，也不复制 payload bytes。Unreal DerivedData 的
metadata/value 分离继续约束 probe 形状，但 Zircon 的 project transaction 是本仓库的 publication owner，
不能被 reference engine 的局部 callback 覆盖。

该切片尚未实现：它至少跨越 `ibl_source_cubemap_staging`、environment importer report/warm probe、
`scan_and_import`、full/targeted prepared generation 与 durable commit outcome。必须先取得完整 exact-path
ownership，且先写 RED 覆盖 prepare-not-terminal、commit-success terminal、commit-failure/recovery reason、
cache miss taxonomy、bounded retention 和 non-destructive snapshot；不得只改 direct importer 或保留
`bool` compatibility path。状态为
`generation_trace_transaction_boundary_review_complete_implementation_pending_atomic_project_owner`。

### 12.39 P0-10 coordinator scope closure and second source review

2026-08-31 对 snapshot 第一切片执行第二轮 integration review。`finish_submission -> operation lock ->
state lock -> projection` 与既有 WGPU 串行查询边界一致；`last_generation` 与共享
`Arc<RenderFrameProfile>` 在同一 state guard 下读取，失配返回 typed error，无 current frame 不发布
default profile。scene completion、probe workload 和 realtime IBL report 保留各自已有 generation/ticket
语义；查询没有调用 `query_stats`、没有 drain timing collector，也没有复制 profile 的 pass/subsystem
vectors。未发现需要在 managed Rust 前猜测性改写的锁序、身份或热路径算法问题。

协调器 immutable scope 也已闭包。四个新路径先通过 request
`1d58777b555f47518c1dd6d39249c5e7` 进入 Shader06；七个已归属但不在 scope 的 facade/wiring 路径以
planless、no-edit bridge 在完全相同 current hash 下执行两段 audited transfer：外移 receipt
`f62060c84f5c4bd6ac63c88b7eaad94a`，回归 Shader06 receipt
`13aa4ef7069a46c489b7d541512e4b43`。因此 status DTO、snapshot/error、trait、WGPU projection、两层
module wiring、graphics status owner 和 source-contract 共 `16` 个 P0-10 dependency paths 现由同一
Shader06 integration owner 持有，后续候选提交不再依赖非法 scope 追加。

Shader06 同时释放了与本计划无原子提交依赖的 mixed manifests。fresh preview baseline `584`、fingerprint
`0CB0B50193C3B7AAA5840C3B74AD0ADDEC7EDCC578B60E2235D6F58530A464CE` 确认三条路径可转移，apply
receipt `91616B0CD88F486CB94DAE68E1BCE9F5` 已把当前 `Cargo.toml`、`Cargo.lock` 和
`zircon_runtime/Cargo.toml` blobs 完整交给 Frameworks01；Shader06 未编辑这些文件。

当前 managed workspace lane 仍由 foreign job `957b46a1eb8c4e61b3a7aa5d53aecaca` 占用，因此没有
启动 raw Cargo 或重复 reservation。状态仍为
`runtime_snapshot_first_slice_complete_static_green_trace_store_and_managed_product_validation_pending`；
受管 Rust/WGPU、current-source PNG、`D:\Tools\renderdoc`、GPU timing、RSS/VRAM、WPR/WPA、matched
Unreal baseline 和 12.38 的 atomic generation trace 均未完成，本节不构成性能收益或里程碑验收。
