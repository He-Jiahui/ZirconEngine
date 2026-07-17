---
related_code:
  - zircon_hub/src/tauri_app/view_model/localized.rs
  - zircon_hub/src/tauri_app/view_model/ui_text.rs
  - zircon_hub/src/tauri_app/view_model/coming_soon.rs
  - zircon_hub/src/tauri_app/view_model/project_templates.rs
  - zircon_hub/src/state/task_status.rs
  - zircon_hub/src/state/action_history.rs
  - zircon_hub/src/state/hub_message
  - zircon_hub/src/error.rs
  - zircon_hub/src/settings/hub_config.rs
  - zircon_hub/src/engines/source_engine_install.rs
  - zircon_hub/src/tauri_app/view_model.rs
  - zircon_hub/src/tauri_app/view_model/action_history.rs
  - zircon_hub/src/tauri_app/view_model/source_engines.rs
  - zircon_hub/src/tauri_app/action_request.rs
  - zircon_hub/src/tauri_app/runtime_state.rs
  - zircon_hub/src/tauri_app/runtime_state/action_tasks.rs
  - zircon_hub/src/tauri_app/runtime_state/action_targets.rs
  - zircon_hub/src/tauri_app/runtime_state/build_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/project_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/project_delivery_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/editor_launch_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/learn_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/output_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/settings_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/quick_actions.rs
  - zircon_hub/web/src/components/shell/TopBar.tsx
  - zircon_hub/web/src/components/overlays/UserMenuPopover.tsx
  - zircon_hub/web/src/pages/ProjectsDashboard.tsx
  - zircon_hub/web/src/types/hub.ts
  - zircon_hub/web/src/data/hubData.ts
  - zircon_hub/tests/project_page_copy_contract.rs
  - zircon_hub/tests/project_workflow_contract.rs
  - zircon_hub/tests/project_source_engine_contract.rs
  - zircon_hub/tests/project_cloud_local_delivery_contract.rs
  - zircon_hub/tests/tauri_react_shell_contract.rs
  - zircon_hub/tests/ui_foundation_contract.rs
  - zircon_hub/tests/ui_shell_header_contract.rs
plan_sources:
  - docs/plans/zircon_hub/index.md
  - docs/plans/zircon_hub/01-action-dispatch-and-typed-payload.md
  - docs/plans/zircon_hub/02-background-task-framework-and-persistence.md
  - docs/plans/zircon_hub/03-project-lifecycle-robustness.md
  - docs/plans/zircon_hub/04-settings-draft-and-source-engine.md
  - docs/plans/zircon_hub/05-frontend-componentization-and-type-safety.md
  - .codex/plans/Zircon Hub 本地闭环 v1 功能实现设计.md
status: planned
---

# 07 本地化消息 schema 与"敬请期待"能力目录

- 失败交接（`open / 待修复`）：[`07/failure-2026-07-11-hub-message-legacy-test-drift.md`](07/failure-2026-07-11-hub-message-legacy-test-drift.md)

## 现状与证据

- `HubTextBundle`（`view_model/localized.rs`，818 行，2026-06-12 实仓终核；01-04 计划并行落地使其自早先 684 行/23 处持续增长）对静态标签用 `(language, label)` 匹配表（`status_label`，70-135 行）——可接受；但对动态 detail 用 37 处 `strip_prefix` 匹配英文原文前缀再重组中文（`status_detail`，142-273 行，如 `detail.strip_prefix("Project template is coming soon: ")`），外加约 92 条英文原文常量词条表（275-463 行）与 4 个后缀/组合解析 helper（`localize_file_count_suffix` / `localize_delivery_log_excerpt` / `localize_project_filter` / `localize_project_sort`，777-818 行）。消息产生点（`task_status` / `action_history` 写入处）与翻译点之间没有任何编译期或测试期同步：后端改一句英文措辞，中文界面静默回落英文（实例：`action_tasks.rs:336` 的 `"Background task panicked: {detail}"` 在词条表中无对应项，中文界面现状直接显示英文）。
- 消息以已渲染英文字符串存进 `action_history` 并持久化到 `hub.toml`（`HubActionRecord.detail/log_excerpt/recovery`，`state/action_history.rs:8-24`）：历史记录的语言被写死为记录时刻语言，切换语言后旧记录不跟随。同样形态还有第三个持久化面：`SourceBuildRecord.detail/log_excerpt`（`engines/source_engine_install.rs:10`，随 `SourceEngineInstall.build_history` 进 `hub.toml`），投影时经 `view_model/source_engines.rs:26-49` 走同一条 `status_detail` 链。
- `view_model/coming_soon.rs`（193 行，2026-06-12 实仓终核）DTO 结构已就绪，且已登记 13 条（早先记录的"缺三条"——Projects 模板预留 / TopBar 通知 / UserMenu sign-out——已由并行进程以 `project-template-2d-scene` / `notification-center` / `sign-out` 补齐，前端 fallback `hubData.ts:84-215` 同步 13 条）。剩余真实缺口：(1) 模板目录有 3 个禁用模板（`2d-scene` / `3d-scene` / `sample-world`，见 `project_templates.rs:78-90`），coming-soon 目录只登记了 `2d-scene`；(2) shell 双源文案——TopBar 通知 tooltip 与 UserMenu sign-out 说明仍从 `ui_text.rs` 的 `notifications_detail` / `sign_out_detail`（`HubShellText`，57 / 42 行字段）取值，与 coming-soon 目录中 `notification-center` / `sign-out` 条目的 detail 文案并存且措辞不一致（"通知中心敬请期待；本地 v1 不启用通知服务。" vs "桌面通知为预留能力；v1 在 Hub 窗口内显示本地任务反馈。"）。（2026-06-12 实仓核对修正）
- `ui_text.rs`（947 行）纯数据结构可接受，但中英 key 覆盖一致性靠肉眼；05.M3 的 `demo_mode_badge` 徽标 key（05 计划已登记、实仓尚未落）等新增 key 需要登记点。

## 目标

1. 动态消息 schema 化：状态/历史的 detail 从"已渲染英文字符串"改为结构化 `HubMessage { id: HubMessageId, params: Vec<String> }`（或带命名参数的小枚举）；`HubTextBundle` 按 `(language, id)` 查模板 + 参数插值渲染；删除全部 37 处 `strip_prefix` 链与常量词条表（数字按 2026-06-12 实仓修正，原记 23 处）。消息产生点（02/03/04 收敛后的 TaskStatus 与 history 写入处）改发 `HubMessageId`。
2. 历史记录语言跟随：`action_history` 持久化 `HubMessageId + params` 而非渲染结果，ViewModel 投影时按当前语言渲染；`hub.toml` 中既有英文字符串记录做一次性读取兼容（读到旧格式按原文显示，不迁移、不写回旧格式）。
3. 模板完备性守卫：测试枚举全部 `HubMessageId` × 两种语言，断言模板存在且参数占位符数量匹配——把"漏翻"从运行时静默降级变成测试期失败。
4. coming-soon 目录填满 v1 设计的能力分类：每条含所属页面、能力名、分类（远程服务/市场/协作/导入）、禁用态说明文案（中英）；前端各页的 disabled 行/按钮统一从该 DTO 取文案，禁止页面内自造"敬请期待"字样。
5. `ui_text.rs` 中英 key 覆盖审计测试：两个语言分支字段逐一非空。

## 非目标

- 不引入 fluent/gettext 等本地化框架（两语言、消息量百级，普通 enum + match 足够）。
- 不做语言热加载与第三语言扩展位（`HubLanguage` 保持双值枚举）。
- 不迁移 `hub.toml` 旧历史记录格式（只读兼容）。

## 里程碑

### M1 HubMessageId 与渲染管线

切片：
1. 落 `HubMessageId` 枚举与双语模板表（按领域分模块：shell / project / engine / build / delivery / process / settings / learn，避免单文件再造 800 行巨表；相对原拟六域增加 shell 与 process 两域：payload/action 通用错误与进程启动消息在现表中各占一组，无法归并进既有六域）。
2. `TaskStatus.detail` / `recovery` 与 `HubActionRecord.detail` / `log_excerpt` / `recovery`、`SourceBuildRecord.detail` / `log_excerpt` 改持 `HubMessage`；ViewModel 投影点统一渲染；删除 `strip_prefix` 链、常量词条表与 4 个解析 helper。
3. 旧 `hub.toml` 历史的字符串 detail 走 `HubMessage::RawText(String)` 只读分支。

#### 目标代码形状

总体改法：消息 id、双语模板、渲染全部归 `src/state/hub_message/` 单一 owner（模板与 id 同处，使"加变体忘加模板"成为编译错误而非测试错误；`view_model::HubTextBundle` 只做按当前语言的委托渲染，避免 `src/error.rs` 等 state 侧调用方反向依赖 view_model）。`view_model/localized.rs` 删除 `status_detail` 与 4 个 helper，新增一行委托 `render_message`；`status_label` / `operation_scope` / `operation_target` / `action_label` / `action_status_label` / `page_title` / `page_subtitle` / `pair` 等静态标签路径不动。

（a）新建 `src/state/hub_message/` 模块。目录与挂接：

```
zircon_hub/src/state/hub_message/
  mod.rs        # 仅 mod 声明 + pub use（遵守 root wiring 薄文件规则）
  message.rs    # HubMessage 结构、serde 兼容、render/is_empty
  id.rs         # HubMessageId 外层枚举 + all()/param_count()/template() 分派 + 完备性测试
  shell.rs  project.rs  engine.rs  build.rs
  delivery.rs  process.rs  settings.rs  learn.rs   # 各域子枚举 + 双语模板表
```

`src/state/mod.rs`（现 21 行）追加：

```rust
mod hub_message;

pub use hub_message::{HubMessage, HubMessageId};
```

`message.rs` 完整形状（serde 用 Repr 中转而非直接 untagged：保证未知 id 的新旧混合文件降级为 `RawText` 可读，而不是整个 `hub.toml` 反序列化失败）：

```rust
use serde::{Deserialize, Serialize, Serializer};

use crate::settings::HubLanguage;

use super::id::HubMessageId;

/// 结构化本地化消息：持久化与内存状态只存 id + 参数，渲染推迟到投影期。
/// RawText 分支承载两类内容：旧 hub.toml 的已渲染英文字符串（只读兼容），
/// 以及刻意不翻译的逐字内容（cargo 日志摘录、项目/引擎显示名、io 错误原文）。
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(from = "HubMessageRepr")]
pub enum HubMessage {
    Structured {
        id: HubMessageId,
        params: Vec<String>,
    },
    RawText(String),
}

impl HubMessage {
    pub fn new(id: HubMessageId) -> Self {
        Self::Structured { id, params: Vec::new() }
    }

    pub fn with_params<P: Into<String>>(id: HubMessageId, params: impl IntoIterator<Item = P>) -> Self {
        Self::Structured { id, params: params.into_iter().map(Into::into).collect() }
    }

    /// 逐字内容入口；刻意不提供 From<String>/From<&str>，
    /// 迫使每个产点显式选择"结构化变体"或"逐字"，杜绝静默双轨。
    pub fn raw_text(text: impl Into<String>) -> Self {
        Self::RawText(text.into())
    }

    pub fn empty() -> Self {
        Self::RawText(String::new())
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::RawText(text) => text.trim().is_empty(),
            Self::Structured { .. } => false,
        }
    }

    pub fn render(&self, language: HubLanguage) -> String {
        match self {
            Self::RawText(text) => text.clone(),
            Self::Structured { id, params } => {
                let mut rendered = id.template(language).to_string();
                for (index, param) in params.iter().enumerate() {
                    rendered = rendered.replace(&format!("{{{index}}}"), param);
                }
                rendered
            }
        }
    }
}

/// serde 中转表示：Structured 先按 raw 字符串 id 读入，
/// 解析失败（未来删除/改名变体后读旧文件）降级为 RawText，整个文件仍可读。
#[derive(Deserialize)]
#[serde(untagged)]
enum HubMessageRepr {
    Structured {
        id: String,
        #[serde(default)]
        params: Vec<String>,
    },
    ArchivedRawText(String),
}

impl From<HubMessageRepr> for HubMessage {
    fn from(repr: HubMessageRepr) -> Self {
        match repr {
            HubMessageRepr::ArchivedRawText(text) => Self::RawText(text),
            HubMessageRepr::Structured { id, params } => match HubMessageId::from_str_id(&id) {
                Some(id) => Self::Structured { id, params },
                None => Self::RawText(if params.is_empty() { id } else { format!("{id}: {}", params.join(", ")) }),
            },
        }
    }
}

impl Serialize for HubMessage {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        match self {
            Self::RawText(text) => serializer.serialize_str(text),
            Self::Structured { id, params } => {
                let mut row = serializer.serialize_struct("HubMessage", 2)?;
                row.serialize_field("id", id.as_str())?;
                row.serialize_field("params", params)?;
                row.end()
            }
        }
    }
}
```

`id.rs` 外层枚举形状（子枚举见各域文件；`as_str` 形如 `"project.template-coming-soon"`，域前缀 + 子枚举 kebab-case，是持久化稳定 id，落定后不得改名）：

```rust
use crate::settings::HubLanguage;

pub use super::{
    build::BuildMessageId, delivery::DeliveryMessageId, engine::EngineMessageId,
    learn::LearnMessageId, process::ProcessMessageId, project::ProjectMessageId,
    settings::SettingsMessageId, shell::ShellMessageId,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HubMessageId {
    Shell(ShellMessageId),
    Project(ProjectMessageId),
    Engine(EngineMessageId),
    Build(BuildMessageId),
    Delivery(DeliveryMessageId),
    Process(ProcessMessageId),
    Settings(SettingsMessageId),
    Learn(LearnMessageId),
}

impl HubMessageId {
    pub fn as_str(self) -> &'static str { /* 分派到子枚举 as_str，带域前缀 */ }
    pub fn from_str_id(id: &str) -> Option<Self> { /* as_str 的逆；按 "domain." 前缀分派 */ }
    pub fn param_count(self) -> usize { /* 分派 */ }
    pub fn template(self, language: HubLanguage) -> &'static str { /* 分派 */ }
    pub fn all() -> impl Iterator<Item = Self> { /* 链接各子枚举 ALL 常量 */ }
}
```

域文件统一形状，以 `shell.rs` 为完整样例（模板原文必须逐字取自 `localized.rs` 现表，括号注行号；`{0}`/`{1}` 为位置参数占位符）：

```rust
use crate::settings::HubLanguage;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellMessageId {
    PayloadRequiredForAction,      // 1 参；localized.rs:142
    InvalidPayloadForAction,       // 2 参；localized.rs:145-149
    UnknownRecentProjectTarget,    // 2 参；localized.rs:268-273
    OpenedPath,                    // 1 参；localized.rs:218
    HubReady,                      // localized.rs:276
    RefreshingCatalogs,            // localized.rs:277-279
    NoRecoveryRequired,            // localized.rs:289
    RecoveryReviewActionTarget,    // localized.rs:437
    RecoveryReviewActionPayload,   // localized.rs:438-440
    StateRefreshAfterCommand,      // localized.rs:441
    RecoveryCheckActionTarget,     // localized.rs:442
    RecoveryCheckConfigPath,       // localized.rs:443-445
    BackgroundTaskPanicked,        // 1 参；新译——现状英文回落（action_tasks.rs:336）
}

pub(super) const ALL: &[ShellMessageId] = &[
    ShellMessageId::PayloadRequiredForAction,
    // ……逐一列全，完备性测试遍历它
];

impl ShellMessageId {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::PayloadRequiredForAction => "shell.payload-required-for-action",
            Self::InvalidPayloadForAction => "shell.invalid-payload-for-action",
            // ……
        }
    }

    pub(super) fn param_count(self) -> usize {
        match self {
            Self::PayloadRequiredForAction | Self::OpenedPath | Self::BackgroundTaskPanicked => 1,
            Self::InvalidPayloadForAction | Self::UnknownRecentProjectTarget => 2,
            _ => 0,
        }
    }

    pub(super) fn template(self, language: HubLanguage) -> &'static str {
        match (language, self) {
            (HubLanguage::English, Self::PayloadRequiredForAction) => "Payload is required for Hub action: {0}",
            (HubLanguage::Chinese, Self::PayloadRequiredForAction) => "Hub 操作缺少 payload：{0}",
            (HubLanguage::English, Self::InvalidPayloadForAction) => "Invalid payload for Hub action {0}: {1}",
            (HubLanguage::Chinese, Self::InvalidPayloadForAction) => "Hub 操作 payload 无效（{0}）：{1}",
            (HubLanguage::English, Self::OpenedPath) => "Opened {0}",
            (HubLanguage::Chinese, Self::OpenedPath) => "已打开 {0}",
            (HubLanguage::English, Self::HubReady) => "Hub is ready",
            (HubLanguage::Chinese, Self::HubReady) => "Hub 已就绪",
            // ……其余变体同模式，中英模板逐字照抄 localized.rs 对应行
            (HubLanguage::English, Self::BackgroundTaskPanicked) => "Background task panicked: {0}",
            (HubLanguage::Chinese, Self::BackgroundTaskPanicked) => "后台任务已中止：{0}",
        }
    }
}
```

完备性测试骨架（`id.rs` 测试区，承接目标 3）：

```rust
#[cfg(test)]
mod tests {
    use super::HubMessageId;
    use crate::settings::HubLanguage;

    #[test]
    fn every_message_id_has_bilingual_templates_with_matching_placeholders() {
        for id in HubMessageId::all() {
            for language in [HubLanguage::English, HubLanguage::Chinese] {
                let template = id.template(language);
                assert!(!template.trim().is_empty(), "{id:?} 缺 {language:?} 模板");
                for index in 0..id.param_count() {
                    assert!(
                        template.contains(&format!("{{{index}}}")),
                        "{id:?} 的 {language:?} 模板缺占位符 {{{index}}}：{template}"
                    );
                }
                assert!(
                    !template.contains(&format!("{{{}}}", id.param_count())),
                    "{id:?} 的 {language:?} 模板引用了越界占位符：{template}"
                );
            }
        }
    }

    #[test]
    fn message_id_round_trips_through_stable_string_ids() {
        for id in HubMessageId::all() {
            assert_eq!(HubMessageId::from_str_id(id.as_str()), Some(id));
        }
    }
}
```

（b）状态与持久化类型切换。

`state/task_status.rs`：现 `detail: String` / `recovery: Option<String>`（4 / 7 行），改：

```rust
pub struct TaskStatus {
    pub label: String,                  // 标签维持字符串 + status_label 表（现状口径不变）
    pub detail: HubMessage,
    pub running: bool,
    pub severity: TaskSeverity,
    pub recovery: Option<HubMessage>,
    // operation/target/progress_percent/task_id 不变
}
```

构造器签名同步：`running` / `success`（53、76 行）detail 参数 `impl Into<String>` → `impl Into<HubMessage>`（无 `From<String>`，调用方必须传 `HubMessage`）；`warning` / `error`（80-94 行）recovery 同理。`detail_with_recovery`（165-175 行）整体删除——其唯一非测试调用 `build_actions.rs:235` 的 `Err(HubError::message(self.task_status.detail_with_recovery()))` 改为携带结构化消息的新 `HubError` 变体（见 c），不再把已渲染英文塞回错误链。

`state/action_history.rs`：`HubActionRecord` 的 `detail: String`（13 行）→ `HubMessage`，`log_excerpt: String`（14-15 行）→ `#[serde(default = "HubMessage::empty")] pub log_excerpt: HubMessage`，`recovery: Option<String>`（16-17 行）→ `Option<HubMessage>`。`engines/source_engine_install.rs`：`SourceBuildRecord.detail/log_excerpt`（10 行起）同改。`hub.toml` 新形状示例（toml crate 自动把表键后置，`HubConfig::save` 的 `toml::to_string_pretty` 路径无需改动，round-trip 测试锚定）：

```toml
[[action_history]]
finished_unix_ms = 9
action = "open-editor"
status = "success"
target = "Game"
detail = { id = "process.started-process", params = ["42"] }   # 旧文件此处为字符串，读取走 RawText
```

（c）错误链贯通。`src/error.rs` 的 `HubError`（1-15 行）追加一个携带结构化消息的变体，使"validation 失败先写好 task_status、再沿 `Err` 冒泡到 `record_background_action_error` 重写状态"的既有路径（`action_tasks.rs:42/51/75/122/274/279` 全部 `error.to_string()`）不丢失结构与 recovery：

```rust
#[error("{}", detail.render(crate::settings::HubLanguage::English))]
Status {
    detail: Box<HubMessage>,
    recovery: Option<Box<HubMessage>>,
},
```

`record_background_action_error`（`action_tasks.rs:300-328`）签名 `detail: impl Into<String>` → `detail: HubMessage, recovery: Option<HubMessage>`（或收 `&HubError` 自行拆解）：`HubError::Status` 取结构化 detail/recovery；其余错误取 `HubMessage::raw_text(error.to_string())` + 既有默认 recovery 变体 `Shell::RecoveryReviewActionTarget`（现 323 行字符串）。`record_background_worker_panic`（330-337 行）改发 `HubMessage::with_params(Shell(BackgroundTaskPanicked), [detail])`。

（d）投影点统一渲染。`view_model/localized.rs` 增（`status_label` 之后）：

```rust
pub(crate) fn render_message(self, message: &HubMessage) -> String {
    message.render(self.language)
}
```

三个消费点切换：
- `view_model.rs::task_summary`（307-327 行）：`text.status_detail(&snapshot.task_status.detail)` → `text.render_message(&snapshot.task_status.detail)`；`recovery` 的 `.map(|recovery| text.status_detail(recovery))`（317-321 行）→ `.as_ref().map(|recovery| text.render_message(recovery))`。
- `view_model/action_history.rs::action_history_row`（47-92 行）：55-60 行三处 `status_detail` → `render_message`；124 行 `log_excerpt.is_empty()` 判断改在渲染后的字符串上（行为不变）。
- `view_model/source_engines.rs`：26、28、49 行三处 `status_detail` → `render_message`。

随后整体删除 `status_detail`（137-466 行）与 `localize_file_count_suffix` / `localize_delivery_log_excerpt` / `localize_project_filter` / `localize_project_sort`（777-818 行）——这是硬切换的删除面，`rg "status_detail" zircon_hub/src zircon_hub/tests` 终态必须零命中。

（e）`strip_prefix` 链逐处盘点表（37 处全量；行号为 2026-06-12 `localized.rs` 实仓；"变体"列为新 `HubMessageId` 归宿，模板中英原文逐字取自该行附近的前缀与 `format!` 重组结果）：

| 行号 | 英文前缀原文 | 新变体 | 参数 |
|---|---|---|---|
| 142 | `"Payload is required for Hub action: "` | `Shell(PayloadRequiredForAction)` | 1 |
| 145 | `"Invalid payload for Hub action "`（`": "` 二分） | `Shell(InvalidPayloadForAction)` | 2 |
| 150 | `"Project location must be an absolute path: "` | `Project(LocationNotAbsolute)` | 1 |
| 153 | `"Project path must be an absolute path: "` | `Project(PathNotAbsolute)` | 1 |
| 156 | `"Import path must be an absolute path: "` | `Project(ImportPathNotAbsolute)` | 1 |
| 159 | `"Import folder must be an absolute path: "` | `Project(ImportFolderNotAbsolute)` | 1 |
| 162 | `"Initial directory must be an absolute path: "` | `Settings(InitialDirectoryNotAbsolute)` | 1 |
| 165 | `"Resource path must be an absolute path: "` | `Learn(ResourcePathNotAbsolute)` | 1 |
| 168 | `"Output path must be an absolute path: "` | `Delivery(OutputPathNotAbsolute)` | 1 |
| 171 | `"Output directory must be an absolute path: "` | `Delivery(OutputDirectoryNotAbsolute)` | 1 |
| 174 | `"Unknown project template: "` | `Project(UnknownTemplate)` | 1 |
| 177 | `"Project template is coming soon: "` | `Project(TemplateComingSoon)` | 1 |
| 180 | `"Project folder does not exist: "` | `Project(FolderDoesNotExist)` | 1 |
| 183 | `"zircon-project.toml was not found in "` | `Project(ManifestNotFound)` | 1 |
| 186 | `"zircon-project.toml could not be parsed in "` | `Project(ManifestUnparsable)` | 1 |
| 189 | `"Project folder was created at "`（`" but Hub failed to record it: "` 二分） | `Project(CreatedButRecordFailed)` | 2 |
| 194 | `"Project root is not valid: "` | `Project(RootNotValid)` | 1 |
| 197 | `"Project has no bound Source Engine: "` | `Engine(ProjectHasNoBoundEngine)` | 1 |
| 200 | `"Project bound Source Engine is unavailable: "` | `Engine(BoundEngineUnavailable)` | 1 |
| 203 | `"Unknown Source Engine: "` | `Engine(UnknownEngine)` | 1 |
| 206 | `"Created "` | `Project(CreatedAt)` | 1 |
| 209 | `"Imported "` | `Project(ImportedAt)` | 1 |
| 212 | `"Output folder does not exist: "` | `Delivery(OutputFolderDoesNotExist)` | 1 |
| 215 | `"Resource file does not exist: "` | `Learn(ResourceFileDoesNotExist)` | 1 |
| 218 | `"Opened "` | `Shell(OpenedPath)` | 1 |
| 221 | `"Editor executable is not available: "` | `Process(EditorExecutableUnavailable)` | 1 |
| 224 | `"Started process "` | `Process(StartedProcess)` | 1 |
| 227 | `"Opening "`（`" (process {1})"` 后缀二分） | `Process(OpeningTargetWithProcess)` | 2 |
| 235 | `"Process "` | `Process(ProcessId)` | 1 |
| 239 | `"Showing "`（接 `localize_project_filter`，803-810 行） | `Project(ShowingAllProjects / ShowingExistingProjects / ShowingMissingProjects)`——按 filter 三选一展开为无参变体，产点直接择一 | 0 |
| 244 | `"Sorting by "`（接 `localize_project_sort`，812-818 行） | `Project(SortedByName / SortedByLastModified)`——同上展开 | 0 |
| 247（`localize_delivery_log_excerpt`，787-801 行） | `"Packaged/Installed {t} to {p} ({n} files)"` | `Delivery(PackagedSummary / InstalledSummary)`，en 模板 `"Packaged {0} to {1} ({2} files)"` | 3 |
| 250（`localize_file_count_suffix`，777-785 行） | `"{prefix} ({n} files)"`（实际产形为 `"Game -> C:\... (2 files)"`，见 localized.rs:675 测试） | `Delivery(HandoffWithFileCount)`，en 模板 `"{0} -> {1} ({2} files)"` | 3 |
| 253 | `"Package directory already exists: "` | `Delivery(PackageDirectoryExists)` | 1 |
| 256 | `"Device install already exists: "` | `Delivery(DeviceInstallExists)` | 1 |
| 259 | `"Unknown Hub language: "` | `Settings(UnknownLanguage)` | 1 |
| 262 | `"Unknown build profile: "` | `Settings(UnknownBuildProfile)` | 1 |
| 265 | `"Unknown settings folder field: "` | `Settings(UnknownFolderField)` | 1 |
| 269 | `"Unknown recent project target for "`（`": "` 二分） | `Shell(UnknownRecentProjectTarget)` | 2 |

注：8 条 `must be an absolute path` 词条在产点是 `action_request.rs:479` 的同一 `format!("{label} must be an absolute path: {}", ...)` 按 label 展开——迁移时在该处按 label → 变体做一次映射（`"Project location"`→`Project(LocationNotAbsolute)` 等 8 对），不引入"label 也是参数"的复合模板。

（f）常量词条表（`localized.rs:275-463`，约 92 条）归域清单。变体名按"英文消息 PascalCase 缩略、recovery 类加 `Recovery` 前缀"规则命名，中英模板逐字照抄对应行，不在此复制全文：

| 域 | 来源行号区间（起始行） | 条数 | 变体名样例 ← 英文原文 |
|---|---|---|---|
| Shell | 276-277、289、437-445 | 8 | `HubReady` ← "Hub is ready"（276）；`RecoveryCheckConfigPath` ← "Check the Hub config path and retry the action"（443） |
| Engine | 280-301 | 9 | `Ready` ← "Source engine is ready"（280）；`CheckoutMissingCargoToml`（282）；`CheckoutMissingRuntimeMember`（283）；`CheckoutMissingBuildScript`（286）；`RecoveryLocateCheckout`（290）；`RecoverySelectRepoRootWithManifest`（293）；`RecoverySelectRepoRootWithRuntimeMember`（296）；`RecoverySelectCompleteCheckout`（299）；`CheckoutDirectoryMissing`（281） |
| Project | 302、329-331、366-410、462 | 22 | `ShowingAllRecentProjects`（302）；`RecoveryFillCreateForm`（366）；`RecoveryChooseRenderableEmpty`（369）；`RecoveryUseImportForExistingFolder`（378）；`CreateKeptFolderUseImport`（381）；`RemovedFromHub`（394）；`DeleteWindowsOnly`（405）；`RecycleBinUnsupportedPlatform`（408）；`NameMustNotBeEmpty`（462） |
| Build | 303-304、310-320 | 7 | `RunningBuildScript` ← "Running tools/zircon_build.py"（303）；`StagedPayload`（304）；`RecoverySelectValidProject`（310）；`RecoveryCheckToolchain`（313）；`RecoveryOpenBuildHistory`（316）；`NoRecentProjectToBuild`（319）；`SelectedProjectUnavailableToBuild`（320） |
| Delivery | 305-308、321-347、424-436 | 21 | `CopyingProjectIntoPackage`（305）；`NoRecentProjectToPackage`（321）；`RecoverySelectProjectBeforePackaging`（325）；`PackageOutputRootRequired`（333）；`DeviceInstallDirInsidePackage`（342）；`OpenOutputTargetRequired`（424）；`RecoveryRerunWorkflow`（431） |
| Process | 309、348-365 | 7 | `LaunchingStagedEditor`（309）；`RecoverySelectProjectOrEmptyEditor`（348）；`RecoveryBuildPayloadBeforeLaunch`（357）；`RecoveryVerifyEditorExecutable`（363） |
| Settings | 446-461 | 14 | `NoFolderSelected`（446）；`RecoveryCheckValues`（451）；`DraftRestoredToSaved`（452）；`DraftRestoredToDefaults`（453）；`PythonRequired`（455）……七条 `"… is required"` 全收（454-461） |
| Learn | 411-423 | 5 | `RecoveryChooseCatalogResource`（411）；`OpenResourceTargetRequired`（414）；`ResourceNotInCatalog`（415）；`RecoveryRefreshCatalogOr`（418）；`RecoveryRefreshCatalogAnd`（421） |

（g）上游计划登记词条的收口（只登记变体，不展开实现）：01 的 6 条 payload 校验词条、02 的 `"Save Hub state failed"` label + `"Check the Hub config path and retry the action"`、03 的 create 补偿（189-193）/`"If the folder already contains..."`（378）/非 Windows 回收站（405-410）、04 的 discard/restore（118-119 label、452-453 detail）均已按旧机制进表，已包含在上面 e/f 清单内（label 类留在 `status_label`，不进枚举）。仍未落仓、需在枚举中预留的变体：04.M2 的 Source Engine 深校验 4 条 detail/recovery（`Engine(CheckoutManifestUnparsable / CheckoutMembersExcludeRuntime / RecoveryFixWorkspaceManifest / RecoverySelectRootWithRuntimeInMembers)`，模板原文以 04 计划实落为准，若 04 晚于本计划落地则在 04 变更内补变体）；02 的 worker panic（`Shell(BackgroundTaskPanicked)`，已列入 shell.rs 样例，属"新增翻译"——现状中文界面回落英文，不受"渲染结果逐字不变"约束保护，契约无断言）。05 的 `demo_mode_badge` 是 `ui_text.rs` key，归 M3，不进消息枚举。

（h）产点迁移盘点（`rg -n "TaskStatus::(error|success|warning|running)" zircon_hub/src` 共约 58 处、`rg -n "HubActionRecord \{" zircon_hub/src` 非测试约 17 处、`SourceBuildRecord` 产点 `build_actions.rs:324` 与 `source_engine_install.rs:59`；2026-06-12 快照，实施时以重新 rg 为准）：

| 文件 | TaskStatus 产点（行） | record 产点（行） |
|---|---|---|
| `runtime_state.rs` | 131、269、345、360、397、415、440、475、487、538、546、554、563 | — |
| `runtime_state/action_targets.rs` | 129（错误产点 35：`"Unknown recent project target for {}: {}"` → `Shell(UnknownRecentProjectTarget)`） | — |
| `runtime_state/action_tasks.rs` | 250、320（含 `record_background_action_error` / `record_background_worker_panic` 签名改造，见 c） | — |
| `runtime_state/build_actions.rs` | 92、147、176、202、233、293 | 132、164、190、221、281；`SourceBuildRecord` 324 |
| `runtime_state/project_actions.rs` | 98、150、204、222、253、268、297、321、518、542 | 559 |
| `runtime_state/project_delivery_actions.rs` | 220、270 | 258、288、311 |
| `runtime_state/editor_launch_actions.rs` | 251 | 225、346 |
| `runtime_state/learn_actions.rs` | 80、116 | 67、103 |
| `runtime_state/output_actions.rs` | 62、133 | 48、120 |
| `runtime_state/quick_actions.rs` | 28 | — |
| `runtime_state/settings_actions.rs` | 110、119、163、171、185、198 | — |
| `tauri_app/action_request.rs` | —（错误产点 363、389、479 改发 `HubError::Status`） | — |

迁移规则（产点逐处适用，无需再做设计决策）：detail/recovery 字符串在 e/f 清单有对应变体的→构造该变体；是项目名/引擎显示名/路径/外部命令日志/`io::Error` 原文等逐字内容的→`HubMessage::raw_text(...)`（已知逐字位点：`runtime_state.rs:397` 的 `display_name`、`runtime_state.rs:440` 的 `engine.display_name`、`runtime_state.rs:563` 的 `"Visual verification success state"` 视觉 fixture 文案、build/delivery 的 `log_excerpt` 真实日志行）；`format!` 拼接的→拆为变体 + params。

#### 文件变更清单

| 路径 | 动作 | 变更内容 |
|---|---|---|
| `zircon_hub/src/state/hub_message/mod.rs` | 新建 | 模块声明 + `pub use`（薄文件） |
| `zircon_hub/src/state/hub_message/message.rs` | 新建 | `HubMessage` + serde 兼容（Repr 中转）+ `render`/`is_empty`/构造器 + 单测 |
| `zircon_hub/src/state/hub_message/id.rs` | 新建 | `HubMessageId` 外层枚举 + `as_str`/`from_str_id`/`param_count`/`template`/`all` 分派 + 完备性测试 |
| `zircon_hub/src/state/hub_message/{shell,project,engine,build,delivery,process,settings,learn}.rs` | 新建 | 各域子枚举 + `ALL` + 双语模板表（按 e/f/g 清单） |
| `zircon_hub/src/state/mod.rs` | 修改 | 挂 `mod hub_message;` 与 re-export |
| `zircon_hub/src/state/task_status.rs` | 修改 | `detail`/`recovery` 改 `HubMessage`，构造器签名同步，删 `detail_with_recovery` |
| `zircon_hub/src/state/action_history.rs` | 修改 | `HubActionRecord.detail/log_excerpt/recovery` 改 `HubMessage` 形态 |
| `zircon_hub/src/engines/source_engine_install.rs` | 修改 | `SourceBuildRecord.detail/log_excerpt` 改 `HubMessage` |
| `zircon_hub/src/error.rs` | 修改 | 增 `HubError::Status { detail, recovery }` 变体 |
| `zircon_hub/src/tauri_app/action_request.rs` | 修改 | 363/389/479 三处错误构造改发结构化变体（绝对路径 8 词条按 label 映射） |
| `zircon_hub/src/tauri_app/runtime_state.rs` 及 `runtime_state/` 下 10 个 actions 文件 | 修改 | 按 h 表逐产点迁移；`record_background_action_error`/`record_background_worker_panic` 签名改造 |
| `zircon_hub/src/tauri_app/view_model.rs` | 修改 | `task_summary` 改 `render_message` |
| `zircon_hub/src/tauri_app/view_model/action_history.rs` | 修改 | 行投影改 `render_message`；单测期望中文逐字保留 |
| `zircon_hub/src/tauri_app/view_model/source_engines.rs` | 修改 | 三处 `status_detail` 改 `render_message`；单测同步 |
| `zircon_hub/src/tauri_app/view_model/localized.rs` | 修改 | 增 `render_message` 委托；删 `status_detail`（137-466）与 4 个 helper（777-818）；测试区 `status_detail(...)` 断言全量迁移为 `render_message(&HubMessage::...)`，期望中文逐字不变 |
| `zircon_hub/src/settings/hub_config.rs` | 修改 | round-trip 测试（423-434、473 行）改构造 `HubMessage`；新增旧字符串/新结构混合文件读取测试 |
| `zircon_hub/tests/project_page_copy_contract.rs` 等（见契约联动） | 修改 | 同变更刷新源码 snippet 断言 |

#### 实施步骤

前置：02/03/04 已收口（index.md §2 阶段 D）；开工时重新 `rg` 核对 h 表行号漂移。步骤 3-4 构成一个逻辑上的硬切换变更（中间提交允许新旧并存以保持可编译，但本里程碑合入时 `status_detail` 必须已删净，不留跨里程碑双轨）。

1. 新建 `src/state/hub_message/` 全部文件（形状 a + e/f/g 清单），`state/mod.rs` 挂接；自带单测：完备性、id round-trip、`toml` 内嵌结构序列化/旧字符串反序列化、未知 id 降级 RawText。此步纯增量，无消费方。
   验证：`cargo test -p zircon_hub --lib hub_message --locked`（注意 `[lib] test = false`，必须显式 `--lib`）；`cargo check -p zircon_hub --locked`。
2. `view_model/localized.rs` 增 `render_message`（形状 d 第一段，旧链暂存，本里程碑内删除）。
   验证：`cargo test -p zircon_hub --lib localized --locked`。
3. 类型切换 + 产点全量迁移（一次提交，机械但大）：按形状 b/c 改 `task_status.rs` / `action_history.rs` / `source_engine_install.rs` / `error.rs` / `action_request.rs`，随后以编译错误为驱动按 h 表迁移全部产点（迁移规则见 h 末段）；同提交内把三个投影点切到 `render_message`（形状 d）。`hub_config.rs` round-trip 测试同步改构造。
   验证：`cargo check -p zircon_hub --locked` 零错误；`cargo test -p zircon_hub --lib --locked`。
4. 删除旧链 + 契约刷新（与步骤 3 同 PR）：删 `status_detail` 与 4 个 helper 及 `localized.rs` 测试区对它们的断言（断言全量迁移为 `render_message` 形式、期望中文逐字保留作渲染等价锚）；按"契约联动"表刷新 6 个契约文件 snippet。终检 `rg -n "status_detail|strip_prefix" zircon_hub/src/tauri_app/view_model zircon_hub/tests` 零命中（`learn/`、`projects/` 下的 `Path::strip_prefix` 是路径运算，不在删除面）。
   验证：`cargo test -p zircon_hub --locked && cargo test -p zircon_hub --lib --locked && cargo fmt --all --check`。
5. 兼容与跟随验收：`hub_config.rs` 新增混合文件测试（fixture 内同一 `action_history` 数组混排字符串 detail 与 `{ id, params }` detail，断言整体可读、RawText 原文保留）；`view_model/action_history.rs` 新增语言跟随测试（同一持久化 record 分别以中英投影，断言两种渲染）。手工：`npm run tauri:dev`（在 `zircon_hub/` 目录，`package.json` 位于 `zircon_hub/` 而非 `web/`）跑一次 build/package 动作后切换语言，确认历史记录与状态横幅即时跟随、旧 `hub.toml` 历史按英文原文显示。
   验证：`cargo test -p zircon_hub --lib hub_config --locked`；全量回归同步骤 4。

#### 契约联动

渲染等价总锚：M1 为机械迁移，两种语言的全部渲染结果逐字不变（唯一例外：g 段标注的"新增翻译"位点，现状即英文回落、无契约断言）。

| 契约文件 | 现有断言（原文片段） | 改成 |
|---|---|---|
| `project_page_copy_contract.rs:117-183` | `"pub(crate) fn status_detail(self, detail: &str) -> String"`、22 组 `"detail.strip_prefix(\"...\")"` + `"return format!(\"项目模板尚未开放：{template}\")"` 等中文重组 snippet | 该块改读新模板文件（`read_crate_file("src/state/hub_message/project.rs")` 等）：锚定 `"pub(crate) fn render_message(self, message: &HubMessage) -> String"`（localized.rs）+ 等价模板原文 snippet（如 `"项目模板尚未开放：{0}"`、`"未在 {0} 找到 zircon-project.toml"`），逐条对应原 22 组 |
| `project_page_copy_contract.rs:189-192` | `"let detail = text.status_detail(&record.detail);"`、`"let log_excerpt = text.status_detail(&record.log_excerpt);"`、`".map(\|recovery\| text.status_detail(recovery))"` | `render_message` 等价形式（`"let detail = text.render_message(&record.detail);"` 等三条） |
| `project_source_engine_contract.rs:232-233` | `"detail: text.status_detail(&record.detail)"`、`"log_excerpt: text.status_detail(&record.log_excerpt)"` | `render_message` 等价形式 |
| `tauri_react_shell_contract.rs:240` | `"let log_excerpt = text.status_detail(&record.log_excerpt);"` | `render_message` 等价形式 |
| `ui_foundation_contract.rs:729-732` | `"let detail = text.status_detail(&record.detail);"` 等四条 | `render_message` 等价形式 |
| `project_workflow_contract.rs:364、373` | `"text.status_detail(\"Check Settings values and save again\")"`、`"text.status_detail(\"Choose an existing local folder or type the path manually\")"` | settings_actions 单测迁移后的等价 snippet（`render_message(&HubMessage::new(...Settings(RecoveryCheckValues)...))` 形式，以实改为准） |

- 必须保持不变（中文渲染期望是等价锚）：`localized.rs` 测试区 552-774 行全部期望中文字符串、`view_model/action_history.rs` 测试区 179-298 行期望中文、`source_engines.rs:118-123` 期望中文——断言输入侧改为构造 `HubMessage`，期望值逐字不动。
- 必须保持不变：`status_label` 表（70-135 行）与其契约（`project_page_copy_contract.rs:108-113`）——label 不进本里程碑。
- 新增测试：`hub_message::id::tests::every_message_id_has_bilingual_templates_with_matching_placeholders`（目标 3）；`hub_message::id::tests::message_id_round_trips_through_stable_string_ids`；`hub_message::message::tests::archived_string_deserializes_into_raw_text_branch`；`hub_message::message::tests::unknown_id_degrades_to_raw_text_instead_of_failing_file_load`；`hub_config::tests::loads_archived_string_action_history_alongside_structured_messages`（风险章节的混合文件测试）；`view_model::action_history::tests::action_history_rows_render_persisted_message_ids_in_current_language`（目标 2）。
- 02 计划承诺的 `persist_failure_sets_recoverable_status_and_recovers_after_retry` 等若已落仓：其断言面是 label（`"Save Hub state failed"`），不受本里程碑影响；若其断言了 detail 英文原文，按渲染等价改为 `HubMessage` 构造断言。

### M2 coming-soon 目录填充

【2026-06-12 落地状态注记】原切片 1 的主体已由并行进程落地：`coming_soon.rs` 已含 13 条（projects 1 / assets 1 / plugins 3 / local-delivery 3 / shell 2 / team 3），`coming_soon_category_label` 已含 `projects`/`shell` 臂（163-173 行），前端 `hubData.ts:84-215` fallback 同步 13 条，四个页面（EditorPage / CatalogPage / CloudPage / TeamPage）均已按 category 过滤消费 `state.comingSoon` 且无页面内"敬请期待"字面量（`rg 敬请期待 web/src` 仅命中 `hubData.ts` fallback 镜像）。本里程碑改为"盘点补缺 + 验收"口径，剩余工作两项：模板预留条目补齐 + shell 双源文案收敛。

切片：
1. 按 v1 设计逐页登记【大半已落地，盘点补缺】：补 `project-template-3d-scene`、`project-template-sample-world` 两条（模板目录 3 个禁用模板只登记了 `2d-scene`）；其余页面条目验收即可。
2. 前端 disabled 行/按钮全部改从 coming-soon DTO 取说明文案【收敛残余】：TopBar 通知 tooltip 与 UserMenu sign-out 说明从 `ui.shell.notificationsDetail` / `ui.shell.signOutDetail` 切到 `comingSoon` 目录条目，删除 `ui_text.rs` 中这两个重复 key——同一文案此后只有目录一个 owner。

#### 目标代码形状

（a）`coming_soon.rs` 条目数组（22-140 行）在 `"project-template-2d-scene"` 元组后插两条，措辞与既有 2d-scene 条目同构：

```rust
(
    "project-template-3d-scene",
    "projects",
    text.pair("3D Scene Template", "3D 场景模板"),
    text.pair(
        "The 3D scene template is reserved until the local authoring workflow is ready.",
        "3D 场景模板会在本地创作工作流就绪后开放。",
    ),
),
(
    "project-template-sample-world",
    "projects",
    text.pair("Sample World Template", "示例世界模板"),
    text.pair(
        "The sample world template is reserved for sample content generation.",
        "示例世界模板为示例内容生成预留。",
    ),
),
```

并补"禁用模板必有目录条目"的结构守卫与双语非空审计（`coming_soon.rs` 测试区，替代人工盘点）：

```rust
#[test]
fn every_disabled_project_template_is_registered_in_coming_soon_catalog() {
    let entries = super::coming_soon_entries(HubLanguage::Chinese);
    for template in crate::projects::project_template_catalog() {
        if template.enabled {
            continue;
        }
        let expected_id = format!("project-template-{}", template.id);
        assert!(
            entries.iter().any(|entry| entry.id == expected_id && entry.category == "projects" && entry.disabled),
            "禁用模板 {} 缺 coming-soon 目录条目 {expected_id}",
            template.id
        );
    }
}

#[test]
fn coming_soon_entries_are_non_empty_in_both_languages() {
    for language in [HubLanguage::English, HubLanguage::Chinese] {
        for entry in super::coming_soon_entries(language) {
            for (field, value) in [("title", &entry.title), ("detail", &entry.detail), ("status", &entry.status), ("meta", &entry.meta), ("category_label", &entry.category_label)] {
                assert!(!value.trim().is_empty(), "{} 的 {field} 在 {language:?} 为空", entry.id);
            }
        }
    }
}
```

说明（避免实施者自造面板）：Projects 页 New Project 对话框的禁用模板行已由 `project_templates.rs` 的 `option_label`/`status`/`disabled_reason` DTO 驱动（21-51 行），M2 不给 Projects 页新增 coming-soon 面板——`projects` 分类条目是能力目录登记位 + 守卫绑定对象，渲染面维持现状。

（b）shell 双源收敛（硬切换：目录条目为唯一 owner，`ui_text` 重复 key 删除；两处 tooltip 的中文渲染随之从 ui_text 措辞换为目录措辞，属文案收敛、非 M1 约束面）。`TopBar.tsx`：

```tsx
// TopBar.tsx —— 组件体内（现 28-33 行常量区）追加：
const comingSoonDetail = (id: string) =>
  state.comingSoon.find((entry) => entry.id === id)?.detail ?? "";

// 现 126 行：
// <HubIconButton label={state.ui.shell.notifications} tooltip={state.ui.shell.notificationsDetail} disabled ...>
// 改为：
<HubIconButton label={state.ui.shell.notifications} tooltip={comingSoonDetail("notification-center")} disabled sx={topIconSx}>

// 现 188-196 行 <UserMenuPopover ... /> 增 prop：
signOutDetail={comingSoonDetail("sign-out")}
```

`UserMenuPopover.tsx`：props 增 `signOutDetail: string`（10-18 行接口），25 行菜单项 `detail: text.signOutDetail` → `detail: signOutDetail`。

`ui_text.rs` 删除面：`HubShellText.sign_out_detail`（42 行字段）与构造（396-401 行）、`HubShellText.notifications_detail`（57 行字段）与构造（445-450 行）、单测 938-941 行对 `notifications_detail` 的断言；`sign_out` / `notifications` label key 保留。前端同步删：`types/hub.ts` 的 `signOutDetail`（297 行）与 `notificationsDetail`（312 行）；`hubData.ts` 的 `signOutDetail`（362 行）与 `notificationsDetail`（387 行）。

（c）`hubData.ts` fallback 的 `comingSoon` 数组（84-215 行）在 2d-scene 条目后补两条中文镜像（与 a 的中文渲染逐字一致）：

```ts
{
  id: "project-template-3d-scene",
  category: "projects",
  categoryLabel: "项目",
  title: "3D 场景模板",
  detail: "3D 场景模板会在本地创作工作流就绪后开放。",
  status: "敬请期待",
  meta: "项目 / 敬请期待",
  disabled: true,
},
{
  id: "project-template-sample-world",
  category: "projects",
  categoryLabel: "项目",
  title: "示例世界模板",
  detail: "示例世界模板为示例内容生成预留。",
  status: "敬请期待",
  meta: "项目 / 敬请期待",
  disabled: true,
},
```

#### 文件变更清单

| 路径 | 动作 | 变更内容 |
|---|---|---|
| `zircon_hub/src/tauri_app/view_model/coming_soon.rs` | 修改 | 增两条模板预留条目 + 模板守卫测试 + 双语非空审计测试 |
| `zircon_hub/src/tauri_app/view_model/ui_text.rs` | 修改 | 删 `sign_out_detail` / `notifications_detail` 字段、构造与单测断言 |
| `zircon_hub/web/src/components/shell/TopBar.tsx` | 修改 | 通知 tooltip 改取 `comingSoon`；向 UserMenuPopover 传 `signOutDetail` |
| `zircon_hub/web/src/components/overlays/UserMenuPopover.tsx` | 修改 | 增 `signOutDetail` prop，sign-out 行 detail 改用之 |
| `zircon_hub/web/src/types/hub.ts` | 修改 | `HubShellText` 删两个 detail 字段 |
| `zircon_hub/web/src/data/hubData.ts` | 修改 | fallback `comingSoon` 增两条镜像；shell 删两个 detail key |
| `zircon_hub/tests/ui_foundation_contract.rs` | 修改 | coming_soon snippet 列表（753-762 行）只增 `"project-template-2d-scene"`/`"project-template-3d-scene"`/`"project-template-sample-world"`/`"notification-center"`/`"sign-out"`；hubData 断言块（796-817 行）只增 `"category: \"projects\""`、`"category: \"shell\""` |
| `zircon_hub/tests/ui_shell_header_contract.rs` | 修改 | 双源收敛涉及的断言刷新（见契约联动） |

#### 实施步骤

1. 后端补条目 + 守卫（形状 a）：`coming_soon.rs` 插两条元组、加两个测试；`ui_foundation_contract.rs` 两处断言块只增 snippet。
   验证：`cargo test -p zircon_hub --lib coming_soon --locked`；`cargo test -p zircon_hub --test ui_foundation_contract --locked`。
2. 前端 fallback 镜像（形状 c）。
   验证：在 `zircon_hub/` 目录 `npm run typecheck`。
3. shell 双源收敛（形状 b，一次提交内删净）：TopBar / UserMenuPopover / `ui_text.rs` / `types/hub.ts` / `hubData.ts` 五处同改；`ui_shell_header_contract.rs` 按契约联动表刷新。终检 `rg -n "signOutDetail|notificationsDetail|sign_out_detail|notifications_detail" zircon_hub/src zircon_hub/web/src zircon_hub/tests` 仅剩 comingSoon 取值路径与契约新断言。
   验证：在 `zircon_hub/` 目录 `npm run typecheck && npm run build`；`cargo test -p zircon_hub --test ui_shell_header_contract --test project_page_copy_contract --locked`。
4. 回归与视觉：`cargo test -p zircon_hub --locked`；视觉验收并入 06.M3 截图矩阵——确认 TopBar 通知 tooltip、UserMenu sign-out、四页 Reserved 面板均显示目录文案且无英文残留。

#### 契约联动

| 契约文件 | 现有断言（原文片段） | 改成 |
|---|---|---|
| `ui_shell_header_contract.rs:192` | `"notificationsDetail: \"通知中心敬请期待；本地 v1 不启用通知服务。\""`（hubData 块） | 删除该条；同块只增 `"id: \"notification-center\""` |
| `ui_shell_header_contract.rs:225` | `"{ id: \"sign-out\", label: text.signOut, detail: text.signOutDetail, Icon: LogoutOutlinedIcon, danger: true, disabled: true }"` | `"{ id: \"sign-out\", label: text.signOut, detail: signOutDetail, Icon: LogoutOutlinedIcon, danger: true, disabled: true }"` |
| `ui_shell_header_contract.rs:244-247` | ui_text.rs 含 `"Remote account service is not enabled in local v1."` / `"Notification center is coming soon; local v1 does not enable a notification service."` 等四条 | 改读 `coming_soon.rs`：断言 `"Remote accounts are disabled for the local-only Hub."`、`"本地版 Hub 不启用远程账号。"`、`"桌面通知为预留能力；v1 在 Hub 窗口内显示本地任务反馈。"`（目录条目为唯一 owner） |
| `ui_shell_header_contract.rs:250-254` | `"signOutDetail: \"本地 v1 不启用远程账号服务。\""`（hubData 块） | 删除该条；改断言 hubData `comingSoon` 含 `"detail: \"本地版 Hub 不启用远程账号。\""` |
| `ui_foundation_contract.rs:753-762、796-817` | coming_soon id 列表缺 projects/shell 五个 id；hubData 块缺 projects/shell category 断言 | 只增（见文件变更清单行） |

- 必须保持不变：`project_cloud_local_delivery_contract.rs:55-120`（Cloud 页仅渲染 disabled `local-delivery` 条目）、`tauri_react_shell_contract.rs:644`（Editor 页 plugins 过滤）、`ui_foundation_contract.rs:838-895`（meta/categoryLabel 组合）、`project_page_copy_contract.rs:235`（`"2D 场景（敬请期待）"` 模板 option label，属 `project_templates.rs` 路径，本里程碑不触碰）。
- 新增测试：`coming_soon::tests::every_disabled_project_template_is_registered_in_coming_soon_catalog`（断言要点：`project_template_catalog()` 每个 `enabled == false` 模板存在 `project-template-{id}`、category `"projects"`、disabled）；`coming_soon::tests::coming_soon_entries_are_non_empty_in_both_languages`（中英 title/detail/status/meta/category_label 非空）。

### M3 ui_text 覆盖审计

切片：中英分支字段非空审计测试；登记 05.M3 新增 key（`demo_mode_badge` 徽标、ErrorBoundary 文案）。

#### 目标代码形状

`HubUiText` 全树（9 个分组结构 + `nav_items`）经 serde 走 JSON 后递归审计字符串叶子，两种语言各跑一遍；已知刻意为空的 key 进显式 allowlist（现仅一个：`catalog.searchPlaceholderSeparator` 中文分支为 `""`，`ui_text.rs:752`）。落 `ui_text.rs` 测试区：

```rust
#[test]
fn every_ui_text_field_is_non_empty_in_both_languages() {
    const ALLOW_EMPTY: &[&str] = &["catalog.searchPlaceholderSeparator"];

    for language in [HubLanguage::English, HubLanguage::Chinese] {
        let tree = serde_json::to_value(super::ui_text(language)).expect("ui_text serializes");
        let mut empty_keys = Vec::new();
        collect_empty_string_keys(&tree, String::new(), &mut empty_keys);
        empty_keys.retain(|key| !ALLOW_EMPTY.contains(&key.as_str()));
        assert!(empty_keys.is_empty(), "{language:?} 存在空文案 key：{empty_keys:?}");
    }
}

fn collect_empty_string_keys(value: &serde_json::Value, path: String, out: &mut Vec<String>) {
    match value {
        serde_json::Value::String(text) => {
            if text.trim().is_empty() {
                out.push(path);
            }
        }
        serde_json::Value::Object(map) => {
            for (key, child) in map {
                let child_path = if path.is_empty() { key.clone() } else { format!("{path}.{key}") };
                collect_empty_string_keys(child, child_path, out);
            }
        }
        serde_json::Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                collect_empty_string_keys(child, format!("{path}[{index}]"), out);
            }
        }
        _ => {}
    }
}
```

说明：`HubUiText` 中英共用同一结构体，"key 是否两语都有"由类型系统保证，审计的真实风险面是空串与漏填——上述测试即覆盖；`serde_json` 已是 `zircon_hub` 直接依赖（Cargo.toml:20），不引新依赖。`nav_items` 数组叶子由 `[index]` 路径覆盖。

05.M3 联动 key 登记清单（核对位，不在本计划实现）：`HubShellText.demo_mode_badge`（`text.pair("Demo Data", "演示数据")`，05 计划 §M3 已登记；落仓后自动进审计覆盖面，无需改本测试）；ErrorBoundary 复用既有 `shell.action_failed` 等四 key，无新增。若 05 先落地，本步只核对该 key 非空即可。

#### 文件变更清单

| 路径 | 动作 | 变更内容 |
|---|---|---|
| `zircon_hub/src/tauri_app/view_model/ui_text.rs` | 修改 | 测试区追加审计测试 + 递归 helper（生产代码零改动） |

#### 实施步骤

1. 落审计测试（目标代码形状）。若此时 M2 已删 `notifications_detail`/`sign_out_detail`、05 已增 `demo_mode_badge`，以实仓结构为准，审计自动适配（按字段树递归，不点名 key）。
   验证：`cargo test -p zircon_hub --lib ui_text --locked`（`[lib] test = false`，必须 `--lib`）。
2. 登记核对：对照 05 计划 §M3 清单确认 `demo_mode_badge` 落仓状态（未落则在 05 变更内落，本计划只留登记）；确认 allowlist 仅含 `catalog.searchPlaceholderSeparator`（若 06 布局计划調整了该分隔符实现则同步收敛 allowlist）。
3. 回归：`cargo test -p zircon_hub --locked && cargo fmt --all --check`。

#### 契约联动

- 必须保持不变：`project_page_copy_contract.rs:197-245` 对 `ui_text.rs` 的正/负向 snippet（`"title: text.pair(\"Projects\", \"项目\").to_string()"` 等）——审计测试是纯增量，不动生产构造。
- `app_error_recovery_contract.rs:59-94` 读 `ui_text.rs` 的断言不受影响（ErrorBoundary 走既有 key）。
- 新增测试：`ui_text::tests::every_ui_text_field_is_non_empty_in_both_languages`（断言要点：两语言全树字符串叶子非空，allowlist 显式列外）。

## 风险与协调

- 必须在 02/03/04 之后执行 M1：消息产生点先收敛（persist 单点、生命周期失败口径定稿、settings 规则表），否则 `HubMessageId` 枚举刚建即返工。2026-06-12 终核：01-04 由并行进程持续落地中，本文档引用的行号（尤其 `localized.rs` / `runtime_state/*` 产点表）随时漂移，实施前逐文件重新 `rg` 盘点，以实仓为准。
- `HubMessage` 进入持久化层会改 `hub.toml` 中 history（及 `engines.build_history`）的序列化形状：serde 上用 Repr 中转兼容枚举确保旧文件可读、未知 id 降级 RawText 而非整文件失败，并补新旧混合文件的读取测试。toml crate 对"标量键 + 表值键混排"的自动后置排序由 round-trip 测试锚定。
- 文案契约（`project_page_copy_contract` 等）断言具体中文字符串：M1 重构渲染路径时保持渲染结果不变（先机械迁移，措辞优化单独切片），降低契约红面。M1 的"新增翻译"位点（`Background task panicked` 等现状英文回落项）与 M2 的 shell 双源收敛（通知/sign-out tooltip 换为目录措辞）是仅有的两类刻意渲染变化，均已在对应契约联动中列明。
- 事实修正注记（2026-06-12 实仓核对）：原"现状与证据"所记 `localized.rs` 684 行 / 23 处 `strip_prefix`、`coming_soon.rs` 164 行 / 10 条均已漂移（现 818 行 / 37 处、193 行 / 13 条），原 M2"缺三条"中的三条已由并行进程落地——现状章节与 M2 已按实仓改写为盘点补缺/验收口径；目标 1 的删除面数字同步修正。
- M2 的 shell 双源收敛会删除 `ui_text.rs` 两个 key 并刷新 `ui_shell_header_contract.rs` 四处断言：该契约同时被 05/06 计划触碰（TopBar 组件化、布局修正），实施前先 `git log`/工作树核对该文件最新形态，只做本计划行内的最小刷新，避免与 05/06 双写冲突。
- `HubError::Status` 变体的 `Display` 以英文渲染（日志/`stderr` 可读性优先），不影响 UI 语言跟随（UI 路径走结构化 detail/recovery，不消费 `Display`）。
