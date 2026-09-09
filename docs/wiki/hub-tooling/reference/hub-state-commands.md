---
related_code:
  - zircon_hub/src/tauri_app/mod.rs
  - zircon_hub/src/tauri_app/commands.rs
  - zircon_hub/src/state/hub_snapshot.rs
  - zircon_hub/src/state/hub_message
  - zircon_hub/src/state/action_history.rs
implementation_files:
  - zircon_hub/src/tauri_app
  - zircon_hub/src/state
plan_sources:
  - docs/plans/zircon_hub/05-frontend-componentization-and-type-safety.md
tests:
  - zircon_hub/tests/tauri_react_shell_contract.rs
  - zircon_hub/tests/hub_docs_contract.rs
  - zircon_hub/tests/app_error_recovery_contract.rs
doc_type: api-reference
---

# Hub 状态、命令与 Tauri 边界

`zircon_hub` 的 Tauri 层是 UI 与 Rust 状态之间的窄边界。`hub_state` 读取可序列化快照；`hub_action` 接收结构化 action request 并启动状态机；账户命令单独走 account broker。Rust 内部错误在边界被转换为 `String`，所以命令实现必须在更早处保留结构化消息和恢复建议。

## 命令入口

```rust
#[tauri::command]
pub fn hub_state(state: State<'_, HubCommandState>)
    -> Result<HubSnapshot, HubError>;

#[tauri::command]
pub fn hub_action(request: HubActionRequest,
                  state: State<'_, HubCommandState>,
                  app: AppHandle)
    -> Result<HubSnapshot, String>;

#[tauri::command]
pub fn account_state(...)
    -> Result<AccountSnapshot, String>;

#[tauri::command]
pub fn account_action(request: AccountActionRequest, ...)
    -> Result<AccountSnapshot, String>;
```

确切的 Tauri state 参数由当前 `commands.rs` 与 `account_commands.rs` 定义；前端只能通过 `invoke("hub_state")`、`invoke("hub_action", request)` 等注册名称访问。不要将内部 `RuntimeState` 或 mutex guard 序列化给 WebView。

## `HubSnapshot`

快照包含当前 `HubScope`、页面、项目视图、引擎目录、任务、最近项目、动作历史和本地化消息所需字段。`scope()` 返回作用域，`filtered_recent_projects()` 返回与选择作用域匹配的历史；快照是读取模型，不是写入 API。

```text
Tauri invoke
  -> command admission
  -> runtime state mutation
  -> background task / process
  -> action history
  -> immutable HubSnapshot
  -> React projection
```

前端投影应保留 `generation` 或 task ID 以丢弃过时响应。刷新不是“重新猜测状态”，而是再次请求权威 snapshot。

## 页面和筛选枚举

`HubPage::{id, from_id}`、`ProjectFilterMode::{id, label, next, from_id}`、`ProjectSortMode`、`ProjectViewMode`、`ProjectSubpage` 都是字符串协议。持久化时写 `id()`；展示时写 `label()`；未知值必须回退而不是 panic。

## 消息协议

```rust
let message = HubMessage::with_params(
    HubMessageId::Project(ProjectMessageId::RootUnavailable),
    [path.to_string_lossy().into_owned()],
);
let text = message.render(HubLanguage::ZhCn);
let with_hint = message.render_with_recovery(Some(&recovery), HubLanguage::ZhCn);
```

`param_count()` 是构造前的契约检查；参数数量不符应在 Rust 侧失败。跨边界传 ID/params/recovery，避免前端按 message 文本判断错误类别。

## 动作历史

```rust
pub struct HubActionRecord { /* action_id, kind, status, timestamps, detail */ }
pub enum HubActionKind { /* source-owned variants */ }
pub enum HubActionStatus { /* source-owned variants */ }

pub fn push_action_record(history: &mut Vec<HubActionRecord>, record: HubActionRecord);
```

`HubActionStatus::succeeded()` 是统一成功判断。历史记录应在任务终态写入，不能只在成功时记录，否则用户无法解释失败动作。

## 前端调用示例

```ts
const snapshot = await invoke<HubSnapshot>("hub_state");
const next = await invoke<HubSnapshot>("hub_action", {
  request: { kind: "refresh_projects", request_id: crypto.randomUUID() }
});
```

TypeScript 的字段名以 `web/src/types/hub.ts` 和命令 DTO 为准；上例强调调用顺序，不能据此创造未注册的 action kind。UI 应处理 pending、success、warning、error、cancelled 五类终态。

## 安全边界

- command 参数必须再次校验，不能信任 UI 已做过的路径检查。
- token、refresh token 和 cookie 不进入 `HubSnapshot`。
- action ID、project key 和路径诊断可记录，但要脱敏。
- Tauri capability 文件限制窗口能调用的命令和插件。
- 文件打开、外部进程和服务请求都要有超时/取消路径。

## 与 Unreal/Godot 的对照

Unreal Editor Utility 常通过消息总线刷新面板；Godot 编辑器插件直接在同进程访问编辑器对象。Zircon Hub 选择 Tauri + snapshot 投影，把 UI 与进程、账户和项目状态隔开，代价是必须维护 DTO、generation 和结构化错误协议。

## 故障恢复

1. invoke 找不到命令：检查 `tauri_app::run` 注册和 capability。
2. 返回旧快照：比较 generation/task ID，重新读取 `hub_state`。
3. 错误只有字符串：回到 command 内部补齐 `HubMessageId` 与 recovery。
4. 前端显示乱码：确认语言、参数数量和 `render` 路径。
5. 任务重复提交：使用 request/action ID 做 admission 去重。

## 测试矩阵

命令合同测试验证 Tauri 注册、项目作用域、选中项目、快速动作和错误恢复；Web 测试验证 projection、导航、账户状态与浏览器回退。修改 DTO 后必须同时更新 Rust 合同测试和 TypeScript projection 测试。

## 相关页面

- [进程、任务与状态协调](process-state.md)
- [UI 与动态 ABI 交接机制](../../mechanisms/ui-render-and-dynamic-abi-handoff.md)
- [UI 文本与输入](../../ui/index.md)
