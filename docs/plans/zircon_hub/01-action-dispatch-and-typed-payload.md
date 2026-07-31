---
related_code:
  - zircon_hub/src/tauri_app/action_request.rs
  - zircon_hub/src/tauri_app/action_id.rs
  - zircon_hub/src/tauri_app/commands.rs
  - zircon_hub/src/tauri_app/runtime_state.rs
  - zircon_hub/src/tauri_app/runtime_state/action_tasks.rs
  - zircon_hub/src/tauri_app/runtime_state/action_targets.rs
  - zircon_hub/src/tauri_app/view_model.rs
  - zircon_hub/src/tauri_app/view_model/settings_dto.rs
  - zircon_hub/src/tauri_app/view_model/localized.rs
  - zircon_hub/web/src/types/hub.ts
  - zircon_hub/web/src/tauri/hubApi.ts
  - zircon_hub/tests/project_workflow_contract.rs
  - zircon_hub/tests/project_quick_actions_contract.rs
  - zircon_hub/tests/ui_foundation_contract.rs
  - zircon_hub/tests/ui_project_scope_contract.rs
  - zircon_hub/tests/ui_input_navigation_api_contract.rs
plan_sources:
  - docs/plans/zircon_hub/index.md
  - .codex/plans/Zircon Hub 本地闭环 v1 功能实现设计.md
  - docs/zircon_hub/pages/actionable-pages.md
status: planned
---

# 01 action 分发与 payload 类型化收敛

## 现状与证据

> 【落地状态终核（2026-06-12）】本节为撰写时（落地前）的现状证据。截至 2026-06-12 工作树（未提交改动）中，本计划 M1/M2/M3 已全部落地：`tauri_app/action_id.rs` 已存在（`HubActionId` 实为 31 个变体），`parse()` 的字符串分支、`commands.rs` 的字符串比对、八个信封结构与九个 `*_payload_from_value` 帮助函数均已删除，M3 双向守卫测试已上线。本节各行号与「问题现状」描述保留为历史快照，不再对应当前源码；当前实仓形态与提案差异见各里程碑的终核注记。

- `HubActionRequest { action_id: String, target_id: Option<String>, payload: Option<Value> }`（`action_request.rs:10-17`）是 IPC 入口，`parse()` 在 `action_request.rs:236-344` 用 30+ 个字符串分支转成 `HubAction` enum——enum 本身已存在，问题在字符串字面量是事实上的协议且无单一来源。
- 后台动作判定在 `commands.rs:73-88` 再次硬编码 `"build-project"`/`"package-project"`/`"install-device"`/`"open-editor"` 四个字符串，与 `parse()` 和 `action_tasks.rs` 的 `BackgroundHubAction` 三处各自维护同一张表。
- payload 按 action 临时 `serde_json::from_value`（如 `create_project_payload_from_value`，`action_request.rs:408-419` 还兼容 `{ project: {...} }` 信封两种形状），无统一校验与错误口径。
- 前端 `web/src/types/hub.ts` 的 `HUB_ACTION` 常量表 + `HubActionPayloadById` 泛型映射与 Rust 字符串靠人工同步，无任何一致性守卫。
- `view_model.rs`（撰写时 1357 行；终核时实测 1367 行）与 `runtime_state.rs`（撰写时 1076 行；终核时实测 1268 行）中 action 相关的路由、目标解析、record 写入相互交织，是 P8 巨型文件的主要成因之一。

## 目标

1. action id 单一来源：Rust 侧一张 `HubActionId`（或扩展现有 `HubAction` 的 discriminant）常量表，`parse()`、后台判定、`BackgroundHubAction`、`HubViewModel::quick_actions` 的 DTO id 全部从它派生；删除所有重复字符串字面量。（注：action history 的 `HubActionKind` 是独立词表——其 id 如 `"build-editor-runtime"`/`"install-project"`/`"open-output"` 与分发 id 不同名，不并入本表，见风险章节。）
2. payload 类型化：每个带 payload 的 action 拥有专用 DTO（多数已存在，如 `HubSettingsPayload`、`CreateProjectActionPayload`），统一经一个 `parse_payload<T>` 入口完成反序列化 + 校验（非空、绝对路径、枚举合法值），失败返回带恢复建议的本地化错误，而不是 serde 原始错误。
3. 前后端契约守卫：契约测试断言 `web/src/types/hub.ts` 的 `HUB_ACTION` 值集合与 Rust action id 表逐一对应（双向：前端没有 Rust 不识别的 id，Rust 的可分发 id 前端都有类型定义）。
4. 移除 `{ project: {...} }` 信封等兼容形状：前端统一发扁平 payload，Rust 删除双形状解析（硬切换，同变更迁移前端调用点）。

## 非目标

- 不改 IPC 形状 `{ actionId, targetId?, payload? }` 本身（契约已锁定）。
- 不引入宏/代码生成框架；常量表用普通 Rust 模块维护即可，避免为 30 个 id 引 build script。
- 不在本计划内拆 `view_model.rs`（DTO 投影侧拆分归 07 与 02 的附带切片）。

## 里程碑

### M1 action id 表收敛

> 【落地状态终核（2026-06-12）】M1 已全部落地，下述切片/实施步骤转为「盘点补缺/验收」口径执行（验收命令 `rg '"build-project"' zircon_hub/src` 已满足：非测试命中仅剩 `action_id.rs:101` 的 `as_str` 单点定义）。与提案的实仓差异，以实仓为准：
> - 实仓枚举为 **31 个变体**（非提案的 29）：并行变更新增了 `DiscardSettingsDraft`/`RestoreDefaultSettings`（`"discard-settings-draft"`/`"restore-default-settings"`），前端 `HUB_ACTION` 已同步为 31 个 id（含 `updateNewProjectDraft`）。提案代码块中的 `ALL: [HubActionId; 29]`、`ALL.len() == 29` 实落为 31。
> - `commands.rs` 落地形态与下方提案代码块不同：`spawn_background_action` 不再按 action id 分派，而是统一调用 `run_background_worker_loop`（`runtime_state/action_tasks.rs`），由 `dispatch_background_request` 经 `BackgroundHubAction::from_request` 单点映射，并新增后台动作 FIFO 队列（`start_background_action_or_record_error`/`take_next_background_action`/`background_worker_active`）；「删除四处字符串比对」的目标已达成，但不存在提案中的 `match HubActionId::from_str` 形态。
> - `action_request.rs` 的 `action()`/`parse_as()` 拆分、`action_targets.rs` 第三参改 `HubActionId`、`runtime_state.rs` 四个后台 arm（现 227-258 行）、`view_model.rs::quick_actions`（现 495-538 行）、`project_delivery_actions.rs` 伪命令行 id（现 331/345 行）均已按提案落地；各契约测试 snippet 已刷新为「契约联动」表「改为」列形态（如 `project_workflow_contract.rs:106-235`）。

切片：
1. 在 `tauri_app/action_request.rs`（或新 `tauri_app/action_id.rs`，root wiring 保持薄）落 `HubActionId` 枚举 + `from_str`/`as_str`，`parse()` 先解析 id 再按枚举分支；未知 id 错误消息保持现有本地化口径。
2. `HubRuntimeSession::should_run_action_in_background` 与 `commands.rs::spawn_background_action` 改为 `match HubActionId`，删除四处字符串比对；`BackgroundHubAction::from(HubActionId)` 单点映射。
3. 同变更内更新 `project_workflow_contract.rs` 等契约对分发器的断言。

#### 目标代码形状

选定落点：新建 `zircon_hub/src/tauri_app/action_id.rs`（`tauri_app/mod.rs` 现为 30 行薄 wiring，只追加一行 `mod action_id;`）。枚举共 29 个变体 = 前端 `HUB_ACTION` 现有 28 个 id（`web/src/types/hub.ts:613-642`）+ `update-new-project-draft`（现仅 Rust 侧支持，`action_request.rs:268` 附近）。【终核修正（2026-06-12）】实落为 31 个变体：在上述 29 个之外，并行变更同时新增了 `discard-settings-draft`/`restore-default-settings` 两个 settings 草稿动作，且前端 `HUB_ACTION`（`hub.ts:617-649`）已同步为 31 个 id；下方代码块按提案原貌保留，盘点时以实仓 `action_id.rs` 为准。

```rust
// 新建 zircon_hub/src/tauri_app/action_id.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HubActionId {
    ShowPage,
    ShowProjectSubpage,
    SearchProjects,
    SetProjectFilter,
    SetProjectSort,
    SetProjectViewMode,
    SelectProject,
    OpenProjectDetail,
    ViewAllProjects,
    NewProject,
    UpdateNewProjectDraft,
    SelectEngine,
    UpdateSettingsDraft,
    SaveSettings,
    BrowseSettingsFolder,
    CreateProject,
    ImportProject,
    PinProject,
    UnpinProject,
    RemoveFromHub,
    RequestDelete,
    CancelDelete,
    ConfirmDelete,
    OpenResource,
    OpenOutputFolder,
    BuildProject,
    PackageProject,
    InstallDevice,
    OpenEditor,
}

impl HubActionId {
    pub(crate) const ALL: [HubActionId; 29] = [
        Self::ShowPage,
        Self::ShowProjectSubpage,
        Self::SearchProjects,
        Self::SetProjectFilter,
        Self::SetProjectSort,
        Self::SetProjectViewMode,
        Self::SelectProject,
        Self::OpenProjectDetail,
        Self::ViewAllProjects,
        Self::NewProject,
        Self::UpdateNewProjectDraft,
        Self::SelectEngine,
        Self::UpdateSettingsDraft,
        Self::SaveSettings,
        Self::BrowseSettingsFolder,
        Self::CreateProject,
        Self::ImportProject,
        Self::PinProject,
        Self::UnpinProject,
        Self::RemoveFromHub,
        Self::RequestDelete,
        Self::CancelDelete,
        Self::ConfirmDelete,
        Self::OpenResource,
        Self::OpenOutputFolder,
        Self::BuildProject,
        Self::PackageProject,
        Self::InstallDevice,
        Self::OpenEditor,
    ];

    /// 唯一的 canonical action id 字符串表。
    /// M3 契约守卫按本函数体内的字符串字面量提取全集；
    /// 历史别名一律放 `from_str`，禁止出现在这里。
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::ShowPage => "show-page",
            Self::ShowProjectSubpage => "show-project-subpage",
            Self::SearchProjects => "search-projects",
            Self::SetProjectFilter => "set-project-filter",
            Self::SetProjectSort => "set-project-sort",
            Self::SetProjectViewMode => "set-project-view-mode",
            Self::SelectProject => "select-project",
            Self::OpenProjectDetail => "open-project-detail",
            Self::ViewAllProjects => "view-all-projects",
            Self::NewProject => "new-project",
            Self::UpdateNewProjectDraft => "update-new-project-draft",
            Self::SelectEngine => "select-engine",
            Self::UpdateSettingsDraft => "update-settings-draft",
            Self::SaveSettings => "save-settings",
            Self::BrowseSettingsFolder => "browse-settings-folder",
            Self::CreateProject => "create-project",
            Self::ImportProject => "import-project",
            Self::PinProject => "pin-project",
            Self::UnpinProject => "unpin-project",
            Self::RemoveFromHub => "remove-from-hub",
            Self::RequestDelete => "request-delete",
            Self::CancelDelete => "cancel-delete",
            Self::ConfirmDelete => "confirm-delete",
            Self::OpenResource => "open-resource",
            Self::OpenOutputFolder => "open-output-folder",
            Self::BuildProject => "build-project",
            Self::PackageProject => "package-project",
            Self::InstallDevice => "install-device",
            Self::OpenEditor => "open-editor",
        }
    }

    pub(crate) fn from_str(value: &str) -> Option<Self> {
        let value = value.trim();
        if let Some(action) = Self::ALL.into_iter().find(|action| action.as_str() == value) {
            return Some(action);
        }
        // 历史别名：仅作入口兼容（现状 parse() 的 "show-page" | "page" 等多模式分支），
        // 不进入 as_str() canonical 表，前端契约守卫不要求别名同步。
        match value {
            "page" => Some(Self::ShowPage),
            "project-subpage" => Some(Self::ShowProjectSubpage),
            "open-project" => Some(Self::SelectProject),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_action_id_round_trips_between_as_str_and_from_str() {
        assert_eq!(HubActionId::ALL.len(), 29);
        for action in HubActionId::ALL {
            assert_eq!(HubActionId::from_str(action.as_str()), Some(action));
        }
    }

    #[test]
    fn legacy_aliases_and_whitespace_resolve_to_canonical_actions() {
        assert_eq!(HubActionId::from_str("page"), Some(HubActionId::ShowPage));
        assert_eq!(
            HubActionId::from_str("project-subpage"),
            Some(HubActionId::ShowProjectSubpage)
        );
        assert_eq!(
            HubActionId::from_str("open-project"),
            Some(HubActionId::SelectProject)
        );
        assert_eq!(
            HubActionId::from_str(" build-project "),
            Some(HubActionId::BuildProject)
        );
        assert_eq!(HubActionId::from_str("upload-to-cloud"), None);
    }
}
```

`action_request.rs`：`parse()`（现为 30+ 个字符串分支，236-344 附近）拆成两段——id 解析与 payload 解析分离，为 M2「payload 错误落 TaskStatus、未知 id 维持 Err」铺路：

```rust
// action_request.rs，替换现 parse()（236-344 附近）
use super::action_id::HubActionId;

impl HubActionRequest {
    pub(crate) fn action(&self) -> Result<HubActionId, HubError> {
        HubActionId::from_str(&self.action_id).ok_or_else(|| {
            HubError::message(format!("Unknown Hub action: {}", self.action_id))
        })
    }

    pub(crate) fn parse(&self) -> Result<HubAction, HubError> {
        let action = self.action()?;
        self.parse_as(action)
    }

    pub(in crate::tauri_app) fn parse_as(&self, action: HubActionId) -> Result<HubAction, HubError> {
        match action {
            HubActionId::ShowPage => Ok(HubAction::ShowPage {
                target_id: self.required_target()?.to_string(),
            }),
            // ……其余 28 个 arm 与现状一一对应，M1 阶段右侧仍调用现有
            // *_payload_from_value 帮助函数（M2 再替换为 parse_payload）。
            HubActionId::BuildProject => Ok(HubAction::BuildProject {
                target_id: self.trimmed_target(),
                payload: project_target_payload_from_value(self.payload.as_ref())?,
            }),
            // ...
        }
    }
}
```

要点：`match` 改为对 `HubActionId` 穷尽匹配后不再需要 `_ =>` 兜底分支——未知 id 在 `action()` 即返回 `Unknown Hub action: {id}`（错误原文不变，`action_request.rs:339-343` 现状口径，单测 `unknown_action_is_rejected_before_runtime_routing` 断言 `"Unknown Hub action: upload-to-cloud"` 原样保留）。

`commands.rs::spawn_background_action`（现为四个 `request.action_id.trim() == "..."` 比对，73-88 附近）：

```rust
// commands.rs
use super::action_id::HubActionId;

fn spawn_background_action(
    request: HubActionRequest,
    session_handle: Arc<Mutex<HubRuntimeSession>>,
    app: tauri::AppHandle,
) {
    thread::spawn(move || {
        match HubActionId::from_str(&request.action_id) {
            Some(HubActionId::BuildProject) => {
                return run_background_build_action(request, session_handle, app);
            }
            Some(HubActionId::PackageProject) => {
                return run_background_package_action(request, session_handle, app);
            }
            Some(HubActionId::InstallDevice) => {
                return run_background_install_action(request, session_handle, app);
            }
            Some(HubActionId::OpenEditor) => {
                return run_background_editor_action(request, session_handle, app);
            }
            _ => {}
        }
        // 通用 apply_action 回退路径（现状 90-103 行）原样保留
        ...
    });
}
```

`action_tasks.rs::BackgroundHubAction`（现 `from_request` 在 19-25 行重复四个字符串）：

```rust
// runtime_state/action_tasks.rs
use crate::tauri_app::action_id::HubActionId;

impl BackgroundHubAction {
    fn from_request(request: &HubActionRequest) -> Option<Self> {
        HubActionId::from_str(&request.action_id).and_then(Self::from_action_id)
    }

    fn from_action_id(action: HubActionId) -> Option<Self> {
        match action {
            HubActionId::BuildProject => Some(Self::BuildProject),
            HubActionId::PackageProject => Some(Self::PackageProject),
            HubActionId::InstallDevice => Some(Self::InstallDevice),
            HubActionId::OpenEditor => Some(Self::OpenEditor),
            _ => None,
        }
    }
    // label()/detail()/operation()/fallback_target() 不变
}
```

（计划原文写 `BackgroundHubAction::from(HubActionId)`；因映射是部分函数（仅 4/29 个 id 是后台动作），`From` trait 无法返回 `Option`，落地为 `from_action_id` 命名。）

`action_targets.rs`：`apply_action_project_target` 第三参由 `&str` 改 `HubActionId`（现签名在 22-27 行），错误消息渲染结果不变（`localized.rs` 现 269 行附近按 `"Unknown recent project target for "` 前缀翻译，不能破坏）：

```rust
// runtime_state/action_targets.rs
pub(in crate::tauri_app) fn apply_request_project_target(
    &mut self,
    request: &HubActionRequest,
) -> Result<(), HubError> {
    let action = request.action()?;            // 现为 request.action_id.trim() 直传
    let payload = request.project_target_payload()?;
    self.apply_action_project_target(request.target_id.as_deref(), payload.as_ref(), action)
}

pub(in crate::tauri_app) fn apply_action_project_target(
    &mut self,
    target_id: Option<&str>,
    payload: Option<&ProjectTargetActionPayload>,
    action: HubActionId,                        // 现为 action_id: &str
) -> Result<(), HubError> {
    // ...
    return Err(HubError::message(format!(
        "Unknown recent project target for {}: {}",
        action.as_str(),
        targets[0]
    )));
    // ...
}
```

`runtime_state.rs::apply_action` 四个后台 arm（现 201-232 行传 `"build-project"` 等字面量）改传 `HubActionId::BuildProject` 等常量；`view_model.rs::quick_actions`（487-531 附近，491/502/516/530 行四个 id 字面量）改为 `HubActionId::BuildProject.as_str()` 等；`project_delivery_actions.rs` 伪命令行（325-349 附近 `"package-project".to_string()` / `"install-device".to_string()`）改 `HubActionId::PackageProject.as_str().to_string()` 等。

#### 文件变更清单

| 路径 | 动作 | 变更内容一句话 |
|------|------|----------------|
| `zircon_hub/src/tauri_app/action_id.rs` | 新建 | `HubActionId` 枚举 + `ALL` + `as_str`/`from_str` + 别名表 + 单元测试，canonical id 唯一来源 |
| `zircon_hub/src/tauri_app/mod.rs` | 修改 | 模块声明区（1-4 行）追加 `mod action_id;` |
| `zircon_hub/src/tauri_app/action_request.rs` | 修改 | `parse()` 拆为 `action()` + `parse_as(HubActionId)`，删 30+ 字符串分支与 `_ =>` 兜底 |
| `zircon_hub/src/tauri_app/commands.rs` | 修改 | `spawn_background_action` 四处 `action_id.trim() == "..."`（73-88）改 `match HubActionId::from_str` |
| `zircon_hub/src/tauri_app/runtime_state/action_tasks.rs` | 修改 | `BackgroundHubAction::from_request` 改经 `HubActionId` + 新增 `from_action_id` 单点映射 |
| `zircon_hub/src/tauri_app/runtime_state/action_targets.rs` | 修改 | `apply_action_project_target` 第三参 `&str` → `HubActionId`；`apply_request_project_target` 先 `request.action()?` |
| `zircon_hub/src/tauri_app/runtime_state.rs` | 修改 | `apply_action` 四个后台 arm（201-232）改传 `HubActionId` 常量 |
| `zircon_hub/src/tauri_app/view_model.rs` | 修改 | `quick_actions`（487-531）四个 DTO id 改 `HubActionId::*.as_str()` |
| `zircon_hub/src/tauri_app/runtime_state/project_delivery_actions.rs` | 修改 | 伪命令行 id 字符串（328/342）改 `as_str()` 派生 |
| `zircon_hub/tests/project_workflow_contract.rs` | 修改 | 刷新 parse 分支 / runtime_state arm / action_tasks 映射的 snippet 断言 |
| `zircon_hub/tests/project_quick_actions_contract.rs` | 修改 | 同上（runtime_state + action_tasks 两个断言块） |
| `zircon_hub/tests/ui_foundation_contract.rs` | 修改 | 同上（action_request + action_tasks 两个断言块） |
| `zircon_hub/tests/ui_project_scope_contract.rs` | 修改 | 刷新 view_model quick_actions 与 runtime_state arm 的 snippet 断言 |

#### 实施步骤

1. 新建 `action_id.rs` 全文（见目标代码形状），在 `tauri_app/mod.rs:1-4` 模块声明区加 `mod action_id;`。此时尚无调用方，仅验证：`cargo check -p zircon_hub --locked`，`cargo test -p zircon_hub --lib tauri_app::action_id --locked`。
2. 改 `action_request.rs`：把 `parse()`（236-344 附近）重写为 `action()` + `parse_as()`，逐 arm 平移（右侧 payload 帮助函数调用原样保留）；删除 `_ =>` 分支；`use super::action_id::HubActionId;`。同步刷新 `project_workflow_contract.rs`（118-129 附近 parse 分支 snippet）与 `ui_foundation_contract.rs`（611-617 附近）。验证：`cargo test -p zircon_hub --lib --locked`、`cargo test -p zircon_hub --test project_workflow_contract --test ui_foundation_contract --locked`。
3. 改 `action_tasks.rs`（`from_request`/`from_action_id`）与 `commands.rs::spawn_background_action`（73-88）；`should_run_action_in_background` 本体不动（仍 `from_request(...).is_some()`）。同步刷新三处 `"\"build-project\" => Some(Self::BuildProject)"` 断言（见契约联动）。验证：`cargo test -p zircon_hub --test project_workflow_contract --test project_quick_actions_contract --test ui_foundation_contract --locked`。
4. 改 `action_targets.rs` 签名 + `runtime_state.rs:201-232` 四个 arm + `view_model.rs:487-531` quick_actions id + `project_delivery_actions.rs:328/342`；同步刷新 `ui_project_scope_contract.rs` 与上述各测试中的 `"\"build-project\","` 类 snippet。验证：`cargo test -p zircon_hub --locked`、`cargo fmt --all --check`。
5. 验收：`rg '"build-project"' zircon_hub/src` 的非测试命中仅剩 `action_id.rs` 的 `as_str` 单点定义；各文件 `#[cfg(test)]` 块内的原始字符串（如 `action_tasks.rs:239` 起的测试构造）有意保留，用于独立锁定 IPC 线缆协议。

#### 契约联动

【落地状态终核（2026-06-12）】本表已执行完毕：「现有断言原文」列为落地前快照（其行号不再对应当前测试文件），当前各测试文件中已是「改为」列形态（如 `project_workflow_contract.rs:134/140-171/200-235`、`ui_foundation_contract.rs:594-635`、`ui_project_scope_contract.rs:165-168/372-381`）。

需同变更刷新的既有断言（原文 → 改为）：

| 文件（测试函数） | 现有断言原文 | 改为 |
|------|--------------|------|
| `project_workflow_contract.rs`（`tauri_runtime_routes_project_workflow_actions_and_persists_state`，83 行起） | `"\"show-page\" \| \"page\" => Ok(HubAction::ShowPage"`（118）、`"\"select-project\" \| \"open-project\" => Ok(HubAction::SelectProject"`（119）、`"\"update-settings-draft\" => Ok(HubAction::UpdateSettingsDraft"`（120）等 parse 分支 | `"HubActionId::ShowPage => Ok(HubAction::ShowPage"` 等同形替换；另增 `"pub(crate) fn action(&self) -> Result<HubActionId, HubError>"` |
| 同上 | `"\"build-project\","`（168）、`"\"package-project\","`（171）、`"\"install-device\","`（174）、`"\"open-editor\","`（177） | `"HubActionId::BuildProject,"` 等 |
| 同上 | `"\"build-project\" => Some(Self::BuildProject)"` 等四条（198-201） | `"HubActionId::BuildProject => Some(Self::BuildProject)"` 等 |
| `project_quick_actions_contract.rs`（`runtime_quick_actions_keep_fallback_and_persisted_history_separate_from_dto_copy`，92 行起） | `"\"build-project\","`（109）等四条与 `"\"build-project\" => Some(Self::BuildProject)"`（200-203）四条 | 同上两组替换 |
| `ui_foundation_contract.rs`（`backend_commands_and_view_model_keep_rust_state_as_source_of_truth`，525 行起） | `"\"show-page\" \| \"page\""`（611）、`"\"build-project\""`（614）等 action_request 块；`"\"build-project\" => Some(Self::BuildProject)"`（644-647）action_tasks 块 | `"HubActionId::ShowPage"`、`"HubActionId::BuildProject"`、`"HubActionId::BuildProject => Some(Self::BuildProject)"` 等 |
| `ui_project_scope_contract.rs`（`tauri_view_model_exposes_project_scope_dtos_and_visible_labels`，132 行起） | view_model 块 `"\"build-project\""`（165）等四条 | `"HubActionId::BuildProject.as_str()"` 等 |
| `ui_project_scope_contract.rs`（`quick_actions_and_workspace_pages_pass_scope_targets_to_runtime`，274 行起） | runtime_state 块 `"\"build-project\","`（372）等四条 | `"HubActionId::BuildProject,"` 等 |

新增测试（在 `action_id.rs` 内）：
- `every_action_id_round_trips_between_as_str_and_from_str`：`ALL.len() == 29`（实落为 31，见 M1 终核注记）+ 全量 round-trip。
- `legacy_aliases_and_whitespace_resolve_to_canonical_actions`：`"page"`/`"project-subpage"`/`"open-project"` 别名与 trim 行为 + 未知 id 返回 `None`。

测试阶段：
- `cargo test -p zircon_hub --locked`（重点 `project_workflow`、`project_quick_actions`、`app_error_recovery`）。
- 验收证据：`rg '"build-project"' zircon_hub/src` 的非测试代码命中仅剩 `action_id.rs` 的 `as_str` 单点定义（`#[cfg(test)]` 块内有意保留原始字符串以独立锁定线缆协议，见实施步骤 5）。

### M2 payload DTO 与统一校验

> 【落地状态终核（2026-06-12）】M2 已全部落地：八个信封结构、九个 `*_payload_from_value` 帮助函数、`settings_dto.rs::settings_payload_from_value` 与字符串简写均已删净；`ValidatePayload` + `parse_payload`/`parse_optional_payload` 已接线全部 arm；提案的六个负向单测（`create_project_rejects_empty_name_with_recoverable_message` 等，`action_request.rs:697-832`）与 `runtime_state.rs:1126` 的 `apply_action_records_payload_validation_failure_as_recoverable_status`（含中文断言）均已存在；`localized.rs` 词条已接通。下述切片/实施步骤转为「盘点补缺/验收」口径。与提案的实仓差异，以实仓为准：
> - 统一入口实形：`parse_payload`/`parse_optional_payload` 为自由函数并共享 `deserialize_payload` 帮助函数；`ValidatePayload::validate` 带默认实现 `Ok(())`，宽松 DTO 写成空 `impl ValidatePayload for X {}`。
> - 提案「宽松校验（仅形状）」未按提案落地：实仓对 `ProjectTargetActionPayload`/`OpenResourcePayload`/`OpenOutputFolderPayload`/`BrowseSettingsFolderPayload` 也做了绝对路径校验，且 `BrowseSettingsFolderPayload.field` 在 `action_request.rs::validate_settings_folder_field` 即有白名单（`settings_actions.rs` 的 `settings_folder_field_from_target` 仍保留既有失败路径，现位于 210 行）。
> - `NewProjectDraftActionPayload` 实仓复用与 create-project 相同的完整校验（`validate_project_creation_payload`：非空 name、绝对路径、模板目录集合），非提案的 `Ok(())` 宽松形。
> - search-projects 实仓未保留 `target_id` 回退：payload 必填（直接 `parse_payload`），提案代码块中的 `None => target_id` 回退分支未落地；对应单测现名 `parses_search_projects_typed_payload`。
> - payload 失败落状态未复用 `record_background_action_error`：实仓在 `runtime_state.rs` 新增专用 `record_action_payload_failure(action_id, detail)`（label `"Action failed"`、recovery `"Review the action payload and retry from Hub"`、operation `Hub`、target 为 action id 字符串）。
> - `localized.rs` 实落绝对路径类 strip_prefix 词条 10 条（Project location / Project path / Import path / Import folder / Initial directory / Resource path / Output path / Output directory 等，现 142-176 行）+ 常量词条 `"Project name must not be empty" => "项目名称不能为空"`（现 462 行），多于提案的 6 条。
> - `view_model.rs` 的 re-export 实为 `pub(crate) use settings_dto::{validate_settings_for_save, HubSettingsActionPayload, HubSettingsPayload};`（现 32-33 行），比提案多保留 `validate_settings_for_save`。

切片：
1. 落 `fn parse_payload<T: DeserializeOwned + ValidatePayload>(payload: Option<&Value>) -> Result<T, HubError>`；`ValidatePayload` 提供 `validate(&self)`，校验规则就近写在各 DTO 上（name 非空、location 绝对路径、template 在 `ProjectTemplate` 集合内、`field` 在 settings 字段白名单内等）。
2. 逐 action 迁移：create-project / import-project / save-settings / update-settings-draft / browse-settings-folder / open-resource / open-output-folder / project-target 类动作；删除 `create_project_payload_from_value` 的信封兼容分支。（修正：经 2026-06-12 实仓核对，前端各调用点已发扁平 payload——`ProjectsDashboard.tsx:93-98` 的 create-project、`SettingsPage.tsx:76-82` 的 settings 族、`BuildsPage.tsx`/`CloudPage.tsx` 的 project-target 族均为扁平对象，`web/src` 无任何 `{ project: {...} }` 信封调用点；本里程碑前端无需迁移，仅 Rust 删双形状分支并跑 typecheck/build 回归，见风险章节注记。）
3. 校验失败统一落 `TaskStatus`（error + recovery）并写 action history，与现有失败口径一致。（范围澄清：action history 仅对已有 lifecycle 记录器的动作保持现状——create-project / import-project 走 `record_lifecycle_failure` 既有路径；parse/validate 层失败统一新增的是「可恢复 `TaskStatus` + 持久化」口径——【终核修正（2026-06-12）】实仓经新增的专用 `record_action_payload_failure` 落地，而非提案的 `record_background_action_error`——不为 settings 等动作发明新的 `HubActionKind`，词表扩展归 07。）

#### 目标代码形状

统一入口落在 `action_request.rs`（payload DTO 所在文件；`action_id.rs` 只放 id 表）。设计要点：(a) 未知 id 错误（`action()`）与 payload 错误（`parse_as()`）分离，前者维持 IPC `Err` 现状口径，后者落 TaskStatus；(b) 信封删除即「canonical 形状唯一」——project/draft/search/folder/resource/output 信封与字符串简写删除、扁平为 canonical，settings 族相反：`{ "settings": {...} }` 包裹是前端 canonical 形状（`hub.ts:674-686`），删除的是 `settings_payload_from_value`（`settings_dto.rs:208-221`）接受顶层扁平字段的兼容分支。

```rust
// action_request.rs —— 新增统一 payload 入口（替换 374-529 行的 8 个 *_payload_from_value 帮助函数）
pub(crate) trait ValidatePayload {
    fn validate(&self) -> Result<(), HubError>;
}

fn parse_payload<T: DeserializeOwned + ValidatePayload>(
    action: HubActionId,
    payload: Option<&Value>,
) -> Result<T, HubError> {
    let Some(payload) = payload else {
        return Err(HubError::message(format!(
            "Payload is required for Hub action: {}",
            action.as_str()
        )));
    };
    let parsed: T = serde_json::from_value(payload.clone()).map_err(|error| {
        HubError::message(format!(
            "Invalid payload for Hub action {}: {error}",
            action.as_str()
        ))
    })?;
    parsed.validate()?;
    Ok(parsed)
}

fn parse_optional_payload<T: DeserializeOwned + ValidatePayload>(
    action: HubActionId,
    payload: Option<&Value>,
) -> Result<Option<T>, HubError> {
    if payload.is_none() {
        return Ok(None);
    }
    parse_payload(action, payload).map(Some)
}
```

各 DTO 的 `validate` 实现（语义校验已有归属的不重复——`HubSettingsPayload` 的非空/枚举校验留在 `apply_to`/`apply_to_draft`（`settings_dto.rs`，现 210-248 行起），browse-settings-folder 的 `field` 白名单提案原拟留在 `settings_actions.rs` 的 `settings_folder_field_from_target`（现 210 行）既有本地化失败路径；【终核（2026-06-12）】实仓在 `action_request.rs::validate_settings_folder_field` 另落了一份入口白名单，与 `settings_actions.rs` 并存，以实仓为准）：

```rust
// action_request.rs
use crate::projects::project_template_catalog;

impl ValidatePayload for CreateProjectActionPayload {
    fn validate(&self) -> Result<(), HubError> {
        if self.name.trim().is_empty() {
            return Err(HubError::message("Project name must not be empty"));
        }
        if !self.location.is_absolute() {
            return Err(HubError::message(format!(
                "Project location must be an absolute path: {}",
                self.location.to_string_lossy()
            )));
        }
        // 目录集合校验（含 disabled 模板）：垃圾 id 在此拒绝；
        // coming-soon（disabled）模板放行，保留 project_actions.rs:27-39 的
        // "Project template is coming soon: ..." 友好失败路径。
        if !project_template_catalog()
            .iter()
            .any(|template| template.id == self.template.as_str())
        {
            return Err(HubError::message(format!(
                "Unknown project template: {}",
                self.template
            )));
        }
        Ok(())
    }
}

impl ValidatePayload for ImportProjectActionPayload {
    fn validate(&self) -> Result<(), HubError> {
        for path in [self.path.as_deref(), self.folder.as_deref()].into_iter().flatten() {
            if !path.is_absolute() {
                return Err(HubError::message(format!(
                    "Import path must be an absolute path: {}",
                    path.to_string_lossy()
                )));
            }
        }
        Ok(())
    }
}

// 宽松校验（仅形状）：草稿允许半成品输入；target/resource/output/search 的语义
// 校验留在 runtime_state 既有失败路径。
impl ValidatePayload for NewProjectDraftActionPayload { fn validate(&self) -> Result<(), HubError> { Ok(()) } }
impl ValidatePayload for ProjectTargetActionPayload { fn validate(&self) -> Result<(), HubError> { Ok(()) } }
impl ValidatePayload for BrowseSettingsFolderPayload { fn validate(&self) -> Result<(), HubError> { Ok(()) } }
impl ValidatePayload for OpenResourcePayload { fn validate(&self) -> Result<(), HubError> { Ok(()) } }
impl ValidatePayload for OpenOutputFolderPayload { fn validate(&self) -> Result<(), HubError> { Ok(()) } }
impl ValidatePayload for SearchProjectsPayload { fn validate(&self) -> Result<(), HubError> { Ok(()) } }
```

settings 族：`settings_dto.rs` 的私有 `HubSettingsActionPayload`（111-115 行）提升为 `pub(crate)` 并实现 `ValidatePayload`（`Ok(())`），整体删除 `settings_payload_from_value`（208-221 行，双形状入口）；`view_model.rs:29` 的 re-export 改为 `pub(crate) use settings_dto::{HubSettingsActionPayload, HubSettingsPayload};`。

`parse_as` 各带 payload 的 arm 切换后形状（对照现状 244-338 行）：

```rust
HubActionId::SearchProjects => Ok(HubAction::SearchProjects {
    // payload 缺省时维持 target_id 回退（单测
    // parses_search_projects_typed_payload_before_target_fallback 锁定 payload 优先）；
    // 删除字符串简写与 { "search": {...} } 信封。
    query: match self.payload.as_ref() {
        None => self.target_id.clone().unwrap_or_default(),
        Some(_) => parse_payload::<SearchProjectsPayload>(action, self.payload.as_ref())?.query,
    },
}),
HubActionId::UpdateNewProjectDraft => Ok(HubAction::UpdateNewProjectDraft {
    payload: parse_payload(action, self.payload.as_ref())?,
}),
HubActionId::CreateProject => Ok(HubAction::CreateProject {
    payload: parse_payload(action, self.payload.as_ref())?,
}),
HubActionId::ImportProject => Ok(HubAction::ImportProject {
    target_id: self.trimmed_target(),
    payload: parse_optional_payload(action, self.payload.as_ref())?,
}),
HubActionId::UpdateSettingsDraft => Ok(HubAction::UpdateSettingsDraft {
    payload: parse_payload::<HubSettingsActionPayload>(action, self.payload.as_ref())?.settings,
}),
HubActionId::SaveSettings => Ok(HubAction::SaveSettings {
    payload: parse_optional_payload::<HubSettingsActionPayload>(action, self.payload.as_ref())?
        .map(|payload| payload.settings),
}),
HubActionId::BrowseSettingsFolder => Ok(HubAction::BrowseSettingsFolder {
    target_id: self.trimmed_target(),
    payload: parse_optional_payload(action, self.payload.as_ref())?,
}),
// pin/unpin/remove-from-hub/request-delete/cancel-delete/confirm-delete/
// build/package/install-device/open-editor 十个 project-target 类 arm 统一为：
HubActionId::PinProject => Ok(HubAction::PinProject {
    target_id: self.trimmed_target(),
    payload: parse_optional_payload(action, self.payload.as_ref())?,
}),
// open-resource / open-output-folder 同 parse_optional_payload 形状。
```

`project_target_payload()`（现 346-351 行）同步改为：

```rust
pub(crate) fn project_target_payload(
    &self,
) -> Result<Option<ProjectTargetActionPayload>, HubError> {
    parse_optional_payload(self.action()?, self.payload.as_ref())
}
```

`runtime_state.rs::apply_action`（139-236 附近，现 `match request.parse()? {`）改为「未知 id 仍 Err、payload 错误落状态」：

```rust
pub(super) fn apply_action(
    &mut self,
    request: HubActionRequest,
) -> Result<HubViewModel, HubError> {
    let action_id = request.action()?; // 未知 id：维持 Err 直达 IPC 的现状口径
    let action = match request.parse_as(action_id) {
        Ok(action) => action,
        Err(error) => {
            // payload 形状/校验失败：复用 action_tasks.rs:130-157 的
            // record_background_action_error（非后台 id 走 "Action failed" +
            // operation Hub + recovery 回退分支，并 persist），返回可恢复状态。
            self.record_background_action_error(&request, error.to_string())?;
            return Ok(self.view_model());
        }
    };
    match action {
        // 现有 28 个 arm 原样
    }
    Ok(self.view_model())
}
```

【终核（2026-06-12）】上方代码块的「未知 id 仍 Err、payload 错误落状态」骨架已按此落地（`runtime_state.rs:154-165`），但错误记录未复用 `record_background_action_error`，而是新增专用 `record_action_payload_failure`（`runtime_state.rs:264-276`）；arm 实为 31 个（含并行新增的 `DiscardSettingsDraft`/`RestoreDefaultSettings`）。

`localized.rs::status_detail`（139-224 附近的 strip_prefix 链）新增词条，保证新错误消息中文可见（`"Action failed" => "操作失败"` 已在 status_label 表 127 行）：

```rust
if let Some(action) = detail.strip_prefix("Payload is required for Hub action: ") {
    return format!("Hub 操作缺少 payload：{action}");
}
if let Some(body) = detail.strip_prefix("Invalid payload for Hub action ") {
    if let Some((action, error)) = body.split_once(": ") {
        return format!("Hub 操作 payload 无效（{action}）：{error}");
    }
}
if let Some(path) = detail.strip_prefix("Project location must be an absolute path: ") {
    return format!("项目位置必须是绝对路径：{path}");
}
if let Some(template) = detail.strip_prefix("Unknown project template: ") {
    return format!("未知项目模板：{template}");
}
if let Some(path) = detail.strip_prefix("Import path must be an absolute path: ") {
    return format!("导入路径必须是绝对路径：{path}");
}
// match detail 常量表（226 行起）追加：
// "Project name must not be empty" => "项目名称不能为空",
```

删除清单（硬切换，同变更删净）：`SearchProjectsEnvelope`（127-131）、`NewProjectDraftActionEnvelope`（143-147）、`CreateProjectActionEnvelope`（159-163）、`ImportProjectActionEnvelope`（173-177）、`ProjectTargetActionEnvelope`（188-192）、`BrowseSettingsFolderEnvelope`（202-206）、`OpenResourceEnvelope`（215-219）、`OpenOutputFolderEnvelope`（229-233）八个信封结构；`search_projects_payload_from_value`/`new_project_draft_payload_from_value`/`create_project_payload_from_value`/`import_project_payload_from_value`/`project_target_payload_from_value`/`browse_settings_folder_payload_from_value`/`required_settings_payload_from_value`/`open_resource_payload_from_value`/`open_output_folder_payload_from_value` 九个帮助函数（374-529）；其中 import/project-target/open-resource/open-output-folder/search 的字符串简写分支（`payload.as_str()` 路径）一并删除（前端从未发字符串 payload）；`settings_dto.rs::settings_payload_from_value`（208-221）；`action_request.rs:8` 对它的 import 与 `view_model.rs:29` 的 re-export 中的该符号。

#### 文件变更清单

| 路径 | 动作 | 变更内容一句话 |
|------|------|----------------|
| `zircon_hub/src/tauri_app/action_request.rs` | 修改 | 新增 `ValidatePayload` + `parse_payload`/`parse_optional_payload` 与各 DTO `validate`；删 8 信封 + 9 帮助函数 + 字符串简写；`parse_as` 各 arm 切换；单测信封 payload 改扁平 |
| `zircon_hub/src/tauri_app/view_model/settings_dto.rs` | 修改 | `HubSettingsActionPayload` 提升 `pub(crate)` + `ValidatePayload`；删 `settings_payload_from_value` 双形状入口；688 行附近单测随迁 |
| `zircon_hub/src/tauri_app/view_model.rs` | 修改 | 29 行 re-export 改为 `{HubSettingsActionPayload, HubSettingsPayload}` |
| `zircon_hub/src/tauri_app/runtime_state.rs` | 修改 | `apply_action` 改 `action()` + `parse_as`，payload 错误落 `record_action_payload_failure`（实仓命名；提案原写 `record_background_action_error`）后返回 `Ok(view_model)` |
| `zircon_hub/src/tauri_app/runtime_state/action_targets.rs` | 修改 | 三个单测（156-216 附近）的 `{"project": {...}}` payload 改扁平 `{"projectId"/"projectPath"}` |
| `zircon_hub/src/tauri_app/view_model/localized.rs` | 修改 | `status_detail` 新增 6 条新错误消息的 strip_prefix/常量词条 |
| `zircon_hub/tests/project_workflow_contract.rs` | 修改 | 刷新 `*_payload_from_value` / `match request.parse()?` / 测试名 snippet |
| `zircon_hub/tests/ui_foundation_contract.rs` | 修改 | 刷新 `"match request.parse()?"`（593 行附近）snippet |

#### 实施步骤

1. `action_request.rs` 落 `ValidatePayload` trait + `parse_payload`/`parse_optional_payload` + 全部 DTO `validate` 实现（暂不接线，旧帮助函数保留）。验证：`cargo check -p zircon_hub --locked`。
2. 切换 create-project / update-new-project-draft 两个 arm 到 `parse_payload`，删除 `CreateProjectActionEnvelope`/`NewProjectDraftActionEnvelope` 与对应帮助函数（395-419）；`action_request.rs` 单测 `parses_create_project_payload_for_create_project_action`（536-558）、`parses_new_project_draft_payload_for_runtime_state_update`（560-584）的 payload 由 `{"project"/"draft": {...}}` 改扁平；同步刷新 `project_workflow_contract.rs:124` snippet。验证：`cargo test -p zircon_hub --lib --locked`、`cargo test -p zircon_hub --test project_workflow_contract --locked`。
3. 切换十个 project-target 类 arm 与 import-project，删除 `ProjectTargetActionEnvelope`/`ImportProjectActionEnvelope`、字符串简写与帮助函数（421-462）；改 `project_target_payload()`；`action_request.rs` 单测 `parses_project_target_payload_for_background_project_actions`（653-678）、`parses_cancel_delete_project_target_payload`（680-705）与 `action_targets.rs` 单测（168-206 附近）payload 改扁平。验证：`cargo test -p zircon_hub --lib --locked`、`cargo test -p zircon_hub project_target --locked`。
4. 切换 settings 族 + open-resource / open-output-folder / search-projects：删 `settings_payload_from_value`（settings_dto.rs:208-221）、`required_settings_payload_from_value`、`BrowseSettingsFolderEnvelope`/`OpenResourceEnvelope`/`OpenOutputFolderEnvelope`/`SearchProjectsEnvelope` 及帮助函数（374-393、464-529）；`HubSettingsActionPayload` 提升 `pub(crate)`；改 `view_model.rs:29` re-export 与 `action_request.rs:8` import；单测 `parses_browse_settings_folder_payload_for_folder_action`（604-629）改扁平、`parses_open_output_folder_wrapped_payload_for_output_action`（707-731）改扁平并更名 `parses_open_output_folder_flat_payload_for_output_action`、settings_dto.rs:688 附近单测随迁；刷新 `project_workflow_contract.rs:122/126/128/133/823` 与 `ui_foundation_contract.rs` 相关 snippet。验证：`cargo test -p zircon_hub --locked`。
5. `runtime_state.rs::apply_action` 改为 `action()` + `parse_as` + 错误落状态（见目标代码形状）；`localized.rs` 增词条；刷新 `project_workflow_contract.rs:143`、`ui_foundation_contract.rs:593` 的 `"match request.parse()?"` snippet；新增负向单测（见契约联动）。验证：`cargo test -p zircon_hub --locked`、`cargo fmt --all --check`。
6. 前端回归（无代码改动，验证扁平契约未破）：在 `zircon_hub/` 目录（package.json 位于 `zircon_hub/`，非 `web/`）执行 `npm run typecheck`、`npm run build`。

#### 契约联动

【落地状态终核（2026-06-12）】本表已执行完毕：「改为」列断言已存在于当前测试文件（如 `project_workflow_contract.rs:139-171/857`、`ui_foundation_contract.rs:576`），「现有断言原文」列及其行号为落地前快照。

需同变更刷新的既有断言（原文 → 改为）：

| 文件 | 现有断言原文 | 改为 |
|------|--------------|------|
| `project_workflow_contract.rs:122` | `"payload: settings_payload_from_value(self.payload.as_ref())?"` | `"parse_optional_payload::<HubSettingsActionPayload>"` |
| `project_workflow_contract.rs:124` | `"payload: create_project_payload_from_value(self.payload.as_ref())?"` | `"payload: parse_payload(action, self.payload.as_ref())?"` |
| `project_workflow_contract.rs:126/128` | `"payload: open_resource_payload_from_value(self.payload.as_ref())?"`、`"payload: open_output_folder_payload_from_value(self.payload.as_ref())?"` | `"payload: parse_optional_payload(action, self.payload.as_ref())?"`（合并为单条） |
| `project_workflow_contract.rs:133` | `"parses_open_output_folder_wrapped_payload_for_output_action"` | `"parses_open_output_folder_flat_payload_for_output_action"` |
| `project_workflow_contract.rs:143` | `"match request.parse()? {"` | `"let action_id = request.action()?;"` + `"request.parse_as(action_id)"` |
| `project_workflow_contract.rs:823` | `"query: search_projects_payload_from_value("` | `"parse_payload::<SearchProjectsPayload>"` |
| `ui_foundation_contract.rs:593` | `"match request.parse()?"` | `"request.parse_as(action_id)"` |

新增测试（测试函数名 + 断言要点）：
- `action_request.rs`：`create_project_rejects_empty_name_with_recoverable_message`（空 name → Err 文本 `"Project name must not be empty"`）；`create_project_rejects_relative_location`（`"projects/Game"` → Err 含 `"must be an absolute path"`）；`create_project_rejects_unknown_template_id`（`"not-a-template"` → Err 含 `"Unknown project template"`；同时保留 disabled 模板（如 `"3d-scene"`）通过 validate 的正向断言，保护 coming-soon 路径）；`project_target_envelope_payload_is_rejected_after_hard_cutover`（`{"project": {...}}` → Err，锁定硬切换不回退）；`missing_required_payload_is_rejected_with_action_id`（create-project 无 payload → Err 文本 `"Payload is required for Hub action: create-project"`）；`settings_payload_requires_settings_wrapper`（顶层扁平 settings 字段 → Err，锁定 wrapped-only）。
- `runtime_state.rs`：`apply_action_records_payload_validation_failure_as_recoverable_status`（对 create-project 发相对路径 payload，断言返回 `Ok`、`task_summary` 为 error 且 `recovery` 非空、IPC 不报 Err）；中文语种下断言 detail 为 `"项目位置必须是绝对路径：..."`（验证 localized 词条接通）。

测试阶段：
- Rust：为每个 DTO 增加非法 payload 用例（缺字段、相对路径、未知模板、空字符串），`cargo test -p zircon_hub --locked`。
- 前端：`npm run typecheck`、`npm run build`（均在 `zircon_hub/` 目录执行——`package.json` 位于 `zircon_hub/`，不在 `web/` 下）。

### M3 前后端 action 契约守卫

> 【落地状态终核（2026-06-12）】M3 已全部落地：`hub.ts` 已补 `updateNewProjectDraft` 常量、`NewProjectDraftPayload` 接口与 `HubActionPayloadById` 对应键；`quoted_values_between` 与三个守卫测试已按提案原文落在 `ui_input_navigation_api_contract.rs`（现 52-141 行，位于既有帮助函数之后、文件前部，而非「末尾」）。前置事实已失效：实仓 Rust canonical 与前端 `HUB_ACTION` 均为 **31 个 id**（并行新增的 `discard-settings-draft`/`restore-default-settings` 两侧已同步），双向守卫零豁免。本里程碑余下工作仅为步骤 3 的守卫自证红灯演练（临时改动不入库）与收尾全量验证。

切片：
1. 新增（或扩展 `ui_input_navigation_api_contract.rs`）：读取 `web/src/types/hub.ts` 源文本提取 `HUB_ACTION` 值集合，与 Rust id 表 `as_str()` 全集双向比对。
2. 对带 payload 的 action，断言 `HubActionPayloadById` 中存在对应键（源文本级断言，沿用现有契约测试的文本匹配风格）。

#### 目标代码形状

前置事实（2026-06-12 核对）：`hub.ts:613-642` 的 `HUB_ACTION` 共 28 个 id，Rust 侧 canonical id 共 29 个——`update-new-project-draft` 仅 Rust 支持，前端无常量、无 payload 类型、无调用点。双向守恒要求先补齐前端（本计划选补齐而非维护豁免表，见风险章节）。【终核修正（2026-06-12）】该缺口已闭合：前端补齐动作已完成，且两侧同步并行新增的两个 settings 草稿 id，现 Rust 与前端均为 31 个（`HUB_ACTION` 现位于 `hub.ts:617-649`）；`update-new-project-draft` 仍无前端调用点，仅类型定义齐备：

```ts
// web/src/types/hub.ts —— HUB_ACTION（613-642 行）在 newProject 后插入一行
export const HUB_ACTION = {
  // ...
  newProject: "new-project",
  updateNewProjectDraft: "update-new-project-draft",
  selectEngine: "select-engine",
  // ...
} as const;

// 652 行起的 payload 接口区新增（字段对齐 Rust NewProjectDraftActionPayload，
// action_request.rs:135-141）：
export interface NewProjectDraftPayload {
  name: string;
  location: string;
  template: string;
  engineId?: string | null;
}

// HubActionPayloadById（698-717 行）新增一键：
export interface HubActionPayloadById {
  // ...
  [HUB_ACTION.updateNewProjectDraft]: NewProjectDraftPayload;
  // ...
}
```

守卫测试落在 `ui_input_navigation_api_contract.rs`（沿用其 `read_crate_file`/`assert_contains_all` 既有帮助函数，1-50 行）：

```rust
// tests/ui_input_navigation_api_contract.rs 新增
use std::collections::BTreeSet;

/// 提取 begin..end 标记之间的全部双引号字符串字面量。
/// - Rust 侧：HubActionId::as_str() 函数体（canonical 表，按约定不含别名，
///   别名在 from_str 中、位于 end 标记之后，不会被采集）。
/// - TS 侧：HUB_ACTION 对象字面量（键为裸标识符，块内引号串恰为 id 值全集）。
fn quoted_values_between(source: &str, begin: &str, end: &str) -> BTreeSet<String> {
    let start = source
        .find(begin)
        .unwrap_or_else(|| panic!("marker {begin:?} should exist"))
        + begin.len();
    let stop = source[start..]
        .find(end)
        .unwrap_or_else(|| panic!("marker {end:?} should exist after {begin:?}"))
        + start;
    source[start..stop]
        .split('"')
        .skip(1)
        .step_by(2)
        .map(str::to_string)
        .collect()
}

#[test]
fn hub_action_id_table_matches_react_hub_action_map_bidirectionally() {
    let action_id = read_crate_file("src/tauri_app/action_id.rs");
    let types = read_crate_file("web/src/types/hub.ts");

    let rust_ids = quoted_values_between(
        &action_id,
        "pub(crate) fn as_str(self) -> &'static str {",
        "pub(crate) fn from_str(",
    );
    let web_ids = quoted_values_between(&types, "export const HUB_ACTION = {", "} as const;");

    assert!(!rust_ids.is_empty(), "Rust action id table must not be empty");
    assert_eq!(
        rust_ids, web_ids,
        "HubActionId::as_str() table and web HUB_ACTION map must expose identical id sets"
    );
}

#[test]
fn hub_action_legacy_aliases_stay_rust_side_only() {
    let action_id = read_crate_file("src/tauri_app/action_id.rs");
    let types = read_crate_file("web/src/types/hub.ts");

    assert_contains_all(
        "action_id.rs",
        &action_id,
        &[
            "\"page\" => Some(Self::ShowPage)",
            "\"project-subpage\" => Some(Self::ShowProjectSubpage)",
            "\"open-project\" => Some(Self::SelectProject)",
        ],
    );
    assert_not_contains_any(
        "types/hub.ts",
        &types,
        &["\"page\",", "\"project-subpage\",", "\"open-project\","],
    );
}

#[test]
fn payload_carrying_actions_keep_typed_entries_in_react_payload_map() {
    let types = read_crate_file("web/src/types/hub.ts");

    assert_contains_all(
        "types/hub.ts",
        &types,
        &[
            "[HUB_ACTION.searchProjects]: SearchProjectsPayload;",
            "[HUB_ACTION.updateNewProjectDraft]: NewProjectDraftPayload;",
            "[HUB_ACTION.createProject]: CreateProjectPayload;",
            "[HUB_ACTION.importProject]: ImportProjectPayload;",
            "[HUB_ACTION.pinProject]: ProjectTargetPayload;",
            "[HUB_ACTION.unpinProject]: ProjectTargetPayload;",
            "[HUB_ACTION.removeFromHub]: ProjectTargetPayload;",
            "[HUB_ACTION.requestDelete]: ProjectTargetPayload;",
            "[HUB_ACTION.cancelDelete]: ProjectTargetPayload;",
            "[HUB_ACTION.confirmDelete]: ProjectTargetPayload;",
            "[HUB_ACTION.buildProject]: ProjectTargetPayload;",
            "[HUB_ACTION.packageProject]: ProjectTargetPayload;",
            "[HUB_ACTION.installDevice]: ProjectTargetPayload;",
            "[HUB_ACTION.openEditor]: ProjectTargetPayload;",
            "[HUB_ACTION.updateSettingsDraft]: UpdateSettingsDraftPayload;",
            "[HUB_ACTION.saveSettings]: SaveSettingsPayload;",
            "[HUB_ACTION.browseSettingsFolder]: BrowseSettingsFolderPayload;",
            "[HUB_ACTION.openResource]: OpenResourcePayload;",
            "[HUB_ACTION.openOutputFolder]: OpenOutputFolderPayload;",
        ],
    );
}
```

约定（写进 `action_id.rs` 的 `as_str` doc 注释，M1 已含）：`as_str` 函数体是 canonical 表的唯一采集面，函数顺序固定 `ALL` → `as_str` → `from_str`，保证 `quoted_values_between` 的 end 标记把别名隔离在采集区之外。

#### 文件变更清单

| 路径 | 动作 | 变更内容一句话 |
|------|------|----------------|
| `zircon_hub/web/src/types/hub.ts` | 修改 | `HUB_ACTION` 增 `updateNewProjectDraft` 键；新增 `NewProjectDraftPayload` 接口；`HubActionPayloadById` 增对应键 |
| `zircon_hub/tests/ui_input_navigation_api_contract.rs` | 修改 | 新增 `quoted_values_between` 帮助函数与三个守卫测试（双向 id 集合、别名只在 Rust、payload 映射键） |

#### 实施步骤

1. 改 `web/src/types/hub.ts`：`HUB_ACTION`（613-642）插入 `updateNewProjectDraft: "update-new-project-draft",`；新增 `NewProjectDraftPayload` 接口；`HubActionPayloadById`（698-717）插入对应键。验证：`zircon_hub/` 下 `npm run typecheck`、`npm run build`。
2. 在 `ui_input_navigation_api_contract.rs` 末尾新增 `quoted_values_between` 与 `hub_action_id_table_matches_react_hub_action_map_bidirectionally`、`hub_action_legacy_aliases_stay_rust_side_only`、`payload_carrying_actions_keep_typed_entries_in_react_payload_map` 三个测试。验证：`cargo test -p zircon_hub --test ui_input_navigation_api_contract --locked`。
3. 守卫自证（临时改动，验证后撤销，不入库）：(a) 在 `hub.ts` 的 `HUB_ACTION` 临时加 `fakeAction: "fake-action",` → 双向比对测试必须报 web 侧多出 id；(b) 在 `action_id.rs` 的 `as_str` 临时把 `"pin-project"` 改 `"pin-projects"` → 必须报集合不等；两者确认红灯后还原绿灯。
4. 收尾全量：`cargo test -p zircon_hub --locked`、`cargo fmt --all --check`。

#### 契约联动

- 既有断言不需修改：`ui_input_navigation_api_contract.rs` 既有七个测试（如 `navigation_components_share_one_action_dispatcher_api` 对 `hubApi.ts` 断言 `"dispatchHubAction<TActionId extends HubActionId>"`、`"request: { actionId, targetId, payload }"`，现 325-403 行；落地前文档误记为「五个测试（234-310 行）」，已据实仓修正）只增不改；`hubApi.ts` 与 `App.tsx` 在本计划全程零改动。
- 新增测试即上述三个守卫（断言要点见目标代码形状）；其失败信息需指明缺失/多余的 id 集合差（`BTreeSet` 的 `assert_eq!` 自带 diff 输出，足够定位）。
- 后续任何新增 action 的变更将同时触碰 `action_id.rs`（枚举 + ALL + as_str）与 `hub.ts`（HUB_ACTION + 可选 PayloadById），守卫缺一侧即红——这是本计划交付给 05/07 等后续计划的稳定接口面。

测试阶段：
- `cargo test -p zircon_hub ui_input_navigation_api --locked`；人为加一个只在一侧存在的 id 验证守卫确实报错后撤销。

## 风险与协调

- `parse()` 周边有大量现有测试依赖字符串 id 的别名（如 `"show-page" | "page"`）：保留别名但收进 id 表的 `from_str`，避免破坏外部调用方。
- 前端扁平 payload 切换与 05 计划的组件拆分都会动 `ProjectsDashboard`：先做本计划 M2，05 在其上拆分。
- `view_model.rs`/`runtime_state.rs` 的行数问题不强行在此解决，但 M1/M2 自然带出的代码删除应使两文件净缩。
- 【2026-06-12 核实修正】action history 的 `HubActionKind`（`src/state/action_history.rs:28-63`）是与分发 id 不同名的独立词表（`"build-editor-runtime"`/`"install-project"`/`"open-output"` vs `"build-project"`/`"install-device"`/`"open-output-folder"`），不能从 `HubActionId` 派生；目标 1 原文「action history 全部从它派生」已据此收窄为 quick_actions DTO id，history 词表 schema 化归 07 计划。
- 【2026-06-12 核实修正】前端各调用点已全部发扁平 payload（`web/src` 中 grep 不到任何 `{ project: {...} }` 信封构造；`ProjectsDashboard.tsx:93-98` 为扁平 create-project）。M2 原文「前端 `ProjectsDashboard` 同变更改为扁平 payload」无事可做，前端在 M2 仅做 typecheck/build 回归；信封与字符串简写只存在于 Rust 解析层与 Rust 单测的构造里，删除范围以 Rust 侧为准。
- 【2026-06-12 核实补充】`update-new-project-draft` 是 Rust 可分发但前端无类型定义的 id（`hub.ts` 的 `HUB_ACTION` 仅 28 个，Rust canonical 29 个）。M3 双向守卫落地前必须先补齐前端常量与 `NewProjectDraftPayload`（M3 步骤 1），否则守卫首跑即红；若评审认为不应在前端暴露未使用的 action，可改为在守卫测试中维护显式 Rust-only 豁免集合——本计划默认选补齐，保持守卫零豁免。【落地状态终核（2026-06-12）】已按「补齐」路线落地：前端常量与 `NewProjectDraftPayload` 均已存在，并行变更另在两侧同步新增了 `discard-settings-draft`/`restore-default-settings`，现 Rust 与前端各 31 个 id，守卫零豁免绿灯。
- 守卫测试以文本标记提取 id 集合（`quoted_values_between`），对 `action_id.rs` 的函数排列顺序（`ALL` → `as_str` → `from_str`）与 `hub.ts` 的 `} as const;` 写法有结构依赖；任何重排这两个区块的重构必须连带跑 `ui_input_navigation_api_contract`，标记缺失会直接 panic 并指明丢失的 marker。
- M2 把 payload 校验失败从「IPC Err → 前端通用 `actionFailed` 文案」改为「`Ok(view_model)` + 可恢复 `TaskStatus`」：`app_error_recovery_contract.rs` 锁定的前端 catch 路径仍保留（传输层错误仍走它），但任何依赖「非法 payload 必然 reject promise」的调用方行为会变；已核实仓内无此类调用方（`App.tsx` 的 catch 只写日志与 snackbar）。

## Code Review 建议 (2026-07-30)

### 与代码现状不符，需修订

- front-matter `status: planned` 与实仓不符：M1/M2/M3 全部落地（文档正文各里程碑「终核」注记已如实反映）。核对 `zircon_hub/src/tauri_app/action_id.rs:2-124`（`HubActionId` 31 变体 + `ALL` + `as_str`/`from_str` + round-trip 单测）、`commands.rs:48-79`（`spawn_background_action` 已收敛为 `run_background_worker_loop`，无字符串比对）。建议把状态改为 `completed`，否则会继续以 planned 调度已完成计划。
- M2「目标代码形状」的 `parse_payload` 签名（第 425-443 行）写为 `parse_payload<T>(action: HubActionId, payload: Option<&Value>)`，但正文 M2 终核注记（第 402 行）已说明实仓落地为「自由函数 + 共享 `deserialize_payload`」。文档的两份签名并存易误导实施者；建议在目标代码形状块顶部直接标注「以实仓 `action_request.rs` 为准」并删除或明确降级过时代码块。
- M2「宽松校验（仅形状）」代码块（第 505-511 行）把 `ProjectTargetActionPayload`/`OpenResourcePayload` 等写成 `validate` 返回 `Ok(())`，但正文终核注记（第 403 行）已指出实仓对这些类型也做了绝对路径校验、`NewProjectDraftActionPayload` 复用完整校验。目标代码形状与实仓相反，属可执行文档里的错误示范，建议整块替换为实仓形态或删除。
- M2 关于 `localized.rs` strip_prefix 词条的目标形状（第 596-616 行）整段已被 07 的 `HubMessage` schema 取代——`localized.rs` 现无 `status_detail`/`strip_prefix`（`grep` 零命中），payload 校验错误改由 `state/hub_message/shell.rs`（`PayloadRequiredForAction`/`InvalidPayloadForAction`）承载。该节应标注「已被 07 计划整体重构，词条落点迁移至 `hub_message` 域文件」。
