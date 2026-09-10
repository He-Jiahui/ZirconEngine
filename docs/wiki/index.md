---
related_code:
  - Cargo.toml
  - zircon_app/src/lib.rs
  - zircon_runtime/src/lib.rs
  - zircon_editor/src/lib.rs
  - zircon_runtime_interface/src/lib.rs
  - zircon_hub/src/main.rs
  - zircon_plugins/Cargo.toml
implementation_files:
  - Cargo.toml
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/plans/mvp/index.md
  - docs/engine-architecture/index.md
tests:
  - docs/plans/mvp/index.md
  - .github/workflows/ci.yml
doc_type: category-index
---

# ZirconEngine 文档

ZirconEngine 是一个以 Rust 为主、采用 Runtime/Editor 分离和插件化扩展的实时引擎工作区。本 Wiki 面向三类读者：使用引擎创建项目的内容作者、编写 Rust 模块或插件的工程师，以及维护宿主、ABI 和构建工具的引擎开发者。

本文档集合借鉴 Unreal Engine 文档的渐进式信息架构：先从概念和快速上手开始，再进入系统专题、工作流指南，最后落到 crate/module/API 参考。页面描述当前源码能证明的行为；路线图、实验能力和 MVP 未验收项会显式标注，不把目标态当成现状。

## 从这里开始

| 目标 | 页面 |
| --- | --- |
| 了解术语、进程和三包架构 | [基础概念](concepts.md) |
| 运行第一个 Runtime/Editor 产品 | [快速开始](getting-started.md) |
| 按产品生命周期理解启动、帧和关停 | [产品生命周期](product-lifecycle.md) |
| 查看当前实现、实验和 MVP 状态 | [功能状态](feature-status.md) |
| 直接查找 Rust 类型、函数和 feature | [Rust API 索引](api-index.md) |
| 逐 crate 核对公开符号、feature 与覆盖状态 | [公开 API 参考](api-reference/index.md) |
| 为 Wiki 网站导入目录和元数据 | [导航清单](navigation.yaml) |
| 按步骤完成一项引擎任务 | [教程集合](tutorials/index.md) |
| 理解状态机、数据流和所有权交接 | [引擎机制指南](mechanisms/index.md) |
| 设计可维护的模块、插件和渲染代码 | [工程最佳实践](best-practices/index.md) |
| 组合一条可落地的生产工作流 | [引擎方案配方](recipes/index.md) |

## 推荐阅读路线

```mermaid
flowchart LR
  A[概念与快速开始] --> B[分步教程]
  B --> C[机制与不变量]
  C --> D[工程最佳实践]
  D --> E[方案配方]
  E --> F[Rust API 与源码]
```

路线图中的每一步都可以独立阅读：教程解决“怎么做”，机制页解释“为什么这样做”，最佳实践约束“怎样长期维护”，方案配方则把多个模块组合成可验收的工作流。

## 文档分区

### 引擎基础

- [架构总览](architecture/index.md)：workspace、crate 层次、依赖方向和边界。
- [Framework 模块目录](architecture/framework-module-catalog.md)：逐项查找 27 个跨模块契约及其 Rust 入口。
- [核心运行时](core-runtime/index.md)：CoreRuntime、模块生命周期、服务解析、事件、时间、任务、Runtime Operation 和诊断日志。
- [场景与资产](scene-assets/index.md)：ECS World、实体/组件、变换、资源注册、项目和持久化。
- [引擎机制指南](mechanisms/index.md)：模块激活、服务解析、帧数据、事件、资产就绪和 UI/ABI 交接。
- [核心运行时接口参考](core-runtime/reference/index.md)：构造、句柄、模块生命周期、任务图、操作服务和诊断 API。
- [场景与资产接口参考](scene-assets/reference/world-entity-query.md)：World/ECS、资产身份、导入、资源就绪、快照与恢复。

### 教程与方案

- [教程集合](tutorials/index.md)：从启动 Runtime、导入项目资产，到编辑器撤销和动态插件会话。
- [工程最佳实践](best-practices/index.md)：Rust 句柄与错误、插件 ABI、编辑器事务、渲染资源生命周期。
- [引擎方案配方](recipes/index.md)：渲染一帧、导入并消费资源、编辑器作者态、宿主世界同步。
- [进阶教程](tutorials/advanced/custom-runtime-module-service.md)：从自定义模块、无头服务器到导出、热重载和 UI/IME 的完整工作流。

### 内容与表现

- [图形与渲染](graphics/index.md)：RHI、WGPU、Render Graph、场景渲染、材质、纹理、Shader 和环境光。
- [UI、文本与输入](ui/index.md)：UI 树、布局、模板、绑定、文本 shaping、平台输入和无障碍。
- [脚本、反射与动画](script-animation/index.md)：Host API、反射注册、动态 API、动画和导航接口。
- [图形接口参考](graphics/reference/render-framework-api.md)：RenderFramework、RenderGraph、RHI、Shader、材质和诊断。
- [UI 接口参考](ui/reference/v2-assets-and-retained-tree.md)：UI v2 资产、保留树、布局、输入、文本和无障碍。
- [脚本与动画接口参考](script-animation/reference/vm-backend-and-manager.md)：VM、反射、Dynamic API V8、动画、导航与热重载。

### 编辑与扩展

- [编辑器](editor/index.md)：作者态会话、命令/事务、场景工具、Viewport、Workbench 和资产编辑器。
- [插件](plugins/index.md)：Plugin SDK、linked/native/dist 路径、manifest、能力协商和插件族。
- [App 与 Runtime API](app-runtime-api/index.md)：入口 profile、动态库会话、ABI DTO、宿主输出和世界同步。
- [编辑器接口参考](editor/reference/host-session-project.md)：Host/Session、命令事务、文档资产、Viewport 和 Workbench。
- [插件接口参考](plugins/reference/index.md)：Descriptor、Manifest、ABI、能力协商、导入器和安全验收。
- [App/Host 接口参考](app-runtime-api/reference/app-entry-runner-api.md)：Entry、ProductHost、Session、窗口输入和导出分发。

### 工具与质量

- [Hub 与工具链](hub-tooling/index.md)：Zircon Hub、`cargo zircon`、导出/打包和 Session Coordinator。
- [GitHub Pages Wiki 发布](hub-tooling/wiki-publishing.md)：本地预览、严格构建、Pages artifact 和自动部署。
- [测试与平台](testing-platform/index.md)：feature/profile 矩阵、Windows 优先验证、诊断、性能和 MVP 验收。
- [真实截图证据画廊](testing-platform/reference/visual-evidence-gallery.md)：只收录编辑器、Runtime、文本、光照和 Hub 的真实截图，并提供不计入证据的执行顺序说明。
- [Rust API 参考](rust-api.md)：命名、错误、句柄、ABI 安全和示例约定。
- [贡献文档](contributing-docs.md)：如何维护本 Wiki、frontmatter 和自动生成网站导航。
- [Hub 与工具链接口参考](hub-tooling/reference/cargo-zircon-cli.md)：项目、构建、Catalog、Receipt、CLI、账户和 Tauri 状态。
- [测试、平台与诊断参考](testing-platform/reference/feature-profile-matrix.md)：feature/profile、契约测试、Windows 验证、性能和 CI 分诊。
- [最佳实践参考](best-practices/reference/api-ownership-and-errors.md)：所有权、背压、缓存、事务、渲染预算、ABI 安全与版本迁移。

## 版本与状态

- 文档快照日期：2026-09-09。
- workspace 当前分支为 `main`；仓库策略要求直接在 `main` 工作。
- MVP 计划 `docs/plans/mvp/index.md` 当前仍为 `in_progress`。因此“已实现”只表示源码中存在生产入口或契约，不表示整条产品验收闭环已经通过。
- 本 Wiki 的状态标签为：`已实现`、`可扩展基础`、`受限`、`实验`、`规划中`、`内部实现`。定义见[功能状态](feature-status.md)。

## 约定

每个代码相关页面都以 YAML frontmatter 开头，至少包含 `related_code`、`implementation_files`、`plan_sources`、`tests` 和 `doc_type`。正文中的源码与测试链接使用固定 `main` 分支的 GitHub `blob/tree` URL，避免静态站点把仓库路径误解析为站内页面；frontmatter 中的路径仍使用仓库相对路径。代码示例优先引用 crate root 的 curated re-export。`pub(crate)`、测试 feature 和仅用于验证的接口不会被描述为第三方稳定 API。

## 贡献入口

发现文档与当前源码不一致时，先以源码和聚焦测试为准，再按[贡献文档](contributing-docs.md)更新对应叶子页及导航。不要在 Wiki 中复制每次提交的 changelog；持久化设计决策应回链到其 owner 计划或测试。
