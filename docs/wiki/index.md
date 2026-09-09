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
| 为 Wiki 网站导入目录和元数据 | [导航清单](navigation.yaml) |

## 文档分区

### 引擎基础

- [架构总览](architecture/index.md)：workspace、crate 层次、依赖方向和边界。
- [Framework 模块目录](architecture/framework-module-catalog.md)：逐项查找 27 个跨模块契约及其 Rust 入口。
- [核心运行时](core-runtime/index.md)：CoreRuntime、模块生命周期、服务解析、事件、时间、任务、Runtime Operation 和诊断日志。
- [场景与资产](scene-assets/index.md)：ECS World、实体/组件、变换、资源注册、项目和持久化。

### 内容与表现

- [图形与渲染](graphics/index.md)：RHI、WGPU、Render Graph、场景渲染、材质、纹理、Shader 和环境光。
- [UI、文本与输入](ui/index.md)：UI 树、布局、模板、绑定、文本 shaping、平台输入和无障碍。
- [脚本、反射与动画](script-animation/index.md)：Host API、反射注册、动态 API、动画和导航接口。

### 编辑与扩展

- [编辑器](editor/index.md)：作者态会话、命令/事务、场景工具、Viewport、Workbench 和资产编辑器。
- [插件](plugins/index.md)：Plugin SDK、linked/native/dist 路径、manifest、能力协商和插件族。
- [App 与 Runtime API](app-runtime-api/index.md)：入口 profile、动态库会话、ABI DTO、宿主输出和世界同步。

### 工具与质量

- [Hub 与工具链](hub-tooling/index.md)：Zircon Hub、`cargo zircon`、导出/打包和 Session Coordinator。
- [GitHub Pages Wiki 发布](hub-tooling/wiki-publishing.md)：本地预览、严格构建、Pages artifact 和自动部署。
- [测试与平台](testing-platform/index.md)：feature/profile 矩阵、Windows 优先验证、诊断、性能和 MVP 验收。
- [Rust API 参考](rust-api.md)：命名、错误、句柄、ABI 安全和示例约定。
- [贡献文档](contributing-docs.md)：如何维护本 Wiki、frontmatter 和自动生成网站导航。

## 版本与状态

- 文档快照日期：2026-09-09。
- workspace 当前分支为 `main`；仓库策略要求直接在 `main` 工作。
- MVP 计划 `docs/plans/mvp/index.md` 当前仍为 `in_progress`。因此“已实现”只表示源码中存在生产入口或契约，不表示整条产品验收闭环已经通过。
- 本 Wiki 的状态标签为：`已实现`、`可扩展基础`、`受限`、`实验`、`规划中`、`内部实现`。定义见[功能状态](feature-status.md)。

## 约定

每个代码相关页面都以 YAML frontmatter 开头，至少包含 `related_code`、`implementation_files`、`plan_sources`、`tests` 和 `doc_type`。正文中的源码与测试链接使用固定 `main` 分支的 GitHub `blob/tree` URL，避免静态站点把仓库路径误解析为站内页面；frontmatter 中的路径仍使用仓库相对路径。代码示例优先引用 crate root 的 curated re-export。`pub(crate)`、测试 feature 和仅用于验证的接口不会被描述为第三方稳定 API。

## 贡献入口

发现文档与当前源码不一致时，先以源码和聚焦测试为准，再按[贡献文档](contributing-docs.md)更新对应叶子页及导航。不要在 Wiki 中复制每次提交的 changelog；持久化设计决策应回链到其 owner 计划或测试。
