# ZirconEngine WGPU 渲染主链闭环计划

## Summary
本计划优先完成可承载后续高级特性的渲染主链：`Scene/ECS Extract -> SRP/RenderFeature -> RenderGraph -> RHI/WGPU -> SceneRenderer -> UI/Overlay/Debug`。第一阶段不一次性实现完整 Lumen/Nanite、SpeedTree、光照烘焙、完整 ShaderGraph、神经网络渲染等大系统，而是先把它们需要的接口、资源生命周期、执行顺序、调试证据和扩展点固定下来。

参考取向：Unity SRP/URP 的 Camera/Layer/RenderQueue/Volume 顺序模型，Bevy 的 Extract/RenderLayers/Material 分层，Unreal RDG/Nanite/Lumen 的资源图和高级特性插件化边界，Slint 的 WGPU UI surface 接入方式。

## Public Interfaces
- 保持固定根结构：`zircon_app`、`zircon_runtime`、`zircon_editor`；不新增非网络 `server` 命名，不增加兼容 shim。
- `zircon_runtime::core::framework::render::RenderFramework` 继续作为外部渲染入口，只提交中立 `RenderFrameExtract`、pipeline asset、capture/debug/status 信息。
- `zircon_runtime::{rhi,rhi_wgpu,render_graph}` 承担底层资源、队列、pass、history、transient/external resource 生命周期；禁止 `app/editor/framework` 直接依赖 `wgpu`。
- `RenderPipelineAsset`、`RendererFeatureAsset`、`RenderFeatureDescriptor` 扩展为主 SRP 描述层：声明 pass stage、queue、executor、extract section、capability、history binding、resource read/write。
- 渲染顺序模型固定为：camera target/order -> render layer mask -> render queue/material queue -> depth/depth bias -> order in layer/z-index。3D、2D、UI 使用同一排序语义，但 UI 保留独立 retained graph 和 damage path。
- VG/HGI、Nanite/Lumen 类重状态保持插件所有权；runtime 只提供 extract DTO、graph executor 注册、readback/status/stats 合约。

## Milestones
### M0 - Baseline And Risk Lock
- 固定当前事实：资产、Mesh/ZMesh、Texture、ZShader/ZMaterial、UI graph/direct surface damage 已作为现有基础复用。
- 将 `RenderFrameExtract` 确认为生产主入口；旧 snapshot adapter 只保留 legacy/test 用途，不再作为新渲染功能入口。
- 默认关闭或强门控 temporal/history 类 pass，直到 history slot、resize、camera cut、motion validity 明确。
- 建立风险清单：clear/load 边界、history ghosting、fixed SceneRenderer path 与 graph path 双轨、executor 缺失、插件 pass 泄漏 runtime 边界。

### M1 - RHI/WGPU Boundary
- 补齐 RHI 描述能力：HDR color、depth/stencil、storage/copy usage、mip、cube、texture array、sampler、compute pipeline、queue capability。
- 收敛 `wgpu` 暴露面：底层类型只允许在 `rhi_wgpu` 和必要的 runtime backend 内部出现。
- WGPU adapter/device capability 进入 runtime capability table，SRP feature 通过 capability gate 启停。
- 测试阶段：
  - `cargo test -p zircon_runtime --locked rhi`
  - 源码边界检查：`zircon_app`、`zircon_editor`、framework facade 不直接导入 `wgpu`。
  - 记录 adapter/capability/debug marker/capture status 的接受证据。

### M2 - RenderGraph As Real Frame Graph
- 用 viewport/extract/pipeline 派生真实 graph resource desc，替换 1x1 占位资源。
- 明确 transient、external、history、swapchain/imported target 的所有权和生命周期。
- 实现 pass read/write、WAW、read-before-produce、clear/load/store、culling、queue fallback 的验证。
- history 资源拆成 current/previous/output slot，禁止同名读写造成隐式覆盖。
- 测试阶段：
  - `cargo test -p zircon_runtime --locked render_graph`
  - 覆盖 pass culling、external root、history slot、clear/load、async fallback、无 producer 读取失败。

### M3 - SRP And Feature Compilation
- 将 Forward+、Deferred、Core2D 默认 pipeline 标准化为 graph pass 序列，而不是只作为统计/声明层。
- `RendererFeatureAsset` reload/set 时验证 executor id、capability、resource I/O、extract section、history binding。
- 内置 feature 覆盖 mesh depth/opaque/alpha/transparent、deferred gbuffer/lighting、sprite、postprocess、UI、overlay、debug。
- 插件 feature 只通过注册 descriptor/executor 接入；未知 executor 在 pipeline 激活阶段失败，不静默 noop。
- 测试阶段：
  - `cargo test -p zircon_runtime --locked graphics::pipeline`
  - 覆盖 Forward+、Deferred、Core2D 编译，缺失 executor，capability gate，feature reload failure rollback。

### M4 - Scene/ECS Extract Canonical Path
- `scene` 在 RenderExtract stage 产出完整 `RenderFrameExtract`：camera、visibility、geometry、sprite、material、lighting、postprocess、particles、VG/HGI sideband。
- camera 支持多 target、多 camera order、render layer mask、HDR/MSAA/clear/exposure；同 target 同 order 产生诊断。
- mesh/sprite/material/light/postprocess extract 均按 active、layer、camera visibility 过滤。
- 引入统一排序键：render queue、material queue、depth、depth bias、order in layer、UI z-index、entity tie-breaker。
- 测试阶段：
  - `cargo test -p zircon_runtime --locked scene::render_extract`
  - 覆盖多 camera、layer intersection、inactive entity、sprite order、transparent reverse depth、opaque early-z eligibility。

### M5 - SceneRenderer Graph Cutover
- 把现有固定渲染函数包装为 graph executor：`mesh.depth-prepass`、`mesh.opaque`、`deferred.gbuffer`、`deferred.lighting`、`sprite.*`、`postprocess.*`、`ui.screen-space`、`overlay.*`。
- `SceneRenderer` 只保留 frame orchestration、target import、executor dispatch、stats/debug marker，不再私下绕过 graph 执行核心 pass。
- graph resource registry 管理 scene-color、final-color、scene-depth、gbuffer、bloom、history、UI target。
- HistoryResolve 只在 motion/resize/camera-cut validity 合格时启用，并在正确 stage 后复制输出到 history。
- 测试阶段:
  - `cargo test -p zircon_runtime --locked graphics::scene_renderer`
  - 运行 Forward+、Deferred、Core2D graph execution 测试，验证 pass order、resource ownership、history invalidation、RenderDoc marker/status。

### M6 - Material, Mesh, Sprite Main-Chain Consumption
- 复用现有 ZShader/ZMaterial/ZMesh/Texture 资产层，建立 renderer-side prepared cache：shader layout、material bind group、texture fallback、uniform payload、alpha mode、queue。
- mesh 支持 static mesh、skinned pose/morph sideband、LOD 输入、normal/tangent/uv/color 属性验证；LOD 策略先进入 extract/sort，不在本阶段实现高级 streaming。
- sprite 支持 atlas、rect、九宫切片所需数据结构、z/order/layer 排序和批处理输入。
- 实现静态批、动态批、GPU instancing 的 queue preparation 合约和 stats；early-z 只对 opaque/alpha-mask 合法对象启用。
- 测试阶段：
  - `cargo test -p zircon_runtime --locked asset::assets::material asset::assets::mesh asset::assets::texture`
  - `cargo test -p zircon_runtime --locked graphics::scene_renderer::mesh graphics::scene_renderer::sprite`
  - 覆盖 material fallback、missing texture、alpha queue、skinned/morph extract、instancing key、batch stats。

### M7 - Postprocess, Volume, UI Final Composition
- 建立 per-camera volume stack：global/local、priority、weight、layer mask、blend；先覆盖 bloom、grading、tonemap/exposure、blur、vignette、grain、chromatic aberration、fog 的参数路径。
- postprocess 作为 graph feature 串联 scene-color/final-color，不直接写 swapchain。
- 保留现有 UI graph/direct surface damage；UI pass 固定在 world/postprocess 之后、overlay/debug 之前或按 pipeline asset 显式声明。
- 文本渲染、tilemap、复杂排版、terrain/tree 在本计划只保留 extract/render feature 扩展点，不直接展开完整编辑器能力。
- 测试阶段：
  - `cargo test -p zircon_runtime --locked graphics::scene_renderer::post_process`
  - `cargo test -p zircon_runtime --locked rhi_wgpu::ui_surface`
  - 覆盖 volume blending、postprocess order、UI damage cache、final composition target。

### M8 - Advanced Feature Slots
- Compute shader 进入 RHI + RenderGraph queue/pass 描述，可执行简单 compute pass；神经网络仅保留 compute-backed feature 插槽和 capability gate。
- VG/HGI 插件通过 neutral extract、pass descriptor、readback DTO、stats 接入，runtime 不保存插件重状态。
- 粒子、reflection probe、light baking、sparse texture、terrain、tree、projector、halo、lens flare、trail、billboard、tilemap、text shaping 分拆为后续子计划，每个子计划必须接入本主链接口。
- 测试阶段：
  - 插件 executor 缺失时 pipeline 激活失败。
  - 插件启用时不引入 runtime -> plugin 反向依赖。
  - compute pass 能完成资源读写、queue fallback、debug marker。

### M9 - Documentation And Acceptance
- 更新 `docs/` 中对应模块文档，记录 related_code、implementation_files、plan_sources、tests。
- 形成最终接受矩阵：
  - `cargo test -p zircon_runtime --locked rhi render_graph graphics scene asset`
  - `cargo test -p zircon_runtime --locked render_framework`
  - 运行 runtime-focused validation matrix。
  - 工作区级 build/test 只在当前 editor/hub 并行工作收敛后作为最终门禁；若失败来源是活跃 editor/hub UI 改动，单独记录为外部阻塞。

## Assumptions
- 第一交付目标是“渲染主链可扩展闭环”，不是一次性完成所有高级渲染算法。
- WGPU 是唯一必须完整落地的 backend；其他 backend 不在本计划内。
- 现有资产、材质、mesh、UI graph 工作尽量复用，不重写。
- VG/HGI、Nanite/Lumen 类能力走插件化和后续专项计划。
- 不触碰当前并行会话中的 editor workbench、hub material UI 范围，除非它们阻塞 runtime 渲染主链验证。
