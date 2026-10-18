# Runtime/Editor 插件化剩余收敛计划

## Summary
当前基线已完成插件标准接口、catalog、编辑器启停状态、SourceTemplate/LibraryEmbed/NativeDynamic 导出闭环、native ABI v1 descriptor/entry 骨架、native loader/load-manifest 诊断边界、runtime/plugin workspace 迁移、插件窗口基础 UI、editor plugin registration gate、以及多轮 hard-cut residue 清理。剩余目标是把已经插件化的实现继续收敛到更干净的边界：继续拆出 VG/GI 重型 graphics state、深化 editor 插件 UI/template/operation payload、清理过时文档与旧迁移期命名，最后在 active sessions 退场后集中回归。

## Key Changes
- **功能实现迁移到 `zircon_plugins`**
  - physics、sound、animation、navigation、particles、net、texture 已迁入 `zircon_plugins` runtime 包并提供真实注册/最小行为；高级渲染能力已通过 plugin render descriptors 接入，但 VG/GI heavy state 仍在 active Stage 4 图形切分中。
  - runtime diagnostics、UI asset authoring、native window hosting 已作为 editor-only plugin 包存在；runtime-backed editor plugin crates 已注册 authoring view/menu/operation/template contribution。
  - runtime 本体只保留 core lifecycle、asset、scene、base render、input、plugin loader、export/profile contract；剩余争议面主要是 graphics heavy state 的迁移边界。
  - editor 本体只保留 workbench shell、Slint host、基础场景编辑、资产浏览、runtime render embed、插件管理；剩余深化点是 Runtime UI/template payload 和真实 editor plugin contribution 深度。

- **插件窗口 UI 接入**
  - 用 `EditorPluginStatusReport` 渲染插件列表、启用状态、required 状态、runtime/editor crate、capabilities、diagnostics。
  - 启停操作统一调用 `EditorManager::set_project_plugin_enabled(...)`。
  - 支持 builtin catalog 与 native discovered plugin 的状态合并展示。
  - 禁用插件后立即刷新 capability snapshot 和 view registry gate。

- **NativeDynamic manifest/diagnostics/export 后端闭环**
  - ABI v2 descriptor/entry handshake 已有 ABI v1 fallback；当前 entry 仍只贡献 owned manifest/diagnostic/能力协商数据。
  - 动态库 entry 已通过 standard registration/report path 进入 runtime/editor 状态面；在完整 Native ABI v2 行为边界验收前，它不承载真实 runtime/editor 行为调用。
  - ABI 版本不匹配、symbol 缺失、manifest 解析失败、entry 调用失败、load-manifest id/path 不一致、导出根目录逃逸、重复 package id、以及 NativeDynamic 导出目录 sanitizer 碰撞均只进 diagnostics，不破坏最小 runtime/editor 启动。

- **导出 runner**
  - `SourceTemplate` 已生成平台入口源码、Cargo manifest、插件选择代码、资源清单。
  - `LibraryEmbed` 已限制为只链接项目启用的 runtime plugin crates。
  - `NativeDynamic` 已复制动态库、`plugin.toml`、资源包，并生成加载清单；native-aware build path 会按需构建 native cdylib 并保存 diagnostics，但不替代 SourceTemplate/LibraryEmbed 对 linked Rust runtime crate 的行为激活。
  - 默认导出仍走 builtin catalog；需要动态包时使用 native-aware export path。剩余 hardening 主要是 active sessions 退场后的最终回归。

- **清理硬切残留**
  - 删除迁移期别名、compat/shim/facade、旧静态模块装配路径。
  - 清理仍留在 core 中但已被 catalog/feature/capability 替代的手写匹配；active render session 继续处理 VG/GI 旧内建/重型 state。
  - 将文档和测试路径统一到标准 plugin/catalog/export 接口；近期清理包括 editor export host folder-backed 化、native loader diagnostics、NativeDynamic 导出目录去重诊断、descriptor default packaging projection、和 `zircon_plugins/target/` 忽略。

## Public Interfaces
- 保留并继续使用：
  - `RuntimePluginDescriptor`, `RuntimePlugin`, `RuntimePluginCatalog`
  - `EditorPluginDescriptor`, `EditorPlugin`, `EditorPluginCatalog`
  - `EditorPluginStatusReport`, `EditorPluginEnableReport`
  - `NativePluginAbiV1`, `NativePluginAbiV2`, `NativePluginDescriptor`
  - `EditorManager::{plugin_status_report,native_plugin_status_report,set_project_plugin_enabled,generate_native_aware_export_plan}`
- 新增 NativeDynamic ABI v1/v2 manifest 闭环接口：
  - runtime native entry：返回 runtime package manifest/diagnostic contribution
  - editor native entry：返回 editor package manifest/diagnostic contribution
  - export runner report：包含 generated files、copied packages、cargo invocation status、diagnostics

## Test Plan
- **Runtime**
  - `core-min` 无可选插件可编译并启动最小模块集。
  - client/server/editor 模式解析不同 runtime module baseline。
  - required 插件缺失报错，optional 插件缺失仅 diagnostics。
  - native ABI 缺失/版本不匹配/manifest 无效均不阻断最小启动。

- **Editor**
  - 默认插件窗口可显示 builtin catalog 状态。
  - 启用/禁用插件会更新项目清单、capability snapshot、可见 view registry。
  - native discovered plugin 能进入状态列表并显示 diagnostics。
  - 无 capability 的 runtime-only 插件可启停项目选择但不改变 editor UI 能力。

- **Export**
  - SourceTemplate 生成 client/server 入口模板。
  - LibraryEmbed 只链接启用插件 runtime crates。
  - NativeDynamic 复制动态包并生成加载清单。
  - native-aware export plan 能合并动态 descriptor-backed package manifest。

- **Final Regression**
  - 先跑窄检查：`zircon_runtime`, `zircon_editor`, 单个 `zircon_plugins` runtime/editor crate。
  - 迁移完成后再跑 `validate-matrix.ps1`。
  - `zircon_plugins` workspace 独立跑检查，避免 root workspace 默认编译所有插件。

## 2026-04-30 Task 12 验收证据
- 使用共享目标目录 `D:\cargo-targets\zircon-runtime-dynamic-milestone` 完成 Runtime Dynamic / pluginized Runtime 里程碑验证。
- 已通过 coordination helper、`git diff --check -- .codex .opencode docs zircon_runtime zircon_plugins zircon_editor zircon_app`、runtime native loader/export/core-min/plugin-render focused checks、`zircon_plugins` runtime/editor workspace checks、editor plugin/report focused checks。
- 最终 `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -TargetDir "D:\cargo-targets\zircon-runtime-dynamic-milestone"` 通过 root workspace `cargo build --workspace --locked` 和 `cargo test --workspace --locked`。
- 剩余不是本里程碑阻塞：NativeDynamic 真实行为 cdylib 迁移仍等待完整 Native ABI v2 行为边界；VG/GI heavy graphics state 继续由 active render/plugin cutover 会话处理。

## Assumptions
- 继续允许破坏性迁移，不保留旧兼容路径。
- 动态库不是唯一插件后端；SourceTemplate 和 LibraryEmbed 仍是 linked Rust runtime/editor 行为激活路径。
- NativeDynamic 当前 v1/v2 handshake 优先服务 manifest discovery、diagnostics、export package 与 loader 闭环；真实行为 cdylib 迁移仍必须等待完整 Native ABI v2 host function table、opaque handle、版本化 DTO 与 unload/state 边界。
- `GraphicsBase`、基础资产、基础场景、基础渲染桥、插件 loader 属于 runtime 最小本体。
- 插件窗口属于 editor 最小本体，其余高级编辑器能力必须经 capability gate。
