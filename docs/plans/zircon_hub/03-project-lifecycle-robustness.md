---
related_code:
  - zircon_hub/src/tauri_app/runtime_state/project_actions.rs
  - zircon_hub/src/tauri_app/runtime_state.rs
  - zircon_hub/src/tauri_app/view_model/localized.rs
  - zircon_hub/src/projects/create_project.rs
  - zircon_hub/src/projects/validation.rs
  - zircon_hub/src/projects/recycle_bin.rs
  - zircon_hub/src/projects/metadata.rs
  - zircon_hub/src/projects/mod.rs
  - zircon_hub/src/state/hub_snapshot.rs
  - zircon_hub/src/process/folder_picker.rs
  - zircon_hub/tests/project_workflow_contract.rs
  - zircon_hub/tests/project_management_contract.rs
  - zircon_hub/tests/project_path_scope_contract.rs
  - zircon_hub/tests/project_page_copy_contract.rs
  - zircon_hub/tests/tauri_react_shell_contract.rs
  - .codex/skills/zircon-project-skills/capture-hub-window-screenshot/scripts/capture-hub-visual-state-matrix.ps1
plan_sources:
  - docs/plans/zircon_hub/index.md
  - docs/plans/zircon_hub/02-background-task-framework-and-persistence.md
  - docs/zircon_hub/projects/lifecycle-workflows.md
status: in_progress
---

# 03 项目生命周期健壮性（create / import / delete / fixture 剥离）

> 2026-08-01 实仓复核：M1 的路径归一、shared manifest 校验、picker 注入、create 保目录恢复与相关单测，M2 的生产 fixture 判定删除与守卫，M3 的 recycler 注入、平台 recovery 与确定性测试均已落在当前源码。本文保留终态设计与验收口径，状态改为 `in_progress`；在受管 Hub package gate 与截图矩阵通过前，不把三个里程碑记为完成。

- 失败交接（`open / 待受管复验`）：[shared recent-project loader test import drift](03/failure-2026-08-27-shared-recent-project-load-import.md)

## 现状与证据

- create-project：`create_project()` 建目录与清单成功后，后续 remember/记录/persist 任一步失败都没有回滚——磁盘上有项目但 Hub 不可见，且重建同名项目会因"目录非空"失败（`runtime_state/project_actions.rs` create 链路，`create_project_from_payload`：22-90 行；`remember_lifecycle_project(...)?` 与 `persist_with_last_project(Some(&project_root))` 的错误直接向上冒泡）。
- import-project：【2026-06-12 修正】folder picker 实际已区分取消与错误——`process/folder_picker.rs:20-65` 返回 `Result<Option<PathBuf>, HubError>`，对话框取消（exit code 2，56-58 行）映射为 `Ok(None)`；`import_project_from_action` 的取消分支（`project_actions.rs:122-130`）走 warning 摘要、不写 history、不 persist。剩余缺口是：该口径无测试锚定（`pick_folder` 直连真实 PowerShell 对话框，session 无注入缝）；项目结构校验仅查 `zircon-project.toml` 文件存在（`projects/validation.rs:10-19`），损坏的 TOML 也能导入；重复导入依赖 `project_metadata_key` 归一（大小写/分隔符/尾斜杠已覆盖，`metadata.rs:24-34`），但入库路径不做 canonicalize——含 `.`/`..` 组件、符号链接、8.3 短名的同目录路径仍会在 `merge_recent_projects`（`editor_recent_sync.rs:123-150`，按 `project_metadata_key` 去重）下产生重复条目。
- delete：三步流（request → cancel/confirm）已按契约实现，回收站失败时保留选中项目与 pending 状态——口径正确（`confirm_project_delete`，`project_actions.rs:270-304`）；但 `recycle_bin.rs` 仅 Windows（PowerShell `Microsoft.VisualBasic` 调用，`windows_delete_directory`：26-40 行），非 Windows 直接报错（`for_project`：18-22 行）；脚本以 `format!` 拼接转义路径（27 行 `replace('\'', "''")`），依赖单引号转义的正确性。
- 视觉 fixture 混入生产：`state/hub_snapshot.rs:76-98` 把 5 个 demo 项目名 + `C:/ZirconProjects/` 路径前缀硬编码进 `ProjectFilterMode::Existing/Missing` 判定（`is_visual_fixture_project`：86-98 行）——真实用户在 `C:\ZirconProjects\` 下创建同名项目时过滤行为错误，属临时代码。
- pin/unpin、remove-from-hub、搜索/筛选/排序/视图模式持久化已落地（契约覆盖），不在本计划重做。

## 目标

1. create-project 事务化：目录与清单创建成功后的任何记录/持久化失败触发补偿——删除本次新建的项目目录（仅当目录是本次创建且仍只含模板产物时），并以 error TaskStatus + recovery 呈现；或者保留目录但给出"目录已创建，可改用导入"的恢复提示——二选一定稿，倾向后者（不删用户数据更稳妥），并把"目标目录非空"错误的 recovery 指向导入流程。【M1 细化已定稿：保留目录 + 恢复提示指向导入】
2. import-project 校验与幂等：路径规范化统一走 `projects/metadata.rs` 的共享 key（契约要求的 shared filesystem key）；除 `zircon-project.toml` 存在性外校验 manifest 可解析（TOML）；picker 取消静默返回（不写 error history），真实错误才落 error。【2026-06-12 修正：原文"`engine` 字段格式合法"不成立——项目清单并无 `engine` 字段（hub 模板清单字段为 `name`/`format_version`/`default_scene`/`library_version`，`create_project.rs:104-113`；截图脚本另有 `[project]` 表形态），引擎绑定存于 `hub.toml` 的 `project_metadata.*.engine_id`，引擎合法性继续由 `resolve_project_engine_id`（`project_actions.rs:340-363`）把关，不进 manifest 校验】
3. fixture 剥离：`is_visual_fixture_project` 退出生产判定路径——demo 项目改由测试/截图流程注入（feature gate `visual-fixtures` 或测试构造 seeded config），`ProjectFilterMode` 只看真实 `path.exists()`。
4. delete 流程跨平台口径定稿：保持"仅 Windows 回收站"为 v1 行为，但非 Windows 的错误文案与 recovery 明确（"当前平台不支持回收站删除，可手动删除后从 Hub 移除"）；PowerShell 调用改为参数传递（`-Command` + 单引号转义现状保留的话，至少补：路径含 `'` 与换行的注入测试）。

## 非目标

- 不引入 trash 类第三方 crate（依赖克制；v2 再决策跨平台删除）。
- 不做项目模板扩展：仍只启用 `renderable-empty`，其余模板保持"敬请期待"（文案归 07）。
- 不改 recent/metadata 的存储格式（`hub.toml` 结构稳定，迁移成本不值得）。

## 里程碑

### M1 create/import 失败路径定稿

切片：
1. create 链路按"保留目录 + 明确恢复提示"实现补偿口径；"目标目录非空"的 recovery 文案指向 Import Project。
2. import：picker 取消返回 `Ok(无操作)`（不污染 history）；manifest 解析校验 + 路径规范化 key 去重；重复导入时选中既有条目而非新增。
3. 为两条链路补失败注入测试：记录阶段失败（persist 目录只读）、manifest 损坏、重复导入、picker 取消。

#### 目标代码形状

（a）路径归一 helper——落 `projects/metadata.rs`（与 `project_metadata_key`/`project_filesystem_path_key` 同 owner；`project_filesystem_path_key`（36-42 行）只产 key 不产可入库路径，本 helper 产「可写入 `hub.toml` 的展示路径」，二者分工保留）：

```rust
// projects/metadata.rs 追加（project_filesystem_path_key 之后）
use std::path::PathBuf;

/// 导入入库前的路径归一：canonicalize 解掉 `.`/`..`/符号链接/8.3 短名，
/// 失败（路径不存在等）时原样返回；Windows canonicalize 产生的 `\\?\`
/// 扩展前缀剥掉，保证入库路径可读且与既有 hub.toml 条目同形。
pub fn normalize_project_root(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    let resolved = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    strip_windows_extended_length_prefix(resolved)
}

fn strip_windows_extended_length_prefix(path: PathBuf) -> PathBuf {
    let text = path.to_string_lossy();
    if let Some(stripped) = text.strip_prefix(r"\\?\UNC\") {
        return PathBuf::from(format!(r"\\{stripped}"));
    }
    if let Some(stripped) = text.strip_prefix(r"\\?\") {
        return PathBuf::from(stripped.to_string());
    }
    path
}
```

`projects/mod.rs:24-27` 的 metadata 导出组追加 `normalize_project_root`。

（b）manifest 可解析校验——`projects/validation.rs`（现 1-19 行，仅 `is_dir` + `is_file` 两查）追加变体与解析查验；`pub fn validate_project_root(path: impl AsRef<Path>) -> ProjectValidation` 签名与 `path.join("zircon-project.toml").is_file()` 片段保持原文（`project_path_scope_contract.rs:87-98` 契约面，只增不改）：

```rust
// projects/validation.rs 终态
use std::fs;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectValidation {
    Valid,
    MissingRoot,
    MissingManifest,
    InvalidManifest,
}

pub fn validate_project_root(path: impl AsRef<Path>) -> ProjectValidation {
    let path = path.as_ref();
    if !path.is_dir() {
        return ProjectValidation::MissingRoot;
    }
    if !path.join("zircon-project.toml").is_file() {
        return ProjectValidation::MissingManifest;
    }
    if !manifest_is_parsable(&path.join("zircon-project.toml")) {
        return ProjectValidation::InvalidManifest;
    }
    ProjectValidation::Valid
}

/// 只验 TOML 可解析，不验字段集合：仓内同时存在顶层 `name = ...`（hub 模板，
/// create_project.rs:104-113）与 `[project]` 表（截图脚本 seeded 清单）两种形态。
fn manifest_is_parsable(manifest_path: &Path) -> bool {
    fs::read_to_string(manifest_path)
        .ok()
        .is_some_and(|text| toml::from_str::<toml::Value>(&text).is_ok())
}
```

注意：`validate_project_root` 另有两处生产调用方（`editor_launch_actions.rs:128`、`project_delivery_actions.rs:168`，均为 `!= ProjectValidation::Valid` 预检）——变严后损坏清单的项目同样无法 open/package，方向一致，无需改调用方代码（见风险注记）。

（c）picker 注入缝——`runtime_state.rs` 的 `HubRuntimeSession`（48-72 行）增函数指针字段（非 trait 对象、无新依赖；生产默认指向真实实现，测试覆写，不构成双轨）：

```rust
// runtime_state.rs —— HubRuntimeSession 结构体追加字段（pending_delete_project_path 之后）
folder_picker: fn(&crate::process::FolderPickerRequest) -> Result<Option<PathBuf>, HubError>,

// load_from_paths（79-134 行）的 Self { ... } 字面量补：
folder_picker: crate::process::pick_folder,
```

同文件追加共享 filesystem-key 查找（与 `find_recent_project`（690-702 行）并列；后者按 `project_metadata_key`/slug 匹配，不解相对组件）：

```rust
// runtime_state.rs —— find_recent_project 之后追加
fn find_recent_project_by_filesystem_key(&self, path: &Path) -> Option<RecentProject> {
    let key = project_filesystem_path_key(path);
    self.config
        .recent_projects
        .iter()
        .find(|project| project_filesystem_path_key(&project.path) == key)
        .cloned()
}
```

（`runtime_state.rs:28-32` 的 `crate::projects` use 组补 `project_filesystem_path_key`。）

（d）import 链路终态——`project_actions.rs::import_project_from_action`（现 92-178 行）。picker 调用从直连 `pick_folder(...)`（104-107 行）改为经字段 `(self.folder_picker)(...)`；取消分支（122-130 行）原样保留——取消口径定稿为：warning 摘要（`"Import cancelled"`，已本地化，`localized.rs:95`）+ 不写 history + 不 persist；校验通过后插入归一与去重：

```rust
// project_actions.rs —— import_project_from_action 中段改造
if project_root.is_none() {
    let text = HubTextBundle::new(self.config.settings.language);
    project_root = match (self.folder_picker)(&FolderPickerRequest::new(
        import_project_picker_title(text),
        Some(self.config.settings.default_project_dir.clone()),
    )) {
        Ok(path) => path,
        Err(error) => { /* record_lifecycle_failure 分支原样（110-118 行） */ }
    };
}

let Some(project_root) = project_root else { /* 取消分支原样（122-130 行） */ };

if let Some(error) = project_validation_error(&project_root) { /* 原样（132-141 行） */ }

// 新增：归一 + 共享 key 去重；重复导入选中既有条目而非新增
let project_root = normalize_project_root(&project_root);
let (display_name, project_root) =
    match self.find_recent_project_by_filesystem_key(&project_root) {
        Some(existing) => (recent_project_display_name(&existing), existing.path),
        None => (
            project_display_name_from_path(&project_root),
            project_root,
        ),
    };

let engine_id = /* resolve_project_engine_id 分支原样（143-156 行） */;
self.remember_lifecycle_project(display_name.clone(), project_root.clone(), engine_id, None)?;
// 后续 push_lifecycle_record / task_status / persist 原样（164-177 行）；
// 现 157 行的 `let display_name = project_display_name_from_path(&project_root);` 删除（上移合并）。
```

去重语义：`remember_lifecycle_project`（306-338 行）内的 `merge_recent_projects` 按 `project_metadata_key` 去重——复用既有条目的 `existing.path` 后 key 必然一致，重复导入只刷新时间戳并选中详情页，`recent_projects` 不增行。

（e）`project_validation_error`（`project_actions.rs:517-529`）补新分支：

```rust
ProjectValidation::InvalidManifest => Some(format!(
    "zircon-project.toml could not be parsed in {}",
    project_root.to_string_lossy()
)),
```

（f）create 补偿口径——`create_project_from_payload`（22-90 行）。两处改动：

其一，`create_project` 失败分支（54-66 行）的 recovery 按错误细分，"目标目录非空"（`create_project.rs:52` 原文 `"Target directory must be empty"`）指向导入：

```rust
Err(error) => {
    let detail = error.to_string();
    let recovery = if detail == "Target directory must be empty" {
        "If the folder already contains a project, use Import Project; otherwise choose an empty target folder"
    } else {
        "Choose an empty target folder and retry project creation"  // 其余错误维持现口径（localized.rs:355-357 词条不动）
    };
    self.record_lifecycle_failure(
        HubActionKind::CreateProject,
        payload.name.clone(),
        detail,
        recovery,
        None,
    )?;
    return Ok(());
}
```

其二，目录建成之后的记录/持久化失败不再向上冒泡（现 69-74 行 `remember_lifecycle_project(...)?` 与 89 行 `persist_with_last_project(...)` 直接 `?`/尾返回），改为补偿记录——保留目录、error TaskStatus、recovery 指向导入：

```rust
let project_root = report.project_root.clone();
if let Err(error) = self.remember_lifecycle_project(
    payload.name.clone(),
    project_root.clone(),
    engine_id,
    Some(template.id().to_string()),
) {
    return self.record_create_project_kept_folder_failure(payload.name, &project_root, error);
}
self.push_lifecycle_record(/* Success 记录原样（75-82 行） */);
self.task_status = /* 原样（83-87 行） */;
self.new_project_name.clear();
if let Err(error) = self.persist_with_last_project(Some(&project_root)) {
    return self.record_create_project_kept_folder_failure(payload.name, &project_root, error);
}
Ok(())
```

```rust
// project_actions.rs —— record_lifecycle_failure（464-484 行）旁新增
const CREATE_KEPT_FOLDER_RECOVERY: &str =
    "The project folder was kept on disk; use Import Project to add it to Hub";

fn record_create_project_kept_folder_failure(
    &mut self,
    target: String,
    project_root: &Path,
    error: HubError,
) -> Result<(), HubError> {
    let detail = format!(
        "Project folder was created at {} but Hub failed to record it: {error}",
        project_root.to_string_lossy()
    );
    self.push_lifecycle_record(
        HubActionKind::CreateProject,
        HubActionStatus::Failed,
        target.clone(),
        detail.clone(),
        Some(CREATE_KEPT_FOLDER_RECOVERY.to_string()),
        Some(project_root.to_path_buf()),
    );
    self.task_status = TaskStatus::error("Create Project failed", detail, CREATE_KEPT_FOLDER_RECOVERY)
        .with_operation(TaskOperationKind::Project, target);
    let _ = self.persist_hub_config(); // 记录阶段已失败，再失败不冒泡为裸 command 错误；02 M2 落地后改 `let _ = self.persist(None);`
    Ok(())
}
```

不走 `record_lifecycle_failure` 的原因：该函数（464-484 行）尾部 `self.persist_hub_config()` 带 `?` 返回——补偿场景里 persist 大概率再次失败，会把"已记录的失败"升级成 command 层错误，与"UI 收到完整状态 + recovery"的目标矛盾。

（g）本地化词条（`view_model/localized.rs`，全部"只增"）：
- `status_detail` strip_prefix 链（参照 142-146 行 `"Invalid payload for Hub action "` 的 split_once 形态）追加：

```rust
if let Some(body) = detail.strip_prefix("Project folder was created at ") {
    if let Some((path, error)) = body.split_once(" but Hub failed to record it: ") {
        return format!("项目目录已创建于 {path}，但 Hub 记录失败：{error}");
    }
}
if let Some(path) = detail.strip_prefix("zircon-project.toml could not be parsed in ") {
    return format!("无法解析 {path} 中的 zircon-project.toml");
}
```

- `status_detail` 常量表（355-362 行附近，与 `"Choose an empty target folder and retry project creation"` 同组）追加：
  - `"If the folder already contains a project, use Import Project; otherwise choose an empty target folder" => "目标文件夹已有内容：若它已是 Zircon 项目请改用「导入项目」，否则请选择空文件夹"`
  - `"The project folder was kept on disk; use Import Project to add it to Hub" => "项目目录已保留在磁盘上，可通过「导入项目」将其加入 Hub"`

#### 文件变更清单

| 路径 | 动作 | 变更内容一句话 |
|------|------|----------------|
| `zircon_hub/src/projects/metadata.rs` | 修改 | 追加 `normalize_project_root` + `strip_windows_extended_length_prefix` 及单测 |
| `zircon_hub/src/projects/mod.rs` | 修改 | metadata 导出组（24-27 行）补 `normalize_project_root` |
| `zircon_hub/src/projects/validation.rs` | 修改 | `ProjectValidation` 增 `InvalidManifest`；`validate_project_root` 增 TOML 解析查验；新增 `manifest_is_parsable` 与单测 |
| `zircon_hub/src/tauri_app/runtime_state.rs` | 修改 | `HubRuntimeSession` 增 `folder_picker` 函数指针字段并在 `load_from_paths` 初始化；新增 `find_recent_project_by_filesystem_key`；use 组补 `project_filesystem_path_key` |
| `zircon_hub/src/tauri_app/runtime_state/project_actions.rs` | 修改 | create 补偿（`record_create_project_kept_folder_failure` + recovery 细分）；import 接 picker 缝 + 归一 + 去重；`project_validation_error` 增 `InvalidManifest` 分支；新增单测 |
| `zircon_hub/src/tauri_app/view_model/localized.rs` | 修改 | 新增两条 strip_prefix detail 与两条 recovery 常量词条及测试断言 |
| `zircon_hub/tests/project_workflow_contract.rs` | 修改 | project_actions 块（288-307 行）只增新 snippet（见契约联动） |
| `zircon_hub/tests/project_path_scope_contract.rs` | 修改 | validation 块（87-98 行）只增 `"InvalidManifest"` |

#### 实施步骤

1. `metadata.rs` 落 `normalize_project_root`（目标代码形状 a）+ 单测 `normalize_project_root_resolves_dot_components_and_strips_extended_prefix`；`projects/mod.rs` 补导出。验证：`cargo test -p zircon_hub --lib metadata --locked`、`cargo test -p zircon_hub --test project_path_scope_contract --locked`。
2. `validation.rs` 落 `InvalidManifest`（目标代码形状 b）+ 单测；`project_actions.rs::project_validation_error`（517-529 行）补分支（目标代码形状 e）；`localized.rs` 补 manifest 解析词条（目标代码形状 g 第二条）；`project_path_scope_contract.rs` validation 块补 `"InvalidManifest"`。验证：`cargo test -p zircon_hub --lib validation --locked`、`cargo test -p zircon_hub --test project_path_scope_contract --test project_page_copy_contract --locked`。
3. `runtime_state.rs` 增 `folder_picker` 字段与 `find_recent_project_by_filesystem_key`（目标代码形状 c）；`import_project_from_action` 改造（目标代码形状 d）；同步在 `project_workflow_contract.rs` project_actions 块只增新 snippet。新增单测：`import_project_duplicate_path_selects_existing_entry_without_new_row`（seed 一条 recent + 真清单目录，payload path 用 `project.join(".")` 触发归一前 key 差异；断言 `recent_projects.len() == 1` 且 `selected_project_path` 指向既有路径）、`import_project_invalid_manifest_is_recoverable_failure`（清单写半截 TOML `name = "x`；断言 `task_status.label == "Import Project failed"`、detail 含 `could not be parsed`、recent 不变）、`import_project_picker_cancel_keeps_state_without_history`（`session.folder_picker = |_| Ok(None);` 后发无 payload 的 `import-project`；断言 `task_status.label == "Import cancelled"`、`action_history` 为空、recent 为空）、`import_project_picker_error_records_failed_history`（`|_| Err(HubError::message("picker boom"))`；断言 history[0] `Failed`、label `"Import Project failed"`）。验证：`cargo check -p zircon_hub --locked`、`cargo test -p zircon_hub --lib project_actions --locked`、`cargo test -p zircon_hub --test project_workflow_contract --locked`。
4. create 补偿（目标代码形状 f）+ `localized.rs` 其余词条（目标代码形状 g）。新增单测：`create_project_recording_failure_keeps_folder_and_points_recovery_to_import`（建好 session 后把 `session.config_path` 指向「父路径是文件」的非法位置使 persist 必败——02 M2 未落地前的可靠注入，02 落地后该注入仍有效；发 `create-project`；断言项目目录与 `zircon-project.toml` 仍在磁盘、`task_status.label == "Create Project failed"`、`task_status.recovery` 含 `Import Project`、history[0] `Failed` 且 `output_dir == Some(project_root)`）、`create_project_non_empty_target_recovery_points_to_import`（预放非空目标目录；断言 recovery 原文为 `"If the folder already contains a project, use Import Project; otherwise choose an empty target folder"`）；`localized.rs` 测试区补两条中文断言。验证：`cargo test -p zircon_hub --lib project_actions --locked`、`cargo test -p zircon_hub --lib localized --locked`。
5. 回归收尾：`cargo test -p zircon_hub --locked`、`cargo test -p zircon_hub --lib --locked`、`cargo fmt --all --check`（注意 `zircon_hub/Cargo.toml` `[lib] test = false`：默认 `cargo test` 不含 src 单测，必须显式 `--lib`，与 02 计划注记一致）。

#### 契约联动

需同变更刷新/只增的既有断言：

| 文件（位置） | 现有断言原文 | 处置 |
|------|--------------|------|
| `project_workflow_contract.rs`（project_actions 块，288-307 行） | `"pub(super) fn import_project_from_action("`、`"FolderPickerRequest::new("`、`"import_project_picker_title(text)"` 等 | 全部保留；只增：`"(self.folder_picker)("`、`"normalize_project_root("`、`"find_recent_project_by_filesystem_key("`、`"fn record_create_project_kept_folder_failure("` |
| `project_path_scope_contract.rs`（validation 块，87-98 行） | `"pub enum ProjectValidation"`、`"Valid"`、`"MissingRoot"`、`"MissingManifest"`、`"pub fn validate_project_root(path: impl AsRef<Path>) -> ProjectValidation"`、`"path.join(\"zircon-project.toml\").is_file()"` | 全部保留；只增 `"InvalidManifest"` |
| `project_page_copy_contract.rs:111` | `"\"Import cancelled\" => \"已取消导入\""` | 不变（取消口径定稿沿用该文案，理由见风险注记） |
| `project_page_copy_contract.rs:122-123` | `"detail.strip_prefix(\"zircon-project.toml was not found in \")"`、`"return format!(\"未在 {path} 找到 zircon-project.toml\")"` | 不变（缺清单与清单损坏是两条独立消息） |
| `project_actions.rs` 既有单测（554-922 行） | `create_project_action_scaffolds_project_and_selects_detail`、`import_project_action_validates_manifest_and_records_recent_project` 等 10 个 | 全部应原样通过（既有成功/失败路径行为不变） |

新增测试（函数名 + 断言要点，全部见实施步骤 3/4）：归一去重、损坏清单、picker 取消零副作用、picker 错误落 history、create 记录失败保目录、非空目录 recovery 指向导入，外加 `metadata.rs`/`validation.rs`/`localized.rs` 各自的单测与中文词条断言。

测试阶段：
- `cargo test -p zircon_hub project_workflow --locked`、`cargo test -p zircon_hub project_management --locked` + 新增用例。
- 补充（2026-06-12）：src 内新增单测须用 `cargo test -p zircon_hub --lib --locked` 显式选中。

### M2 fixture 剥离

切片：
1. 删除 `hub_snapshot.rs` 的 `is_visual_fixture_project` 生产分支；`Existing/Missing` 仅以 `path.exists()` 判定。
2. 截图/视觉流程需要的 demo 项目改为：契约/截图脚本启动前写入临时 `hub.toml`（seeded config 已是现有截图方案的一部分，对齐 `tauri-react-shell.md` 的 seeded Elysium 用法），必要时配 `#[cfg(test)]` 辅助构造器。
3. 同变更刷新依赖 fixture 行为的契约测试。

#### 目标代码形状

【2026-06-12 核实结论，缩小切片 2 工作量】截图脚本早已是 seeded config 方案且项目目录**真实存在**：`capture-hub-project-pages.ps1:163-191` 在隔离 `$ConfigRoot\C\ZirconProjects` 下逐个 `New-Item` 建目录并写入 `zircon-project.toml`，193-216 行把 `GetFullPath` 后的真实路径写进 seeded `hub.toml`；`capture-hub-visual-state-matrix.ps1:87-124` 同理（`New-VisualProject` + 真实路径入 config）。因此删除 fixture 判定后 `path.exists()` 对 seeded demo 项目恒为真，截图矩阵不会退化为空态——切片 2 无需新建注入机制、无需 `#[cfg(test)]` 构造器，收敛为"跑一遍矩阵脚本回归确认"。

`state/hub_snapshot.rs` 删 86-98 行整个 `is_visual_fixture_project`，`includes`（76-84 行）终态：

```rust
impl ProjectFilterMode {
    fn includes(self, project: &RecentProject) -> bool {
        match self {
            Self::All => true,
            Self::Existing => project.path.exists(),
            Self::Missing => !project.path.exists(),
        }
    }
}
```

同文件 `#[cfg(test)]` 区追加锚定测试（路径取 `C:/ZirconProjects/` 前缀 + 进程唯一后缀，命中旧判定条件但磁盘不存在，锁死"按名单豁免"不回潮）：

```rust
#[test]
fn fixture_named_projects_follow_real_path_existence() {
    let missing_fixture_path = format!(
        "C:/ZirconProjects/ElysiumMissing-{}",
        std::process::id()
    );
    let project = RecentProject::new("Elysium Chronicles", &missing_fixture_path, 10);

    assert!(!ProjectFilterMode::Existing.includes(&project));
    assert!(ProjectFilterMode::Missing.includes(&project));
}
```

（`includes` 为私有方法，测试与其同文件，可直接调用；如保持现有测试风格也可经 `filtered_recent_projects` 断言。）

契约加固：`tauri_react_shell_contract.rs:248-262` 的静态 fixture 禁用循环现在只检查 `commands`/`runtime_state`/`quick_actions` 三个源文件——同变更把 `src/state/hub_snapshot.rs` 读入并加入该循环，且 forbidden 列表追加 `"ZirconProjects"`（demo 项目名与路径前缀都不得再出现于这些生产文件）。

#### 文件变更清单

| 路径 | 动作 | 变更内容一句话 |
|------|------|----------------|
| `zircon_hub/src/state/hub_snapshot.rs` | 修改 | 删 `is_visual_fixture_project`（86-98 行）与 `includes` 中两处 fixture 分支（80-81 行）；增锚定单测 |
| `zircon_hub/tests/tauri_react_shell_contract.rs` | 修改 | 静态 fixture 禁用循环（248-262 行）读入 `hub_snapshot.rs` 并追加 forbidden `"ZirconProjects"` |

#### 实施步骤

1. `hub_snapshot.rs` 删 fixture 判定 + 增 `fixture_named_projects_follow_real_path_existence` 单测。验证：`cargo test -p zircon_hub --lib hub_snapshot --locked`、`cargo test -p zircon_hub --test project_management_contract --locked`（其 `filtered_recent_projects_respects_existing_missing_filter`（333-352 行）用真实临时目录断言，应原样通过）。
2. `tauri_react_shell_contract.rs` 扩 forbidden 检查面（目标代码形状末段）。验证：`cargo test -p zircon_hub --test tauri_react_shell_contract --locked`。
3. 回归与验收：`cargo test -p zircon_hub --locked`、`cargo test -p zircon_hub --lib --locked`；`rg "ZirconProjects" zircon_hub/src` 零命中、`rg "is_visual_fixture" zircon_hub` 零命中；按 `capture-hub-window-screenshot` skill 跑一次 `capture-hub-visual-state-matrix.ps1` 确认 seeded demo 项目仍呈现（含 `project-browser-empty` 空态用例）。

#### 契约联动

- 必须保持不变：`project_management_contract.rs::filtered_recent_projects_respects_existing_missing_filter`（333-352 行，真实目录注入，删除 fixture 后语义不变）；`hub_snapshot.rs` 既有单测 `filtered_recent_projects_applies_path_filter_before_sorting`（171-215 行）。
- 只增：`tauri_react_shell_contract.rs` forbidden 循环新增 `hub_snapshot` 源与 `"ZirconProjects"` 词条；`hub_snapshot.rs` 新增锚定单测。
- 范围注记：`view_model.rs::project_cover_id`（952-962 行）保留 5 个 demo 项目名→封面 id 的映射——它只做封面展示回退（缺名时落 `"elysium"`），不含路径、不影响过滤正确性；其去留归 05/06 计划的前端资产口径，本里程碑不动。

测试阶段：
- `cargo test -p zircon_hub --locked` 全量；视觉截图流程跑通一次确认 demo 项目仍可呈现。
- 验收证据：`rg "ZirconProjects" zircon_hub/src` 生产代码零命中。

### M3 delete 注入与平台口径

切片：
1. `recycle_bin.rs` 增加路径含单引号、空格、中文、换行的转义单测（不真删，抽出脚本构造函数单测其输出）。
2. 非 Windows 错误消息与 recovery 文案定稿（默认中文，归 07 的消息表）；确认失败后 pending 状态保留行为有测试锚定。

#### 目标代码形状

（a）转义单测——脚本构造已经独立成 `RecycleDeleteCommand::windows_delete_directory`（`recycle_bin.rs:26-40`，无 `cfg` 门、全平台可编译），无需再抽函数；直接在同文件 `#[cfg(test)]`（既有 `windows_recycle_command_uses_shell_recycle_bin_api`、`recycle_command_rejects_empty_path` 测试旁）补：

```rust
#[test]
fn windows_recycle_script_escapes_quotes_spaces_unicode_and_newlines() {
    for (raw, expected_fragment) in [
        ("E:/Projects/Designer's Game", "Designer''s Game"),
        ("E:/Projects/My Game", "My Game"),
        ("E:/项目/我的 游戏", "我的 游戏"),
        ("E:/Projects/Line1\nLine2", "Line1\nLine2"),
        ("E:/Projects/It's '; Remove-Item x", "It''s ''; Remove-Item x"),
    ] {
        let command = RecycleDeleteCommand::windows_delete_directory(Path::new(raw));
        let script = &command.args[3];

        assert!(script.contains(expected_fragment), "raw={raw}");
        // 单引号必须成对：路径片段内不得残留能闭合 PS 字符串的孤立引号
        assert_eq!(script.matches('\'').count() % 2, 0, "raw={raw}");
        assert!(script.contains("SendToRecycleBin"));
    }
}
```

（PowerShell 单引号字符串接受内嵌换行，`-Command` 实参经 `std::process` 原样传递；本测试只构造命令、不执行，全平台可跑。）

（b）非 Windows 口径定稿——错误消息维持 `recycle_bin.rs:18-22` 原文 `"Project deletion is only available on Windows in this Hub build"`（`project_management_contract.rs:283-287` 非 Windows 分支断言 `for_project(...).is_err()` 依赖该错误路径），但该消息现在不在 `localized.rs` 任何词表中（中文环境漏翻）；recovery 按平台细分。`confirm_project_delete` 失败分支（`project_actions.rs:293-303`）改为：

```rust
Err(error) => {
    self.pending_delete_project_path = Some(project.path.clone());
    let recovery = if cfg!(target_os = "windows") {
        "The project remains in Hub; fix the filesystem issue or cancel delete" // 现口径保留（localized.rs:376 词条不动）
    } else {
        "Recycle Bin deletion is not supported on this platform; delete the folder manually, then use Remove from Hub"
    };
    self.record_lifecycle_failure(
        HubActionKind::DeleteProject,
        display_name,
        error.to_string(),
        recovery,
        Some(project.path),
    )
}
```

`localized.rs` 只增两条（落点：detail 消息进 `status_detail` 常量表 426 行 `_ => detail` 之前；07 计划 schema 化后整体迁移，本里程碑先确保无英文裸文案）：
- `"Project deletion is only available on Windows in this Hub build" => "当前 Hub 构建仅支持在 Windows 上将项目移入回收站"`
- `"Recycle Bin deletion is not supported on this platform; delete the folder manually, then use Remove from Hub" => "当前平台不支持回收站删除：请手动删除项目文件夹，再使用「从 Hub 移除」"`

（c）回收站注入缝——与 M1 `folder_picker` 同模式，让"确认删除失败保留 pending"在任何平台可确定性单测（现状只能在非 Windows 靠真实错误、Windows 上会真删）：

```rust
// runtime_state.rs —— HubRuntimeSession 结构体追加字段（folder_picker 之后）
recycle_delete: fn(PathBuf) -> Result<(), HubError>,

// load_from_paths 的 Self { ... } 字面量补（recycle_delete_project 为
// `impl Into<PathBuf>` 泛型签名，经非捕获闭包强转 fn 指针）：
recycle_delete: |path| crate::projects::recycle_delete_project(path),
```

`confirm_project_delete`（270-304 行）的 `match recycle_delete_project(project.path.clone())` 改 `match (self.recycle_delete)(project.path.clone())`；`project_actions.rs:1-9` use 组删除 `recycle_delete_project` 导入。

（d）pending 保留与成功路径锚定单测（`project_actions.rs` `#[cfg(test)]`，沿用 `session_with_source`/`temp_test_dir` fixture，924-959 行）：

```rust
#[test]
fn confirm_delete_failure_keeps_pending_state_with_recovery() {
    // seed recent + selected + request-delete 后：
    session.recycle_delete = |_| Err(HubError::message("Recycle Bin deletion failed with status 1"));
    // apply confirm-delete：
    // 断言 pending_delete_project_path 仍 == Some(project)
    // 断言 recent_projects 不变、task_status.label == "Delete Project failed"
    // 断言 history[0].status == Failed 且 recovery.is_some()
}

#[test]
fn confirm_delete_success_with_injected_recycler_drops_project_only_from_hub() {
    session.recycle_delete = |_| Ok(());
    // apply confirm-delete：
    // 断言 recent_projects 为空、pending 为 None、label == "Project deleted"
    // 断言磁盘目录仍存在（证明走了注入而非真删）
}
```

#### 文件变更清单

| 路径 | 动作 | 变更内容一句话 |
|------|------|----------------|
| `zircon_hub/src/projects/recycle_bin.rs` | 修改 | 只增转义单测（单引号/空格/中文/换行/注入串 + 引号配对断言） |
| `zircon_hub/src/tauri_app/runtime_state.rs` | 修改 | `HubRuntimeSession` 增 `recycle_delete` 函数指针字段并初始化 |
| `zircon_hub/src/tauri_app/runtime_state/project_actions.rs` | 修改 | `confirm_project_delete` 经注入缝调用 + recovery 平台分支；删 `recycle_delete_project` use；增两个锚定单测 |
| `zircon_hub/src/tauri_app/view_model/localized.rs` | 修改 | `status_detail` 只增两条非 Windows 删除词条及测试断言 |
| `zircon_hub/tests/project_workflow_contract.rs` | 修改 | project_actions 块只增 `"(self.recycle_delete)("` snippet |

#### 实施步骤

1. `recycle_bin.rs` 补转义单测（目标代码形状 a）。验证：`cargo test -p zircon_hub --lib recycle --locked`（注意 `[lib] test = false`，必须带 `--lib`）。
2. `runtime_state.rs` 增 `recycle_delete` 字段（目标代码形状 c）；`confirm_project_delete` 切到注入缝 + recovery 平台分支（目标代码形状 b）；`project_workflow_contract.rs` project_actions 块只增 snippet。验证：`cargo check -p zircon_hub --locked`、`cargo test -p zircon_hub --test project_workflow_contract --test project_management_contract --locked`。
3. 补两个锚定单测（目标代码形状 d）+ `localized.rs` 两条词条与中文断言。验证：`cargo test -p zircon_hub --lib project_actions --locked`、`cargo test -p zircon_hub --lib localized --locked`。
4. 回归收尾：`cargo test -p zircon_hub --locked`、`cargo test -p zircon_hub --lib --locked`、`cargo fmt --all --check`；Windows 手工验证一次真实回收站删除（建临时项目 → request → confirm → 回收站可见、Hub 列表移除）。

#### 契约联动

- 必须保持不变（消息/命令构造是契约面）：`project_management_contract.rs::recycle_delete_command_targets_windows_recycle_bin_without_deleting`（283-297 行，断言 `command.program == "powershell"`、`args[3].contains("Microsoft.VisualBasic.FileIO.FileSystem")`、`contains("SendToRecycleBin")`、非 Windows 分支 `for_project(...).is_err()`）与 `recycle_delete_command_escapes_single_quotes`（300-308 行，断言 `args[3].contains("Designer''s Game")`）——本里程碑只增不改。
- 必须保持不变：`localized.rs:370-377` 既有 delete 词条（`"Confirm delete to move the project to the Windows Recycle Bin"`、`"Moved project to Windows Recycle Bin"`、`"The project remains in Hub; fix the filesystem issue or cancel delete"`）。
- 只增：`project_workflow_contract.rs` project_actions 块补 `"(self.recycle_delete)("`；`localized.rs` 测试区补 `bundle.status_detail("Project deletion is only available on Windows in this Hub build")` 与新 recovery 的中文断言。
- 新增测试：`recycle_bin.rs::windows_recycle_script_escapes_quotes_spaces_unicode_and_newlines`、`project_actions.rs::confirm_delete_failure_keeps_pending_state_with_recovery`、`project_actions.rs::confirm_delete_success_with_injected_recycler_drops_project_only_from_hub`（断言要点见目标代码形状 a/d）。
- 范围注记：`request_project_delete` 的 warning 文案（`project_actions.rs:236-240`，"Confirm delete to move the project to the Windows Recycle Bin"）在非 Windows 平台语义略有出入，文案统一归 07 消息表，本里程碑不改。

测试阶段：
- `cargo test -p zircon_hub --lib recycle --locked` 及 delete 相关契约（`--test project_management_contract --test project_workflow_contract`）；Windows 手工验证一次真实回收站删除。
- 原文 `cargo test -p zircon_hub recycle --locked` 仅能命中集成契约中名字含 recycle 的测试（`[lib] test = false`），单测过滤须显式 `--lib`（2026-06-12 注记）。

## 风险与协调

- M1 的"保留目录"口径如与既有 `project_workflow_contract.rs` 断言冲突，按契约现状先核对：契约要求失败保留 Hub 状态，与本口径一致。
- M2 触及截图/视觉验证基线（`docs/zircon_hub` 的 design reference matrix）：剥离后首帧 Projects 页在干净环境为空态，截图脚本必须改为 seeded config 启动，避免视觉契约回归。【2026-06-12 核实更新：该风险已大幅收窄——两个截图脚本（`capture-hub-project-pages.ps1:163-216`、`capture-hub-visual-state-matrix.ps1:87-124`）本就以 seeded config + 真实落盘项目目录启动，`path.exists()` 恒真，删除 fixture 判定不会让截图退化为空态；M2 仅需矩阵脚本回归确认，无需改脚本。】
- 依赖 02 的 persist 单点：M1 的"记录阶段失败"测试以 02 的可注入 persist 错误为前提；若 02 未完成，先用只读目录模拟。【2026-06-12 细化更新：失败注入统一改为「`session.config_path` 指向父路径是文件的非法位置」（Windows 只读目录属性不阻止目录内写入，"只读目录"注入不可靠，与 02 计划风险注记同口径）；该注入在 02 落地前后都有效。M1 目标代码形状按现名 `persist_with_last_project`/`persist_hub_config` 书写，02 M2 落地后由其盘点表（含 `project_actions.rs:89、177、202、226、245、267、291、483`）机械替换为 `self.persist(...)`，两计划无冲突。】
- 【2026-06-12 核实修正】原"现状与证据"中"picker 取消与真实错误共用错误通道"不成立：`folder_picker.rs` 早已返回 `Result<Option<PathBuf>, HubError>` 且取消 = `Ok(None)`（exit code 2，56-58 行），import 取消分支也已不写 history。04 计划里程碑文本（其 35、61 行）所列"`folder_picker.rs` 返回类型改造"实为现状已满足——04 细化时与本计划对齐：返回类型不动，工作量在调用点取消语义锚定与（04 范围的）`browse_settings_folder` 接 `folder_picker` 注入缝；本计划 M1 已落 session 级 `folder_picker` 字段，settings 调用点迁移归 04，同一落点不双轨。
- 【2026-06-12 设计注记】"picker 取消静默"定稿为"零持久副作用"：不写 history、不 persist、不改选中态，但保留 `"Import cancelled"` warning 摘要——该文案已本地化且被 `project_page_copy_contract.rs:111` 与 `localized.rs:95、659` 锁定，删除它属契约面收缩，无收益不做。
- 【2026-06-12 核实修正】目标 2 原文"manifest `engine` 字段格式合法"与实仓矛盾：项目清单（hub 写出与截图脚本两种形态）均无 `engine` 字段，引擎绑定在 `hub.toml` `project_metadata.*.engine_id`。已修正为"仅校验 TOML 可解析"；字段级 schema 校验（若将来需要）另行立项。
- 【2026-06-12 设计注记】`validate_project_root` 增 `InvalidManifest` 后整体变严，波及另两处预检调用方（`editor_launch_actions.rs:128` open 预检、`project_delivery_actions.rs:168` package 预检）：损坏清单的项目将无法 open/package——方向正确（早失败优于把损坏项目交给 editor/打包），其错误消息沿用各自既有口径，无新词条。
- 【2026-06-12 设计注记】`folder_picker`/`recycle_delete` 注入缝采用 session 上的函数指针字段（生产默认 = 真实实现，测试覆写），不引入 trait 对象、不加第三方 mock 依赖；这是单一调用路径上的可替换默认值，不构成 §3 禁止的兼容双轨。
- 【2026-06-12 协调注记】01/02 计划在同一未提交工作树持续落地（`action_id.rs`/`HubActionId` 已存在，`commands.rs` 行号会漂移）；本文行号均为 2026-06-12 快照，实施每个切片前先以 `rg` 复核行号与断言原文，以届时工作树为准。M2 的 seeded 截图矩阵与 06 计划 M3 共用 `capture-hub-visual-state-matrix.ps1`，06 细化已注明 "seeded config（03.M2 产物）"——实际上 seeded 机制现成，03.M2 只交付"fixture 退出生产判定"这一前置，06 无需等待额外产物。
- 【2026-07-23 性能交接】`zircon_runtime_interface/src/project/**` 39/39静态复核确认Hub create/recent仍重复TOML→JSON→typed，template复制全部embedded bytes并重复parse/encode。Hub03按PERF-MVP-568只消费Runtime04发布的content fingerprint/generation与last-good summary；stable recent list不重读/重parse，create直接流式写共享template artifact，不建立Hub私有manifest parser/cache。mtime仅作便宜失效提示，损坏/未来版本/恢复语义保持typed。
