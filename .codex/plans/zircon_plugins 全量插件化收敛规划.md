# zircon_plugins 全量插件化收敛规划

## Summary
- 按“分阶段全量收敛”执行：插件注册到 runtime 启动链、已实现 runtime 模块迁移、非空壳 runtime 插件、editor-only 插件包、SourceTemplate/LibraryEmbed/NativeDynamic 导出闭环已经进入当前基线。
- 当前分类：`physics`、`animation`、`sound`、`net`、`navigation`、`particles`、`texture` 已作为 `zircon_plugins` runtime 包提供真实注册/最小行为；`runtime_diagnostics`、`ui_asset_authoring`、`native_window_hosting` 已作为 editor-only 插件包存在；`virtual_geometry` / `hybrid_gi` 已通过插件 render feature descriptor 接入，且重型 runtime state/feedback/completion 源码已从 `zircon_runtime::graphics` 物理迁到对应 plugin runtime crate。
- 剩余收敛重点：继续把 VG/GI plugin-owned prepare/feedback/runtime host 接回 linked descriptor pipeline、保持 editor 插件 UI/operation capability gate 与真实模板/菜单/操作贡献一致，并清理文档/计划中的旧静态实现描述；native load-manifest/ABI 诊断边界已覆盖导出根目录逃逸、重复 package id、以及 NativeDynamic sanitizer 输出目录碰撞的非致命报告。当前 Native ABI v1/v2 handshake 下的 NativeDynamic 仍是 manifest/diagnostics/export package 后端，真实行为 cdylib 迁移必须等待完整 ABI v2 行为边界。
- 参考依据：现有三份 Runtime/Editor 插件化计划、hard-cutover 规则、Bevy `Plugin` 生命周期、Fyrox `PluginContext`、Godot module init level、Unity Graphics renderer feature 列表式启停模型。

## Public APIs / Interfaces
- 扩展 `RuntimeExtensionRegistry`：新增 `register_module(ModuleDescriptor)`、`register_render_feature(RenderFeatureDescriptor)`、必要的 asset/importer contribution 入口；保留 manager/component/ui-component 注册。
- 扩展 `RuntimePluginRegistrationReport`：携带 package manifest、project selection、module descriptors、render feature descriptors、diagnostics。
- 新增启动链入口：`EntryRunner` / `BuiltinEngineEntry` 支持接收 linked plugin registration reports；`EntryConfig` 继续只保存可序列化 project/export 配置。
- 更新 SourceTemplate/LibraryEmbed：生成的 `zircon_plugins.rs` 必须 import 已启用 runtime crate 并调用 `plugin_registration()`，不再只生成 manifest。
- Editor 侧保持 `EditorPlugin` / `EditorPluginRegistrationReport` 形态，但每个 editor crate 必须实现真实 `register_editor_extensions(...)`，贡献 view/menu/drawer/template/operation，而不是只声明 capability。

## Implementation Changes
- **阶段 1：插件注册闭环（当前基线已完成，继续做回归守护）**
  - runtime module loading 已走“core modules + linked plugin registration reports”。
  - `zircon_runtime` 不再通过 `plugin-sound/plugin-net/...` feature 持有可选实现；required 插件未链接时报错，optional 插件未链接只进 diagnostics。
  - `NativeDynamic` 继续只作为 manifest/diagnostics/export package 后端；真正 Rust 实现激活走 SourceTemplate/LibraryEmbed linked crates，直到完整 Native ABI v2 提供 host function table、opaque handles、版本化 DTO、panic/unload 和 state migration 边界。

- **阶段 2：激活已搬迁实现（当前基线已完成，继续守护无旧路径回退）**
  - `physics/runtime`、`animation/runtime` 已接入 `module.rs/service_types.rs/tests.rs` 并使用 `zircon_runtime::` 合同路径。
  - runtime crate 已提供 `RuntimePlugin` 实现，并在 registration 中注册对应 `ModuleDescriptor`。
  - `zircon_runtime/src/physics`、`zircon_runtime/src/animation` 旧实现路径已不再作为 runtime 本体实现入口。

- **阶段 3：迁移已有 runtime extensions（当前基线已完成，继续防止空壳回退）**
  - `sound`、`net` 的真实 driver/manager/config/tests 已硬切到对应 plugin runtime crate。
  - `sound` 保留对 runtime asset/framework 合同的依赖；`net` 保留 `std-udp` MVP；旧 runtime extension 路径不再作为实现入口。
  - `navigation`、`particles`、`texture` 已补最小 manager/runtime 激活点和行为测试，不再允许只保留 descriptor 空壳。

- **阶段 4：高级渲染插件**
  - `virtual_geometry`、`hybrid_gi` 通过 render feature contribution 接入；`GraphicsBase` 仍留在 runtime core。
  - heavy runtime state、prepare/feedback、GPU completion、Nanite/scene-representation runtime host 源码已迁入对应插件；runtime 只保留中性 render DTO、基础 renderer、通用 feature registry。
  - 当前 base submit path 已移除 VG/GI previous runtime state 持久化与 core record/update feedback 应用；下一步是让 linked plugin execution layer 持有并调度 plugin-owned prepare/feedback/runtime host。
  - 移除 `BuiltinRenderFeature` 中对 VG/GI 的强耦合启用路径，改为 descriptor/capability gate。

- **阶段 5：Editor 插件（包与注册路径已建立，继续收敛真实 UI 深度）**
  - `runtime_diagnostics`、`ui_asset_authoring`、`native_window_hosting` 已存在 editor-only plugin 包。
  - animation/physics/sound/net/navigation/particles/texture/VG/GI 的 editor crates 已通过 shared editor support 注册 authoring view、menu、operation、component drawer 或 UI template contribution。
  - 插件窗口继续显示 builtin + native discovered 状态；启停后刷新 capability snapshot、view registry、menu/operation registry。剩余工作集中在真实模板文档、菜单/操作 payload 深度和 active Runtime UI sessions 的 visual/interaction closure。

- **阶段 6：清理与文档（持续执行）**
  - 删除旧 runtime/editor optional 实现路径、compat/shim/pub-use 迁移层、旧 feature tests。
  - 更新 `docs/engine-architecture/runtime-editor-pluginized-export.md`、runtime extension docs、editor tooling docs，并补 machine-readable related-code header。
  - 近期清理已包括 native load-manifest 诊断一致性、NativeDynamic sanitizer 输出目录去重、descriptor 默认 packaging 投影、`zircon_plugins/target/` 工作区产物忽略，以及已拆分模块的 folder-backed 文档同步。

## Test Plan
- Runtime：`core-min` 无可选插件可启动；linked required 插件可注册并 resolve manager；required 未链接报错；optional 未链接只 diagnostics。
- Plugin crates：逐包跑 `cargo test -p zircon_plugin_<id>_runtime --locked`，再跑 `cargo test --workspace --locked` 于 `zircon_plugins`。
- Migration regression：physics/animation/sound/net 现有行为测试迁到插件 workspace 后仍通过。
- Placeholder closure：navigation/particles/texture 必须有最小功能测试，禁止只测 descriptor 存在。
- Render：VG/GI 未启用时 base render 不拉入 heavy pipeline；启用插件后 feature descriptors、prepare/update/feedback 流程通过原图形测试迁移版。
- Editor：插件启停改变 capability snapshot；插件 view/menu/operation 进入 registry；禁用后不可见不可打开。
- Export：SourceTemplate 生成的 Cargo.toml 链接启用插件 crate，生成代码调用 `plugin_registration()`；LibraryEmbed 只链接 enabled runtime plugins。
- Final：先窄跑 `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked`，再跑 `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1`。

## 2026-04-30 Task 12 验收证据
- Runtime Dynamic / pluginized Runtime 收敛里程碑使用 `D:\cargo-targets\zircon-runtime-dynamic-milestone` 完成 focused checks 与最终 workspace validator。
- `zircon_plugins` 仍保持独立 workspace；runtime plugin crate 通过 `cargo test --manifest-path "zircon_plugins\Cargo.toml" ... --locked --jobs 1` 验证，editor plugin crate 通过 `cargo check --manifest-path "zircon_plugins\Cargo.toml" ... --locked --jobs 1` 验证。
- Root workspace validator `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -TargetDir "D:\cargo-targets\zircon-runtime-dynamic-milestone"` 已通过 `cargo build --workspace --locked` 和 `cargo test --workspace --locked`。
- 剩余 handoff：VG/GI heavy state 抽出继续归 active render/plugin cutover；NativeDynamic 真实行为 cdylib 化继续等待完整 Native ABI v2 行为边界，当前 NativeDynamic 仍只作为 manifest/diagnostics/export/load-manifest 后端。

## Assumptions
- 执行硬切迁移：不保留旧路径 re-export、shim、compat、bridge。
- `zircon_plugins` 保持独立 workspace，不加入 root workspace default members。
- `zircon_runtime::ui`、asset、scene、input、base render、plugin loader 留在最小本体。
- 当前未跟踪/已修改的插件化工作视为后续实现基线，不回滚。
