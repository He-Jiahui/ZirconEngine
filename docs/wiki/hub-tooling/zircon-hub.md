---
related_code:
  - zircon_hub/src/lib.rs
  - zircon_hub/src/tauri_app/commands.rs
  - zircon_hub/src/tauri_app/runtime_state.rs
  - zircon_hub/src/projects/mod.rs
  - zircon_hub/src/engines/mod.rs
  - zircon_hub/src/process/editor_launch.rs
  - zircon_hub/src/account/mod.rs
implementation_files:
  - zircon_hub/src/tauri_app
  - zircon_hub/src/projects
  - zircon_hub/src/engines
  - zircon_hub/src/process
  - zircon_hub/src/account
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/plans/zircon_hub/03-project-lifecycle-robustness.md
tests:
  - zircon_hub/src/projects/create_project_request.rs
  - zircon_hub/src/projects/validation.rs
  - zircon_hub/src/tauri_app/runtime_state/tests.rs
  - zircon_hub/src/process/editor_handshake/tests.rs
doc_type: module-reference
---

# Zircon Hub

Zircon Hub 是 ZirconEngine 的桌面入口。它把项目管理、源码引擎登记、插件/资产/学习内容目录、构建、编辑器进程和账户服务汇总为一个任务驱动的 Tauri shell。Hub 的可见状态来自 `HubViewModel`，而不是让前端直接读取文件系统或持有 Rust 服务对象。

## 启动和状态流

1. `tauri_app::run` 创建 `HubCommandState` 与账户命令状态。
2. `HubRuntimeSession::load` 读取 Hub 设置、最近项目和本地 catalog。
3. 前端调用 `hub_state` 获取完整快照，调用 `hub_action` 提交一个 `HubActionRequest`。
4. 可耗时 action（构建、安装、编辑器启动、包下载）进入后台 worker；状态改变通过 `hub-state-changed` 事件广播。
5. 窗口重新获得焦点时，Hub 以节流门重新读取共享最近项目，避免多个 Hub 进程互相覆盖。

状态更新采用“动作输入 -> session 变更 -> view model -> 事件”顺序。前端不应根据按钮状态自行推断任务完成；应等待新的 view model，并用任务 id 关联日志和取消操作。

## 项目工作流

### 创建

`CreateProjectRequest` 包含 `project_name`、父目录 `location` 和 `ProjectTemplateId`。当前 `project_template_catalog()` 只有 `renderable-empty` 可创建；`2d-scene`、`3d-scene` 和 `sample-world` 会列出但 `enabled = false`。`validate_launch_fields` 拒绝空位置、非法名称、路径分隔符和保留设备名。

```rust
use zircon_hub::projects::{CreateProjectRequest, enabled_project_template_id};

let template = enabled_project_template_id("renderable-empty")
    .expect("the current MVP template");
let request = CreateProjectRequest::new("MyGame", "E:/Projects", template);
request.validate_launch_fields()?;
assert_eq!(request.target_root(), "E:/Projects/MyGame".into());
```

### 打开和校验

`validate_project_root` 按顺序检查目录、`zircon-project.toml` 和 `ProjectManifestSummary::parse_toml_bytes`。结果是 `Valid`、`MissingRoot`、`MissingManifest` 或 `InvalidManifest`。Hub 应把这四类结果显示为可操作的恢复提示，不能将无效 manifest 当作空项目。

### 引擎登记和构建

`SourceEngineInstall` 保存源码根、显示名、稳定 id、staged 目录和最近的 `SourceBuildRecord`。`validate_source_engine` 至少要求根目录、根 `Cargo.toml`、`zircon_runtime` workspace member 与 `tools/zircon_build.py`。构建请求由 `BuildCommand::for_editor_runtime` 生成，`BuildExecutionReport` 汇总 exit code、日志摘要和恢复建议。

## 编辑器进程

`EditorLaunchRequest` 只接受经过验证的项目根、源码引擎和 profile。`staged_editor_executable` 计算 staged 二进制路径；`editor_handshake` 通过 mailbox 等待启动确认；`SupervisedChild` 和 `EditorChildReaper` 负责异常退出与孤儿子进程回收。启动成功不是进程创建成功，而是握手收到兼容的运行时 epoch 和 project identity。

## 目录与账户

`discover_plugin_catalog`、`discover_asset_catalog` 和 `discover_learn_catalog` 分别扫描引擎级与项目级来源，并用规范化路径去重。账户 broker 在 `account-broker` feature 下启用 OIDC、Windows credential store 和包下载；本地服务在 `local-service` 下启用 SQLite、JWT、云 blob 与组织 API。服务配置要求绝对数据库/secret 路径，并默认只允许 HTTPS；只有显式 `allow_loopback_http` 才允许回环 HTTP。

## Tauri 调用接口

```typescript
// 前端只传序列化 action，返回新的 HubViewModel。
const initial = await invoke("hub_state");
const next = await invoke("hub_action", { request });
listen("hub-state-changed", event => render(event.payload));
```

Rust 端入口是 `hub_state(State<HubCommandState>)` 与 `hub_action(HubActionRequest, State<...>, AppHandle)`。错误统一转换为字符串给 Tauri；详细的 `HubError` message id 仍由 Rust 日志和 view model 保留。账户对应 `account_state` / `account_action`，只有启用 `account-broker` 时才编译。

## 限制与安全规则

- Hub 是桌面工具，不是 Runtime 公共 API；第三方模块不应依赖 `tauri_app` 内部类型。
- 项目包输出不能位于项目源目录内；复制时跳过 `.git` 和 `target`。
- 文件操作使用 regular-file/no-reparse 检查，避免通过 junction 或 symlink 逃逸根目录。
- 账号 token 不写入 view model；凭据由平台 credential store 管理。
