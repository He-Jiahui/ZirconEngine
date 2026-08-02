---
related_code:
  - zircon_hub/src/tauri_app/runtime_state/settings_actions.rs
  - zircon_hub/src/tauri_app/view_model/settings_dto.rs
  - zircon_hub/src/settings/hub_config.rs
  - zircon_hub/src/settings/paths.rs
  - zircon_hub/src/engines/registry.rs
  - zircon_hub/src/engines/validation.rs
  - zircon_hub/src/engines/source_engine_paths.rs
  - zircon_hub/src/process/folder_picker.rs
  - zircon_hub/src/tauri_app/action_id.rs
  - zircon_hub/src/tauri_app/action_request.rs
  - zircon_hub/src/tauri_app/runtime_state.rs
  - zircon_hub/src/tauri_app/view_model.rs
  - zircon_hub/src/tauri_app/runtime_state/build_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/editor_launch_actions.rs
  - zircon_hub/src/tauri_app/view_model/localized.rs
  - zircon_hub/web/src/pages/SettingsPage.tsx
  - zircon_hub/web/src/types/hub.ts
  - zircon_hub/tests/project_source_engine_contract.rs
  - zircon_hub/tests/project_workflow_contract.rs
  - zircon_hub/tests/ui_input_navigation_api_contract.rs
plan_sources:
  - docs/plans/zircon_hub/index.md
  - docs/zircon_hub/pages/settings-status.md
  - docs/zircon_hub/state/foundations.md
status: in_progress
---

# 04 settings 草稿生命周期与 Source Engine 校验

> 2026-08-01 实仓复核：共享 settings field spec、save 必填校验、discard/restore 动作、picker 注入与取消零副作用、Source Engine workspace 深校验均已落在当前源码。open-editor 对已绑定 active Source Engine 的健康预检已补入 `editor_launch_actions.rs`，避免源码检出失效后仅凭 staged executable 继续启动；状态改为 `in_progress`，等待受管 Hub package gate 后再确认里程碑完成。

## 现状与证据

- 契约口径已定：`update-settings-draft` 只改 `settings_draft` 并重算 Configuration Health，不 persist；`save-settings` 才持久化、注册 Source Engine、刷新 source-scoped catalogs；`browse-settings-folder` 接收 `{ field, initialDir, settings }`，取消与错误不得污染已存设置（`docs/zircon_hub/pages/settings-status.md`）。
- 实仓缺口（`runtime_state/settings_actions.rs`，457 行）：
  - draft 生命周期缺"放弃修改"（cancel-draft / 回到已存设置）与"恢复默认"（restore-defaults）两个动作，前端只能靠重启丢弃脏 draft。【2026-06-12 终核快照】该缺口正在被并行工作树按本计划 M2 同口径补齐：typed id（`action_id.rs` 31 变体，as_str 88-89 行）、解析/路由臂、`settings_actions.rs` 两方法与单测、DTO 按钮词条、前端 `HUB_ACTION`（`hub.ts:632-633`）与 SettingsPage 按钮、`project_workflow_contract.rs:107`（已改 `[HubActionId; 31]`）在本文档细化期间陆续落仓——M2 据此从"新增"调整为"盘点补缺 + 验收"，明细见 M2 落地状态终核。
  - 【2026-06-12 修正】"folder picker 的取消与失败在部分路径上共用错误通道，取消可能落 error history"不成立：`process/folder_picker.rs:20-65` 早已返回 `Result<Option<PathBuf>, HubError>`、取消（对话框 exit code 2，56-58 行）= `Ok(None)`；settings browse 的取消分支走 warning 摘要（`"Folder selection cancelled"`），不写 history、不 persist。03 计划的 session 级 `folder_picker` 函数指针注入缝已落仓（`runtime_state.rs:65` 字段、118 行初始化；import 链路已走缝，`project_actions.rs:131`），且终核时 settings browse 也已切到 `(self.folder_picker)`（`settings_actions.rs:155`）并出现取消注入测试——剩余缺口收窄为失败注入测试与取消零副作用断言面核对。
  - save 时对 draft 的路径校验逐字段散落（`settings_dto.rs::apply_to` 逐字段 `trimmed_required`/`path_from_required`），与 health 计算（`settings_dto.rs::settings_health` 手写 7 行 `vec![...]`）两处维护，无统一"字段 → 校验规则 → health 贡献"表；且 `save_settings`（`runtime_state.rs:437` 行起）无 payload 分支直接 `self.config.settings = self.settings_draft.clone()`，**不经任何必填校验**——含空必填字段的 draft 可直接落盘。【2026-06-12 终核】并行工作树已按本计划 M1 口径落地 spec 表：`SETTINGS_FIELD_SPECS` 内联于 `settings_dto.rs`（非独立模块）、`settings_health` 已遍历该表（412 行起 `settings_field_row`）、browse/update 链路已统一 `apply_to_draft`；`apply_to` 仅剩 `runtime_state.rs::save_settings`（451 行）一处生产调用，save 必填校验缺口仍在。
- 【2026-06-12 修正】"Source Engine 校验过浅：仅验 source dir 存在 `Cargo.toml`；不验 `tools/zircon_build.py` 存在"已部分过时：`engines/validation.rs:37-49` 现已依次校验目录存在、`Cargo.toml` 存在、`tools/zircon_build.py` 存在（`MissingRoot`/`MissingWorkspaceManifest`/`MissingBuildTool` 三变体）。仍缺：不验 `Cargo.toml` 可解析、不验 workspace members 含 `zircon_runtime`——空文件或任意 TOML 都算"有效引擎"。
- 【2026-06-12 修正】"active engine 失效无预检"对 build 不成立：`build_actions.rs::validate_active_source_engine_for_build`（210-236 行）已在 prepare 阶段做 `validate_source_engine` 预检并落 `"Source Engine invalid"` error + recovery。仍缺：open-editor 链路（`editor_launch_actions.rs::prepare_editor_launch`：97-114 行）只查 staged 可执行文件（`ensure_editor_available`：256-265 行），引擎目录被移走时错误不指向根因；save-settings 注册引擎与启动注册（`runtime_state.rs:129` 调 `register_source_engine_from_settings`：548 行起）完全不校验，失效引擎静默入册。

## 目标

1. draft 生命周期补全为五动作闭环：`update-settings-draft` / `browse-settings-folder` / `save-settings` / `discard-settings-draft`（回到已存设置）/ `restore-default-settings`（回到内置默认值，仅改 draft 不落盘）。新动作走 01 计划的 typed id + payload。
2. 校验规则单点化：落一张 `SettingsFieldSpec` 表（字段 id、是否必填、校验器、health 权重、recovery 文案 key），save 校验与 Configuration Health 同源计算；health 百分比由真实字段状态推导，无任何硬编码常量。
3. picker 取消语义统一：取消 = 零持久副作用（不写 history、不改 draft、不 persist）；失败 = error TaskStatus + recovery。【2026-06-12 修正】`process/folder_picker.rs` 返回类型**无需改造**——现已是 `Result<Option<PathBuf>, HubError>` 且取消 = `Ok(None)`；本目标收敛为：settings browse 链路迁移到 03 计划的 `folder_picker` 注入缝并补取消/失败锚定测试；取消保留既有 `"Folder selection cancelled"` warning 摘要口径（与 03 计划 import 取消保留 `"Import cancelled"` warning 同原则，该摘要已被 `project_workflow_contract.rs:361` 契约锁定，删除属契约面收缩，无收益不做）。
4. Source Engine 注册校验加深：`Cargo.toml` 可解析且 workspace members 含 `zircon_runtime` + `tools/zircon_build.py` 存在，缺一项则 save-settings 链路注册失败（不 upsert 引擎记录）并给出指向性 recovery；同时为 open-editor 增加执行前 active engine 健康预检（build 预检已存在，升级为含上述深校验），失败早报。
5. save-settings 成功后行为保持契约：注册/更新 Source Engine、刷新 catalogs、persist 一次（走 02 的单点 persist）。

## 非目标

- 不做 settings 的多 profile / 导入导出。
- 不验证工具链可执行性（不真跑 `python --version`/`cargo --version`；只查路径存在与文件类型，执行期校验归 build 动作自身）。
- 不改 `hub.toml` 字段集合（新增动作不新增持久化字段）。

## 里程碑

### M1 字段规则表与 health 同源化

切片：
1. 在 `settings_actions.rs`（规则表数据可落 `view_model/settings_dto.rs` 邻近模块）建 `SettingsFieldSpec` 表；save 校验与 health 计算改为遍历该表；删除散落的逐字段 if 链与任何硬编码 health 数值。
2. 契约 `project_workflow_contract.rs` 中 update-draft "仅 draft + health 重算"断言同步刷新。

【2026-06-12 落地状态终核】本里程碑大部已由并行工作树按同口径落地：`SETTINGS_FIELD_SPECS` 与遍历式 `settings_health` 已在 `settings_dto.rs` 内联落仓（落点为 settings_dto 内联而非下文"目标代码形状 a"的独立模块——按硬切换原则以实仓落点为准，不回搬；文件清单"新建模块"行相应作废），browse/update 链路已切 `apply_to_draft`。实施者先 `rg "SETTINGS_FIELD_SPECS|validate_settings_for_save" zircon_hub/src` 盘点，可见剩余缺口：`save_settings` 仍调 `apply_to`（`runtime_state.rs:451`）且无 payload 分支不经必填校验（目标代码形状 c/d 的接线与硬切换删除）、必填拒绝测试、契约 snippet 增补。下文四块内容继续作为终态规格与验收依据。

#### 目标代码形状

（a）【已由并行工作树落地，规格供核对】`SettingsFieldSpec` 表。字段集合 = 现 health 的 7 行（`"python-path"`/`"cargo-path"`/`"rustup-path"`/`"project-dir"`/`"source-dir"`/`"build-output"`/`"device-install"`，行 id 与标题原文不变）；`save_label` 沿用 `apply_to` 时代的错误原文（`"Python executable"` 等），使 `localized.rs:447-453` 的七条 `"… is required"` 中文词条继续命中。实仓落点为 `settings_dto.rs` 内联（以实仓为准）；下列为本细化原拟的独立模块形状，核对字段语义用：

```rust
// 规格参考（实仓已内联于 settings_dto.rs，命名以实仓为准）
use std::path::Path;

use crate::error::HubError;
use crate::settings::HubSettings;

/// settings 字段的稳定标识；穷尽 match 保证 spec 表与取值器同步演进。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SettingsFieldId {
    PythonPath,
    CargoPath,
    RustupPath,
    DefaultProjectDir,
    DefaultSourceDir,
    DefaultBuildOutputDir,
    DefaultDeviceInstallDir,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SettingsFieldKind {
    /// 校验器 = settings_dto::executable_row（空值/路径存在/PATH 命中）
    Executable,
    /// 校验器 = settings_dto::directory_row（空值/is_dir/使用时创建）
    Directory,
}

pub(crate) enum SettingsFieldValue<'a> {
    Executable(&'a str),
    Directory(&'a Path),
}

/// 单字段规则：save 必填校验与 Configuration Health 行同源于此。
#[derive(Clone, Copy, Debug)]
pub(crate) struct SettingsFieldSpec {
    pub(crate) id: SettingsFieldId,
    pub(crate) kind: SettingsFieldKind,
    /// save 必填开关；v1 全部 true，表结构保留开关供未来可选字段使用。
    pub(crate) required: bool,
    /// Configuration Health 行 id（DTO 契约面，保持现值不变）。
    pub(crate) health_row_id: &'static str,
    /// health 行标题（en/zh），喂给 HubTextBundle::pair。
    pub(crate) title_en: &'static str,
    pub(crate) title_zh: &'static str,
    /// save 校验错误 "{save_label} is required" 的原文 label（recovery 文案 key），
    /// 与 localized.rs 既有词条逐字对齐。
    pub(crate) save_label: &'static str,
    /// health 行 selected 位（现状仅 python 行为 true）。
    pub(crate) selected: bool,
}

pub(crate) const SETTINGS_FIELD_SPECS: [SettingsFieldSpec; 7] = [
    SettingsFieldSpec { id: SettingsFieldId::PythonPath, kind: SettingsFieldKind::Executable, required: true, health_row_id: "python-path", title_en: "Python", title_zh: "Python", save_label: "Python executable", selected: true },
    SettingsFieldSpec { id: SettingsFieldId::CargoPath, kind: SettingsFieldKind::Executable, required: true, health_row_id: "cargo-path", title_en: "Cargo", title_zh: "Cargo", save_label: "Cargo executable", selected: false },
    SettingsFieldSpec { id: SettingsFieldId::RustupPath, kind: SettingsFieldKind::Executable, required: true, health_row_id: "rustup-path", title_en: "Rustup", title_zh: "Rustup", save_label: "Rustup executable", selected: false },
    SettingsFieldSpec { id: SettingsFieldId::DefaultProjectDir, kind: SettingsFieldKind::Directory, required: true, health_row_id: "project-dir", title_en: "Project Directory", title_zh: "项目目录", save_label: "Default project directory", selected: false },
    SettingsFieldSpec { id: SettingsFieldId::DefaultSourceDir, kind: SettingsFieldKind::Directory, required: true, health_row_id: "source-dir", title_en: "Source Checkout", title_zh: "源码检出目录", save_label: "Default source directory", selected: false },
    SettingsFieldSpec { id: SettingsFieldId::DefaultBuildOutputDir, kind: SettingsFieldKind::Directory, required: true, health_row_id: "build-output", title_en: "Build Output", title_zh: "构建输出", save_label: "Default build output directory", selected: false },
    SettingsFieldSpec { id: SettingsFieldId::DefaultDeviceInstallDir, kind: SettingsFieldKind::Directory, required: true, health_row_id: "device-install", title_en: "Device Install", title_zh: "设备安装", save_label: "Default device install directory", selected: false },
];

impl SettingsFieldSpec {
    pub(crate) fn value<'a>(&self, settings: &'a HubSettings) -> SettingsFieldValue<'a> {
        match self.id {
            SettingsFieldId::PythonPath => SettingsFieldValue::Executable(&settings.python_path),
            SettingsFieldId::CargoPath => SettingsFieldValue::Executable(&settings.cargo_path),
            SettingsFieldId::RustupPath => SettingsFieldValue::Executable(&settings.rustup_path),
            SettingsFieldId::DefaultProjectDir => SettingsFieldValue::Directory(&settings.default_project_dir),
            SettingsFieldId::DefaultSourceDir => SettingsFieldValue::Directory(&settings.default_source_dir),
            SettingsFieldId::DefaultBuildOutputDir => SettingsFieldValue::Directory(&settings.default_build_output_dir),
            SettingsFieldId::DefaultDeviceInstallDir => SettingsFieldValue::Directory(&settings.default_device_install_dir),
        }
    }

    pub(crate) fn is_empty(&self, settings: &HubSettings) -> bool {
        match self.value(settings) {
            SettingsFieldValue::Executable(text) => text.trim().is_empty(),
            SettingsFieldValue::Directory(path) => path.as_os_str().is_empty(),
        }
    }

    pub(crate) fn validate_for_save(&self, settings: &HubSettings) -> Result<(), HubError> {
        if self.required && self.is_empty(settings) {
            return Err(HubError::message(format!("{} is required", self.save_label)));
        }
        Ok(())
    }
}

/// save-settings 的统一必填校验入口；与 health 的"空值 → error 行"判定同源
/// （二者都经 is_empty / SETTINGS_FIELD_SPECS）。
pub(crate) fn validate_settings_for_save(settings: &HubSettings) -> Result<(), HubError> {
    for spec in &SETTINGS_FIELD_SPECS {
        spec.validate_for_save(settings)?;
    }
    Ok(())
}
```

设计决策（无需再决策）：
- "health 权重"实现为均匀权重——`completion` 仍由 `ready_count * 100 / rows.len()` 推导（现 `settings_dto.rs:366-370` 逻辑原样保留），spec 表不引入 weight 字段：当前无差异化权重消费场景，引入即硬编码常量，与目标 2"无任何硬编码常量"冲突。
- "校验器"列即 `kind`：`Executable`/`Directory` 分别绑定既有 `executable_row`（397-449 行）与 `directory_row`（504-542 行），两函数本体不动。
- `build_profile`/`jobs`/`language` 不进 spec 表：它们是枚举/数值类型，非法值在 payload 反序列化与 `apply_to_draft`（`from_ui_value` 失败即错）层已拒绝，health 也从不展示它们。

（b）【已由并行工作树落地，规格供核对】`settings_dto.rs::settings_health`：删除手写 7 行 `vec![...]`，改为遍历 spec 表（实仓已落 `settings_field_row` 形态，412 行起）；`ready_count`/`completion`/`has_error`/label/detail/tone 推导逻辑原样保留：

（b 续）形状 b 的代码骨架（实仓已等价落地，函数名以实仓 `settings_field_row` 为准）：

```rust
// settings_dto.rs —— settings_health 头部改造 + spec 行构造
fn settings_health(settings: &HubSettings) -> HubSettingsHealthSummary {
    let text = HubTextBundle::new(settings.language);
    let rows: Vec<HubSettingsHealthRow> = SETTINGS_FIELD_SPECS
        .iter()
        .map(|spec| spec_health_row(spec, settings))
        .collect();
    // ready_count / completion / has_error / label / detail / tone 推导原样
    ...
}

fn spec_health_row(spec: &SettingsFieldSpec, settings: &HubSettings) -> HubSettingsHealthRow {
    let text = HubTextBundle::new(settings.language);
    let title = text.pair(spec.title_en, spec.title_zh);
    match spec.value(settings) {
        SettingsFieldValue::Executable(value) => {
            executable_row(spec.health_row_id, title, value, settings.language, spec.selected)
        }
        SettingsFieldValue::Directory(path) => {
            directory_row(spec.health_row_id, title, path, settings.language, spec.selected)
        }
    }
}
```

（c）硬切换删除 `apply_to`：`settings_dto.rs::HubSettingsPayload::apply_to`（终核快照 210-246 行）与其专属 helper `trimmed_required`/`path_from_required`（669-684 行）整体删除；终核快照下 `apply_to` 仅剩 2 个调用点（`runtime_state.rs:451`、`settings_dto.rs:770` 测试——`settings_actions.rs` 的 browse/update 链路已被并行工作树切到 `apply_to_draft`），全部同变更切到 `apply_to_draft`（保留为唯一 payload 应用路径；其 `BuildProfile`/`HubLanguage` 非法值错误消息与 `apply_to` 逐字相同，`save_settings_validation_errors_return_localized_view_model` 的 `"未知 Hub 语言：Klingon"` 断言不变）。必填校验后移到 save 链路的 `validate_settings_for_save`，错误原文不变。

（d）`runtime_state.rs::save_settings`（终核快照 437 行起；`apply_to` 调用在 451 行）终态（M1 版；M3 再叠加引擎深校验门控）。行为变化点：现状无 payload 分支不校验直接落盘，改后空必填字段被拒；persist 走 02 已落地的单点 `self.persist(None)`（现状即此形状）：

```rust
fn save_settings(
    &mut self,
    settings_payload: Option<HubSettingsPayload>,
) -> Result<(), HubError> {
    let mut settings = match &settings_payload {
        Some(_) => self.config.settings.clone(), // payload 分支基于已存设置，语义与现状一致
        None => self.settings_draft.clone(),
    };
    if let Some(settings_payload) = settings_payload {
        if let Err(error) = settings_payload.apply_to_draft(&mut settings) {
            self.record_settings_save_failure(error.to_string());
            return Ok(());
        }
    }
    if let Err(error) = validate_settings_for_save(&settings) {
        self.record_settings_save_failure(error.to_string());
        return Ok(()); // draft 保留用户输入待修正，不回滚
    }
    self.config.settings = settings;
    self.register_source_engine_from_settings();
    self.refresh_source_scoped_views()?;
    self.persist(None)?;
    self.settings_draft = self.config.settings.clone();
    self.task_status = TaskStatus::success(
        "Settings saved",
        self.config_path.to_string_lossy().into_owned(),
    )
    .with_operation(TaskOperationKind::Settings, "Hub settings");
    Ok(())
}
```

（`runtime_state.rs:44` 的 `use super::view_model::{...}` 补 `validate_settings_for_save`，经 `view_model.rs` 再导出。）

#### 文件变更清单

| 路径 | 动作 | 变更内容一句话 |
|------|------|----------------|
| `zircon_hub/src/tauri_app/view_model/settings_dto.rs` | 修改（部分已落地） | spec 表与遍历式 `settings_health` 已落地；剩余：删 `apply_to`/`trimmed_required`/`path_from_required`、770 行测试切 `apply_to_draft`、补 `validate_settings_for_save`（若实仓未落）与 spec 同步单测 |
| `zircon_hub/src/tauri_app/view_model.rs` | 修改（复核） | 若 `validate_settings_for_save` 落在 settings_dto，则仅补 `pub(crate) use` 导出供 `runtime_state` 调用；以实仓挂接为准 |
| `zircon_hub/src/tauri_app/runtime_state.rs` | 修改 | `save_settings` 按目标代码形状 d 重写；451 行 `apply_to` 调用消失 |
| `zircon_hub/src/tauri_app/runtime_state/settings_actions.rs` | 修改 | 新增 save 必填拒绝单测（browse/update 链路已是 `apply_to_draft`，无需再改） |
| `zircon_hub/tests/project_workflow_contract.rs` | 修改 | settings_dto 块与 settings_actions 块只增新 snippet（见契约联动） |

#### 实施步骤

1. 复核 spec 表落地形态（`rg "SETTINGS_FIELD_SPECS|validate_settings_for_save|settings_field_row" zircon_hub/src`）：若 `validate_settings_for_save` 未落，按目标代码形状 a 末段补在 spec 表同文件，并补单测 `field_spec_table_covers_all_health_rows_and_required_labels`（断言 7 条 spec 的 `health_row_id` 序列 == `["python-path","cargo-path","rustup-path","project-dir","source-dir","build-output","device-install"]`；逐条把对应字段置空后 `validate_for_save` 报 `"{save_label} is required"`）。验证：`cargo check -p zircon_hub --locked`、`cargo test -p zircon_hub --lib settings_dto --locked`（注意 `zircon_hub/Cargo.toml` `[lib] test = false`：src 内单测必须显式 `--lib`）。
2. 补 health 同源锚定单测 `settings_health_rows_follow_field_spec_table`（默认 settings 下 rows 的 id/title 序列与 spec 表一致；全部字段指向真实存在的临时目录/PATH 命中值时 `completion == 100`；既有 `settings_health_includes_rustup_path_status`、`settings_health_checks_path_command_availability` 应原样通过）。验证：`cargo test -p zircon_hub --lib settings_dto --locked`。
3. 硬切换删 `apply_to`（目标代码形状 c）：改 `runtime_state.rs:451` 与 `settings_dto.rs:770`（测试）两个调用点后整体删除 `apply_to`/`trimmed_required`/`path_from_required`；`save_settings` 重写为目标代码形状 d。验证：`cargo check -p zircon_hub --locked`、`rg "fn apply_to\(|trimmed_required|path_from_required" zircon_hub/src` 零命中、`cargo test -p zircon_hub --lib --locked`。
4. 补 save 必填拒绝测试（`settings_actions.rs` 测试区，沿用 `temp_test_dir` fixture 205-457 行风格）：`save_settings_rejects_empty_required_draft_field_without_persisting`——中文 config 下先发 `update-settings-draft` 把 `pythonPath` 置 `""`（既有 `update_settings_draft_recomputes_health_without_persisting` 已锚定该步），再发无 payload 的 `save-settings`；断言 `model.task_summary.label == "保存设置失败"`、`model.task_summary.detail == "需要 Python 可执行文件"`（`localized.rs:447` 既有词条）、`HubConfig::load(&config_path)` 的 `python_path` 未变、`model.settings_draft.python_path == ""`（draft 保留待修正）。验证：`cargo test -p zircon_hub --lib save_settings --locked`。
5. 契约刷新（见契约联动）+ 里程碑回归：`cargo test -p zircon_hub --test project_workflow_contract --locked`、`cargo test -p zircon_hub settings --locked`、`cargo test -p zircon_hub --lib --locked`、`cargo fmt --all --check`。

#### 契约联动

| 文件（位置） | 现有断言原文 | 处置 |
|------|--------------|------|
| `project_workflow_contract.rs`（settings_dto 块，375 行 `"view_model/settings_dto.rs"` 起） | `"fn executable_row("`、`"path_command_exists(trimmed)"`、`"fn directory_row("`、`"\"python-path\""` 等 | 全部保留（行构造函数与行 id 不变）；只增：`"SETTINGS_FIELD_SPECS"`、`"fn settings_field_row("`（实仓行构造函数名）、`"validate_settings_for_save"` |
| `project_workflow_contract.rs`（settings_actions 块，346 行 `"runtime_state/settings_actions.rs"` 起） | `"pub(super) fn save_settings_from_action("`、`"self.save_settings(settings_payload)"`、`"update_settings_draft_recomputes_health_without_persisting"`、`"save_settings_validation_errors_return_localized_view_model"` | 全部保留；只增：`"save_settings_rejects_empty_required_draft_field_without_persisting"` |
| `settings_actions.rs` 既有单测 | `settings_draft_folder_changes_wait_for_save_settings`（245-295 行）、`update_settings_draft_recomputes_health_without_persisting`（298-343 行）、`save_settings_validation_errors_return_localized_view_model`（404-445 行） | 全部应原样通过（默认 settings 七字段非空，必填校验不触发；非法 language 错误消息原文不变） |
| `runtime_state.rs` 既有单测 | `save_settings_action_applies_typed_payload_and_refreshes_source_engine`（882 行起）、`save_settings_refreshes_source_scoped_catalogs_in_returned_view_model`（956 行起） | M1 应原样通过（payload 全字段非空）；M3 需升级 fixture（见 M3） |

新增测试：`field_spec_table_covers_all_health_rows_and_required_labels`、`settings_health_rows_follow_field_spec_table`、`save_settings_rejects_empty_required_draft_field_without_persisting`（断言要点见实施步骤 1/2/4）。

测试阶段：
- 新增测试：每个字段的非法值 → save 拒绝且 health 对应项标红（spec 单测逐字段覆盖 + save 链路抽 python 一例）；全部合法 → health 100%（`settings_health_rows_follow_field_spec_table` 内以全部存在的临时目录/PATH 命中值断言 `completion == 100`）。
- `cargo test -p zircon_hub settings --locked` + `cargo test -p zircon_hub --lib --locked`（`[lib] test = false`，src 单测必须显式 `--lib`）。

### M2 draft 闭环动作与 picker 语义

切片：
1. 新增 `discard-settings-draft` / `restore-default-settings` 两个 action（Rust + `HUB_ACTION` + SettingsPage 按钮），语义只改 draft。
2. 【2026-06-12 修正】`folder_picker.rs` 返回类型已是 `Result<Option<PathBuf>, HubError>`，本切片改为：settings browse 链路切到 session 级 `folder_picker` 注入缝（03 计划 M1 落点），取消零副作用补测试锚定。
3. 前端 SettingsPage 增加"放弃修改 / 恢复默认"入口（组件落点与 05 计划的 SettingsSection 拆分对齐）。

【2026-06-12 落地状态终核】三个切片的代码面已由并行工作树基本落地：typed id 31（`action_id.rs:88-89`）、`parse_as` 两臂（`action_request.rs:260-261`）、路由两臂（`runtime_state.rs:194-195`）、`settings_actions.rs::discard_settings_draft`/`restore_default_settings`（108-126 行，raw 字符串 + view-model 期本地化）及单测（`discard_settings_draft_restores_saved_settings_without_persisting`、`restore_default_settings_updates_draft_without_persisting`）、DTO 按钮词条（`settings_dto.rs:63-64、331-332`）、localized 词条（实落中文为 `"已放弃设置修改"`，118-119 行）、前端 `HUB_ACTION` 两 id（`hub.ts:632-633`）与 SettingsPage 两按钮（112-117 行）、`project_workflow_contract.rs:107` 已改 31、browse 已走 `(self.folder_picker)`（155 行）且已有取消注入测试（`browse_settings_folder_cancel_keeps_existing_draft`，438 行起）。实落文案/测试名与下文规格有措辞差异（如 detail `"Draft restored to built-in defaults"`）——以实仓为准，不回改。实施者 `rg` 盘点后的可见剩余项：picker 失败注入测试（`browse_settings_folder_picker_error_sets_recoverable_status` 口径）、parse 无 payload 单测、取消测试是否断言 history/`hub.toml` 零副作用（缺则补强）、`docs/zircon_hub/pages/settings-status.md` 增补、`npm run typecheck && npm run build` 回归。下文四块内容继续作为终态规格与验收依据。

#### 目标代码形状

（a）typed id——`action_id.rs`：枚举在 `SaveSettings` 与 `BrowseSettingsFolder` 之间插入两个变体；`ALL` 同位序插入、长度 29 → 31；`as_str` 同位序插入两臂。【2026-06-12 终核】该子项已由并行工作树落仓（枚举 31 变体、`as_str` 88-89 行、round-trip 测试断言 31、前端两 id 亦已落），实施时以 `rg "DiscardSettingsDraft"` 复核即可。两表 id 集合由 `ui_input_navigation_api_contract.rs:70-86` 的 `hub_action_id_table_matches_react_hub_action_map_bidirectionally` 守卫——`quoted_values_between`（52 行）返回 `BTreeSet<String>`，集合比较、对插入位序不敏感，断言原文 `assert_eq!(rust_ids, web_ids, "HubActionId::as_str() table and web HUB_ACTION map must expose identical id sets")`：

```rust
// action_id.rs —— 枚举与 ALL 在 SaveSettings 与 BrowseSettingsFolder 之间插入
DiscardSettingsDraft,
RestoreDefaultSettings,

// as_str —— Self::SaveSettings 臂之后插入（现 88-89 行已落）
Self::DiscardSettingsDraft => "discard-settings-draft",
Self::RestoreDefaultSettings => "restore-default-settings",
```

（b）action 解析——`action_request.rs`：`HubAction` 枚举在 `BrowseSettingsFolder` 变体（64 行）旁加两个无字段变体；`parse_as` 在 `HubActionId::BrowseSettingsFolder` 臂（262 行）旁加两臂（无 target、无 payload，多余 payload 直接忽略，与 `ViewAllProjects`/`NewProject` 同型）。【2026-06-12 终核】并行工作树已开始落该面（`rg "DiscardSettingsDraft" action_request.rs` 已有命中），实施前复核补缺：

```rust
// HubAction 枚举追加
DiscardSettingsDraft,
RestoreDefaultSettings,

// parse_as 追加
HubActionId::DiscardSettingsDraft => Ok(HubAction::DiscardSettingsDraft),
HubActionId::RestoreDefaultSettings => Ok(HubAction::RestoreDefaultSettings),
```

（c）路由与动作实现——`runtime_state.rs::apply_action` 的 `HubAction::BrowseSettingsFolder` 臂旁加（【2026-06-12 终核】路由臂亦在并行落仓中，复核补缺）：

```rust
HubAction::DiscardSettingsDraft => self.discard_settings_draft(),
HubAction::RestoreDefaultSettings => self.restore_default_settings(),
```

`settings_actions.rs`（`record_settings_save_failure` 之前）落两个方法——只改 draft、不 persist、不写 history（`HubSettings` 已在 5 行 use）：

```rust
pub(super) fn discard_settings_draft(&mut self) {
    self.settings_draft = self.config.settings.clone();
    let text = HubTextBundle::new(self.settings_draft.language);
    self.task_status = TaskStatus::success(
        text.status_label("Settings draft discarded"),
        text.status_detail("Draft restored to saved settings"),
    )
    .with_operation(
        TaskOperationKind::Settings,
        text.pair("Hub settings", "Hub 设置"),
    );
}

pub(super) fn restore_default_settings(&mut self) {
    self.settings_draft = HubSettings::default();
    let text = HubTextBundle::new(self.settings_draft.language);
    self.task_status = TaskStatus::success(
        text.status_label("Default settings restored"),
        text.status_detail("Draft reset to built-in defaults; save to apply"),
    )
    .with_operation(
        TaskOperationKind::Settings,
        text.pair("Hub settings", "Hub 设置"),
    );
}
```

设计决策（无需再决策）：`restore-default-settings` 重置**全部**字段含 `language`（`HubSettings::default()` 的 language 为 Chinese，`hub_config.rs:218-240`）；状态文案语言取恢复**后**的 draft 语言（与 settings 域其余动作"draft 语言驱动文案"一致，`settings_actions.rs:135` 同口径）。两动作不调用任何 persist 变体——契约"仅改 draft 不落盘"。

（d）本地化——`localized.rs` 只增四条：`status_label` 表（92-121 行区段）加 `"Settings draft discarded" => "已放弃设置草稿修改"`、`"Default settings restored" => "已恢复默认设置"`；`status_detail` 常量表（440-441 行 `"No folder was selected"`/`"Choose a folder or keep the current setting"` 同组）加 `"Draft restored to saved settings" => "草稿已恢复为已保存设置"`、`"Draft reset to built-in defaults; save to apply" => "草稿已重置为内置默认值，保存后生效"`。

（e）picker 注入缝迁移——【2026-06-12 终核：已由并行工作树落地，规格供核对】`settings_actions.rs::browse_settings_folder` 的 picker 调用已是 `(self.folder_picker)(&FolderPickerRequest::new(...))`（终核快照 155 行），use 行仅剩 `FolderPickerRequest`。注入缝本体由 03 计划落仓：`HubRuntimeSession.folder_picker` 字段（`runtime_state.rs:65`，`fn(&FolderPickerRequest) -> Result<Option<PathBuf>, HubError>`）、`load_from_paths` 初始化为 `crate::process::pick_folder`（118 行）、import 链路同走缝（`project_actions.rs:131`）。取消/成功/失败三分支行为与文案原样保留（取消 warning 摘要为契约面）；剩余工作 = 失败注入测试与取消零副作用断言面核对（见实施步骤 3）。

（f）前端——`web/src/types/hub.ts` 的 `HUB_ACTION`（615-645 行）在 `saveSettings` 之后、`browseSettingsFolder` 之前插入（与 Rust `as_str` 同位序；parity 契约为集合比较，位序仅为可读性）：

```ts
discardSettingsDraft: "discard-settings-draft",
restoreDefaultSettings: "restore-default-settings",
```

（两动作无 payload，`HubActionPayloadById`（708 行起）不加条目。）

`settings_dto.rs::HubSettingsText`（57-77 行）`save_button` 后加 `pub discard_button: String,`、`pub restore_defaults_button: String,`；`for_language`（242-311 行）`save_button` 行（253 行）后加：

```rust
discard_button: text.pair("Discard Changes", "放弃修改").to_string(),
restore_defaults_button: text.pair("Restore Defaults", "恢复默认").to_string(),
```

`web/src/types/hub.ts` 的 settings text interface 同步加 `discardButton: string;`、`restoreDefaultsButton: string;`。`SettingsPage.tsx` 头部按钮区（106-113 行）在 Projects 按钮与 Save 按钮之间插入最小接线（版式重排归 05/06）：

```tsx
<HubButton onClick={() => void onAction(HUB_ACTION.restoreDefaultSettings)}>
  {settingsText.restoreDefaultsButton}
</HubButton>
<HubButton onClick={() => void onAction(HUB_ACTION.discardSettingsDraft)}>
  {settingsText.discardButton}
</HubButton>
```

（按钮文案全部来自 Rust DTO，前端零新增硬编码业务文案。）

#### 文件变更清单

| 路径 | 动作 | 变更内容一句话 |
|------|------|----------------|
| `zircon_hub/src/tauri_app/action_id.rs` | 修改（复核） | 两 id 同位序插入、`ALL` 29→31、round-trip 测试改 31——2026-06-12 终核已由并行工作树落仓，实施时 `rg` 复核补缺 |
| `zircon_hub/src/tauri_app/action_request.rs` | 修改（复核） | `HubAction` 加两个无字段变体；`parse_as` 加两臂；新增解析单测（并行落仓中，复核补缺） |
| `zircon_hub/src/tauri_app/runtime_state.rs` | 修改（复核） | `apply_action` 加两臂（并行落仓中，复核补缺；`folder_picker` 缝已存在，65、118 行） |
| `zircon_hub/src/tauri_app/runtime_state/settings_actions.rs` | 修改（部分已落地） | 两方法、picker 缝迁移、取消注入测试已由并行工作树落地；剩余：失败注入测试、取消零副作用断言面核对、（如缺）parse 无 payload 测试 |
| `zircon_hub/src/tauri_app/view_model/localized.rs` | 修改 | 只增两条 label + 两条 detail 词条及中文断言 |
| `zircon_hub/src/tauri_app/view_model/settings_dto.rs` | 修改 | `HubSettingsText` 加 `discard_button`/`restore_defaults_button` 两字段与词条 |
| `zircon_hub/web/src/types/hub.ts` | 修改 | `HUB_ACTION` 同位序插入两 id；settings text 接口加两个按钮字段 |
| `zircon_hub/web/src/pages/SettingsPage.tsx` | 修改 | 头部按钮区加"恢复默认/放弃修改"两个最小按钮 |
| `zircon_hub/tests/project_workflow_contract.rs` | 修改（部分已落地） | `ALL` 长度断言已由并行工作树改 31（107 行）；settings_actions / parse_as / types / SettingsPage 各块只增新 snippet（缺则补） |
| `docs/zircon_hub/pages/settings-status.md` | 修改 | 增补 draft 闭环两动作与取消语义描述（machine-readable header 与 `hub_docs_contract.rs:171-186` 既有 snippet 保持） |

#### 实施步骤

1. Rust id 与解析（先 `rg "DiscardSettingsDraft|RestoreDefaultSettings" zircon_hub/src zircon_hub/tests zircon_hub/web/src` 盘点并行工作树已落子项，仅补缺）：`action_id.rs`（目标代码形状 a，终核已落）+ `action_request.rs`（形状 b）+ `runtime_state.rs::apply_action` 两臂（形状 c 前半，终核已落于 194-195 行）；若缺则在 `action_request.rs` 测试区加 `parses_settings_draft_lifecycle_actions_without_payload`（两 id 各 parse 一次，断言落到对应无字段变体；payload 传 `Some(json!({}))` 亦成功）。`project_workflow_contract.rs:107` 的 `[HubActionId; 31]` 与 parse 臂 snippet 组（143 行 `"HubActionId::SaveSettings => Ok(HubAction::SaveSettings"` 同组）复核只增 `"HubActionId::DiscardSettingsDraft => Ok(HubAction::DiscardSettingsDraft)"`、`"HubActionId::RestoreDefaultSettings => Ok(HubAction::RestoreDefaultSettings)"`。验证：`cargo test -p zircon_hub --lib action_id --locked`、`cargo test -p zircon_hub --lib action_request --locked`、`cargo test -p zircon_hub --test project_workflow_contract --locked`。
2. 动作实现与本地化（终核已落，本步为验收 + 差异核对）：`settings_actions.rs` 两方法（形状 c 后半；实落为 raw 字符串 + view-model 期本地化，语义等价）与单测（实落名 `discard_settings_draft_restores_saved_settings_without_persisting`、`restore_default_settings_updates_draft_without_persisting`）、`localized.rs` 词条（实落中文 `"已放弃设置修改"` 等，以实仓为准）。核对要点：两方法不得调用任何 persist 变体；restore 后 `settings_draft == HubSettings::default()`；若实落单测未断言磁盘 `hub.toml` 不变则补强。验证：`cargo test -p zircon_hub --lib settings_actions --locked`、`cargo test -p zircon_hub --lib localized --locked`。
3. picker 注入缝（迁移本体已落地，本步为测试补强）：复核 `(self.folder_picker)` 调用（终核快照 `settings_actions.rs:155`）与既有取消注入测试 `browse_settings_folder_cancel_keeps_existing_draft`（438 行起）的断言面——若未断言 `config.action_history` 为空与 `HubConfig::load(&config_path)` 不变，按 `browse_settings_folder_cancel_keeps_draft_and_history_silent` 口径补强；新增 `browse_settings_folder_picker_error_sets_recoverable_status`（`session.folder_picker = |_| Err(HubError::message("picker boom"));` → 断言 `task_status.label == "Browse folder failed"`、detail 含 `picker boom`、draft 不变、history 为空）。同步 `project_workflow_contract.rs` settings_actions 块只增 `"(self.folder_picker)("`（既有 `"FolderPickerRequest::new("` 保留）。验证：`cargo test -p zircon_hub --lib browse_settings_folder --locked`、`cargo test -p zircon_hub --test project_workflow_contract --locked`。
4. 前端接线（终核已落，本步为验收 + 契约补增）：复核 `hub.ts:632-633` 两 id、settings text 接口字段与 `SettingsPage.tsx:112-117` 两按钮、`settings_dto.rs:63-64、331-332` DTO 词条；`project_workflow_contract.rs` types 块（680 行 `"types/hub.ts"` 起）复核只增 `"discardSettingsDraft: \"discard-settings-draft\""`、`"restoreDefaultSettings: \"restore-default-settings\""`，SettingsPage 块（820 行 `"SettingsPage.tsx"` 起）复核只增 `"void onAction(HUB_ACTION.discardSettingsDraft)"`、`"void onAction(HUB_ACTION.restoreDefaultSettings)"`（缺则补）。验证（前端命令在 `zircon_hub/` 下执行，package.json 位于 `zircon_hub/package.json`）：`npm run typecheck`、`npm run build`；`cargo test -p zircon_hub --test ui_input_navigation_api_contract --test project_workflow_contract --locked`。
5. 文档与回归：`docs/zircon_hub/pages/settings-status.md` 增补两动作描述；`cargo test -p zircon_hub --test hub_docs_contract --locked`、`cargo test -p zircon_hub --lib --locked`、`cargo test -p zircon_hub --locked`、`cargo fmt --all --check`。

#### 契约联动

| 文件（位置） | 现有断言原文 | 处置 |
|------|--------------|------|
| `project_workflow_contract.rs:107` | `"pub(crate) const ALL: [HubActionId; 31]"`（终核快照；细化期间为 29，已由并行工作树刷新） | 复核为 31 即可 |
| `action_id.rs:132`（单测） | `assert_eq!(HubActionId::ALL.len(), 31);` | 已由并行工作树改 31，复核即可 |
| `ui_input_navigation_api_contract.rs:70-86` | `assert_eq!(rust_ids, web_ids, "HubActionId::as_str() table and web HUB_ACTION map must expose identical id sets")`（`quoted_values_between` 返回 `BTreeSet<String>`，集合比较） | 断言本体不动；前端 `HUB_ACTION` 补两 id 后即恢复绿灯 |
| `project_workflow_contract.rs`（settings_actions 块，346 行 `"runtime_state/settings_actions.rs"` 起） | `"FolderPickerRequest::new("`、`"text.status_label(\"Folder selection cancelled\")"`、`"text.status_label(\"Browse folder failed\")"` | 全部保留（取消 warning 摘要为契约面）；只增 `"(self.folder_picker)("`、`"pub(super) fn discard_settings_draft("`、`"pub(super) fn restore_default_settings("` 与两个新测试名 |
| `project_workflow_contract.rs`（types 块 680 行起 / SettingsPage 块 820 行起） | `"browseSettingsFolder: \"browse-settings-folder\""`、`"void onAction(HUB_ACTION.browseSettingsFolder, field, { field, initialDir, settings: draft });"` | 保留；只增两 id 与两个按钮 onAction snippet |
| `hub_docs_contract.rs:171-186` | settings-status 文档需含 `"browse-settings-folder"`、`"save-settings"` 等 | 文档只增内容，断言不动 |

新增测试：`parses_settings_draft_lifecycle_actions_without_payload`、`discard_settings_draft_restores_saved_settings_without_persisting`、`restore_default_settings_resets_draft_to_builtin_defaults_without_persisting`、`browse_settings_folder_cancel_keeps_draft_and_history_silent`、`browse_settings_folder_picker_error_sets_recoverable_status`（断言要点见实施步骤）。

测试阶段：
- Rust：draft 修改 → discard → 与已存设置一致；restore-defaults → 与内置默认一致且未 persist。
- `npm run typecheck && npm run build`（`zircon_hub/` 下）。

### M3 Source Engine 深校验与预检

切片：
1. `engines/validation.rs` 增加 workspace 解析（toml 读 members）与 `tools/zircon_build.py` 存在性校验（【2026-06-12 修正】后者已存在于 45-47 行，本切片实际只新增"可解析 + members 含 `zircon_runtime`"两级）；注册失败 recovery 指明缺失项。
2. build / open-editor prepare 阶段调用同一 engine 健康预检；active engine 失效时错误文案指向 Settings 页修复。【2026-06-12 修正】build 预检已存在（`validate_active_source_engine_for_build`，自动获得深校验）；package 不依赖 staged engine 产物（`pending_project_package_from_project`（`project_delivery_actions.rs:166-194`）只用项目路径 + 输出目录构造 `ProjectPackageRequest`），不加引擎预检。
3. 启动时 `register_source_engine_from_settings` 失败不再静默：落一条 warning 级 TaskStatus。

【2026-06-12 落地状态终核】切片 1 已由并行工作树部分落地：`validation.rs` 已含 `MissingRuntimeWorkspaceMember` 变体与 `workspace_members_include_zircon_runtime`（members 解析），单测 fixture 已写 members——但快照中**未见** `InvalidWorkspaceManifest` 变体（Cargo.toml 解析失败的归类以实仓为准核对：若实落将解析失败并入 `MissingRuntimeWorkspaceMember`，则下文形状 a 的双变体拆分按实仓收敛，localized 词条相应减一条）。save 注册门控、open-editor 预检、启动 warning 在终核快照中 `rg "warn_on_invalid|ensure_active_source_engine_healthy|Source Engine needs attention"` 零命中，仍按本里程碑实施。

#### 目标代码形状

（a）`engines/validation.rs` 终态——变体追加（既有四变体与三条既有检查原文保留，`project_source_engine_contract.rs:180-193` 的 `"path.join(\"Cargo.toml\").is_file()"`、`"path.join(\"tools\").join(\"zircon_build.py\").is_file()"` 等 snippet 只增不改）；检查顺序：目录 → Cargo.toml 存在 → build 脚本存在 → Cargo.toml 可解析 → members 含 `zircon_runtime`：

```rust
// validation.rs —— 枚举追加两变体（MissingBuildTool 之后）
InvalidWorkspaceManifest,
MissingRuntimeWorkspaceMember,

// summary 追加
Self::InvalidWorkspaceManifest => "Source checkout Cargo.toml could not be parsed",
Self::MissingRuntimeWorkspaceMember => {
    "Source checkout workspace members do not include zircon_runtime"
}

// recovery_hint 追加
Self::InvalidWorkspaceManifest => {
    "Fix the workspace Cargo.toml so it parses as TOML, or select a complete ZirconEngine checkout"
}
Self::MissingRuntimeWorkspaceMember => {
    "Select the ZirconEngine repository root whose [workspace] members include zircon_runtime"
}

// validate_source_engine 尾段（既有三查之后、`SourceEngineValidation::Valid` 之前）
let manifest_path = path.join("Cargo.toml");
let Ok(manifest_text) = std::fs::read_to_string(&manifest_path) else {
    return SourceEngineValidation::InvalidWorkspaceManifest;
};
let Ok(manifest) = toml::from_str::<toml::Value>(&manifest_text) else {
    return SourceEngineValidation::InvalidWorkspaceManifest;
};
if !workspace_members_include_zircon_runtime(&manifest) {
    return SourceEngineValidation::MissingRuntimeWorkspaceMember;
}

fn workspace_members_include_zircon_runtime(manifest: &toml::Value) -> bool {
    manifest
        .get("workspace")
        .and_then(|workspace| workspace.get("members"))
        .and_then(|members| members.as_array())
        .is_some_and(|members| {
            members
                .iter()
                .filter_map(|member| member.as_str())
                .any(is_zircon_runtime_member)
        })
}

fn is_zircon_runtime_member(member: &str) -> bool {
    let normalized = member.trim().trim_end_matches('/').replace('\\', "/");
    normalized == "zircon_runtime" || normalized.ends_with("/zircon_runtime")
}
```

（`toml` 已是 zircon_hub 依赖——`hub_config.rs:48、58` 在用，零新增依赖。不解析 glob member（如 `"crates/*"`）：实仓根 `Cargo.toml:2-10` members 为字面列举且含 `"zircon_runtime"`；脏工作区只读文件解析，不跑 `cargo metadata`，与风险章节"解析不得过严"一致。）

（b）save-settings 注册门控——`runtime_state.rs::save_settings`（M1 形状 d 基础上叠加；`register_source_engine_from_settings();` 与 `task_status` 赋值两处改）：

```rust
let engine_validation =
    crate::engines::validate_source_engine(&self.config.settings.default_source_dir);
if engine_validation == crate::engines::SourceEngineValidation::Valid {
    self.register_source_engine_from_settings();
}
self.refresh_source_scoped_views()?;
self.persist(None)?;
self.settings_draft = self.config.settings.clone();
self.task_status = if engine_validation == crate::engines::SourceEngineValidation::Valid {
    TaskStatus::success(
        "Settings saved",
        self.config_path.to_string_lossy().into_owned(),
    )
} else {
    TaskStatus::warning(
        "Settings saved",
        engine_validation.summary(),
        engine_validation.recovery_hint(),
    )
}
.with_operation(TaskOperationKind::Settings, "Hub settings");
```

设计决策（无需再决策）：注册门控**仅在 save 链路**。启动（`load_from_paths` 的 129 行）与 build prepare（`build_actions.rs:79`）对 `register_source_engine_from_settings` 的调用保持无条件——这是对既有 `hub.toml` 引擎列表的修复/同步语义，门控会在磁盘状态暂时退化（如检出目录改名）时静默丢弃用户引擎记录；失效在启动侧以 warning（形状 d）、执行侧以预检 error 呈现。settings 本身照常 persist（用户输入不丢），仅引擎记录不更新。

（c）open-editor 预检——`editor_launch_actions.rs` 新增方法（`ensure_editor_available`（256-265 行）旁），并在 `prepare_project_editor_launch` 的 `ensure_editor_available` 调用（146 行）之前、`prepare_empty_editor_launch` 的 `ensure_editor_available` 调用（173 行）之前各插一次：

```rust
fn ensure_active_source_engine_healthy_for_launch(
    &mut self,
    target: String,
) -> Result<(), HubError> {
    let source_dir = self.config.settings.default_source_dir.clone();
    if source_dir.as_os_str().is_empty() {
        return Ok(()); // 未配置源码引擎时维持现状：由 staged 可执行文件检查驱动错误
    }
    let validation = crate::engines::validate_source_engine(&source_dir);
    if validation == crate::engines::SourceEngineValidation::Valid {
        return Ok(());
    }
    let detail = validation.summary().to_string();
    self.record_editor_launch_failure(
        target,
        detail.clone(),
        Vec::new(),
        validation.recovery_hint(),
    )?;
    Err(HubError::message(detail))
}
```

（recovery 原文即指向 Settings 修复：`MissingRoot` 的 `"Locate an existing ZirconEngine checkout or update Settings > Source Checkout"` 等，`validation.rs:21-34`，无需新文案。空 source dir 跳过是为兼容 `editor_launch_actions.rs` 测试 fixture `session_with_project`（492-511 行）的 `default_source_dir = PathBuf::new()`（500 行）——其"缺 staged 可执行文件"既有断言面不受影响。）

（d）启动 warning——`runtime_state.rs` 新增方法并在 `load_from_paths` 的 `session.register_source_engine_from_settings();`（129 行）之后、`session.apply_visual_task_state_override_from_env();`（137 行）之前调用（视觉覆盖 env 仍可压过该 warning）：

```rust
fn warn_on_invalid_startup_source_engine(&mut self) {
    let source_dir = self.config.settings.default_source_dir.clone();
    if source_dir.as_os_str().is_empty() {
        return;
    }
    let validation = crate::engines::validate_source_engine(&source_dir);
    if validation == crate::engines::SourceEngineValidation::Valid {
        return;
    }
    self.task_status = TaskStatus::warning(
        "Source Engine needs attention",
        validation.summary(),
        validation.recovery_hint(),
    )
    .with_operation(
        TaskOperationKind::SourceEngine,
        source_dir.to_string_lossy().into_owned(),
    );
}
```

（e）本地化——`localized.rs` 只增五条：`status_label` 表加 `"Source Engine needs attention" => "源码引擎需要处理"`（92 行 `"Source Engine invalid"` 同组）；`status_detail` 常量表（280-284 行既有三条 validation summary 同组）加 `"Source checkout Cargo.toml could not be parsed" => "无法解析源码检出的 Cargo.toml"`、`"Source checkout workspace members do not include zircon_runtime" => "源码检出的 workspace members 不含 zircon_runtime"`；（287-293 行既有 recovery 同组）加 `"Fix the workspace Cargo.toml so it parses as TOML, or select a complete ZirconEngine checkout" => "修复 workspace Cargo.toml 使其可解析，或选择完整的 ZirconEngine 检出"`、`"Select the ZirconEngine repository root whose [workspace] members include zircon_runtime" => "选择 [workspace] members 包含 zircon_runtime 的 ZirconEngine 仓库根目录"`。

（f）测试 fixture 升级（深校验变严的机械后果）：
- `build_actions.rs::create_source_engine_root`（521-527 行）的 `fs::write(source.join("Cargo.toml"), "[workspace]\n")`（524 行）改写 `"[workspace]\nmembers = [\"zircon_runtime\"]\n"`。
- `validation.rs` 既有单测 `source_engine_validation_requires_manifest_and_build_tool`（57-87 行，名称保留——`project_source_engine_contract.rs` validation 块 snippet 锁定）：`fs::write(root.join("Cargo.toml"), "[workspace]")`（70 行）后断言序列扩展为 `MissingBuildTool` →（写 build 脚本后）`MissingRuntimeWorkspaceMember` →（改写含 members 的 Cargo.toml 后）`Valid`；另插一步写非法 TOML（如 `"[workspace"`）断言 `InvalidWorkspaceManifest`。
- `runtime_state.rs` 两个 save 测试（`save_settings_action_applies_typed_payload_and_refreshes_source_engine`：882 行起；`save_settings_refreshes_source_scoped_catalogs_in_returned_view_model`：956 行起）的 `source_path` 由裸目录升级为完整引擎根（补写含 members 的 `Cargo.toml` 与 `tools/zircon_build.py`），其注册/`"Settings saved"` 断言（946 行等）全部原样保留。

#### 文件变更清单

| 路径 | 动作 | 变更内容一句话 |
|------|------|----------------|
| `zircon_hub/src/engines/validation.rs` | 修改 | 加 `InvalidWorkspaceManifest`/`MissingRuntimeWorkspaceMember` 两变体、解析与 members 检查、summary/recovery；既有单测扩断言序列 |
| `zircon_hub/src/tauri_app/runtime_state.rs` | 修改 | `save_settings` 加注册门控与 warning 分支；新增 `warn_on_invalid_startup_source_engine` 并接入 `load_from_paths`；两个 save 测试 fixture 升级 |
| `zircon_hub/src/tauri_app/runtime_state/editor_launch_actions.rs` | 修改 | 新增 `ensure_active_source_engine_healthy_for_launch` 并接入两条 prepare 路径；新增预检单测 |
| `zircon_hub/src/tauri_app/runtime_state/build_actions.rs` | 修改 | `create_source_engine_root` 清单补 members；新增 members 缺失预检单测 |
| `zircon_hub/src/tauri_app/view_model/localized.rs` | 修改 | 只增一条 label + 四条 detail/recovery 词条及中文断言 |
| `zircon_hub/tests/project_source_engine_contract.rs` | 修改 | validation 块（180-193 行）只增新变体与新函数 snippet |

#### 实施步骤

1. `validation.rs` 落两变体 + 解析/​members 检查（目标代码形状 a），既有单测扩序列（形状 f 第二条）；`project_source_engine_contract.rs` validation 块只增 `"InvalidWorkspaceManifest"`、`"MissingRuntimeWorkspaceMember"`、`"fn workspace_members_include_zircon_runtime("`。验证：`cargo test -p zircon_hub --lib validation --locked`、`cargo test -p zircon_hub --test project_source_engine_contract --locked`。
2. fixture 升级（形状 f 第一、三条）：`build_actions.rs:524` 清单补 members；`runtime_state.rs` 两个 save 测试的 source_path 建满引擎根。验证：`cargo test -p zircon_hub --lib build_actions --locked`、`cargo test -p zircon_hub --lib save_settings --locked`（此时既有断言应全绿，行为尚未变化）。
3. save-settings 注册门控（形状 b）+ `localized.rs` 五词条（形状 e）。新增单测（`runtime_state.rs` 测试区）：`save_settings_with_invalid_source_engine_persists_settings_with_warning_and_skips_registration`——source dir 用裸临时目录发 save-settings payload；断言 `session.config.settings.default_source_dir` 已更新且 `HubConfig::load(&config_path)` 同步（settings 照常落盘）、`session.config.engines` 为空（未注册）、`task_status.label == "Settings saved"` 且 detail `== "Source checkout is missing Cargo.toml"`、recovery `== "Select the ZirconEngine repository root that contains the workspace Cargo.toml"`（`validation.rs:16、27-29` 原文）。验证：`cargo test -p zircon_hub --lib save_settings --locked`、`cargo test -p zircon_hub --lib localized --locked`。
4. open-editor 预检（形状 c）。新增单测（`editor_launch_actions.rs` 测试区，沿用 `session_with_project`/`create_project_root` fixture）：`editor_launch_preflight_fails_when_active_source_engine_is_invalid`——session 的 `config.settings.default_source_dir` 置为不存在路径后发起 launch prepare；断言 `prepare_background_editor_launch()` 返回 `Ok(None)`、`config.action_history[0]` 为 `OpenEditor`/`Failed`、detail `== "Source checkout directory is missing"`、recovery 含 `Settings`。同步 `project_workflow_contract.rs` 若 editor_launch 块存在则只增 `"fn ensure_active_source_engine_healthy_for_launch("`。验证：`cargo test -p zircon_hub --lib editor_launch --locked`、`cargo test -p zircon_hub --test project_workflow_contract --locked`。
5. 启动 warning（形状 d）。新增单测：`startup_with_invalid_source_engine_surfaces_warning_task_status`——config 写一个存在但缺 `Cargo.toml` 的 source dir，`load_from_paths` 后断言 `task_status.label == "Source Engine needs attention"`、detail `== "Source checkout is missing Cargo.toml"`、recovery 非空；以及 `build_actions.rs` 加 `build_preflight_rejects_workspace_without_zircon_runtime_member`（fixture 写 `"[workspace]\n"` 无 members；断言 prepare 返回 `Ok(None)`、`task_status.label == "Source Engine invalid"`、detail 为新 summary 原文）。验证：`cargo test -p zircon_hub --lib runtime_state --locked`、`cargo test -p zircon_hub --lib build_actions --locked`。
6. 回归收尾：`cargo test -p zircon_hub project_source_engine --locked`、`cargo test -p zircon_hub --locked`、`cargo test -p zircon_hub --lib --locked`、`cargo fmt --all --check`；按 `capture-hub-window-screenshot` skill 跑一次视觉矩阵确认 seeded config 不出现非预期启动 warning（见风险与协调）。

#### 契约联动

| 文件（位置） | 现有断言原文 | 处置 |
|------|--------------|------|
| `project_source_engine_contract.rs`（validation 块 180-193 行） | `"pub enum SourceEngineValidation"`、`"MissingRoot"`、`"MissingWorkspaceManifest"`、`"MissingBuildTool"`、`"path.join(\"Cargo.toml\").is_file()"`、`"path.join(\"tools\").join(\"zircon_build.py\").is_file()"`、`"source_engine_validation_requires_manifest_and_build_tool"` | 全部保留（检查与测试名不变）；只增 `"InvalidWorkspaceManifest"`、`"MissingRuntimeWorkspaceMember"`、`"fn workspace_members_include_zircon_runtime("` |
| `project_page_copy_contract.rs:152` | `"\"Source checkout is missing Cargo.toml\" => \"源码检出缺少 Cargo.toml\""` | 不变；`localized.rs` 新词条为只增 |
| `runtime_state.rs:946` 等 save 测试断言 | `assert_eq!(view_model.task_summary.label, "Settings saved");` 及同测试内的注册断言 | 保留——靠 fixture 升级为完整引擎根维持成功路径（形状 f） |
| `build_actions.rs` 既有单测（350-469 行） | `background_build_prepares_command_without_running_or_recording_history` 等 | 保留——靠 `create_source_engine_root` 清单补 members 维持绿灯 |
| `editor_launch_actions.rs` 既有单测（382-433 行） | `background_editor_launch_prepare_records_missing_executable_failure_without_spawn`（断言 recovery 含 `"editor/runtime"`） | 保留——预检对空 source dir 跳过（形状 c），该 fixture `default_source_dir = PathBuf::new()` 不触发预检 |

新增测试：`save_settings_with_invalid_source_engine_persists_settings_with_warning_and_skips_registration`、`editor_launch_preflight_fails_when_active_source_engine_is_invalid`、`startup_with_invalid_source_engine_surfaces_warning_task_status`、`build_preflight_rejects_workspace_without_zircon_runtime_member`，以及 `validation.rs` 既有测试的 `InvalidWorkspaceManifest`/`MissingRuntimeWorkspaceMember` 序列断言（要点见实施步骤）。

测试阶段：
- 新增测试：缺 build 脚本 / members 不含 zircon_runtime / 目录被移走三类注册与预检失败（分别由 `validation.rs` 序列断言、`build_preflight_rejects_workspace_without_zircon_runtime_member`、`editor_launch_preflight_fails_when_active_source_engine_is_invalid` + `startup_..._warning` 覆盖）。
- `cargo test -p zircon_hub project_source_engine --locked` 回归。

## 风险与协调

- 新增 action 依赖 01 计划的 id 表与契约守卫先行；若 01 未完成，新 id 暂按现有字符串模式加入但登记到 01 的迁移清单。【2026-06-12 更新】01 的 typed id 已在工作树落仓（`src/tauri_app/action_id.rs` 的 `HubActionId` + `ALL` + `as_str`/`from_str`），且同日终核时 `DiscardSettingsDraft`/`RestoreDefaultSettings` 两个新 id 已被并行工作树先行加入（31 变体）——本计划 M2 据此从"新增"调整为"盘点补缺"；两 id 仍须登记进 01 计划的 HubActionId 枚举提案/迁移清单（联动登记，不在本计划内改 01 文档）。
- 【2026-06-12 快照注记】`ui_input_navigation_api_contract.rs` 的 id 表双向 parity（70-86 行）以 `BTreeSet` 集合比较（`quoted_values_between`，52 行），对插入位序不敏感。本文档细化期间观测到中间态红灯（Rust 侧先到 31、前端/`project_workflow_contract.rs:107` 暂为 29）；终核时前端两 id（`hub.ts:632-633`）与 107 行（已改 31）均已补齐——M2 收口时跑全量契约确认归绿。
- workspace 解析不得过严：源引擎可能处于脏工作区（大量未提交改动是常态），只验结构存在性与 TOML 可解析，不验 `cargo metadata` 可运行；glob member 不解析（实仓根 `Cargo.toml` members 为字面列举且含 `"zircon_runtime"`，2-10 行）。
- M2 的前端切片与 05 计划在 SettingsPage 上交叠：先做本计划动作接线（最小按钮），版式重排交给 05/06。
- 【2026-06-12 核实修正】本计划"现状与证据"与目标 3 / M2 切片 2 原文所称"`folder_picker.rs` 返回类型需改造（`Ok(None)` vs `Err` 区分）"失实：`process/folder_picker.rs:20-65` 现状已返回 `Result<Option<PathBuf>, HubError>` 且取消（exit code 2）= `Ok(None)`，settings 取消分支已走 warning、不写 history。已据实修正：返回类型不动，M2 工作量为调用点迁移到 03 计划的 session 级 `folder_picker` 函数指针注入缝（2026-06-12 复核：缝已落仓——`runtime_state.rs:65、118`，import 与 recycle 已迁，settings browse 是最后一个直连 `pick_folder` 的调用点）+ 取消零副作用测试锚定；取消语义定稿为"零持久副作用 + 保留 `"Folder selection cancelled"` warning 摘要"（与 03 计划 import 的 `"Import cancelled"` 同原则；该摘要被 `project_workflow_contract.rs` settings_actions 块锁定，完全静默属契约面收缩，不做）。
- 【2026-06-12 核实修正】"现状与证据"原文"仅验 Cargo.toml 存在、不验 zircon_build.py"与"active engine 无预检"均已部分过时（`validation.rs:45-47` 已查 build 脚本；`build_actions.rs:210-236` 已有 build 预检），M3 范围据实收窄为：可解析 + members 深校验、open-editor 预检、save 注册门控、启动 warning；package 经核实不依赖 staged engine 产物，不加引擎预检。
- 【2026-06-12 设计注记】save-settings 的"注册失败"定稿为：深校验非 Valid 时**跳过 upsert**（已注册引擎记录不动）、settings 照常 persist、`task_status` 为 warning（label 仍 `"Settings saved"`，detail/recovery 取 validation 原文）；启动与 build prepare 的 `register_source_engine_from_settings` 保持无条件（修复语义），避免磁盘状态暂时退化时丢弃用户引擎列表。受影响的既有 save 测试以"fixture 升级为完整引擎根"方式保持断言面不缩。
- 【2026-06-12 设计注记】`restore-default-settings` 重置全部字段含 `language`（内置默认中文）；状态文案取恢复后 draft 语言。
- 02 联动：【2026-06-12 终核更新】02 的 persist 单点已落仓——session 级 `persist(&mut self, last_project_path: Option<&Path>)` + `persist_unchecked`（`runtime_state.rs:461、476`），`save_settings` 现已是 `self.persist(None)?`（451 行）；本计划 M1/M3 对 `save_settings` 的重写直接保持该调用形状，无遗留替换项。
- M3 启动 warning 触及视觉基线：截图矩阵以 seeded `hub.toml` 启动，若其 `default_source_dir` 非完整引擎根，Settings/Projects 首帧将出现 `"Source Engine needs attention"` warning 横幅；M3 收尾跑一遍 `capture-hub-visual-state-matrix.ps1` 确认（`ZIRCON_HUB_VISUAL_TASK_STATE` env 覆盖在 warning 之后应用，可压过；与 03.M2 / 06 计划共用该脚本，协调时间窗）。
- 行号均为 2026-06-12 工作树快照，且本文档细化期间实测到同日内 01/02/03 持续落地造成多处行号漂移（`persist` 单点、`folder_picker`/`recycle_delete` 缝、两个新 action id 均在数小时内出现），并行推进速度高于一般预期——实施每个切片前**必须**先以 `rg` 复核行号、断言原文与"哪些子项已被并行完成"，以届时工作树为准，只补缺、不重做、不回滚他人改动。
