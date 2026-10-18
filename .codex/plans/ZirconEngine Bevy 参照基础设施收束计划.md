# ZirconEngine Bevy 参照基础设施收束计划

## Summary

以 Bevy 作为第一参照，但不机械复制 Bevy crate 布局。ZirconEngine 应保留当前 `zircon_app` / `zircon_runtime` / `zircon_editor` / `zircon_runtime_interface` 架构，把 Bevy 的价值转译为：清晰的默认模块组合、稳定公共基础设施、可诊断插件能力、通用 ECS/反射/资产模型，以及运行时可复用的输入、UI、渲染、远程控制和可访问性基础。

主要来源：Bevy feature profiles 与默认插件来自 [dev/bevy/Cargo.toml](<E:/Git/ZirconEngine/dev/bevy/Cargo.toml:134>)、[default_plugins.rs](<E:/Git/ZirconEngine/dev/bevy/crates/bevy_internal/src/default_plugins.rs:5>)；Bevy app/plugin/ECS/reflect/asset/render 参考来自 [app.rs](<E:/Git/ZirconEngine/dev/bevy/crates/bevy_app/src/app.rs:86>)、[plugin_group.rs](<E:/Git/ZirconEngine/dev/bevy/crates/bevy_app/src/plugin_group.rs:204>)、[world/mod.rs](<E:/Git/ZirconEngine/dev/bevy/crates/bevy_ecs/src/world/mod.rs:98>)、[reflect.rs](<E:/Git/ZirconEngine/dev/bevy/crates/bevy_reflect/src/reflect.rs:421>)、[assets.rs](<E:/Git/ZirconEngine/dev/bevy/crates/bevy_asset/src/assets.rs:288>)、[camera.rs](<E:/Git/ZirconEngine/dev/bevy/crates/bevy_render/src/camera.rs:81>)。Zircon 当前基线来自 [Cargo.toml](<E:/Git/ZirconEngine/Cargo.toml:1>)、[zircon_runtime/Cargo.toml](<E:/Git/ZirconEngine/zircon_runtime/Cargo.toml:8>)、[runtime lib.rs](<E:/Git/ZirconEngine/zircon_runtime/src/lib.rs:1>)、[runtime_modules.rs](<E:/Git/ZirconEngine/zircon_runtime/src/builtin/runtime_modules.rs:30>)、[CoreRuntime](<E:/Git/ZirconEngine/zircon_runtime/src/core/runtime/runtime.rs:22>)、[World](<E:/Git/ZirconEngine/zircon_runtime/src/scene/world/world.rs:16>)、[Schedule](<E:/Git/ZirconEngine/zircon_runtime/src/scene/components/schedule.rs:4>)、[zircon_plugins/Cargo.toml](<E:/Git/ZirconEngine/zircon_plugins/Cargo.toml:1>)，以及 [core runtime service registry doc](<E:/Git/ZirconEngine/docs/engine-architecture/core-runtime-service-registry.md:1>)、[plugin feature bundle doc](<E:/Git/ZirconEngine/docs/engine-architecture/plugin-optional-feature-bundles.md:1>)、[runtime diagnostics doc](<E:/Git/ZirconEngine/docs/engine-architecture/runtime-diagnostics-contract.md:1>)。

## Comparison

| Bevy 模块族 | Zircon 当前状态 | 缺失或不一致点 |
|---|---|---|
| `app` / `prelude` / `state` / `time` / `tasks` / `log` / `diagnostic` | 部分支持。Zircon 已有 `CoreRuntime`、模块描述符、生命周期、service registry、rayon job scheduler、诊断文档。 | 缺少 Bevy 式 `DefaultPlugins` / `MinimalPlugins` / `PluginGroup` 组合层；缺少稳定 `prelude`、通用 `State`、`Time`、任务池抽象、默认 log/diagnostics/dev profile。 |
| `ecs` / `reflect` / `scene` / `transform` | 部分支持。Zircon 有 scene `World`、entity、transform、stage schedule、序列化与 editor scene 边界。 | 当前更像固定组件 scene world，不是 Bevy 式通用 ECS；缺少统一 typed component/resource/query、change detection、observers/lifecycle events、derive-style reflection/type registry。 |
| `asset` / `image` / `gltf` / `shader` / `mesh` / `material` | 部分支持。Zircon 有 asset 模块、项目资产、importer plugins、artifact/export 方向。 | 缺少统一公共 `Assets<T>` / `Handle<T>` / `AssetEvent` / `LoadState` 风格模型；导入器插件丰富，但默认加载面和能力诊断不够统一。 |
| `render` / `core_pipeline` / `camera` / `light` / `pbr` / `post_process` / `anti_alias` / `solari` | 强但不均衡。Zircon 渲染、RHI、render graph、virtual geometry、hybrid GI 投入较深。 | 基础分层不如 Bevy 清晰：camera/image/mesh/material/shader/light/core pipeline/PBR/sprite/UI render/default 2D/3D profile 需要显式化；高级渲染不应替代基础渲染产品面。 |
| `ui` / `text` / `ui_widgets` / `feathers` / `input_focus` / `a11y` | UI 与 editor UI 有较多实现，runtime interface 也有 UI contract。 | `a11y` 基本缺失；runtime 级 input focus、标准 widget、styled editor widgets、UI picking 与 UI render 边界需要收束。 |
| `input` / `window` / `winit` / `gilrs` / `platform` | 基础存在。 | 缺少 Bevy 式 platform feature matrix、gamepad/gilrs、窗口/事件循环默认平台组合、跨端能力声明。 |
| `picking` / `gizmos` / `camera_controller` / `dev_tools` / `remote` | editor 侧有 viewport/overlay/handles，runtime 有 diagnostics/dynamic_api/interface。 | 缺少运行时可复用 picking、gizmos、camera controller、remote protocol；当前更偏 editor 内部能力，不是公共 runtime primitive。 |
| `animation` / `audio` / `sprite` / `particles` / `navigation` / `net` | plugin workspace 已覆盖很多名字，animation/physics/sound 有真实 runtime plugin。 | 很多能力仍是 optional/externalized 或较薄插件；`RuntimePluginId` 声明广，但 builtin resolver 对多数插件只给 “externalized” 警告，缺少成熟度分类和 profile 级选择。 |

## Key Changes

1. 建立 Bevy-inspired 的 Zircon profile/module group 层  
   定义 `MinimalRuntime`、`DefaultRuntime`、`Client2d`、`Client3d`、`EditorHost`、`HeadlessServer`、`DevTools` 等组合。保留现有 `target-client` / `target-server` / `target-editor-host`，但把 Cargo features、project plugin manifest、runtime descriptors 合并到一个可诊断 resolver。resolver 需要支持 enable/disable/order/replace，并输出缺失插件、外部插件、stub/no-op、editor-only 的结构化报告。

2. 收束核心基础设施  
   在 `zircon_runtime` 内补齐或标准化 `Time`、`State`、`TaskPool/AsyncTask`、`Diagnostics`、`Log`、`Color`、`A11y`、`RemoteControl` 的公共 contract。当前三根包结构作为权威架构；旧文档中独立 `zircon_core` / `zircon_manager` / `zircon_framework` 的表述应迁移为 `zircon_runtime::core::{runtime, manager, framework, math, resource}` 内部 spine。

3. 建立 ECS / reflection / asset 公共模型  
   不立即替换现有 scene world，而是在其上或其下加入通用 component/resource/type registry、query、change/lifecycle 事件和 reflection serialization。editor inspector、scene serialization、dynamic API、plugin metadata 都应使用同一套 type registry。资产层增加 typed handle、asset event、load state、dependency load state，把现有 importer/artifact pipeline 挂接进去。

4. 把交互能力从 editor 内部提升为 runtime primitive  
   picking、gizmos、camera controller、input focus、gamepad/window events、accessibility tree 都应是 runtime/interface 层可复用能力，editor 只消费和扩展它们。remote protocol 基于 `zircon_runtime_interface` 与 `dynamic_api` 提供 scene/assets/diagnostics/query/control 的稳定外部接口。

5. 做一次渲染与插件成熟度分层  
   渲染公共 API 按 camera、image、mesh、material、shader、light、core pipeline、PBR、sprite/2D、post process、anti alias、UI render 分层。`virtual_geometry`、`hybrid_gi`、ray tracing 类能力保持 optional feature bundle。对每个 `RuntimePluginId` 输出成熟度：`default`、`available_external`、`stub_or_noop`、`editor_only`、`missing`，避免“名字存在但能力不完整”的误判。

## Test Plan

- Profile resolver tests：覆盖 Minimal、Default、2D、3D、UI、EditorHost、HeadlessServer、DevTools 的模块选择、顺序、禁用、替换、缺失报告。
- ECS/reflection tests：注册 reflected component/resource，查询、修改、序列化/反序列化、editor inspect、dynamic component plugin roundtrip。
- Asset tests：typed handle 生命周期、asset event、load state、dependency load state、hot reload/importer fallback。
- Runtime interaction tests：input focus、picking、gizmos、camera controller、gamepad/window event、a11y tree update 不依赖 editor 私有状态。
- Remote/diagnostics tests：远程查询 scene/assets/diagnostics；未启用、未授权、插件缺失时返回结构化错误。
- Render profile tests：2D、3D、UI render profile 在无 virtual geometry / hybrid GI 时仍能建立基础 extract/render contract。
- Validation commands：`cargo check --workspace --locked`，`cargo test -p zircon_runtime --lib`，`cargo test -p zircon_app --lib`，`cargo test -p zircon_editor --lib`，再对 `zircon_plugins` 中 animation/physics/sound/rendering/asset importers 做 targeted tests。

## Assumptions

- Bevy 是基础设施与产品面参照，不是 crate 结构或 API 名字的直接复制对象。
- Zircon 当前根架构以 `zircon_app`、`zircon_runtime`、`zircon_editor`、`zircon_runtime_interface` 为准。
- 高成本能力继续作为插件存在；基础 contract 必须落在 runtime/interface 层。
- 本轮仅做只读分析和计划，没有修改仓库文件。
