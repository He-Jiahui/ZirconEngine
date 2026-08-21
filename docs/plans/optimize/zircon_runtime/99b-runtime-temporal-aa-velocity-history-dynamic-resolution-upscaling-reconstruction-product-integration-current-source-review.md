---
title: Runtime Temporal AA、Velocity、History、Dynamic Resolution、Upscaling、Reconstruction 与 Product Integration 当前源码工程化差距
category: zircon_runtime
report_id: Runtime101
review_date: 2026-08-22
baseline_head: be5a281c96b6dc9d33b5c9d0f2699a8bf75afcf1
baseline_epoch: 336
related_code:
  - zircon_runtime/src/core/framework/render/anti_alias
  - zircon_runtime/src/core/framework/render/view_family.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/anti_alias.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/anti_alias.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/resource_descriptors.rs
  - zircon_runtime/src/graphics/runtime/history
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract
  - zircon_runtime/src/graphics/scene/resources/pipeline
  - zircon_runtime/src/graphics/scene/scene_renderer/anti_alias
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record
  - zircon_runtime/src/scene/world/render.rs
tests:
  - zircon_runtime/src/graphics/tests/render_debugger_and_history.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge/history.rs
  - zircon_runtime/src/graphics/tests/render_product_anti_alias.rs
  - zircon_runtime/src/graphics/tests/render_product_anti_alias/particle.rs
  - zircon_runtime/src/graphics/tests/render_product_anti_alias/reactive_mask.rs
  - zircon_runtime/src/graphics/tests/render_product_particle_velocity.rs
  - zircon_runtime/src/graphics/tests/render_product_post_process/motion_blur.rs
  - zircon_runtime/src/graphics/tests/render_product_post_process_full_chain.rs
  - zircon_runtime/src/graphics/tests/render_product_post_process_full_chain/fixture.rs
  - zircon_runtime/src/graphics/tests/render_product_post_process_full_chain/visual_export.rs
  - zircon_runtime/src/graphics/tests/render_product_post_process_terminal.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/09h1-temporal-aa-velocity-history-upscaling-review.md
  - docs/plans/optimize/zircon_runtime/89-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/94-runtime-visibility-spatial-index-bounds-frustum-occlusion-hzb-culling-batching-instancing-gpu-scene-indirect-submission-instance-lifecycle-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99-runtime-volumetric-fog-froxel-local-fog-volume-lighting-shadow-history-temporal-reprojection-product-integration-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/TemporalAA.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/TemporalSuperResolution.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/VelocityRendering.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/DynamicResolution.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/DynamicResolutionProxy.h
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/STP
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Upscaling
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/PostProcessing/Shaders/TemporalAntialiasing.hlsl
  - dev/godot/servers/rendering/renderer_rd/effects/taa.cpp
  - dev/godot/servers/rendering/renderer_rd/effects/fsr2.cpp
  - dev/godot/servers/rendering/renderer_rd/effects/smaa.cpp
  - dev/bevy/crates/bevy_anti_alias/src
  - dev/Fyrox/fyrox-impl/src/renderer/fxaa.rs
  - dev/Fyrox/fyrox-impl/src/renderer/shaders/fxaa.shader
doc_type: current_source_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Runtime Temporal AA、Velocity、History、Dynamic Resolution、Upscaling、Reconstruction 与 Product Integration 当前源码工程化差距

## 1. 结论

当前Zircon的时域渲染不是空壳。它已经有typed AA settings与fallback report、TAA history双缓冲、camera/object/particle velocity GPU executor、mesh skin/morph previous deformation、reactive mask、history bind-group identity cache、per-camera key、成功提交后才翻转history、基础统计，以及一个设计方向正确但尚未接入产品提交的`ViewFamilyPipeline`。这些底座适合保留为characterization oracle，不能在重构时退回单帧后处理或把previous transform重新塞回每个feature。

但当前产品链存在五个确定性阻断。第一，`HistoryValidationKey`把world、camera、mesh transform/material、lighting、animation、post-process、particle和effective features的整帧相等当作history兼容条件；正常相机或物体运动会返回`FrameInputsChanged`并把`previous_history_available`置为false，TAA因而恰好在最需要reprojection时失去history。第二，完整`ViewFamilyPipeline`只存在于类型与测试，生产`build`没有安装，render graph继续使用`size/render_size`标量。第三，DRS开启时TAA把render-size current attachment与display-size history attachment绑定为同一render pass MRT，同时executor又把display size传给render-size depth/velocity，资源尺寸和坐标域都不成立。第四，默认feature descriptor没有particle velocity，透明、sprite、overlay、half-res/OIT/transmission和procedural deformation也没有统一velocity/reactive producer。第五，Scene只持久化`msaa_samples`，Editor没有AA/DRS/upscaler authoring；当前TAA主要由测试手工改`RenderFrameExtract`触发，不是可保存、可重开的产品能力。

算法身份同样不能按枚举名验收。当前FXAA是5 tap邻域模糊；SMAA是无area/search LUT、无stencil、无diagonal/corner search的三阶段自定义平滑；spatial upscale只有一次bilinear sample；CAS与DLSS没有executor/provider；MSAA pipeline固定sample count 1且render pass没有resolve target。基础TAA只有单张RGBA16F history、round后单texel读取、3x3 YCoCg clip，以及用当前帧中心深度和当前帧邻域深度差冒充disocclusion。它不是Unreal TSR、Unity STP、Godot FSR2，也不能证明达到这些实现的画质或性能。

旧`09H1`登记的 **10项P0全部保持开放并继续由其唯一计数**；本报告不重复制造父P0，使用`Runtime101-P0-*`逐项重验其当前状态，并新增 **31项P1、10项P2与44个资格门**。当前24个名称匹配artifact总计154,612 bytes，没有直接TAA/FXAA/SMAA/upscale artifact；两张velocity图几乎全黑，仅有细边缘颜色，GI与volumetric temporal图也只是其他consumer的合成样例。因此在唯一View/History/Resolution合同、完整velocity/reactive覆盖、工程级reconstruction、provider ABI、Scene/Editor闭环和同场景竞争性证据完成前，本域必须标记为`experimental/degraded`，不得宣称production-ready、TSR-equivalent或优于当前Unreal。

## 2. 审查边界、currentness与证据

### 2.1 冻结语料

| 范围 | 文件 / 行 / 非空行 / bytes / test attributes / ignored | 证据等级 | fingerprint |
|---|---:|---|---|
| Runtime temporal reconstruction current-source语料 | **193 / 45,449 / 未归一 / 1,713,638 / 344 / 0** | E3覆盖neutral contract、history、ViewFamily、graph、resource、shader、velocity、submission、Scene与Editor入口 | `cbd39439f719dbacae5b6adec7cfd1fd99e95f1e8a28e7a21326ef10a42f4081` |
| focused tests与support | **11 / 5,009 / 未归一 / 179,635 / 42 / 2** | E3覆盖history state、AA、particle velocity、reactive、motion blur与full-chain exporter | `790818a6b72a836ab3c13396aec5651218885a8c72cb3f716b5194e7713aa3f9` |
| 名称命中的历史artifact | **24 / 未归一 / 未归一 / 154,612 / 未归一 / 未归一** | E1/E2：16个velocity、2个Hybrid GI MSAA、6个GI/volumetric temporal；直接AA/upscale为0 | `3dfbe697212347cedfaef1ebaa92297c63f59fe4d01408c34e6e98ea5e9464c8` |
| 五引擎参考切片 | **28 / 17,477 / 14,835 / 750,409 / 未归一 / 未归一** | E2/E3读取Unreal、Unity Graphics、Godot、Bevy与Fyrox具体算法和owner | `f109744c7352ce0b180a6a51f386002c1dee561538d21a487bf3adba27353be1` |

fingerprint算法为：相对路径与每文件SHA-256组成按路径排序的manifest，以TAB分隔字段、LF分隔记录，再对UTF-8 manifest执行SHA-256。行数按PowerShell `Get-Content`统计。冻结对象是2026-08-22共享working tree，基线HEAD为`be5a281c96b6dc9d33b5c9d0f2699a8bf75afcf1`，coordinator epoch为336；它不是只读HEAD快照，进入实现前必须重取fingerprint并逐项复核。

Bevy、Fyrox、Godot与Unity Graphics revision分别为`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`与`a7e4c051d256a781ab362c64316b125a1e104694`。Unreal目录不是独立Git checkout，本报告以列出的五个源码文件及其manifest fingerprint为锚，不伪造Unreal revision。

### 2.2 数据链读取深度

本轮从`SceneCameraAsset/CameraComponent`开始，追踪World camera snapshot、`RenderFrameExtract`、requested/effective AA、capability fallback、built-in feature descriptor、ViewFamily构造、pipeline compile options、resource descriptor、graph executor、persistent history allocation、validation key、previous camera/mesh/skin/morph/particle state、velocity/reactive draw、TAA bind group、WGSL、history copy/flip、viewport record、diagnostics、Editor symbol与artifact exporter。不是只按`taa`文件名检索。

shader审查逐项核对current/render/display/history extent、viewport origin、jitter与matrix domain、previous depth/velocity、sampling filter、disocclusion、color clipping、confidence、exposure、reactive/translucency、MRT attachment一致性和history publication。FXAA核对edge orientation、endpoint search与subpixel；SMAA核对LUT、stencil、search、diagonal、corner和preset；upscale核对provider lifecycle、motion/depth convention、resolution negotiation、reset、exposure、sharpness和fault fallback；MSAA核对attachment、pipeline key和resolve，而不是只查枚举或descriptor。

### 2.3 明确未做

本轮只做静态current-source review、既有PNG/TXT视觉复核和文档审计。没有修改Rust/WGSL/Scene/Editor实现，没有运行Cargo、Editor/App、WGPU、RenderDoc、参考引擎、camera path、4K/8K、XR、device loss、VRAM pressure或同画质benchmark。静态源码足以证明资源尺寸、坐标域、producer可达性和authoring断链，但不能替代修复后的GPU产品验收。tooling按用户要求不纳入本轮。

## 3. 当前产品链与可保留基础

### 3.1 typed requested/effective AA可以保留

`core/framework/render/anti_alias`已经把mode、TAA quality、fallback和effective report分开；graphics stats也能记录effective mode。这比在shader里猜测capability更接近正确边界。重构应保留typed request，但把`requested -> eligible -> admitted -> prepared -> active/degraded`统一为一份frame receipt，并删除名存实亡的枚举。

### 3.2 history双缓冲的成功提交语义可以保留

`TemporalHistoryStore`维护previous/current slot，只有成功写入才翻转；TAA bind-group cache按真实texture identity区分slot。该语义能够避免failed frame发布半更新history。需要重构的是validation domain、generation、per-view资源与last-good/fault生命周期，而不是推翻success-only publication。

### 3.3 previous deformation与多类velocity executor可以保留

camera、rigid mesh、skin/morph和particle都有真实GPU路径，mesh提交也保存previous deformation所需状态。它们证明Zircon已具备构建统一Motion Producer Registry的局部组件。正确方向是由RenderScene/GPU Scene发布current/previous identity和deformation packet，让各render path共享，而不是每个后处理自己维护previous world。

### 3.4 reactive mask与material strength是可迁移底座

reactive pass能接收材质强度并写入mask，测试覆盖zero/full strength、透明alpha、history slot flip和bind-group复用。它仍缺少particle、translucency composition、specular、emissive、disocclusion和upscaler provider所需分类，但现有ABI可作为迁移入口。

### 3.5 `ViewFamilyPipeline`的类型设计方向正确

当前类型已经表达display/primary/secondary extents、phase target、upscale requirement、temporal history key、DRS controller/scope/sample/decision。这是应成为唯一分辨率与view合同的候选，而不是删除。问题在于production build没有安装，consumer继续依赖标量尺寸；重构必须hard cut到不可缺省的compiled view-family snapshot。

### 3.6 per-camera key和previous-camera清理语义可以保留

camera history key包含entity、order、type、target、viewport和layers，能区分stack与sub-view；camera cut会清理previous jitter/camera状态。这比单个viewport-wide previous matrix更安全。下一步要把key提升为稳定ViewHistoryId，加入generation、last-used与显式销毁。

### 3.7 统计与测试骨架可以保留

现有测试会断言effective AA、executor id、TAA resolve、reactive command、velocity readback、particle order、sample-count normalization和history cache复用。这些适合作为L1/L2 characterization gate。它们不能继续被当作画质/性能oracle，必须扩展到普通Scene producer、连续运动、DRS、camera stack、provider failure与可复现sequence artifact。

## 4. 五引擎参考对照

### 4.1 Unreal：history是带viewport、exposure与有效性的产品对象

`TemporalAA.cpp`显式处理input/output viewport rect、history buffer size、UV bounds、screen-to-history transform、jitter、camera cut、velocity/stencil、pre-exposure correction和history metadata。`TemporalSuperResolution.cpp`进一步拥有sample count/metadata、shading rejection、flicker与moire处理、thin geometry、translucency rejection、decimation、spatial AA、history resurrection、persistent frames、guide/uncertainty/coverage和debug resources。Zircon当前单RGBA16F history加whole-frame equality不能表达这些状态。

`VelocityRendering.cpp`把velocity作为renderer-wide责任：材质/WPO、translucent velocity、previous transform、clipped depth和pass routing都有显式资格。`DynamicResolution.cpp`与`DynamicResolutionProxy.h`消费GPU timing history、frame budget、headroom、change period、threshold/hysteresis、over-budget panic和third-party upscaler的上下限。Zircon虽有同名controller DTO，却没有生产owner或timestamp feedback。

### 4.2 Unity Graphics：STP和IUpscaler冻结了provider级时域合同

STP输入current color/depth/motion、可选stencil/debug、current/previous/previous-previous matrices、hardware DRS、motion scaling、delta time、current/output size和valid-history；history按DepthMotion、Luma、Convergence、Feedback分类并由per-context ping-pong/hash负责重建。pre-TAA在input resolution执行，TAA在output resolution执行，分辨率域不是一个`size`参数。

`IUpscaler`还定义temporal/sharpen/XR能力、jitter、render-resolution negotiation、per-camera context、mip bias、last-used/cleanup、pre/previous-pre/post resolution、motion vector尺寸/方向、exposure/pre-exposure、reset、frame time、DRS与subpixel jitter。DLSS/FSR2实现负责availability、context create/destroy、option validation、quality negotiation、depth/motion/jitter convention和fallback。Zircon的`Dlss`/`Cas`枚举没有这些生命周期和ABI。

### 4.3 Godot：轻量TAA、真实SMAA和FSR2边界可直接区分

Godot TAA读取current/previous velocity、depth与history，用closest-depth motion、Catmull-Rom 9 tap、previous velocity disocclusion和variance clip。它仍是相对轻量实现，但已经明显超过Zircon的nearest history和current-neighbor depth差。Godot SMAA有edge/weight/blend三阶段、AreaTex/SearchTex、stencil、search steps、diagonal和corner；Zircon同名路径缺少算法身份的核心资源。

Godot FSR2不是把vendor名写入AA枚举，而是创建SDK context/pipeline/resource，输入color/depth/motion/reactive/exposure，输出display-size结果，并传递jitter、motion scale、reset、internal/display size、sharpness、time和camera参数。这说明provider lifecycle与neutral IO必须先于具体实现。

### 4.4 Bevy与Fyrox：baseline实现仍保留准确命名

Bevy TAA按camera组件拥有history，要求depth/motion prepass，支持reset、ping-pong与viewport，shader使用closest depth、过滤history、variance clip和confidence。Bevy SMAA带两张LUT、stencil、三pass与preset，FXAA实现edge方向、endpoint search和subpixel；CAS是独立可组合post stage。Fyrox明确集成NVIDIA FXAA 3.11 shader。即使目标只是轻量baseline，Zircon当前5 tap blur和custom edge blur也不能沿用FXAA/SMAA正式名称。

### 4.5 对照结论

参考实现共同要求的不是“更多阈值”，而是稳定View identity、input/output/history rect、previous surface数据、motion convention、exposure、per-context lifecycle、DRS feedback、provider capability、fault/reset和可观察debug。Zircon必须先修正这些结构合同，再选择baseline TAA、temporal upscaler、spatial AA与vendor provider；直接继续调当前shader权重只会固化错误边界。

## 5. P0当前源码重验

### Runtime101-P0-01：全帧输入相等仍被错误用作temporal history compatibility

`graphics/runtime/history/validation_key.rs`把world、完整camera descriptor、mesh transform/material、lighting、animation、post-process、particle和effective feature list写入key；`is_compatible.rs`只要key变化就返回`FrameInputsChanged`。测试还明确断言camera/mesh motion会失效。资源handle可以复用，但`previous_history_available=false`会阻断TAA accumulation。正确合同必须只因ViewHistoryId/generation、extent/format、camera cut、projection discontinuity、provider change、device generation等时域不兼容事件失效；正常连续运动应依赖velocity重投影，而不是清空history。

### Runtime101-P0-02：全局history bit和巨型texture bundle仍把不同consumer绑成同一生命周期

`SceneFrameHistoryTextures`同时拥有TAA、GI、volumetric、AO、SSR、HZB、exposure等资源，render-size变化或validation failure按整组处理。不同consumer的有效性、分辨率、格式、camera cut、quality/provider和last-good语义完全不同。必须拆成typed `HistoryDomain`，每域拥有generation、descriptor、valid rect、reason、previous/current publication、memory/eviction和device-loss策略；frame receipt不能再只有一个“previous history available”。

### Runtime101-P0-03：`ViewFamilyPipeline`仍未进入生产提交上下文

`core/framework/render/view_family.rs`已有完整display/primary/secondary extents、phase target和DRS数据，但production `build.rs`只读取phases与`upscale_required`，构造context时没有`.with_view_family_pipeline(...)`。context字段保持`Option`，getter会在缺失时`expect`；当前生产之所以不panic，只是没有consumer真正调用它。必须让编译后的ViewFamily成为submit context必填值并删除scalar fallback，所有resource descriptor、pass、shader params与history owner只从同一snapshot读取。

### Runtime101-P0-04：DRS下TAA attachment尺寸和坐标域确定性不成立

`resource_descriptors.rs`只把`UPSCALED/FINAL_COMPOSITED`设为display size，`TAA_OUTPUT`与scene inputs仍是render size；persistent TAA history却按display size创建。TAA executor把render-size output和display-size history-current作为同一MRT，WGPU要求attachment extent一致。graph executor还把`stack.target.size`的display size传给render-size color/depth/velocity，shader直接用display coordinate `textureLoad` current inputs。必须明确TAA与TAAU两条合同：baseline TAA同分辨率，temporal reconstruction以input/output rect和独立history output执行，不能靠混尺寸MRT隐式完成upscale。

### Runtime101-P0-05：camera velocity在DRS/sub-view下继续混用display与render矩阵

scene uniform的current matrix按render size构建，previous matrix却使用`frame.viewport_size` display size；executor又把display size传给render-size depth/velocity，参数没有viewport origin、input/output ratio、jitter delta或history transform。velocity compatibility还要求`dynamic_resolution`完全相等，scale变化直接禁用previous。结果在DRS变化、split viewport、camera stack和odd extent下会产生错误motion或清空history。必须冻结motion vector单位/方向、jittered与unjittered矩阵、viewport rect/origin、current/previous render extent和display extent，并由同一个ViewSnapshot生成。

### Runtime101-P0-06：particle velocity组件存在，但默认产品feature图仍不可达

`particle.velocity` executor已注册，GPU pipeline与测试也存在；但production built-in temporal feature descriptor只有camera/object velocity，默认pipeline没有`particle-velocity` pass。仓库中这个pass名主要出现在compile test fixture。必须由Motion Producer Registry依据实际visible contributor生成particle work并进入默认产品graph，而不是要求测试手工注入descriptor。

### Runtime101-P0-07：透明、sprite、overlay和程序化变形没有完整velocity/reactive覆盖

object velocity主要覆盖mesh stream；透明mesh没有统一自身motion，sprite、overlay UI、half-res transparent、OIT、transmission、procedural WPO和部分particle composition缺producer。reactive writer主要从mesh/material alpha与strength推导，无法表达translucency composition、emissive/specular变化、particles和reconstruction provider要求。必须以render contributor为单位声明velocity、reactive、composition mask与coverage资格，并在缺失时结构化degrade，不能静默写零。

### Runtime101-P0-08：MSAA仍是descriptor surface，不是实际GPU render/resolve architecture

mesh、GBuffer、velocity、particle和post-process pipeline广泛使用`wgpu::MultisampleState::default()`，sample count为1；render pass几乎都`resolve_target: None`。graph descriptor虽可表达大于1的sample count，当前capability却把max MSAA限制为1，测试只证明descriptor编译而非匹配的WGPU pipeline/resolve。必须让sample count进入attachment、PSO key、depth/color resolve、camera stack与post-process边界；在完成前`Msaa`不能作为supported product mode。

### Runtime101-P0-09：requested、supported、active、provider和持久化authoring仍没有唯一真值

capability报告offscreen支持FXAA/TAA、不支持SMAA/CAS/DLSS、max MSAA=1；public enum却同时暴露Off/Auto/Fxaa/Msaa/Taa/Smaa/Cas/Dlss，`Auto`固定优先FXAA且不会选择TAA。build在requested TAA时把history可用性强制为true以避免首帧fallback，实际shader仍收到invalid history，造成receipt语义失真。`SceneCameraAsset`和`CameraComponent`只保存`msaa_samples`，World构造snapshot时不传DRS，Editor也没有AA/DRS/provider consumer。必须建立Scene-owned settings asset、capability/provider registry和逐帧effective receipt，普通create/save/reopen/cook路径必须可达。

### Runtime101-P0-10：仍没有能支撑“达到或超过Unreal”的质量、性能和回归证据

24个名称匹配artifact中没有文件名含TAA、FXAA、SMAA、anti-alias或upscale。ignored reactive exporter声明的`render18_taa_reactive_mask_wgpu_20260813.png`仍不存在。velocity图片几乎全黑，GI/volumetric temporal图片验证的是其他consumer；没有连续camera path、reference sequence、history/rejection heatmap、GPU timestamp、VRAM/bandwidth、driver/provider/settings或source manifest。必须先冻结场景、硬件、输出分辨率、画质metric和统计方法；单帧changed-pixel、pass名或CPU submit time不能用于性能声明。

## 6. P1工程差距

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| Runtime101-P1-01 | TAA没有previous depth/normal/material或surface identity history | typed previous-surface histories与统一reprojection validity |
| Runtime101-P1-02 | `depth_delta`比较当前中心与当前邻域，不是disocclusion | previous-depth reconstruction、closest-depth velocity与surface rejection |
| Runtime101-P1-03 | history坐标round后单次`textureLoad` | Catmull-Rom/bicubic或可配置高质量history filter，带valid rect clamp |
| Runtime101-P1-04 | uniform缺input/output/history transform、rect origin和jitter delta | versioned reconstruction constants，由compiled ViewSnapshot生成 |
| Runtime101-P1-05 | 没有sample count、moments、lock/status、coverage或confidence模型 | 分层history metadata与可视化生命周期 |
| Runtime101-P1-06 | quality preset只切同一baseline的8个阈值 | 明确sample/filter/rejection/sharpen/cost的质量配方与预算 |
| Runtime101-P1-07 | camera cut依赖硬编码位移/旋转启发式 | Director/Scene显式cut、projection discontinuity与fallback heuristic |
| Runtime101-P1-08 | jitter固定8周期Halton(2,3) | provider/quality决定的sequence、phase count、reset与per-view sample index |
| Runtime101-P1-09 | temporal frame index为viewport全局值，非每history generation | `ViewHistoryId + generation + successful_write_index` |
| Runtime101-P1-10 | DRS controller/scope/sample/decision只有DTO，没有runtime owner | GPU timestamp mailbox、budget、hysteresis、panic、decision receipt与trace replay |
| Runtime101-P1-11 | render-size变化整组reallocate/reset | per-history resize policy、resample/discard条件、last-good与atomic publication |
| Runtime101-P1-12 | static mobility对象违规运动没有检测 | mobility invariant diagnostic、previous transform修复或显式velocity资格 |
| Runtime101-P1-13 | transparent mesh没有统一object velocity | transparent contributor velocity contract与composition-aware motion |
| Runtime101-P1-14 | particle velocity每帧CPU展开quad并创建vertex buffer | persistent GPU buffers、simulation output复用、budget与indirect submission |
| Runtime101-P1-15 | particle reactive/translucency分类缺失 | VFX material输出reactive/composition/coverage元数据 |
| Runtime101-P1-16 | reactive mask直接用alpha代表拒绝强度 | material hint、opacity delta、luminance change、translucency和provider policy |
| Runtime101-P1-17 | WPO/procedural deformation没有统一previous-state ABI | material/deformer versioned velocity ABI，Forward/Deferred/Shadow共享 |
| Runtime101-P1-18 | 缺camera-relative/large-world motion精度策略 | relative origin generation、high/low transform或稳定previous-origin变换 |
| Runtime101-P1-19 | FXAA实际上是5 tap blur | 真实FXAA版本、准确命名、edge search/subpixel与oracle |
| Runtime101-P1-20 | SMAA是无LUT/无stencil的custom edge blur | SMAA 1x三阶段、Area/Search LUT、preset、diagonal/corner与residency |
| Runtime101-P1-21 | spatial upscale只有bilinear sample | 明确spatial filter/quality/sharpness/ringing与viewport transform |
| Runtime101-P1-22 | CAS只有模式名，没有真实stage | 独立可组合CAS/RCAS stage，不再冒充AA mode |
| Runtime101-P1-23 | provider不能扩展FSR2/DLSS/XeSS/STP-like实现 | neutral upscaler ABI、capability negotiation、context/fault/version lifecycle |
| Runtime101-P1-24 | TAA/particle等热路径仍创建部分per-frame GPU对象 | persistent resource/PSO/bind-group pool与generation cache |
| Runtime101-P1-25 | 2D、overlay、UI、stereo/XR/multiview AA边界未定义 | 每surface phase、view family、late UI与XR provider合同 |
| Runtime101-P1-26 | 测试只证明静态或重新稳定后的收敛 | 连续运动、disocclusion、shimmer、exposure、DRS与stack sequence oracle |
| Runtime101-P1-27 | Scene/asset不保存AA mode、TAA quality、DRS policy、provider和sharpness | versioned camera/render-quality authoring schema与hard-cut migration |
| Runtime101-P1-28 | Editor没有Inspector、profile、debug或Problems闭环 | transaction/undo/redo/multi-edit/save/reopen/cook与runtime preview |
| Runtime101-P1-29 | `ViewportRecord`的camera history、motion、particle与runtime state maps无统一淘汰 | last-used、camera removal、budget、eviction、rename/rekey和device teardown |
| Runtime101-P1-30 | history texture bundle跨TAA/GI/fog/AO/SSR/HZB/exposure共生共灭 | typed HistoryRegistry与各consumer独立descriptor/generation/validity |
| Runtime101-P1-31 | Runtime虽有AA统计，Editor不消费history rejection、DRS或provider状态 | machine-readable diagnostics、debug view和capture receipt贯通Editor |

## 7. P2治理差距

| ID | 当前差距 | 治理要求 |
|---|---|---|
| Runtime101-P2-01 | `Auto`固定优先FXAA | 按project quality、motion、provider、budget和surface类型解析 |
| Runtime101-P2-02 | 只有TAA拥有quality字段 | 所有算法/provider使用versioned quality recipe和cost class |
| Runtime101-P2-03 | camera cut、TAA阈值和blend常量来源不可观察 | 进入配置/recipe并记录effective值与修改原因 |
| Runtime101-P2-04 | TAA bind-group cache固定最多8项且无预算/淘汰统计 | capacity、bytes、hit/miss、eviction和pressure receipt |
| Runtime101-P2-05 | 缺velocity/history/rejection/lock/coverage debug view | RenderDebugger与Editor overlay提供逐view可视化 |
| Runtime101-P2-06 | 文档、capability与当前算法身份会漂移 | 从同一feature registry生成状态或在promotion gate交叉审计 |
| Runtime101-P2-07 | `Dlss`固化在core AA enum | generic provider id，vendor名只进入provider metadata/settings |
| Runtime101-P2-08 | 画质/性能判定无统计显著性 | warmup、重复次数、置信区间、outlier与driver/hardware provenance |
| Runtime101-P2-09 | history invalidation reason过粗且无法按domain聚合 | stable reason taxonomy、counter、event trace与per-view drill-down |
| Runtime101-P2-10 | DRS/provider decision不能录制和确定性重放 | timing/decision trace schema与offline replay harness |

## 8. 目标架构与owner边界

| Owner | 唯一职责 | 明确不拥有 |
|---|---|---|
| `Scene Render Quality Truth` | 持久AA/DRS/upscaler/quality settings与migration | GPU resource与provider context |
| `ViewFamilyService` | 编译view identity、rect、display/render/history extent、jitter与phase | 算法私有history |
| `ViewHistoryRegistry` | per-view/per-domain generation、descriptor、publication、eviction、fault | whole-frame scene equality |
| `MotionProducerRegistry` | camera/object/deformation/particle/transparency motion与coverage | TAA算法本身 |
| `ReactiveCompositionService` | reactive/translucency/exposure/composition masks | vendor provider实现 |
| `TemporalReconstructionService` | baseline TAA/TAAU input contract、history与debug outputs | Scene authoring与GPU timing决策 |
| `UpscalerProviderRegistry` | provider capability、context、IO convention、reset/fallback/version | core enum扩张 |
| `DynamicResolutionService` | GPU timing、budget、hysteresis、scope、decision与telemetry | 自行修改camera truth |
| `SpatialAntiAliasService` | FXAA/SMAA等准确命名的spatial stage | temporal history |
| `MultisampleSurfaceService` | sample count、PSO、attachment与resolve | 用descriptor假装pipeline支持 |
| `TemporalDiagnosticsService` | stats、debug view、capture manifest、sequence metric | 从单帧图片推导产品完成度 |
| `Editor Rendering Quality Workspace` | transaction、preview、Problems、profile、save/reopen/cook | 复制Runtime算法或状态机 |

固定边界仍是公开包`zircon_app`、`zircon_runtime`、`zircon_editor`；neutral contract归`zircon_runtime::core::framework::render`，具体GPU资源和executor归graphics，Scene保存可序列化真值，Editor只拥有authoring操作。不得留下旧scalar-size、full-frame history equality、vendor enum或test-only descriptor兼容shim。

## 9. 依赖顺序与重构里程碑

### M0：冻结characterization与算法身份

保留现有history flip、velocity、reactive、fallback和stats测试；新增普通Scene producer基线。将当前FXAA/SMAA/CAS/DLSS/MSAA状态诚实标为`prototype/unsupported`，避免重构期间继续以名称积累错误兼容性。

### M1：ViewFamily与Resolution hard cut

让compiled `ViewFamilySnapshot`成为submission必填值；统一view id、rect origin、display/render/output/history extent、jittered/unjittered matrices与phase。删除`size/render_size`标量fallback和optional getter。

### M2：typed History Registry

拆分TAA、GI、fog、AO、SSR、HZB、exposure history；加入descriptor、generation、valid rect、successful write index、reason、last-good、resize、eviction、memory、device-loss和per-camera removal。删除whole-frame validation key。

### M3：统一Motion Producer Registry

从RenderScene/GPU Scene发布camera、rigid、skin、morph、WPO、particle、sprite、transparent和overlay current/previous packet；冻结motion单位、方向、viewport和jitter convention；加入large-world origin generation。

### M4：Reactive与Composition合同

统一mesh/VFX/translucency/OIT/transmission/half-res contributor的reactive、composition、coverage和exposure元数据；缺producer时提供结构化reason，不再默认零mask。

### M5：修复baseline TAA

加入previous depth/surface、closest-depth velocity、filtered history、input/output/history transform、camera cut、exposure correction、sample/moment/confidence、robust clip与sharpen；先在同分辨率合同下通过运动sequence。

### M6：Temporal Reconstruction与provider ABI

建立neutral IO、context create/destroy、availability、quality negotiation、jitter、motion/depth convention、pre-exposure、reset、fault和fallback；随后接入至少一个真实开源temporal provider并保留高质量自研路径。

### M7：GPU-time Dynamic Resolution

用timestamp mailbox驱动有界历史、budget/headroom、hysteresis、change period、panic、min/max、multi-view scope和provider约束；decision与history transition必须可记录/重放。

### M8：准确的spatial AA与sharpen stage

实现真实FXAA与SMAA 1x；Area/Search LUT进入resource residency；CAS/RCAS作为独立stage。删除custom blur沿用正式算法名的路径。

### M9：真实MSAA architecture

sample count进入attachment、PSO key、depth/color resolve、forward/deferred、alpha-to-coverage、velocity、post-process、camera stack和UI；无硬件/路径支持时不得报告active。

### M10：Scene、Editor与scalability闭环

添加versioned render-quality asset/camera override、profile layering、undo/redo、multi-edit、save/reopen、cook validation、runtime preview、Problems与migration；普通项目入口必须能选择并复现effective mode。

### M11：Diagnostics、artifact与fault lifecycle

交付velocity/history/rejection/lock/coverage/DRS debug view、GPU timestamp、VRAM/bandwidth、provider context、cache/eviction和device-loss telemetry；artifact绑定source/scene/shader/provider/device manifest。

### M12：hard cut与竞争性产品gate

删除full-frame history equality、global history bit、optional ViewFamily、test-only particle descriptor、custom-SMAA同名路径、vendor core enum与descriptor-only MSAA。按同场景同质量对照Unreal TSR/TAA、Unity STP/HDRP、Godot FSR2、Bevy和Fyrox；未超过项继续保留为公开gap。

## 10. 验收资格门

| Gate | 验收要求 |
|---|---|
| G01 | 普通Scene/Camera可创建、保存、重开AA、DRS、provider、quality与sharpness设置 |
| G02 | Editor undo/redo、multi-edit、copy/paste、profile override与cook保留同一真值 |
| G03 | requested/eligible/admitted/prepared/active/degraded状态和reason唯一且可查询 |
| G04 | `ViewFamilySnapshot`是production submission必填值，scalar fallback为0引用 |
| G05 | display/render/output/history extent和rect origin在所有pass/shader一致 |
| G06 | baseline TAA所有MRT attachment尺寸、sample count和layer完全一致 |
| G07 | TAAU以明确input/output rect执行，不以混尺寸MRT隐式upscale |
| G08 | camera/mesh正常连续运动不会触发whole-frame history失效 |
| G09 | camera cut、projection jump、provider change和device generation会精确失效 |
| G10 | 每个history domain有独立generation、validity、reason和publication |
| G11 | failed frame不翻转history，last-good资源与metadata同generation发布 |
| G12 | camera删除、rename/rekey、viewport关闭与device teardown释放所有history/state |
| G13 | DRS resize按domain resample或discard，不整组无条件reset |
| G14 | current/previous matrices使用同一relative origin和明确jitter convention |
| G15 | velocity单位、方向、viewport、extent与provider motion scale有machine test |
| G16 | rigid、skin、morph、WPO和LOD transition拥有正确previous deformation |
| G17 | particle velocity由普通产品descriptor可达且不每帧重建CPU quad buffer |
| G18 | sprite、transparent、OIT、transmission、half-res与overlay声明motion资格 |
| G19 | reactive/composition覆盖mesh、VFX、translucency、emissive与specular变化 |
| G20 | missing velocity/reactive producer产生结构化degraded reason，不静默写零 |
| G21 | TAA读取previous depth/surface并通过真实disocclusion测试 |
| G22 | history filter、valid rect、UV clamp和odd resolution通过边界oracle |
| G23 | sample count、moments、confidence/lock与history weight可debug读取 |
| G24 | exposure/pre-exposure jump不产生闪烁、拖影或能量跳变 |
| G25 | thin geometry、foliage、specular、moire、translucency各有sequence metric |
| G26 | FXAA实现选定正式版本并通过edge orientation/endpoint/subpixel oracle |
| G27 | SMAA三阶段、Area/Search LUT、stencil、diagonal/corner和preset全部真实执行 |
| G28 | CAS/RCAS是可组合stage，feature off时exact baseline |
| G29 | provider ABI支持context create/destroy、availability、reset、fault和version |
| G30 | provider明确depth/motion/jitter/exposure convention与input/output resolution |
| G31 | FSR2/DLSS/XeSS/STP-like provider可在不扩core enum的情况下接入 |
| G32 | provider failure原子fallback且不发布半更新history或错误active receipt |
| G33 | DRS消费真实GPU timestamp，不用CPU submit time冒充GPU frame cost |
| G34 | DRS budget、headroom、hysteresis、change period、panic和min/max可配置可观测 |
| G35 | steady/spike/oscillation/multi-view场景满足scale稳定与p95/p99预算 |
| G36 | DRS decision trace可确定性重放并关联history transition |
| G37 | MSAA attachment、pipeline key、depth/color resolve sample count完全一致 |
| G38 | Forward/Deferred、alpha-to-coverage、velocity、post-process和UI MSAA边界明确 |
| G39 | 2D、camera stack、split viewport、stereo/XR与multiview合同通过 |
| G40 | debug views覆盖velocity、history、rejection、lock、coverage、reactive与DRS |
| G41 | GPU timing、VRAM、bandwidth、cache/eviction和provider context纳入capture receipt |
| G42 | device loss、OOM/pressure、shader compile和provider unavailable可恢复或fail closed |
| G43 | required-GPU lane无adapter时失败或输出受管skip receipt，exporter不得默认ignore |
| G44 | 同场景同质量对Unreal/Unity/Godot记录画质误差、GPU ms、VRAM、稳定性与provenance |

## 11. 测试与artifact判定

focused tests共11个文件、5,009行、42个`#[test]`和2个ignored exporter。它们对history invalidation、effective AA、velocity executor、reactive mask、particle order、motion blur和full-chain pass order有价值；同时也暴露出产品断链：测试经常手工构造`RenderFrameExtract`、feature descriptor、history和GPU资源，绕过Scene producer、Editor transaction、save/reopen与正常built-in graph。

静态空场景TAA测试只证明seed后相同帧可byte-stable，关闭history也可能满足部分断言；camera/mesh motion invalidation测试反而锁住了错误的whole-frame compatibility。MSAA测试证明descriptor可编译4 samples，却没有证明真实WGPU mesh/gbuffer/velocity pipeline和resolve匹配。particle velocity测试证明局部executor存在，不证明默认产品descriptor可达。

24个历史artifact中，16个velocity相关文件主要是2026-07-03/04命令日志与两张128x128图片；视觉复核显示图片几乎全黑，只有细薄青/洋红边缘。2个MSAA文件属于Hybrid GI current-frame post，6个temporal文件属于Hybrid GI/Volumetric；GI rejection图是小尺寸合成色块/三角形，volumetric图是狭窄网格体。它们最多是L1/L2 path evidence，不验证AA reconstruction。仓库没有声明的`render18_taa_reactive_mask_wgpu_20260813.png`。

新的产品artifact必须至少携带source fingerprint、scene/camera-path hash、shader hashes、GPU/driver/backend、output/render resolution、effective AA/provider/quality、history generation、GPU timestamps、VRAM/bandwidth、sequence metric、debug atlas和non-ignored test identity。单张PNG、changed pixel count、pass名字存在、CPU submit time或可打开RDC都不能独立关闭G01-G44。

## 12. 完成定义与退出条件

Runtime101只在以下条件同时成立时关闭：旧09H1的10项P0全部由其owner依据当前源码关闭；31项P1完成或存在被接受且有期限的明确降级；10项P2进入持续治理；G01-G44都有machine-readable证据；ViewFamily/History/Motion/Provider/DRS/Scene/Editor唯一owner完成hard cut；普通Scene与Editor入口可达；真实GPU fault、scale与连续sequence通过；与参考引擎的同场景同质量差异可复现。

在这些退出条件前，`implementation_status`保持`not_started`。当前基础可描述为“存在baseline组件和characterization tests”，不得从枚举、pass、shader、静态测试或早期artifact推导“完整”“production-ready”“TSR-equivalent”或“超过Unreal”。
