# Rendering 插件选项补齐计划

## Summary
- 目标：把计划中 Rendering 池的 8 个选项逐个支持：`post_process`、`ssao`、`decals`、`reflection_probes`、`baked_lighting`、`ray_tracing_policy`、`shader_graph`、`vfx_graph`。
- 主参考：Unreal 的 `Renderer/RenderCore/RHI` 分层、`PostProcessMaterialChainGraph`、`GPULightmass`、`Niagara` 插件模块形态；交叉参考 Unity Graphics 的 `ScriptableRendererFeature` / RenderGraph / ShaderGraph / VFXGraph。
- 落点：新增 `rendering` 伞形插件包，8 个选项作为 owner feature；`zircon_runtime` 只保留中性 RHI、RenderGraph、RenderFeature contract，不反向依赖 `zircon_plugins`。

## Public Interfaces
- 新增 `RuntimePluginId::Rendering`，`parse_key("rendering")` 可识别；`RuntimePluginDescriptor::builtin_catalog()` 加入 `rendering`，目标仅 `client_runtime` + `editor_host`。
- `PluginPackageManifest` 增加默认字段 `category = "rendering"`；`rendering/plugin.toml` 声明 8 个 `optional_features`，每个 feature 提供 runtime/editor crate、capability、默认启用状态和依赖。
- 复用现有 `RuntimePluginFeature`、`RuntimeExtensionRegistry::register_render_feature`、`register_render_pass_executor`；若 shader/vfx 需要资产编译，先在 rendering 插件内落本地 compiler contract，不扩散成新的 root crate。
- 默认策略：为保持现有渲染行为，`post_process`、`ssao`、`reflection_probes`、`baked_lighting` 默认启用；`decals`、`ray_tracing_policy`、`shader_graph`、`vfx_graph` 默认关闭。

## Milestones
- M0 基线收束与证据固化：记录 Unreal/Graphics 参考文件；确认当前 `RenderFeatureDescriptor`、plugin catalog、default pipeline 的实际行为；补一个 docs 小节说明 Rendering 插件 owner/feature 模型。
- M1 Rendering 伞形插件与选项可见：创建 `zircon_plugins/rendering/{runtime,editor}` 和 `features/<feature>/...` 结构；让 Plugin Manager/catalog 能列出 8 个 Rendering 选项；项目 manifest 能保存 nested feature selection。
- M2 RenderFeature 接线硬切：把当前默认管线中的 `post_process`、`ssao`、`reflection_probes`、`baked_lighting` 改为由 rendering feature 注册；关闭 feature 时 compiled graph 不再含对应 pass，开启时保持原有 pass 顺序和 history binding。
- M3 已有能力插件化：按 `post_process -> ssao -> reflection_probes -> baked_lighting` 顺序迁移现有 descriptor、executor id、quality/profile gate、editor capability；GPU 代码可先继续由现有 renderer 内部承载，但必须只由 feature metadata 触发。
- M4 Decals：新增 decal runtime extract/component/descriptor，支持 screen-space/deferred decal pass；editor 插件提供 Decal Projector authoring、inspector drawer 和禁用后 UI 消失的 capability gate。
- M5 Ray Tracing Policy：新增 policy feature，不实现完整硬件 RT；只负责 backend capability 检测、AS/inline ray/ray pipeline gate、依赖诊断，并为 reflection/baked/HGI 等后续 RT 路径提供统一 policy surface。
- M6 Shader Graph：新增 shader graph asset DTO、最小 WGSL compiler、material/post-process target、editor asset shell；首版只支持 constants、texture sample、基础 math、color/material output。
- M7 VFX Graph：依赖 `particles` 和 `shader_graph` capability；新增 VFX graph asset DTO、compile report、runtime simulation/update pass、transparent render pass、editor authoring shell；参考 Niagara 的 runtime/editor/shader 模块拆分。
- M8 集成清理与文档：更新 rendering/plugin docs、render framework docs、插件目录 README；移除旧的内建-only 渲染选项假设，不保留 compat re-export 或旧路径 shim。

## Test Plan
- Catalog/manifest：`rendering` 插件、8 个 feature、workspace members、`plugin.toml` roundtrip、默认启用状态、target modes 全覆盖。
- Dependency/capability：feature 缺 owner、缺 dependency、server target、backend caps 不满足时产生结构化 blocked/diagnostic；不会隐式启用依赖。
- Pipeline graph：启用/禁用每个 feature 后，compiled graph pass、history bindings、required extract sections、executor registry 都符合预期。
- Runtime/editor：禁用插件后不初始化对应 heavy state，editor menus/views/drawers/operations 消失；`zircon_runtime` 不依赖 `zircon_plugins`。
- Commands：里程碑内用 scoped `cargo check`；测试阶段跑 `cargo test -p zircon_runtime --locked plugin_extensions graphics::tests::pipeline_compile`、`cargo test --manifest-path zircon_plugins/Cargo.toml --workspace --locked`，最终跑仓库 validator。

## Assumptions
- “支持”按 V1 解释为：可选择、可保存、可注册、可编译进 RenderGraph、可按 capability 启停，并在已有实现处接入真实行为；不要求一次性完成完整 Lumen/Nanite/Unreal Material Editor/Niagara 级功能。
- Unreal 是模块生命周期和大型系统边界主参考；Unity Graphics 是 SRP/RendererFeature/pass 组织辅助参考；最终命名和边界服从当前 Zircon 三包结构。
- 不新增 `render_server` 或独立 rendering root crate；不复制 UE/Unity API；所有开发继续在当前 `main` checkout。
