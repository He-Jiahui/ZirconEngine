---
related_code:
  - zircon_runtime/src/core/framework/render/post_process
  - zircon_runtime/src/asset/assets/scene/post_process.rs
  - zircon_runtime/src/asset/assets/texture/cube_lut.rs
  - zircon_runtime/src/asset/importer/ingest/import_cube_lut.rs
  - zircon_runtime/src/scene/components/scene/post_process.rs
  - zircon_runtime/src/scene/world/project_io/post_process.rs
  - zircon_runtime/src/scene/world/render_post_process.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/descriptor_filtering.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/resource_descriptors.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/resolve_history_handle.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/history
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors.rs
  - zircon_runtime/src/graphics/tests/render_product_post_process.rs
  - zircon_runtime/src/graphics/tests/render_product_post_process_full_chain.rs
  - zircon_runtime/src/graphics/tests/render_product_post_process_full_chain/visual_export.rs
  - zircon_runtime/src/graphics/tests/render_product_post_process_volume.rs
  - zircon_runtime/src/scene/tests/render_post_process_extract.rs
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/rendering/workbench_extension_post_process_workspace.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/render_asset_vfx.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/09h2-exposure-color-bloom-dof-motion-blur-ssr-terminal-review.md
  - docs/plans/optimize/zircon_runtime/99b-runtime-temporal-aa-velocity-history-dynamic-resolution-upscaling-reconstruction-product-integration-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/PostProcessEyeAdaptation.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/PostProcessLocalExposure.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/PostProcessCombineLUTs.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/PostProcessBloomSetup.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/PostProcessFFTBloom.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/DiaphragmDOF.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/PostProcessMotionBlur.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/PostProcessDeviceEncodingOnly.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ScreenSpaceRayTracing.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ScreenSpaceReflectionTiles.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/PostProcessing/Components
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/PostProcessing/Shaders
  - dev/godot/servers/rendering/renderer_rd/effects/bokeh_dof.cpp
  - dev/godot/servers/rendering/renderer_rd/effects/luminance.cpp
  - dev/godot/servers/rendering/renderer_rd/effects/tone_mapper.cpp
  - dev/godot/servers/rendering/renderer_rd/shaders/effects/screen_space_reflection.glsl
  - dev/godot/servers/rendering/renderer_rd/shaders/effects/screen_space_reflection_resolve.glsl
  - dev/bevy/crates/bevy_post_process/src/auto_exposure
  - dev/bevy/crates/bevy_post_process/src/bloom
  - dev/bevy/crates/bevy_post_process/src/motion_blur
  - dev/bevy/crates/bevy_core_pipeline/src/tonemapping
  - dev/Fyrox/fyrox-impl/src/renderer/bloom
  - dev/Fyrox/fyrox-impl/src/renderer/hdr
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime Exposure、Color、Tonemap、LUT、Bloom、DOF、Motion Blur、SSR、Output Transfer 与 Terminal Composition 当前源码工程化差距

## 1. 结论

当前后处理不是“完全占位”：typed stack、pass graph、真实曝光 compute、3D LUT upload/binding、DOF prepare、motion-vector tile/neighbor、SSR HZB/roughness pyramid/temporal resolve、camera-keyed history、camera stack terminal owner以及产品测试骨架都已存在。它们是后续 hard cut 的迁移基础，不能推倒回单一匿名 fullscreen pass。

但它仍不具备工程级 HDR/color/post-process 产品资格。当前 baked LUT 会在 tone curve 前把 scene HDR clamp 到 `[0,1]`；`TonemapOperator::None` 仍写固定 `Rgba8Unorm`；`Hdr10Pq` 与 `LinearExtended` 没有进入 shader、surface capability或metadata链；Scene save/load 会静默丢失 LUT、Blur、Motion Blur、DOF、SSR 与 Exposure；Editor 的 Post Process Workspace 是静态示例数据和 canned feedback；Motion Blur 的 shutter 双单位与 neighbor-max 主方向仍错误；SSR 最终仍以无 BRDF 的 `mix` 合成。

本轮还确认了一个旧09H2未登记的当前源码P0：DOF、Motion Blur、Blur、SSR/Fog先由分离pass写回，再被未屏蔽的uber重复执行；Bloom则在Bloom pass和uber中使用同一intensity两次，形成近似`intensity^2`响应。这不是画质偏好，而是确定的phase/ownership错误与额外带宽。

旧09H2的8项P0仍未全部关闭：P0-4的“全局history”部分已经由per-viewport-camera history handle修复，必须撤销旧描述；但固定`1/60`和缺少effect-specific timing/reset/provenance仍让该记录保持开放。其余7项仍由09H2唯一计数。Runtime102新增1项P0，另登记40项P1、10项P2与44项资格门。任何“完整HDR”“ACES compliant”“cinematic DOF”“physically based SSR”“production-ready postprocess”或“超过Unreal”的描述目前都不成立。

## 2. 审查边界、currentness与证据

### 2.1 冻结语料

| 范围 | 文件 / 行 / 非空行 / bytes / test attributes / ignored | 证据等级 | fingerprint |
|---|---:|---|---|
| Zircon production review slice | 270 / 33,972 / 31,610 / 1,334,276 / 223 / 0 | E3主链逐段读取；E2支持文件清单与调用扫描 | `2ecab8bc899a17a12711c9a8885c4c0fa213b457e6ebe6ed2bb27c3e71398b90` |
| dedicated relevant tests | 34 / 10,658 / 10,039 / 389,520 / 108 / 1 | E2/E3断言分类；未执行 | `37f6cd44b7ee8b1e79668a798728d38c89bdce622007c5f35422a1961b36a929` |
| existing relevant artifact | 67 / 48,982,850 bytes | E1旧日志、1 PNG、8 RDC；不绑定当前源码 | 09H2集合未变化 |
| Unreal reference slice | 10 / 10,719 / 8,825 / 464,839 | E3 | `cb772ec9e7071b6224d14f0e6b7eb78f3036328273481b33912e7bd2497e074b` |
| Unity HDRP reference slice | 85 / 11,567 / 9,589 / 445,231 | E3/E2 | `7bf8beb330782240eca271f5ad2f6c0ee4dbffaebf9dbce78ec2191c512d1ec7` |
| Godot reference slice | 5 / 1,590 / 1,261 / 80,487 | E3 | `656ab83cb3b08cdac520096896a46315b90f56129fee045baa6a08331c437d57` |
| Bevy reference slice | 19 / 3,924 / 3,501 / 148,222 | E3 | `ccd493a959ad95bd402c852cf5c95ebdddc31c0d3d3248f15bf39c8dde072465` |
| Fyrox reference slice | 7 / 998 / 890 / 35,462 | E3 | `d215d71ee63aaea43e2859605cc7b84a173e64b7b7475649d0635418e279db18` |
| combined reference slice | 126 / 28,798 / 24,066 / 1,174,241 | E3/E2 | `25004219a1a78b7e3cd0ae76519a3c7fdff60e217e294aa6159ad50aa0374229` |

fingerprint算法为：路径排序后，对每个working-tree文件计算SHA-256，再对UTF-8 `path<TAB>hash<LF>` manifest计算SHA-256。冻结对象是2026-08-22共享working tree，基线HEAD为`be5a281c96b6dc9d33b5c9d0f2699a8bf75afcf1`，coordinator epoch为336；不是只读HEAD快照，进入实现前必须重取指纹并复核所有结论。

Bevy、Fyrox、Godot与Unity Graphics revision分别为`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`与`a7e4c051d256a781ab362c64316b125a1e104694`。Unreal目录不是独立Git checkout，因此只使用上述10文件及manifest fingerprint，不伪造revision。

### 2.2 数据链读取深度

本轮沿`Scene/Volume asset -> project_io -> runtime settings/registry -> camera resolve -> PostProcessStackDescriptor -> descriptor filtering -> graph resource -> executor -> WGSL -> history/readback/stats -> product tests/artifact -> Editor workspace`读取。对270文件的完整owner边界做清单和调用扫描；对颜色、曝光、效果、终端、Scene持久化、Editor入口与对应shader/executor做逐段E3读取。Runtime101已拥有TAA/velocity/DRS/upscale主问题，本报告只记录它们对后处理输入/history/output的交叉影响，不重复计数。

### 2.3 明确未做

本轮是review-only，没有修改Rust、Cargo、shader、asset、Editor或tooling；没有运行Cargo、WGPU、Editor、RenderDoc、参考引擎、真实HDR monitor或GPU profiler，也没有重导出artifact。没有执行HDR ramp、PQ/scRGB、camera exposure transition、LUT domain chart、Bloom firefly、foreground DOF、fast thin-object motion blur、rough/offscreen SSR、split viewport、camera stack、4K/8K、stereo/XR、device loss、VRAM pressure或同画质benchmark。tooling按用户要求排除，后续将迁移Rust。

## 3. 当前产品链与可保留基础

### 3.1 typed stack与显式pass graph可以保留

`PostProcessEffectKind`、chain slot、resource name、required/produced inputs和executor id已形成可分析图。Exposure、Bloom、DOF、Motion Blur、SSR、Scene Composite、LUT Bake、Uber、Upscale与Output Transfer不是字符串脚本临时拼接。重构应收紧stage ownership与color domain，不能退回隐式大pass。

### 3.2 Exposure compute与per-camera history是实质进展

Histogram和resolve是实际compute pipeline，previous/current exposure buffer可翻转并有readback report。`ViewportRecord.camera_histories`按camera/target key保存history，renderer以`FrameHistoryHandle`索引，因此09H2“所有camera共享全局曝光history”的描述已过时。保留这一身份模型，再把Exposure从巨型history bundle中拆成typed consumer lifecycle。

### 3.3 3D LUT已进入真实资源链

`.cube`可导入D3 texture，streamer能准备3D view，shader使用线性sampler；baked LUT也是`rgba16float` 3D storage texture。旧报告若把3D LUT描述成“不可绑定”已不再成立。当前缺口是domain、precision、schema与quality，而不是D3资源完全不存在。

### 3.4 SSR不是占位shader

当前SSR shader包含scene reflection pyramid、coarse pyramid、HZB trace、4步refine、roughness mip选择、temporal history与specular occlusion。应保留这些分层资源和debug入口；问题在ray model、temporal validation、material BRDF、fallback与budget，不应误判为零实现。

### 3.5 camera stack terminal owner已经接入

每个camera会解析post-process stack与history handle，terminal node由stack/output policy决定，success-only history更新也已存在。报告不再重复旧“没有camera terminal owner”的结论；HDR UI composition与Overlay自身authoring truth仍需补齐。

### 3.6 resource readiness与readback report是可迁移骨架

Color LUT missing/stale状态、Exposure readback、effect-stack report和graph execution stats提供了诊断入口。它们目前未形成统一per-effect timing/quality/degraded reason，也没有被Editor工作区消费，但不应删除。

## 4. 五引擎参考对照

### 4.1 Unreal：颜色、曝光和输出是同一display pipeline

`PostProcessEyeAdaptation.cpp`消费真实`DeltaWorldTime`并连接histogram、compensation与pre-exposure；`PostProcessLocalExposure.cpp`有独立局部曝光阶段；`PostProcessCombineLUTs.cpp`同时消费working color space和output device；`PostProcessDeviceEncodingOnly.cpp`按display output format构建真实编码分支。Zircon当前把curve、LUT与copy-only terminal分开暴露enum，却没有共同的Display/Color plan。

### 4.2 Unreal：DOF、Motion Blur与SSR按work classification扩展

`DiaphragmDOF.cpp`提供resolution divisor、CoC tile/dilate、foreground/background gather/scatter/recombine；`PostProcessMotionBlur.cpp`区分velocity flatten、tile、scatter/half-res gather与quality；`ScreenSpaceRayTracing.cpp`和`ScreenSpaceReflectionTiles.cpp`有roughness/quality/tile/denoiser路径。工程差异不在“kernel更多”，而在Zircon没有work classification、预算、material/fallback和可观测quality identity。

### 4.3 Unity HDRP：模块化stage是产品下限

HDRP 85文件切片明确分开Exposure、Bloom Prefilter/Blur/Upsample、DOF CoC/TileMax/Dilate/Mip/Prefilter/Gather/Combine、Motion Vector Prep/Tile Merge/Neighborhood/Filter、LUT Builder与UberPost。Zircon当前分离pass存在，但又在uber重复执行，说明“有多个pass名”不等于stage ownership正确。

### 4.4 Godot：轻量实现仍保留层级和材质邻域

Godot SSR沿mip/cell推进并在resolve读取depth、normal、roughness与mip；Bokeh DOF、luminance reduction和tone mapper各有独立owner。即使不追求Unreal的全部档位，Zircon也不能以neighbor-max reprojection、scene RGB clamp和无BRDF mix作为工程终点。

### 4.5 Bevy与Fyrox：baseline也比当前部分临时实现完整

Bevy auto exposure已有metering mask和compensation curve，Bloom明确使用downsample/upsample mip与soft threshold，Motion Blur把`0.5`定义为180度的frame fraction。Fyrox把HDR luminance/adaptation与Bloom blur拆开。metering mask、mip Bloom和唯一shutter单位不是大型商业引擎专属要求。

## 5. P0当前源码重验

### Runtime102-P0-01：baked LUT仍在tone curve前截断HDR domain（继承09H2-P0-1）

`color_lut_bake.wgsl`只在归一化`[0,1]^3`格点烘焙曝光、tone、grading；uber的baked模式直接以scene value采样，`sample_effect_lut_3d`先clamp输入。大于1的高光在curve前折叠，negative/over-range和wide-gamut也没有定义。必须引入明确scene-linear到shaper domain变换，或把tone/output留在analytic stage。

### Runtime102-P0-02：`TonemapOperator::None`仍把HDR写入固定8-bit LDR（继承09H2-P0-2）

`TONEMAPPED`与`UPSCALED`固定为`Rgba8Unorm`，`FINAL_COMPOSITED`固定为`Rgba8UnormSrgb`；None又是默认operator。None因此不是“保留scene linear”，而是隐式clip/quantize。默认SDR必须选择合法tone/output transform；真正None只允许写可表达HDR/linear的目标。

### Runtime102-P0-03：`Hdr10Pq`与`LinearExtended`仍是假能力面（继承09H2-P0-3）

`RenderOutputTransfer`只存在于neutral类型和测试。35行`output_transfer.wgsl`只做integer load/copy，uniform只有viewport origin；executor没有transfer、gamut、paper white、peak nits或metadata参数，最终target仍是sRGB8。surface capability、swapchain format、EOTF/gamut和fallback必须由同一个Output Device plan决定。

### Runtime102-P0-04：Exposure全局history已修复，但固定时间步与consumer lifecycle仍错误（09H2-P0-4部分修复）

当前history handle已按viewport-camera隔离，旧“全局history串camera”结论撤销。可是`EXPOSURE_ADAPTATION_DELTA_SECONDS`仍硬编码`1/60`，30/60/144Hz与暂停/慢放会产生不同时间常数；Exposure buffer仍住在广义`SceneFrameHistoryTextures`，缺少独立validity、settings signature、metering source、cut/reset reason与frame timestamp。该旧记录不能关闭，只能标记partial progress。

### Runtime102-P0-05：Scene/Volume save-load仍静默丢失运行时效果（继承09H2-P0-5）

runtime effect stack包含Tonemap、Color Lookup、Blur、Motion Blur、DOF、SSR、Vignette、Grain、Dither、Chromatic与Fog；`ScenePostProcessEffectStackAsset`只保存Tonemap/Vignette/Grain/Dither/Chromatic/Fog。`project_io`用default补回缺失字段并在保存时再次丢弃。`PostProcessSettingsComponent`也没有Exposure字段。必须hard cut到versioned typed component/profile schema，未知component和resource reference可round-trip。

### Runtime102-P0-06：Motion Blur shutter unit与最终velocity source仍冲突（继承09H2-P0-6）

字段名是`shutter_angle`，调用与测试同时出现`0.5/1.0`和`90.0`；upload不除以360，shader将其直接作为fraction。最终gather的中心方向与每个sample方向都读取`MOTION_VECTOR_NEIGHBOR_MAX`，不是中心surface velocity，静态背景会被邻近高速前景拖拽。必须用typed shutter fraction/angle/exposure time之一，并分离per-pixel direction与tile envelope。

### Runtime102-P0-07：SSR最终合成仍没有material BRDF与能量合同（继承09H2-P0-7）

trace虽增强，但`apply_scene_composite`仍执行`mix(scene, resolved.rgb, alpha)`。shader只读取roughness，不读取metallic/specular/F0/Fresnel；没有probe/planar/RT/environment fallback的统一indirect-specular owner。硬编码visibility比例不能代替BRDF。应由Indirect Specular Compositor接收radiance、confidence、hit/fallback provenance和material lobes。

### Runtime102-P0-08：现有artifact仍不能证明当前颜色与效果正确（继承09H2-P0-8）

67个artifact仍是58份2026-07-04/05日志、1张旧PNG和8个旧RDC，共48,982,850 bytes。ignored exporter声明的`20260801_cold.png`、`20260801.png`与`20260801.json`不存在。已有文件没有本轮source fingerprint、HDR/EXR、settings、GPU/driver、reference metric或完整sequence，因此只能证明某个旧二进制曾运行。

### Runtime102-P0-09：分离效果pass与uber重复执行，Bloom intensity也被重复应用（新增）

stack在effect stack启用时同时生成DOF、Motion Blur、SSR Resolve/Scene Composite、Blur和Uber。分离executor只局部清零参数：DOF只清零general blur，Blur只清零DOF；Motion Blur、Scene Composite和Uber继续使用原始effect stack。`fs_main`再次调用blur/DOF、motion blur和scene composite，所以这些效果被处理两次。Bloom pass已经返回`bloom * intensity`，uber又执行`color + bloom * blends.w`，而`blends.w`来自同一Bloom intensity，响应近似平方。必须定义每个effect唯一执行owner，并在编译stack时生成stage-local参数，而不是靠executor临时mask。

## 6. P1工程差距

1. **Runtime102-P1-01**：Histogram以64 bin全分辨率均匀采样每个pixel，没有metering mask、center/spot/ROI或downsample work plan。
2. **Runtime102-P1-02**：Exposure没有ISO、shutter speed、aperture、calibration constant、pre-exposure与physical camera联动。
3. **Runtime102-P1-03**：没有compensation curve、local exposure、highlight/shadow detail balance与局部阶段debug。
4. **Runtime102-P1-04**：Exposure缺少独立history signature、settings change/camera cut reset、真实delta和pause/time-domain合同。
5. **Runtime102-P1-05**：名为`Aces`的2.51/0.03/2.43/0.59/0.14逐通道拟合不是完整ACES RRT/ODT或ACES 2 pipeline。
6. **Runtime102-P1-06**：没有明确working color space、chromaticities、white adaptation、gamut compression和output gamut transform。
7. **Runtime102-P1-07**：Color Grading只有exposure、contrast、saturation、gamma、tint，缺少white balance、CDL/range controls、curves与versioned order。
8. **Runtime102-P1-08**：2D strip LUT使用nearest `textureLoad`，没有trilinear/tetrahedral oracle、layout validation与precision contract。
9. **Runtime102-P1-09**：`.cube`忽略`DOMAIN_MIN/MAX`和input range、拒绝1D shaper、把finite sample clamp/quantize到RGBA8 UNORM。
10. **Runtime102-P1-10**：baked LUT固定32³；64³常量只被导出/测试，未进入resource descriptor、quality tier、cache key或budget。
11. **Runtime102-P1-11**：Bloom为全分辨率单pass固定5x5/25-load，radius只改变stride，没有mip pyramid和稳定的屏幕空间radius。
12. **Runtime102-P1-12**：Bloom使用`sample_color * max(luminance-threshold,0)`，亮度增长超线性且无soft knee、anti-firefly或energy normalization。
13. **Runtime102-P1-13**：Bloom没有scatter、anamorphic、dirt、convolution/FFT、lens asset、temporal stability和quality/budget档。
14. **Runtime102-P1-14**：DOF CoC仍是focus-distance/range启发式，不是由focal length、f-number、sensor与projection定义的thin-lens模型。
15. **Runtime102-P1-15**：DOF prepare和gather全分辨率，固定环形samples并在shader硬限约12px，没有tile/indirect/adaptive work reduction。
16. **Runtime102-P1-16**：DOF near/far、foreground dilation、occlusion-aware recombine、transparent/translucent与alpha语义不完整。
17. **Runtime102-P1-17**：blade count/rotation只是轻量形状参数，没有cat-eye、chromatic bokeh、scatter highlight energy与quality tier。
18. **Runtime102-P1-18**：Motion Vector tile-max只做2x2降采样、再粗化到quarter、最后3x3扩张，缺少depth/min-max和surface classification。
19. **Runtime102-P1-19**：Motion Blur只有固定1..32 sample count，没有camera/object separation、pixel radius budget、half-res/scatter或adaptive quality。
20. **Runtime102-P1-20**：Motion Blur depth rejection固定`smoothstep(0.01,0.05)`，没有projection/unit、velocity-depth envelope与薄物体策略。
21. **Runtime102-P1-21**：SSR已有HZB和refine，但没有material tile classification、indirect dispatch、adaptive step/ray budget与hit/reject counters。
22. **Runtime102-P1-22**：rough SSR仍追单条perfect reflection；roughness只选择pyramid mip，没有stochastic GGX、多ray或spatial denoiser。
23. **Runtime102-P1-23**：SSR temporal用neighbor-max做reprojection，并用当前scene RGB 3x3 min/max clamp history；没有SSR moments、depth/normal/material disocclusion。
24. **Runtime102-P1-24**：SSR visibility硬乘`0.18`并封顶`0.35`，这是画面压暗启发式，不是confidence、BRDF或energy policy。
25. **Runtime102-P1-25**：SSR miss/offscreen/thin geometry没有显式probe/planar/RT/IBL fallback provenance与连续过渡。
26. **Runtime102-P1-26**：General Blur复用DOF gather family和全套绑定，缺少separable/Gaussian/Kawase等准确算法身份与成本模型。
27. **Runtime102-P1-27**：Film Grain与Dither使用静态、单通道相关procedural噪声，没有frame sequence、blue noise、bit-depth/output-aware dither。
28. **Runtime102-P1-28**：Chromatic Aberration只是水平R/B整数偏移；screen fog又与真实volumetric fog形成重复且不守物理domain的轻量路径。
29. **Runtime102-P1-29**：Local Volume只可靠支持Box/Sphere，Capsule/Cylinder/Convex/TriangleMesh/HeightField/Compound会退化或不投影，用户诊断不足。
30. **Runtime102-P1-30**：runtime registry的typed override没有等价的持久化resource parameter、schema migration与unknown component round-trip。
31. **Runtime102-P1-31**：HDR terminal之后没有UI/debug overlay的reference white、gamut/EOTF、alpha/composition plane与capture policy。
32. **Runtime102-P1-32**：没有SDR/PQ/scRGB/headless/scene-linear EXR的target negotiation、metadata与unsupported fallback矩阵。
33. **Runtime102-P1-33**：没有统一scalability profile、per-effect GPU/VRAM/bandwidth预算、降级顺序和effective quality report。
34. **Runtime102-P1-34**：多个分离pass每帧创建bind group和临时params buffer，缺少frame parameter arena、descriptor cache与generation reuse。
35. **Runtime102-P1-35**：本组render/compute pass普遍`timestamp_writes: None`，无法得到effect级GPU成本和预算闭环。
36. **Runtime102-P1-36**：约910行uber shader与约854行SSR source被组合给多个pipeline，形成monolithic coupling、重复编译和variant膨胀风险。
37. **Runtime102-P1-37**：Bloom/DOF等固定全分辨率资源与split+uber重复读写放大transient bytes，格式/extent缺少quality-dependent plan。
38. **Runtime102-P1-38**：已有readback/report未统一携带requested/effective/degraded reason、history generation、resource bytes、GPU timing与source hash，Editor也不消费。
39. **Runtime102-P1-39**：测试主要断言pass顺序、resource backing、changed pixel/RGB sum；无法识别HDR clamp、错误curve、DOF occlusion、motion bleed或SSR energy。
40. **Runtime102-P1-40**：缺少settings change、missing/stale LUT、resize、camera cut、device loss、OOM/VRAM pressure与shader/pipeline failure的连续fault矩阵。

## 7. P2产品与治理差距

1. **Runtime102-P2-01**：缺少lens distortion、panini、lens flare、sensor dirt与统一lens pipeline。
2. **Runtime102-P2-02**：缺少display calibration、ICC/EDID-like capability、creative/technical LUT分层与project color policy。
3. **Runtime102-P2-03**：custom post-process injection point没有与typed Volume/Profile、resource lifetime、security和plugin ABI统一。
4. **Runtime102-P2-04**：Editor没有live histogram、waveform、vectorscope、false color、gamut warning、LUT preview与A/B compare。
5. **Runtime102-P2-05**：缺少per-effect debug atlas、intermediate capture、ray rejection view、tile occupancy与pass disable isolation。
6. **Runtime102-P2-06**：缺少asset schema version migration、unknown component/resource round-trip和old-project batch upgrade工具链合同。
7. **Runtime102-P2-07**：缺少orthographic、2D、split-screen、stereo/XR、foveated与multi-display的effect/output边界。
8. **Runtime102-P2-08**：缺少deterministic offline color reference、scene-linear EXR、CPU oracle与高精度capture路径。
9. **Runtime102-P2-09**：缺少同场景、同输出、同quality target下与Unreal/Unity/Godot/Bevy/Fyrox的可复现质量性能基线。
10. **Runtime102-P2-10**：Post Process Editor没有事务/undo、asset picker、multi-selection、override visualization、sequence authoring与cook validation闭环。

## 8. Scene、Editor、诊断与产品真值

### 8.1 Scene asset与runtime registry不是同一schema

asset固定struct与runtime registry已经发生结构漂移。当前load通过default掩盖缺字段，导致项目看似能打开、实则保存后永久丢authoring。目标不是继续向固定struct追加字段，而是`component id + schema version + enabled/override state + typed payload/resource refs`；builtin与plugin走同一serialization/migration路径，未知payload保留。

### 8.2 Editor工作区是静态演示，不是authoring产品

`.zui`硬编码`Bloom 0.65`、`Filmic exposure +0.4`、`LUT_CityWarm 33 cube`和`Interior volume EV +2.1`。navigation只注册action id；feedback返回`Preview queued`、`Apply queued`等固定字符串。未发现这些action对Scene component、Volume profile、asset transaction、undo/redo或runtime snapshot的真实读写。因此Editor UI存在不等于功能可用，必须接唯一Scene/Profile truth和transaction service。

### 8.3 failure与diagnostics没有形成闭环

builtin shader validation失败会在pipeline构造阶段panic；missing LUT有fallback/report，但Editor无资源修复入口；所有effect没有统一requested/effective/degraded reason。需要把startup invariant、hot-reload failure、resource stale、unsupported target和device loss区分为typed fault，定义last-known-good、disable-effect、fallback-output与用户可操作恢复路径。

## 9. 目标架构与owner边界

### 9.1 `ColorPipelinePlan`

由neutral framework定义scene-linear working space、pre-exposure、grading domain/order、tone mapper identity/version、output gamut/EOTF/luminance、alpha与capture domain。graphics只实现已解析plan；surface/capture提供capability，不能由shader enum假装支持。

### 9.2 `ExposureService`与typed history

输入真实frame delta、camera physical settings、metering source/mask、compensation curve和cut/reset event；输出pre-exposure/current EV/local exposure及provenance。history按consumer schema、extent、settings signature和generation管理，不再借一个广义valid bit。

### 9.3 `PostProcessComponentRegistry`与versioned profile asset

registry拥有parameter schema、default、sanitize、interpolation、resource dependency、editor metadata和migration；Scene asset保存component records与unknown payload。Editor、runtime、cook和plugin只读写这一真值，不保留fixed six-field兼容shim。

### 9.4 `PostProcessExecutionPlanner`

把resolved components编译为唯一stage DAG和stage-local params，明确scene-linear/display-referred域、唯一执行owner、resolution tier、history/resource需求和fallback。split pass与uber只能互补，不能重复执行同一effect。

### 9.5 scalable effect services

Bloom、DOF、Motion Blur和SSR各自拥有classification、quality tier、work budget、resource plan、timestamps与debug output。低档可选择轻量算法，但名称、质量身份和降级原因必须准确。

### 9.6 `IndirectSpecularCompositor`

SSR、planar、probe、IBL和hardware RT输出统一radiance/confidence/provenance；compositor结合material F0/Fresnel/roughness/visibility合成并保证能量合同。SSR不再直接mix scene color。

### 9.7 `OutputDeviceService`与`PostProcessAuthoringService`

Output owner协商surface/capture format、gamut、EOTF、luminance、metadata、UI reference white和fallback；Authoring owner通过transaction/undo、asset picker、override schema、live diagnostics和cook validation编辑同一profile truth。

## 10. 依赖顺序与重构里程碑

### M0：冻结characterization与重复执行测试

锁定本报告fingerprint，增加split-vs-uber、Bloom intensity响应、HDR ramp、None tone、output enum false-surface和Scene round-trip characterization；修复前不得更新golden掩盖缺陷。

### M1：冻结color/output contract

定义working space、pre-exposure、grading/tone/LUT顺序、output gamut/EOTF/luminance和capture domain，删除语义不明的None/HDR enum路径。

### M2：hard cut `ColorPipelinePlan`与Output Device

接入surface/capture capability，完成SDR、PQ、scRGB、headless和EXR plan；unsupported选择必须显式失败或降级并报告原因。

### M3：修复HDR LUT domain

引入shaper transform或把tone留在analytic stage；`.cube`支持domain、float precision和可选1D shaper；32/64与cache/budget由quality plan决定。

### M4：Exposure真实时间与独立history

传入真实delta，按camera consumer管理validity/reset/signature；完成30/60/144Hz、pause、cut、resize和settings-change序列。

### M5：Exposure product能力

增加metering mask/modes、physical camera、compensation curve、pre-exposure与local exposure，并提供readback/debug/Editor闭环。

### M6：Profile schema hard cut

以versioned component records替换固定Scene effect struct；所有builtin、plugin、resource ref和unknown payload通过save/reload/cook/migrate。

### M7：修复execution ownership

planner生成stage-local参数，消除DOF/Motion/Blur/SSR/Fog重复执行和Bloom双intensity；用exact baseline、effect-on和组合测试验证phase。

### M8：Scalable Bloom与DOF

Bloom实现soft-knee mip pyramid/scatter/anti-firefly；DOF实现physical CoC、tile/dilate、near/far gather/scatter/recombine与透明策略。

### M9：Motion Blur contract与filter

唯一shutter单位，per-pixel direction与tile envelope分离，加入depth classification、adaptive samples与camera/object/skin/particle动态序列。

### M10：Hierarchical rough SSR与denoiser

增加tile classification、adaptive hierarchical trace、stochastic rough rays、moments/depth-normal temporal rejection和debug counters。

### M11：Indirect specular integration

SSR/planar/probe/IBL/RT按material BRDF和confidence合成，明确miss/offscreen/fallback与history provenance。

### M12：Quality、performance与fault lifecycle

所有effect提供requested/effective tier、GPU timestamp、resource/history bytes、budget降级、device loss与last-known-good恢复。

### M13：Editor authoring与diagnostics

静态workspace替换为Scene/Profile transaction、undo、asset picker、override状态、scope/debug view和cook validation；删除canned apply/preview假反馈。

### M14：artifact与竞争性产品gate

按当前fingerprint导出SDR/HDR/EXR sequence、RDC、GPU profile、graph/resource manifest和reference metrics；完成同场景同画质基线后才允许具体“优于Unreal”结论。

## 11. 验收资格门

| Gate | 资格条件 |
|---|---|
| G01 | scene-linear HDR ramp在tone/LUT前保留大于1、negative与wide-gamut值，误差有CPU/reference上限 |
| G02 | 默认SDR path总有合法tone/output transform，None不写无法表达其domain的target |
| G03 | baked与analytic color path在声明domain内一致，LUT sample不发生pre-tone clamp |
| G04 | `.cube` domain、1D shaper、3D order、float precision、invalid input与round-trip通过 |
| G05 | 32/64 LUT quality、cache key、async bake、stale generation和memory budget可观测 |
| G06 | SDR sRGB output的EOTF、gamut、alpha和capture与reference vector一致 |
| G07 | HDR10 PQ的Rec.2020/P3 mapping、paper white、peak nits、metadata与surface capability闭合 |
| G08 | scRGB/linear extended只在真实支持的surface/capture上激活，unsupported有明确fallback |
| G09 | UI/overlay在SDR/HDR具有reference white、gamut、alpha和composition policy |
| G10 | scene-linear EXR/headless capture不经过8-bit terminal clip |
| G11 | Exposure在30/60/144Hz真实delta下时间常数一致 |
| G12 | camera cut、pause、resize、settings change和history eviction有typed reset reason |
| G13 | 多viewport/多camera Exposure无串扰，history generation与settings signature可追踪 |
| G14 | manual、histogram、mask/spot/center、physical与compensation curve有reference序列 |
| G15 | pre-exposure与local exposure在bright/dark transition中稳定且无TAA/history冲突 |
| G16 | Scene component与global/local profile保存所有builtin effect和Exposure |
| G17 | unknown/plugin component及resource ref可save/reload/cook/migrate且不丢payload |
| G18 | Editor transaction/undo、multi-edit、override状态与runtime resolved值同真值 |
| G19 | 每个effect只有一个执行owner，split pass与uber组合无重复处理 |
| G20 | Bloom强度为单次、可预测响应，组合测试不会变成intensity平方 |
| G21 | Bloom soft-knee、firefly、small/large radius与mip reconstruction通过HDR reference |
| G22 | Bloom quality tiers满足GPU ms、transient bytes与能量/半径误差预算 |
| G23 | DOF CoC与physical lens reference一致，perspective/orthographic边界明确 |
| G24 | DOF near/far occlusion、foreground dilation、highlight energy和transparent策略通过 |
| G25 | DOF tile/gather/scatter/recombine occupancy与质量档满足预算 |
| G26 | Motion Blur shutter单位唯一，angle/fraction/exposure time不存在混用 |
| G27 | Motion Blur使用中心surface velocity定方向，tile envelope不拖拽静态背景 |
| G28 | camera/object/skin/morph/particle与fast thin edge动态序列通过 |
| G29 | Motion Blur depth rejection和adaptive samples在不同projection/resolution下稳定 |
| G30 | SSR trace有material tile classification、层级推进、ray/step budget和hit counters |
| G31 | rough SSR采用roughness-aware sampling/denoise，而不是单ray+mip伪装 |
| G32 | SSR temporal用pixel velocity、depth/normal/material/moments拒绝disocclusion |
| G33 | SSR smooth/rough metal/dielectric的F0/Fresnel/energy与reference一致 |
| G34 | offscreen/miss/thin geometry与probe/planar/IBL/RT fallback连续且带provenance |
| G35 | Grain/dither按frame和output bit depth稳定，chromatic/fog算法身份准确 |
| G36 | Volume shape、priority、blend、camera stack与unsupported geometry有可见诊断 |
| G37 | 每个effect报告requested/effective/degraded reason、resource/history bytes与generation |
| G38 | 每个GPU stage有timestamp，4K/8K与quality tier满足明确frame budget |
| G39 | bind group/params/pipeline缓存无无界增长，warm frame无不必要创建和编译 |
| G40 | shader/pipeline/LUT失败、OOM、device loss和stale resource走typed fault与恢复 |
| G41 | feature-off exact baseline、registered-empty inert、active visible effect均自动验证 |
| G42 | full-chain动态sequence覆盖曝光、Bloom、DOF、Motion、SSR、TAA、Fog、UI与camera stack |
| G43 | artifact携source/scene/shader hash、GPU/driver/backend、settings、metrics、EXR/PNG/RDC/profile |
| G44 | 同场景同output同quality benchmark可复现；未超过参考引擎的项目继续公开为gap |

## 12. 测试与artifact判定

34个dedicated相关test文件共10,658行、108个`#[test]`和1个ignored exporter。非ignored测试能证明stack编译、executor可达、resource backing、readback可解析与单帧输出发生变化；不能证明color science、temporal stability、physical lens、BRDF energy或同画质性能。production slice另含大量inline structural/string tests，其中部分只断言shader包含函数名或pass顺序，不能升级为pixel/reference evidence。

唯一full-chain exporter仍被`#[ignore]`，写出的目标固定为`20260801` PNG/JSON并要求RenderDoc注入。本轮检查时这些目标不存在。当前目录中1张旧PNG和8个旧RDC来自2026-07-30的Render17/PFM1链，58份相关日志来自2026-07-04/05；它们既不绑定本轮fingerprint，也没有HDR、EXR、reference vector和effect级GPU timestamp。因此本轮动态状态为unknown，不把旧pass或可打开capture记作当前产品验收。

## 13. 完成定义与退出条件

Runtime102只有在G01-G44全部由当前source fingerprint自动证据关闭，并且以下hard cut完成后才能从`pending`改为完成：旧normalized HDR LUT、None-to-LDR、copy-only HDR output、fixed Scene effect struct、fixed-1/60 Exposure、shutter双单位、neighbor-only motion direction、scene-mix SSR、split/uber重复执行和Bloom双intensity均不得保留compatibility shim。

还必须由独立color-science、render architecture、visual quality、performance、Scene/asset、Editor UX和fault-lifecycle review确认无Critical/Important遗留。达到这些条件前，capability、Editor、文档、plugin manifest和release note必须准确标注unsupported/degraded状态，不得以类型名、pass名、单帧pixel delta或旧artifact宣称工程化完成。
