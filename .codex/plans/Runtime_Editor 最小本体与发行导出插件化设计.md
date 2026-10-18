# Runtime/Editor 最小本体与发行导出插件化设计

## Summary
- Runtime 本体保留最小可运行内核：生命周期/服务注册、运行模式、资产管理、基础场景、基础渲染、插件清单解析与加载。
- Editor 本体保留最小可用编辑器：编辑器核心、Slint/UI host、运行时渲染嵌入、基础场景编辑、项目/资产浏览、插件管理窗口。
- 其余功能迁入 `zircon_plugins`，插件包按 `runtime` crate + `editor` crate 分离。
- 编辑器导出运行时可执行文件时支持两条发行路径：生成平台入口源码再编译，或链接/嵌入 Rust 库；由项目清单和导出 profile 决定。

## Key Changes
- Runtime 最小层：
  - 保留 `zircon_app` 作为入口/profile/export host。
  - 保留 `zircon_runtime::core`、资产/项目 manifest、基础 scene world、基础 render bridge、基础 input、插件 loader。
  - `GraphicsBase` 作为基础渲染留在 runtime；高级渲染能力如 VG/GI、物理、声音、动画、导航、粒子、网络迁到插件。
- Editor 最小层：
  - 保留 workbench shell、Slint host、项目/session、资产浏览、基础场景编辑、runtime render embed、插件管理窗口。
  - 高级 Inspector 扩展、动画编辑、UI asset authoring、runtime diagnostics、native window host、高级调试工具都走插件能力门控。
- `zircon_plugins` 结构：
  - `zircon_plugins/<plugin_id>/plugin.toml`
  - `zircon_plugins/<plugin_id>/runtime/Cargo.toml`
  - `zircon_plugins/<plugin_id>/editor/Cargo.toml`
  - runtime crate 不依赖 `zircon_editor`；editor crate 可依赖 `zircon_editor` 和 runtime 契约。
- 项目清单：
  - `zircon-project.toml` 增加 `plugins` 和 `export_profiles`。
  - 插件启停、目标模式、平台、打包策略都以项目清单为主，环境变量只作调试覆盖。
- 导出系统：
  - `SourceTemplate`：编辑器按平台生成入口源码、Cargo manifest、插件选择代码和资源清单，再调用 Cargo 编译。
  - `LibraryEmbed`：导出工程链接 `zircon_runtime` 与选中的插件 runtime crates，适合快速本机/CI 构建。
  - `NativeDynamic`：复制动态插件包并生成加载清单，优先用于编辑器开发期或需要热插拔的发行包。
  - 默认推荐 `SourceTemplate + LibraryEmbed` 混合：生成清晰源码入口，同时用 Rust crate 链接保证稳定。

## Public Interfaces / Types
- 新增：
  - `RuntimeCoreProfile`
  - `EditorCoreProfile`
  - `PluginPackageManifest`
  - `ProjectPluginManifest`
  - `ExportProfile`
  - `ExportTargetPlatform`
  - `ExportPackagingStrategy::{SourceTemplate, LibraryEmbed, NativeDynamic}`
  - `ExportBuildPlan`
  - `EditorCapabilitySnapshot`
- `EntryConfig` 接收 `RuntimeTargetMode`、项目插件清单、导出 profile。
- `EditorManager` 提供插件目录、能力快照、启停插件、生成导出计划、执行导出构建的入口。
- `ViewDescriptor` 使用 `required_capabilities`，所有窗口注册和打开都走能力检查。

## Test Plan
- Runtime：
  - 最小 runtime 不加载可选插件也能启动。
  - client/server/editor 模式解析出不同基础模块集合。
  - required 插件缺失时报错，optional 插件缺失只产生诊断。
  - runtime core 不依赖 editor。
- Editor：
  - 默认只显示最小编辑器能力和插件管理窗口。
  - 禁用插件时相关视图不可见、不可打开。
  - 启用插件后能力快照、窗口列表、项目清单同步更新。
- Export：
  - 同一项目可生成 client/server 两种入口模板。
  - `SourceTemplate` 产物包含平台入口、Cargo manifest、插件选择代码。
  - `LibraryEmbed` 只链接项目启用的 runtime 插件 crates。
  - 动态插件加载失败不破坏最小运行时启动。
- 验证：
  - root workspace 跑现有 `validate-matrix.ps1`。
  - `zircon_plugins` 独立跑插件 workspace 测试，避免每次默认编译全部插件。

## Assumptions
- 运行时发行包不强制动态链接；动态库只是插件加载后端之一。
- 编辑器比运行时更强调灵活启停、诊断和开发期热插拔。
- 基础渲染、资产管理、插件加载属于 runtime 最小本体。
- 基础场景编辑、运行时渲染嵌入、Slint/UI host、插件管理属于 editor 最小本体。
