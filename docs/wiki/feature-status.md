---
related_code:
  - zircon_runtime/Cargo.toml
  - zircon_app/src/entry/entry_profile.rs
  - zircon_app/src/entry/product_host_config
  - zircon_editor/Cargo.toml
  - zircon_plugins/Cargo.toml
  - docs/plans/mvp/index.md
implementation_files:
  - zircon_runtime/Cargo.toml
  - zircon_app/src/entry
  - zircon_editor/Cargo.toml
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/plans/mvp/index.md
  - docs/runtime-plugins/profile-selection.md
tests:
  - zircon_runtime/src/tests
  - zircon_app/src/tests
  - .github/workflows/ci.yml
doc_type: milestone-detail
---

# 功能状态

状态页把源码能力、feature 门控和 MVP 计划分开。它不是提交日志，也不替代 owner 计划的验收记录。

## 标签定义

| 标签 | 判定标准 |
| --- | --- |
| **已实现** | 当前源码有生产入口，至少有直接测试或宿主调用证据；不代表所有平台均已验收。 |
| **可扩展基础** | 公共 descriptor/contract/registry 已存在，可供后续实现接入，但完整产品路径或 UI 尚不齐全。 |
| **受限** | 功能可用，但受 feature、profile、平台、session、ABI payload limit 或宿主能力约束。 |
| **实验** | 代码或插件存在，但 maturity/默认 profile/稳定性不承诺。 |
| **规划中** | 只在计划、设计或 reference 文档中出现，当前源码不能证明可调用。 |
| **内部实现** | `pub(crate)`、测试 feature 或宿主私有细节；不应作为第三方 API。 |

## 当前功能矩阵

| 功能域 | 主要入口 | 当前判断 |
| --- | --- | --- |
| CoreRuntime、模块生命周期、manager resolver | `zircon_runtime::core` | 已实现；高级生命周期/热重载能力需看具体模块 |
| ECS World、场景组件、项目资产与资源 registry | `zircon_runtime::scene`、`asset`、`resource` | 已实现；MVP 持久化闭环仍由计划验收 |
| RHI/WGPU、Render Graph、场景渲染 | `zircon_runtime::rhi`、`graphics`、`render_graph` | 已实现/受限；平台和 feature 组合影响可用性 |
| UI 树、布局、模板、文本、输入 | `zircon_runtime::ui`、`text`、`input` | 已实现/可扩展基础；部分 v2/editor surface 仍持续收敛 |
| 脚本 Host、反射、动态 API | `script`、`dynamic_api`、`zircon_runtime_interface` | 受限；需要对应 feature 和 ABI lockstep |
| Editor 作者态、命令/事务、Viewport | `zircon_editor` | 已实现；若干编辑器工具为可扩展基础或内部 API |
| Plugin SDK、linked/native/dist | `zircon_plugins/plugin_sdk`、runtime loader | 已实现/受限；插件成熟度逐包不同 |
| Hub、导出、打包、Session Coordinator | `zircon_hub`、`tools/cargo-zircon`、`zircon_runtime` bins | 已实现/受平台和验证器约束 |
| AI、网络、物理、声音等可选族 | contract/插件 feature | 多数是可扩展基础或实验，不默认承诺产品闭环 |

## Feature/profile 解释

`zircon_runtime` 的默认 feature 是 `target-client`，它组合 `core-min`、诊断、动态 API、graphics、navigation、script、text、ui 和默认平台输入。`target-server` 走 headless、最小核心和诊断；`target-editor-host` 包含作者态所需的图形/UI/文本/脚本能力。

feature 只说明编译面，不等于运行时 capability。产品入口还要检查 manifest、插件 maturity、平台后端、窗口/表面和资产 build set。关闭一个可选域后，调用方应得到明确的 unavailable/capability error，而不是空实现。

## MVP 事实

`docs/plans/mvp/index.md` 目前将 F0-F5 标为依赖链，并要求：创建/打开同一项目、加载 registry/settings、渲染 camera/primitive/light、接收输入、通过 editor command 修改 transform、保存/重开、连续两次产品运行。Wiki 页面因此把这些能力标成“源码有入口但整体验收仍受计划状态约束”。

## 如何更新状态

更新叶子页时必须同时检查：

1. crate root 是否仍 re-export 该类型/函数；
2. feature gate 是否变化；
3. 直接测试或 CI 是否仍覆盖；
4. owner 计划是否把能力晋级或回退；
5. 旧描述是否会让读者误以为实验能力稳定。

不要用“有一个 struct”作为已实现依据，也不要把历史 validation receipt 当作当前源码证据。
