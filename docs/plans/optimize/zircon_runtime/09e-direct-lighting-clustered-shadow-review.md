---
related_code:
  - zircon_runtime/src/scene/components/scene/lighting.rs
  - zircon_runtime/src/asset/assets/scene/lighting.rs
  - zircon_runtime/src/scene/world/render/lights.rs
  - zircon_runtime/src/core/framework/render/light
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow
  - zircon_runtime/src/graphics/visibility/view_context/build_views.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/clustered_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/computed_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_shading_standard_pbr.wgsl
  - zircon_plugins/rendering/features/contact_shadow
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/performance/01/2026-07-18-graphics-lighting-static-review.md
  - docs/plans/performance/01/2026-07-18-graphics-shadow-static-review.md
  - docs/plans/zircon_runtime/render/05-lighting-shadows.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/LightGridInjection.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/LightRendering.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ShadowSetup.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ShadowDepthRendering.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/VirtualShadowMaps/VirtualShadowMapArray.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/VirtualShadowMaps/VirtualShadowMapCacheManager.h
  - dev/bevy/crates/bevy_light/src/directional_light.rs
  - dev/bevy/crates/bevy_light/src/point_light.rs
  - dev/bevy/crates/bevy_light/src/spot_light.rs
  - dev/bevy/crates/bevy_light/src/cluster
  - dev/bevy/crates/bevy_pbr/src/cluster
  - dev/bevy/crates/bevy_pbr/src/contact_shadows.rs
  - dev/bevy/crates/bevy_pbr/src/render/light.rs
  - dev/godot/servers/rendering/renderer_rd/cluster_builder_rd.cpp
  - dev/godot/servers/rendering/renderer_rd/storage_rd/light_storage.cpp
  - dev/godot/servers/rendering/renderer_rd/shaders/area_lights_inc.glsl
  - dev/Fyrox/fyrox-impl/src/renderer/light.rs
  - dev/Fyrox/fyrox-impl/src/renderer/shadow/csm.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/LightLoop/LightLoop.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/Shadow/ContactShadows.compute
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Material/LTCAreaLight/LTCAreaLight.hlsl
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/ForwardLights.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/Passes/AdditionalLightsShadowAtlasLayout.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 09E · Direct Lighting / Clustered Light Grid / Shadow 工程化差距

## 1. 结论

Zircon并非完全没有直接光照和阴影基础。当前已有稳定排序的方向光、点光、聚光与矩形光snapshot，128-byte `GpuLightData` ABI，GPU Scene增量light upload，CPU z-bin/tile bitmask grid，保留上一帧slot并带generation/preemption的shadow atlas allocator，方向光cascade分割与texel snapping，点光六面/聚光shadow view，PCF采样，shadow visibility view，以及能够区分static caster revision的`ShadowCache` policy kernel。这些局部实现可作为重构输入，不能被另起一套系统全部抛弃。

但产品链尚未形成工程级闭环。场景组件和scene asset根本没有shadow字段，world extract又对四类光全部硬编码`shadow: None`，所以普通项目资产无法启用已有shadow path；现有shadow product tests通过手工构造render snapshot绕过了这条断链。light layer只写入GPU结构却没有任何直接光照或shadow caster shader消费；`strength`与`normal_bias`也只被打包而未进入最终采样。`RenderLightReadinessReport`仍把全部方向光、点光和聚光按数量直接标为ready，无法反映这些失效字段、atlas拒绝、grid截断或pipeline/device失败。

默认Forward+和Deferred profile声明的“Clustered Lighting”不是实际的cluster light assignment。产品先在CPU构造真正被材质消费的z-bin/tile mask并`queue.write_buffer`，随后另一个标成`AsyncCompute`的`zircon-cluster-pipeline`只读取固定上限的方向光，写出一张二维tile颜色摘要；post process明确把对应blend权重固定为0。render graph却宣称该compute shader写入`LIGHT_GRID_PARAMS`、`LIGHT_ZBINS`、`LIGHT_TILE_MASKS`和`LIGHT_LIST`四个资源。这里同时存在默认GPU浪费、resource access谎报、queue语义谎报和功能命名谎报，必须硬切换，不能继续把无效dispatch算作clustered lighting完成度。

CPU light grid本身还有可见性正确性错误：正交相机的canonical `ortho_size`是half-height，builder却再次乘0.5；影响球与near plane相交时，若光心位于相机后方或`clip.w <= 0`，整灯会被丢弃。点光、聚光和矩形光还统一用sphere粗包围，矩形光的width/height虽然进入`GpuLightData`，所有直接光照WGSL仍把它当作带单面dot项的point light；距离衰减只是`(1 - d/r)^2`，没有lux/lumen/candela与inverse-square/solid-angle约定。这既不能提供稳定的物理authoring，也不能与Unreal rect light、Unity HDRP LTC或Godot area light达到同一表现层级。

shadow planner同样是“算法存在、产品未闭环”。它只选择第一个投影阴影的方向光，固定使用0.1 near plane与默认4 cascade；visibility却为每一个方向光都生成shadow view，未启用shadow的方向光仍生成1个，启用的全部生成4个。allocator的priority/rejection没有完整进入最终计划与readiness，logical 256-slot append顺序又是所有point先于spot，可能覆盖物理allocator的优先级。`ShadowCache`和静态revision计算只有测试/导出，没有renderer consumer；atlas每帧先清空，所有slot重新绘制，且每slot创建uniform buffer、bind group、pass name和render pass，再扫描整条shadow command stream。默认scene profile还会预留4096x4096 `Depth32Float` atlas，约64 MiB，不论本帧是否存在可投影光源。

可选Contact Shadow插件在启用后也不是按光源的screen-space ray shadow。其WGSL没有inverse view-projection、world position、light ID、type或direction，只比较12个邻近的非线性depth值、一个HZB mip和`normal.z`；输出单张RGBA8 visibility。最终post process把它乘到已经包含直接光、ambient、IBL、baked、reflection与emissive的整张scene color上。它会把无关能量一起压暗，不能被继续描述为Unreal/HDRP式contact shadow。

本轮登记10项P0、17项P1、5项P2。P0必须先修通authoring、真实GPU cluster、light-grid正确性、light layer、物理光度、面积光、shadow字段消费、统一shadow plan/visibility、真实cache与按光源contact shadow；P1再完成单一packed generation、精确bounds、预算/过载、atlas边界、批量depth submission、设备恢复、统一shader owner、诊断和产品验证；P2才进入virtual shadow map、ray-traced/MegaLights级many-light、时域高级软阴影和多GPU/注视点策略。完成同画质、同硬件、同场景的CPU/GPU/RAM/VRAM基准前，静态报告不能声称性能或表现已优于当前Unreal。

## 2. 审查边界与覆盖

### 2.1 已读范围

| 子域 | 文件 / 物理行 | 本轮判定 |
|---|---:|---|
| light authoring / asset / world extract | 3 Rust / 458 | E3：组件、scene schema、方向约定、layer、mobility与snapshot producer |
| core light ABI / snapshot / readiness / shadow settings | 5 Rust / 419 | E3：GPU layout、family readiness、shadow public contract |
| light packing / CPU grid / GPU grid consumer | 4 Rust + 1 WGSL / 1,189 | E3：packing、z-bin、tile mask、upload、stats与shader traversal |
| shadow plan / atlas / cache / raster / sampling | 13 Rust + 1 WGSL / 4,475 | E3：allocation、cascade、slot、view、cache policy、draw replay与PCF |
| shadow visibility view producer | 1 Rust / 456 | E3：directional/point/spot view创建与caster过滤 |
| direct-light shader consumers | 1 Rust + 6 WGSL / 2,407 | E3：deferred、fallback、generated standard PBR、GPU ABI与cookie交叉消费 |
| clustered graph / compute / post consumer | 12 Rust + 2 WGSL / 2,344 | E3：descriptor、declared access、host write、compute dispatch与zero blend |
| Contact Shadow runtime/editor plugin | 7 Rust + 1 WGSL / 1,502 | E3：registration、resource contract、dispatch、shader与WGPU product tests |
| 合计 | 57 / 13,250 | 85个inline test属性；focused fingerprint `09cd7df6ede3608c9da977497bef549393eae7731c31a21648a7cc737f731865` |

另抽查6个独立shadow product test文件、2,677行、18个test属性，用于判断真实产品入口与截图覆盖；这些独立test行数未计入focused fingerprint。物理行统计包含空行和同文件局部tests。“E3”表示读到实现、调用链和失败/生命周期语义，不表示真实GPU动态验收已经完成。

### 2.2 本轮归属与后续边界

- 09A拥有RHI handle、device generation、queue/fence、render graph version/access和resource retirement；09E只能消费这些能力，不能在shadow/cluster内部再造第二套GPU lifetime。
- 09B拥有persistent RenderScene、统一visibility truth、GPU Scene与instance/caster bounds；09E从该generation取得light/caster/view数据，不再从frame extract重复推导一套scene truth。
- 09C拥有Shader/PSO generation与共享include/module authority；09E负责direct-light/shadow算法模块，不允许forward/deferred/fallback继续手写三份漂移代码。
- 09D拥有texture/buffer residency、upload和预算会计；09E的cookie/LTC table/shadow page等资源必须通过其residency owner。
- 09F1单独审查environment、sky/IBL与reflection probe，09F2审查baked lighting/lightmap/irradiance volume，09F3审查Hybrid GI；本篇只记录ambient字段在direct producer被丢弃，不展开其全部算法。
- 09G单独审查froxel/volumetric、cookie、reflection、SSS等advanced lighting；本篇只处理它们与共享light ABI/grid/shadow generation的边界。
- 09H单独审查post/temporal；Contact Shadow在本篇处理，因为其正确语义必须在逐光源direct visibility内完成，不能作为全画面post occlusion。

### 2.3 参考引擎边界

- Unreal `LightGridInjection.cpp`是真正的GPU light-grid injection：具备可配置XY/Z分辨率、HZB cell cull、rect light refined bounds、linked-list/two-level路径、16-bit buffer、async compute选择、debug与feedback status；scene extension还按light change set更新稳定索引。Zircon的二维方向光颜色summary不能等价映射到该机制。
- Unreal `ShadowSetup.cpp`/`ShadowDepthRendering.cpp`把screen influence、resolution/fade、whole-scene cache MB、point/spot cache update cap、static/dynamic split、preshadow/per-object、CSM reuse与VSM纳入同一shadow policy。Zircon只有atlas packing和未消费的cache decision，是远低于该目标的局部基础。
- Bevy光源组件至少使用lux/lumen，暴露cast/contact/soft shadow、depth/normal bias、near-z和per-light cascade config；只有启用shadow才生成对应frustum。它的CPU/GPU cluster同时覆盖punctual light、probe/decal并有GPU overflow readback/resize，是Zircon必须越过的下限，而不是最终性能目标。
- Godot `ClusterBuilderRD`以sphere、cone/large-angle sphere和box分别raster omni、spot与area light volume，再compute pack成着色器消费的3D cluster；LightStorage暴露area size、soft shadow size、projector、directional split/mode与shadow atlas生命周期。
- Fyrox按camera distance选择point/spot shadow cascade，暴露shadow distance/fade/soft setting，并允许方向光absolute/relative frustum split。其传统volume/deferred路径规模有限，只作为基础完整性对照。
- Unity HDRP `LightLoop`拥有真实tile/big-tile/cluster list compute、depth prepass变体、debug和间接dispatch；Contact Shadow重建world position、沿逐光源方向ray march，并把结果编码为逐光源mask；LTC area light使用roughness/NdotV LUT与polygon irradiance。URP的CPU z-bin/tile mask和shadow atlas layout与Zircon局部结构相似，但其persistent NativeArray/job化与完整atlas降级仍只是较低端路径，不应成为“优于Unreal”的终点。
- 仓内Unity Graphics是SRP源码，不包含Unity native scene/RHI/shadow cache全部实现；本篇只引用可见的SRP算法和资源契约，不根据缺失源码推断native core。

### 2.4 明确未做

本轮没有修改production code，没有运行Cargo、Editor、真实GPU、PIX/RenderDoc、WPR、device loss、跨平台、视觉golden、光源过载、atlas thrash、相机穿灯、长时间移动光源或soak。静态审查可以证明字段/调用链断裂和算法风险，不能证明任何硬件上的最终帧耗时、噪声、闪烁或画质。

## 3. 当前必须保留并迁移的基础

### 3.1 `GpuLightData`固定ABI和GPU Scene dirty upload可保留

128-byte、16-byte对齐的light ABI已有布局测试，GPU Scene也能对light buffer做range diff而不是无条件重建整块资源。重构应version该ABI并由唯一`PackedLightingFrameGeneration`生产，不能让cluster、shadow、volumetric和mesh各自重新pack；稳定generation没有字段变化时应产生0 light upload。

### 3.2 shadow allocator的retention、generation和preemption是有效policy基础

allocator已经能保留上一帧rect、对tier做降级、记录rejection、维护slot generation，并通过持续竞争阈值避免立即抖动。需要修复的是输入priority、directional统一分配、logical slot编排、gutter和产品诊断，而不是回退到每帧顺序切固定网格。

### 3.3 cascade split、texel snapping和per-view visibility可以迁入统一planner

现有cascade范围、方向光stable snap、point/spot camera和`VisibilityViewKey`能够成为`ShadowViewGeneration`的数学/身份基础。唯一要求是planner先完成实际仲裁和allocation，再只为已分配slot创建view；visibility不能提前猜测planner结果。

### 3.4 shadow cache identity kernel值得接入产品

`ShadowCacheInput`已经同时考虑light parameter hash、static caster revision和atlas slot generation，并对动态/无authoritative revision输入fail closed。这是正确的cache identity方向。必须补上persistent depth owner、copy/reuse、dynamic overlay、fence和统计，不能继续让它只存在于测试。

### 3.5 当前局部tests可以转化为新contract回归

GPU ABI、grid mask、allocator reuse/preemption、cascade、slot flag、cache invalidation和WGPU shadow capture都已有可复用fixture。重构时应先把它们改为generation/authoring/product契约，再删除依赖旧假表面的source-string断言。

## 4. P0 差距清单

### P0-1：scene authoring到render extract无法启用shadow

`DirectionalLight`、`PointLight`、`SpotLight`、`RectLight`组件及对应scene asset没有`LightShadowSettings`或等价字段；`collect_*_lights`又对全部类型写`shadow: None`。这意味着allocator、cascade、point/spot shadow和PCF不是正常项目资产可达能力，测试手工snapshot不能证明产品交付。

必须在versioned scene schema建立逐光源shadow authoring：casts mode、method、strength、depth/normal/slope bias、resolution policy、near plane、max distance/fade、softness/source radius、cascade config与cache/mobility policy。component -> asset import/load -> property/transaction -> World -> render extract -> GPU/plan必须有roundtrip和migration test；任何中间层不得静默写None。

### P0-2：“Clustered Lighting”compute与render graph契约是假的

CPU `build_light_grid_for_frame`构造并host-write三个真正被direct shader读取的buffer；`execute_clustered_lighting`只接收directional lights并写二维`LIGHT_LIST`颜色。post params tests明确要求`blends[1] == 0`与`blends[2] == 0`，最终像素不消费该颜色。descriptor却把四个资源全声明为compute storage write，并把pass放在`QueueLane::AsyncCompute`。

必须二选一硬切换：要么实现真实GPU cluster assignment，让compute shader确实写入由forward/deferred/volumetric消费的light list并由graph追踪实际read/write；要么在迁移期删除该dispatch、`LIGHT_LIST`和“AsyncCompute”声明，仅把现有CPU grid诚实命名为CPU tiled light assignment。禁止长期保留两个同名authority，也禁止用统计record伪装GPU完成工作。

### P0-3：near-plane/camera-inside与正交相机grid会丢失可见光

`sphere_influence`允许影响球跨near plane，但`sphere_tile_rect`随后因光心`clip.w <= 0`返回None；相机位于point/spot/rect range内、或大光源中心在相机后方时可能整灯消失。正交projection又将canonical half-height `ortho_size`乘0.5，使grid使用的投影比真实view窄一倍。现有tests没有camera-inside、near crossing或orthographic edge coverage。

必须用统一view projection/bounds library生成conservative screen/depth bounds：camera inside时覆盖全屏或解析投影sphere/cone/OBB边界，near-plane交叉不能按center拒绝；orthographic直接消费canonical projection matrix。修复必须覆盖reversed-Z/oblique/jittered/stereo/viewport subrect，不能在grid再手写一套相机约定。

### P0-4：light layer被打包但没有逐物体直接光照与caster语义

world extract只按camera layer先过滤light，随后`layer_mask`写入`shadow_slot_layer.y`。所有direct/shadow WGSL均不读取该字段，surface/primitive light mask也未参与light loop；shadow visibility只判断`relevance.shadow_caster()`，不与light layer求交。因此留在同一camera内的光会照亮全部对象，所有shadow caster也可能投到不应受该光影响的对象上。

必须定义统一`LightingChannelMask` ABI并贯穿primitive、light、direct evaluation、shadow caster inclusion、volumetric与debug。cluster list可以按view收集候选，但最终surface/light求交必须正确；shadow planner生成caster set时也必须使用同一mask。无对应consumer前不得在readiness中声称layer功能有效。

### P0-5：光度单位与衰减不是可迁移的物理contract

方向光默认强度2、点光4、聚光8、矩形光1,000,000，字段都只叫`intensity`；punctual attenuation固定为`pow(clamp(1-d/r), 2)`，没有inverse-square、solid angle、source size、exposure/pre-exposure或lux/lumen/candela定义。相同数值在不同light type没有可预测意义，也无法稳定导入DCC/UE/Unity资产或建立画质基准。

必须确定并version photometric contract：directional用lux，point用lumen或candela，spot明确cone lumen/candela转换，rect用lumen/nit与面积归一；颜色支持linear RGB与temperature/tint，range只作为smooth cutoff而非替代物理衰减。旧scene通过显式legacy migration转换，不能偷偷改变已有字段语义。GPU端需要pre-exposed radiance与finite/NaN admission校验。

### P0-6：RectLight被公开为类型，实际不是面积光且没有shadow实现

producer把每个RectLight标成`renderer_degraded: true`，但仍将width/height写入`spot_angles_size.zw`并参与direct loop。deferred、fallback和standard PBR三套WGSL都忽略rect half-size，只在point-like attenuation上乘一个单面direction dot；shadow requests完全不包含rect。结果既不是LTC/analytic area light，也不是诚实的unsupported状态。

必须选择工程级目标并一次切换：实现矩形面积光的diffuse/specular积分、basis/size/barn door/two-sided/source texture、精确cluster bounds和可验证shadow策略，首选LTC作为raster baseline；或者在完成前从shipping authoring和readiness中禁用该类型并明确错误。不能继续用百万级默认强度掩盖point approximation。

### P0-7：shadow strength、normal bias与readiness是false surface

`LightShadowSettings`公开`strength`和`normal_bias`。pack后`light.shadow_params.x/y`没有direct/shadow shader consumer；`GpuShadowSlot.params.y`保存normal bias但`zr_shadow.wgsl`只用`params.x`作为receiver depth bias。`RenderLightReadinessReport`对directional/point/spot直接以total作为ready，不检查authoring可达性、slot assignment、atlas rejection、field consumption、pipeline或device状态。

必须先定义每个setting的唯一消费点和单位：strength在逐光源visibility混合，normal/slope bias在receiver offset或caster raster policy中按world/texel尺度生效，并有正负面artifact测试。readiness必须从最终generation生成`Ready/Degraded/Rejected/Unsupported/Failed`及原因，不得由light count推断。

### P0-8：shadow plan、visibility与light assignment没有共同authority

planner只取第一个shadow-casting directional；visibility却遍历所有directional，未启用shadow也生成1个view，启用则固定4个。point/spot visibility在allocator之前按casts flag生成，即使之后atlas拒绝仍支付culling。plan固定0.1 near与默认cascade，visibility有自己的常量/相机函数。两个系统可以对同一帧生成不同view数量、矩阵或工作量。

必须由`ShadowPlanGeneration`先根据view、quality、budget、screen influence、history和capability完成全类型仲裁/slot allocation，再产出唯一`ShadowViewDescriptor[]`。visibility和raster只消费该数组；未分配slot绝不创建view。多方向光必须有显式选择/混合/拒绝政策及diagnostic，而不是“第一个”加沉默丢弃。

### P0-9：ShadowCache没有产品consumer，atlas清空与全slot重绘使其不可能命中

`ShadowCache`仅决定identity，产品renderer没有`evaluate/commit`调用。shadow atlas pass在第一slot使用graph attachment clear，之后load；每帧所有slot都进入draw replay。即使未来简单跳过某slot，开头的whole-atlas clear也会抹掉缓存。4096平方D32 atlas约64 MiB会在scene profile构造时分配，和本帧shadow demand无关。

必须实现persistent cached depth ownership：静态depth region或page保留、slot generation与fence关联，cache hit复制/复用static depth，再叠加dynamic caster；clear必须按region/page或使用独立static/dynamic atlas。资源分配与resolution tier进入09D/09A预算，零shadow帧不得保留不必要的full atlas。cache hit/miss/invalidation/copy bytes/update cap必须可观测。

### P0-10：Contact Shadow不是按光源shadow，却全局压暗最终scene color

插件shader没有camera/world/light输入，固定以屏幕邻域非线性depth阈值估算一个全局visibility；post process再将其与AO一起乘到完成光照的scene color。它会错误压暗ambient、IBL、baked、reflection和emissive，且阈值随near/far/FOV/分辨率变化。现有product tests只证明画面变暗，反而固化了错误语义。

必须把contact shadow移入逐光源direct visibility：重建world position，沿directional或punctual light direction进行world/screen length ray march，使用HZB traversal、thickness、bias、distance fade、jitter/history与disocclusion filter，并以light mask/index在light loop中消费。低端单directional模式也必须明确只影响该光的direct term；禁止继续作为全画面occlusion post。

## 5. P1 差距清单

### P1-1：light packing在多个consumer重复

light grid build、GPU Scene sync以及advanced-light consumer可重复调用pack/cookie planning；volumetric ID通过逐light `Vec::contains`形成O(L*V)。需要唯一`PackedLightingFrameGeneration`，一次完成stable index、photometric conversion、cookie/IES handle、shadow request key、channel mask和changed ranges，并被所有consumer按只读slice使用。

### P1-2：真实GPU cluster需要容量、overflow和fallback contract

现有CPU bitmask把light count静默clamp到65,535，固定8,192 tile words会随light数增大tile size，z-bin数也随words减少；没有“哪些light被丢弃/精度降级”的报告。GPU实现必须有per-view配置、buffer capacity、atomic/list overflow flag、异步readback、增长/降级策略、capability fallback和shipping threshold。silent clamp禁止进入shipping。

### P1-3：不同light type需要精确conservative volume

directional全cluster合理，但point/spot/rect目前都用sphere。spot应使用cone或保守cone/sphere hybrid；rect应使用oriented box/frustum/polyhedral bounds；source radius/length需要扩张。大角度、相机inside、near clipping与极远范围必须有解析/数值稳定策略，并通过CPU reference与GPU list对拍。

### P1-4：physical atlas priority与logical slot顺序冲突

physical allocator按priority排序，但frame plan先append全部point六面，再append spot；当256 logical slot先耗尽时spot会无视物理priority全部失去slot。point只要任一face未分配就六面全弃，也没有低成本降级。logical indirection必须由allocator结果统一排序，支持atomic six-face group、重要性/屏幕尺寸/history/mobility、partial fallback策略和明确拒绝原因。

### P1-5：cascade配置、camera near与caster fit均为硬编码

方向光固定`CascadeSplitConfig::default()`、0.1 near、最多4 cascade和150m类默认范围；没有per-light/per-camera/quality配置。light-space depth fit主要按camera slice sphere与padding，未纳入实际receiver/caster bounds和off-camera caster。必须支持authorable split/count/distance/fade、实际camera near、scene depth/caster bounds fit、stable snap与camera-cut reset。

### P1-6：atlas没有gutter，PCF clamp允许跨slot采样

allocation rect没有padding。shader把sample UV clamp到精确slot min/max，linear comparison sampler在边界仍可能混合邻接texel；High quality只取正负8 texel的稀疏3x3，核形状与分辨率/penumbra无一致物理意义。必须按最大filter footprint分配gutter/guard texel，clamp到half-texel安全范围，并为不同tier定义可测的Poisson/PCSS/EVSM或其他quality policy。

### P1-7：point cube face没有seam处理

receiver按dominant axis只选一个face，边界没有跨face采样、seam fixup或octahedral替代。动态物体跨face会出现可见断层。必须选择cubemap array/seam-aware face transform或dual-paraboloid/octahedral方案，并以旋转light和跨face blocker做golden。

### P1-8：每slot CPU/RHI热路径分配与全command扫描

每个slot创建immutable scene uniform buffer、scene bind group、formatted String和render pass，构造`BTreeSet`可见entity，并扫描完整shadow command stream；还创建一个forward receiver bind group及一套neutral environment texture/sampler/BRDF/specular/irradiance/SH9资源，只为满足过宽scene layout。应改成persistent ring/dynamic offset、按view预分桶的draw packet、multi-viewport批量record和depth-only专用layout。

### P1-9：shadow update scheduler没有CPU/GPU/byte预算

allocator只预算atlas面积，不预算本帧需要cull多少view、提交多少draw、多少static cache copy、多少dynamic overlay和GPU时间。必须有按quality tier的view/draw/triangle/copy byte/GPU time/update count预算，超载时按稳定priority延迟、降resolution、复用上一代或关闭低价值shadow，并输出原因。

### P1-10：shadow资源生命周期与device loss依赖09A仍未接入

atlas、sampler、buffers、pipeline/bind group和Contact Shadow pipeline都是raw WGPU owner；多个ensure路径使用`expect`，插件用mutex lazy pipeline且没有device generation终态。必须统一迁入09A的generation-aware resource/PSO registry，device loss后last-good generation有明确失效、重建、降级和超时状态。

### P1-11：forward/deferred/generated PBR直接光算法存在多份owner

`punctual_light_visibility`、rect approximation、BRDF与shadow调用在deferred、fallback、standard PBR/basic PBR和builtin shader source重复。修复一处物理衰减或area light极易产生路径漂移。09C必须提供单一typed shader module/include，permutation只选择feature，不复制算法正文；离线compile验证所有consumer ABI。

### P1-12：light方向authoring约定不一致

Directional/Spot使用component内`direction`向量，Rect使用entity transform的`forward()`；scene asset还同时存在方向字段。旋转entity对不同light type产生不同结果。必须确定transform-owned或property-owned方向，支持look-at/rotation编辑与migration，并在负scale/zero direction下给出validation，不得静默normalize为不可预测方向。

### P1-13：输入admission缺少finite/range/angle校验

color、intensity、range、rect size、inner/outer angle、bias和strength可进入负值、NaN/Inf或inner/outer倒置。packing主要依赖shader `max/clamp`兜底，错误资产可能污染bounds、priority和GPU。scene load、Editor transaction和runtime mutation必须共享validator与repair policy，错误要携带entity/property路径。

### P1-14：ambient direct producer丢弃authoring字段

`AmbientLight.affects_lightmapped_meshes`能序列化和roundtrip，但`RenderAmbientLightSnapshot`没有该字段；renderer把所有ambient相加，无ambient时还注入固定0.2 fallback。global environment/IBL由09F1处理，lightmap能量归09F2，但09E必须先保证producer不丢字段，并区分editor preview fallback与shipping scene truth。

### P1-15：readiness、stats与debug view没有共同truth

当前light readiness按count，light-grid stats需要遍历`bin * tile * word`笛卡尔积，shadow rejection/cache又不进入同一report；cluster debug buffer本身无效。必须从最终generations低成本累计accepted/rejected/truncated/overflow/cache/update与timing，提供cluster occupancy、light bounds、shadow atlas/page、cascade、channel mask和contact ray debug view。

### P1-16：稳定帧仍有CPU allocation与全量buffer write

CPU grid每build重新分配z-bin、tile mask和min/max数组，并每帧全写params/zbin/mask；stats又扫描全部组合。shadow slots也全量upload并为stale tail临时分配disabled vector。需要persistent capacity/scratch arena、changed range或GPU producer、可选采样stats和zero-change fast path；所有目标必须用allocation counter与queue-write bytes验证。

### P1-17：产品测试覆盖绕过真实authoring且缺少规模/故障证据

shadow captures直接注入snapshot，不能发现`shadow: None`产品断链；Contact Shadow tests只断言darkening；大量测试锁定source string/resource name。必须建立asset -> World -> capture的产品场景，并覆盖camera-inside、orthographic edge、light layers、多方向光仲裁、rect size response、所有bias/strength、atlas gutter/point seam、cache hit/miss、overflow、device loss、camera cut、动态caster和长时间thrash。

## 6. P2 差距清单

### P2-1：Virtual Shadow Map / clipmap / page cache

在稳定传统atlas、cache、预算和诊断后，引入virtual page table、physical page pool、per-page caster invalidation、clipmap scrolling、HZB/page request与GPU-driven page render。不得把“4096 atlas切小块”改名为VSM，也不得在09A/09B尚未统一generation前并行建立独立scene/page authority。

### P2-2：ray-traced shadow与many-light/MegaLights级路径

硬件RT、software ray query、stochastic many-light sampling与denoise需要独立capability/quality path，并与raster fallback共享photometric light ABI、channel、cookie、shadow policy和diagnostics。只有基线direct/cluster正确后才有资格比较Unreal MegaLights或ray-traced rect shadow。

### P2-3：高质量软阴影的时空采样

PCSS/contact hardening、stochastic area-light shadow、temporal accumulation、spatial denoise和disocclusion rejection应按light/source size与motion truth设计。不能把当前固定8-texel稀疏PCF继续扩张为“ultra”。

### P2-4：GPU-driven shadow caster submission

依赖09B的persistent GPU Scene、instance culling与indirect draw，由GPU生成每shadow view/page caster list和draw args，支持meshlet/Nanite类geometry path。CPU command replay必须保留可验证fallback，但不能成为大型场景唯一shipping path。

### P2-5：multi-GPU、foveated与超大视图族调度

在单GPU、多view/stereo generation稳定后，再定义per-GPU shadow page ownership、cross-adapter copy、foveated cluster/shadow density和view-family共享。当前不应预留无法验证的公共ABI或伪能力flag。

## 7. 目标架构

### 7.1 单一generation链

```text
AuthoredLightingGeneration
  -> ExtractedLightingGeneration
  -> PackedLightingFrameGeneration
  -> ClusterAssignmentGeneration
  -> ShadowPlanGeneration
  -> ShadowViewGeneration
  -> ShadowVisibilityGeneration
  -> ShadowDepthGeneration
  -> DirectLightingEvaluationGeneration
  -> LightingDiagnosticsGeneration
```

每一代必须包含`world_id + view_family_id + frame/generation + source revision + device generation`。下游只能消费同代或显式last-good generation；不得按裸slice/count拼接来自不同帧的数据。09F1-09F3/09G消费`PackedLightingFrameGeneration`和shadow visibility，但不能重新提取/打包light。

### 7.2 owner职责

| Owner | 唯一职责 | 禁止行为 |
|---|---|---|
| scene light schema | versioned photometric/shadow/channel authoring与migration | render extract静默补功能默认值 |
| lighting extractor | 从World生成immutable light records与dirty set | 创建WGPU资源、分配atlas |
| packed lighting owner | stable GPU index、单位转换、ABI、changed ranges | 每个consumer各自pack/cookie plan |
| cluster owner | 精确bounds、GPU assignment、capacity/overflow/fallback | 用无consumer颜色buffer冒充light list |
| shadow planner | 仲裁、预算、slot/page、view descriptor、cache decision | visibility提前猜测slot、按source顺序分配logical slot |
| shadow visibility/depth | 消费planned views并生成caster/draw/depth | 重算cascade/near、扫描无关全部command |
| direct shader module | 唯一BRDF、attenuation、area、channel、shadow消费 | forward/deferred/fallback复制算法 |
| diagnostics | 聚合最终generation状态、cost与降级原因 | 由source count推断ready |

### 7.3 关键数据合同

`PackedLightRecord`至少需要stable 64-bit identity或generation-safe index、type、photometric radiance、position/basis/source shape、range cutoff、channel mask、shadow request handle、cookie/IES handle、volumetric/indirect flags和dirty bits。GPU可采用SoA或压缩结构，但不能再把layer、低32位light ID和shadow flags混装在无schema的`shadow_slot_layer`后靠约定猜测。

`ClusterAssignmentGeneration`必须记录view dimensions、projection identity、tile/cluster dimensions、light index list、capacity、overflow、truncated IDs、build path CPU/GPU、dispatch/submission ticket与timing。shader只读取该generation内的buffers。

`ShadowPlanGeneration`必须记录每个light的accepted/rejected/deferred状态、priority组成、requested/allocated resolution、slot/page generation、view descriptor、cache decision、static/dynamic caster policy、预计/实际cost与fallback。`Ready`只在深度generation和direct binding均可消费时成立。

## 8. Hard Cutover 规则

1. scene schema升级后，旧`intensity`与无shadow字段通过显式migration得到legacy-compatible值；新资产只写photometric schema。禁止两套authoring长期双写。
2. 真实GPU cluster落地时删除二维方向光summary、zero post blend及虚假的四resource write声明。CPU fallback使用同一输出ABI，禁止保留第二套shader遍历格式。
3. `PackedLightingFrameGeneration`接管后删除grid/GPU Scene/advanced consumer的独立pack调用；cookie/volumetric membership预建index/bitset，不再逐light线性`contains`。
4. `ShadowPlanGeneration`接管后删除`build_views.rs`中的独立cascade/point/spot shadow camera推导，只保留对planned descriptor的visibility执行。
5. cache产品接入必须同时替换whole-atlas clear；禁止先添加“cache hit”统计而仍清空/重画深度。
6. shared direct-light module完成后删除deferred/fallback/generated PBR重复函数；不接受仅用source-string同步测试维持多份正文。
7. Contact Shadow ABI升级为逐光源mask/visibility后删除全画面post multiply和RGBA8四通道标量资源；旧插件manifest需要breaking capability/version迁移。
8. 不新增root crate。实现留在`zircon_runtime`既有scene/graphics边界，插件只拥有可选feature adapter；RHI、asset、shader和visibility复用09A-09D owner。

## 9. 分层实施里程碑

### M0：冻结当前失败证据与目标ABI

添加authoring-to-capture失败fixture、camera-inside/ortho edge、layer、rect size、multi-directional、atlas rejection、cache与contact semantic tests；记录当前CPU allocation、queue writes、dispatch、shadow draws、VRAM和golden。产出versioned light/shadow/cluster generation schema及迁移RFC。

### M1：修通scene authoring与photometric contract

实现组件/asset/property/transaction/World/extract roundtrip，加入lux/lumen/candela/area单位、temperature、source shape、channel与完整shadow settings，提供legacy migration/validator。退出条件是普通scene asset可以启用方向/点/聚光shadow，且每字段有最终consumer或明确unsupported错误。

### M2：建立唯一Packed Lighting Generation

统一stable light index、GPU ABI、unit conversion、cookie/IES/shadow handles、dirty ranges和membership index；GPU Scene/grid/volumetric只读消费。退出条件是稳定无变化帧0次重复pack、0 light upload、0 heap growth。

### M3：先修CPU reference正确性

修复canonical projection、near/camera-inside、ortho、cone/rect bounds和light channels，建立CPU reference list与随机/property tests。该reference既是GPU对拍oracle，也是无storage/compute设备fallback；不能把旧错误CPU grid直接当最终fallback。

### M4：硬切换真实GPU cluster

实现clear/count/prefix-or-linked-list/fill或等价GPU pipeline，写入真实consumer buffers，接入render graph实际access/barrier和capability-aware async lane；删除fake cluster summary。加入overflow feedback、resize/degrade和debug occupancy。GPU结果逐帧抽样与CPU oracle对拍。

### M5：统一direct-light shader与面积光

09C提供共享module，forward/deferred/fallback使用同一photometric attenuation、BRDF、channel与shadow入口；RectLight实现LTC/analytic diffuse+specular、basis/size/barn door和精确cluster bounds。退出条件是各path在容差内一致，rect size/roughness响应有golden。

### M6：统一shadow planner、allocation与visibility

planner接收实际camera/view family、quality、budget、screen influence和history，统一directional/punctual/rect request、logical slot/page与rejection；只为accepted request产生view descriptor。删除visibility侧重复相机数学，并加入multi-directional与layer caster policy。

### M7：修复atlas采样与软阴影基线

实现gutter/half-texel安全范围、seam-aware point shadow、world/texel一致bias、strength/normal bias消费和按source size的filter policy。以acne、peter-panning、neighbor bleed、cascade transition、cube seam与移动相机golden验收。

### M8：接入static cache与更新预算

实现persistent static depth、cache evaluate/commit、region/page preserve或copy、dynamic overlay、slot generation/fence、update count/time/byte budget。零shadow帧按需释放/降配atlas；cache hit帧不得重绘static caster。

### M9：批量化shadow visibility与depth submission

按planned view构建预分桶caster/draw packets，使用persistent uniform ring/dynamic offsets、depth-only layout和parallel/GPU-driven record；移除per-slot buffer/bind/String/BTreeSet及全stream扫描。保留deterministic CPU fallback与统计。

### M10：重做逐光源Contact Shadow

world reconstruction、light-direction ray/HZB traversal、per-light mask、world/screen length、thickness/bias/fade和时域稳定接入direct light loop；删除全画面post乘法。低/中/高quality分别有步数、分辨率、history和budget合同。

### M11：设备恢复、错误终态与诊断

迁入09A generation-aware resource/PSO owner，消除render-path `expect`，支持device loss/recreate/last-good/degraded。统一light/cluster/shadow/cache/contact readiness与debug views，Editor后续只消费该公共诊断，不重建真值。

### M12：产品规模与竞争性验收

完成desktop/low-end/integrated、perspective/orthographic/stereo、1/100/10k lights、static/dynamic mix、camera cut/teleport、atlas pressure、device loss、24h soak。用同硬件、同分辨率、同可见光数量、同阴影覆盖与匹配画质对照Unreal；所有原始capture、配置、commit与统计进入可复现artifact。

## 10. 验收矩阵

| 场景 | 必须证明 | 失败信号 |
|---|---|---|
| scene asset roundtrip | shadow/channel/photometric/source shape从磁盘到像素保持语义 | extract写None、字段丢失、测试专用注入 |
| camera inside / near crossing | 影响相机的point/spot/rect不消失，CPU/GPU list一致 | light pop、NaN bounds、center-behind整灯剔除 |
| orthographic / oblique / jitter | grid与真实view projection一致，边缘无漏光 | `ortho_size`重复缩放、jitter导致list闪烁 |
| light channels | direct、shadow caster、volumetric使用同一mask | camera可见但错误物体受光/投影 |
| photometric scene | 不同light type单位可预测，range只平滑截断 | arbitrary intensity、曝光变化导致非物理跳变 |
| rect area light | size/basis/roughness/one-two sided/door响应正确 | width/height变化不影响照明、point-like高光 |
| cluster overload | overflow被检测并稳定降级/扩容，无OOB | silent 65,535 clamp、粗tile无原因、无效dispatch |
| multi-directional shadows | 显式选择/预算/拒绝，可见性只为accepted view | 只渲染第一个但全部做culling |
| atlas boundary / point seam | filter不跨slot，cube face边界连续 | 邻slot漏影、旋转light出现接缝 |
| static cache | exact hit不重画static，任一依赖变化可靠invalidate | 伪hit后仍clear/redraw或复用stale depth |
| contact shadow | 只衰减对应light direct term，history稳定 | ambient/emissive变暗、FOV/near改变阈值失控 |
| device loss / resize | generation失效、重建、降级有界 | panic、stale WGPU handle、永久false-ready |
| soak / thrash | memory、slot/page、pipeline和history有上界 | atlas/preemption闪烁、资源/Map单调增长 |

### 10.1 性能门禁

- warm stable frame：light extract/pack/grid/shadow plan不得有无界heap分配；无变化generation的light/shadow metadata upload bytes必须为0或有明确GPU-produced理由。
- cluster：记录CPU prepare、GPU build、buffer bytes、occupancy分布、overflow与consumer GPU time；fake/unused dispatch数必须为0。
- shadow：记录planned/culled/rendered/cached/deferred view、caster/draw/triangle、static copy bytes、atlas/page bytes、CPU record与GPU depth time；预算超限必须稳定降级而非hitch或silent drop。
- contact：记录active lights、ray steps/HZB visits、resolution、history rejection、GPU time和mask容量；不能只记录“pass dispatched”。
- 竞争目标：在匹配画质和功能集合下，Zircon的p50/p95/p99 render-thread time、GPU direct+shadow time、RAM/VRAM峰值和camera-motion stability均不得显著劣于选定Unreal版本；“优于”必须由预注册场景和统计显著性证明，不能以关闭阴影、减少光源或降低filter质量取得。

## 11. 参考实现映射

| Zircon目标 | Unreal主参考 | 次级/下限参考 | 不能直接照搬的部分 |
|---|---|---|---|
| GPU cluster | `LightGridInjection.cpp` | Bevy GPU cluster、Godot ClusterBuilder、HDRP LightLoop | URP固定word CPU grid不是高端终点 |
| photometric light ABI | light scene proxy/rendering data | Bevy lux/lumen、Godot LightStorage | 各引擎曝光/单位版本不同，需Zircon schema migration |
| rect area light | RectLightSceneProxy/LightRendering | HDRP LTC、Godot area light | 只复制LUT不等于完整basis/bounds/shadow |
| shadow planning | `ShadowSetup.cpp` | URP atlas layout、Fyrox distance tiers | Unreal内部scene/RHI类型不能越过Zircon 09A/09B owner |
| shadow cache | ShadowSetup/ShadowDepthRendering | Bevy/Godot基础atlas | cache identity必须结合Zircon resource/caster generation |
| virtual shadow | VSM array/cache/clipmap | 无等价最终目标 | 仅在传统baseline完成后进入P2 |
| contact shadow | LightRendering contact params | HDRP ContactShadows、Bevy raymarch | 当前全画面AO式乘法必须删除 |
| diagnostics | light-grid feedback/debug、shadow dump/stats | HDRP tile/cluster debug、Godot debug | debug view不能成为独立真值或隐藏overflow |

## 12. 证据缺口与风险

### 12.1 已有证据

- light ABI/layout、packing order、grid bitmask、allocator tier/retention/preemption、cascade/slot和cache invalidation有较多局部unit tests。
- WGPU shadow product tests覆盖方向/点光多灯等部分capture，证明手工snapshot条件下存在真实depth/shading路径。
- Contact Shadow插件有WGPU测试，证明资源绑定、dispatch和最终darkening可发生。
- render stats能看到部分light-grid occupancy和shadow replay统计，提供迁移挂点。

### 12.2 仍缺失的决定性证据

- 没有普通scene asset启用shadow的end-to-end测试；现有capture绕过P0-1。
- 没有camera-inside、near-plane crossing、orthographic edge、light-channel和multi-directional arbitration测试。
- 没有证明rect width/height改变BRDF响应或产生面积光shadow的golden。
- 没有证明strength/normal bias消费、atlas gutter、cube seam与cascade stable motion的图像证据。
- 没有ShadowCache产品hit，也没有static/dynamic分层depth与fence测试。
- 没有cluster CPU/GPU oracle对拍、overflow、10k light、stereo或真实async queue时间线。
- 没有Contact Shadow逐光源语义、camera projection invariance、temporal/disocclusion测试。
- 没有device loss、VRAM pressure、atlas thrash、24h soak或与Unreal匹配画质benchmark。

### 12.3 实施风险

- photometric迁移会改变旧scene观感，必须提供版本化转换和before/after capture，不能用“更物理”掩盖breaking change。
- cluster与shadow同时依赖09A-09D；若先在旧raw WGPU/frame extract上快速实现，会形成第三套待迁移authority。
- true area light和soft shadow会显著增加shader/PSO/资源组合，必须先由09C统一permutation与artifact，避免compile storm。
- cache与temporal算法最容易产生stale generation；所有reuse必须携带scene/view/device/slot/page identity并fail closed。

## 13. 完成定义

09E实现只能在以下条件全部满足后标记complete：

1. 普通versioned scene asset可完整author方向/点/聚光/矩形光的photometric、channel与shadow字段，并通过World/extract/render roundtrip。
2. fake clustered dispatch、zero blend和虚假resource write全部删除；真实CPU/GPU fallback输出同一consumer ABI，overflow可见且有界。
3. camera-inside、near crossing、orthographic、light channel、rect area response和多方向光均有CPU oracle、GPU/product与golden证据。
4. shadow strength/normal/slope bias、cascade config、atlas gutter、point seam、priority/rejection和readiness均有真实consumer与测试。
5. shadow plan先于visibility且是唯一view authority；未分配shadow不产生culling/depth工作。
6. static shadow cache在产品命中，whole-atlas clear不破坏reuse，dynamic overlay与fence/device generation正确。
7. shadow recording不再每slot创建临时buffer/bind/String并扫描整stream；稳定帧allocation/upload/draw成本有预算与profile。
8. Contact Shadow只影响对应light direct term，不再全局乘scene color，并通过projection/motion/temporal测试。
9. desktop/low-end/stereo/overload/device-loss/soak矩阵通过，原始性能/画质artifact可复现。
10. 与选定Unreal版本的同硬件同画质基准完成；报告只根据数据陈述是否达到“优于”目标，未达到时保留差距，不以功能降级换取结论。

在此之前，`docs/plans/zircon_runtime/render/05-lighting-shadows.md`中的局部milestone完成状态只能表示代码片段已存在，不能表示工程级Lighting/Shadow产品完成。
