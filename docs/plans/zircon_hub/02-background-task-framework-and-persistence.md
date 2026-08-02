---
related_code:
  - zircon_hub/src/tauri_app/commands.rs
  - zircon_hub/src/tauri_app/runtime_state/action_tasks.rs
  - zircon_hub/src/tauri_app/runtime_state/build_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/project_delivery_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/editor_launch_actions.rs
  - zircon_hub/src/state/task_status.rs
  - zircon_hub/src/settings/hub_config.rs
  - zircon_hub/src/projects/package.rs
  - zircon_hub/src/projects/device_install.rs
  - zircon_hub/src/projects/local_paths.rs
  - zircon_hub/src/tauri_app/runtime_state.rs
  - zircon_hub/src/tauri_app/runtime_state/quick_actions.rs
  - zircon_hub/src/tauri_app/view_model.rs
  - zircon_hub/src/tauri_app/view_model/localized.rs
  - zircon_hub/src/state/hub_snapshot.rs
  - zircon_hub/web/src/types/hub.ts
  - zircon_hub/web/src/App.tsx
  - zircon_hub/web/src/data/hubData.ts
  - zircon_hub/tests/project_workflow_contract.rs
  - zircon_hub/tests/project_quick_actions_contract.rs
  - zircon_hub/tests/ui_foundation_contract.rs
  - zircon_hub/tests/tauri_react_shell_contract.rs
  - zircon_hub/tests/project_management_contract.rs
  - zircon_hub/tests/ui_shell_navigation_contract.rs
  - zircon_hub/tests/ui_selected_project_runtime_contract.rs
  - zircon_hub/tests/project_page_copy_contract.rs
plan_sources:
  - docs/plans/zircon_hub/index.md
  - docs/plans/zircon_hub/01-action-dispatch-and-typed-payload.md
  - docs/zircon_hub/ui/tauri-react-shell.md
status: in_progress
---

# 02 后台任务框架与持久化一致性

> 2026-08-01 current-source 状态：M1/M2/M3 主实现已存在。本轮修复了排队任务的 task id 续接和 Windows config “先删旧再 rename” 风险；两个聚焦受管 Cargo 门均为 GREEN。完整 Hub lib/integration 回归及本节列出的 worker FIFO/panic/poison 直接行为门仍未补齐，因此计划保持 `in_progress`。

## 现状与证据

- 四个后台动作已由 `runtime_state/action_tasks.rs::{execute_background_task, dispatch_background_request, run_background_worker_loop}` 统一驱动；`commands.rs` 只负责 Tauri spawn 入口。旧的四份执行函数和每队列项重开线程路径已删除。
- `TaskStatus` 已带单调 `task_id`，snapshot/view-model/frontend 已投影 queue 长度；本轮补齐排队项出队时建立新 running id，以及 prepared/success/error 保持同一 id 的行为门。
- session 落盘已收敛到 `persist`/`persist_unchecked`。`HubConfig::save` 同目录写 tmp；Windows 已存在目标使用 `ReplaceFileW(REPLACEFILE_WRITE_THROUGH)`，失败保留旧 target 并清理 tmp。
- package/install 已通过原子占位与 owned-dir cleanup 约束“只清本次创建的输出”；collision 由 typed delivery message 承载。安装收据 helper 的无用参数与 0 调用 summary loader 已在本轮删除。
- worker 的 poisoned mutex 与 panic 已有恢复/诊断路径；排队取消仍无 action/API/竞争测试，并明确留给 v2。

## 目标

1. 后台执行框架收敛为单一泛型路径：`trait BackgroundTask { type Output; fn run(&self) -> Result<Output, HubError>; }` + 一个 `execute_background_task` 驱动函数，prepare/complete 以闭包或关联函数注入；四份重复函数删除。
2. 任务可观测：`TaskStatus` 增加单调递增 `task_id` 与排队信息（queue 长度 / 当前任务标签），进度档位语义由任务自身上报（至少 start/prepared/done/failed 四态，保留百分比字段兼容前端）。
3. 持久化纪律：`HubConfig` 落盘收敛到单一 `persist` 入口，写文件改为 tmp 文件 + rename 原子替换；persist 失败必须反映到 `TaskStatus`（error + recovery），不得静默。
4. 交付链路清理：package/install 失败时删除本次创建的输出目录（只删本次新建的，不碰既有产物）；`device_install` 用 `fs::create_dir`（非 `create_dir_all` 的存在性双检）原子占位消除 TOCTOU。
5. lock poisoned / 线程 panic 路径有日志与状态恢复：worker 线程 panic 不得让 `background_worker_active` 永久卡 true。

## 非目标

- 不做多 worker 并行（FIFO 单工是有意设计：build 写共享 `CARGO_TARGET_DIR`，并行有害）。
- 不引入细粒度增量事件协议：v1 维持 `hub-state-changed` 全量 ViewModel 推送（契约已锁定），任务可观测性通过 ViewModel 内字段表达。
- 任务取消整体留给 v2 产品决策；当前既没有 running 强杀，也没有 queued cancel action，不把未实现的“移除排队任务”列为 v1 已支持能力。

## 里程碑

### M1 泛型后台执行框架

切片：
1. 在 `runtime_state/action_tasks.rs` 落 `BackgroundTask` trait 与 `execute_background_task(request, session_handle, app, prepare, complete)` 驱动；`PendingEditorRuntimeBuild` / `PendingProjectPackage` / `PendingDeviceInstall` / 编辑器启动 pending 类型实现该 trait。
2. `commands.rs` 四个 `run_background_*_action` 删除，`spawn_background_action` 按 `HubActionId`（01 计划产物）映射到任务构造器；`emit_and_continue` 与队列续接逻辑只保留一份。
3. worker 线程体包 `catch_unwind`：panic 时记录 error TaskStatus、重置 `background_worker_active`、续接队列。
4. 任务可观测（承接目标 2，2026-06-12 补切片）：`TaskStatus` 增单调递增 `task_id`，`HubSnapshot` 增 `queued_background_actions`（队列长度），`HubTaskSummary` DTO 与前端 `HubTaskSummary` 类型同步 `taskId`/`queued` 字段；进度四态沿用现有 `running` + `progress_percent` + `severity` 组合（started=10 / prepared=35 / done=100 / failed=0，常量在 `task_status.rs:13-16`），不新增进度枚举。

#### 目标代码形状

总体改法：现状四份 `run_background_*_action`（`commands.rs:110-324` 附近）共享同一骨架（lock → `apply_request_project_target` → `prepare_background_*` → emit 中间态 → drop lock 跑外部命令 → relock → `complete_background_*` → `emit_and_continue`），差异只有 prepare/complete 的具体类型。改为：框架（trait + 泛型驱动 + 分发 + worker 循环）全部落 `runtime_state/action_tasks.rs`，`commands.rs` 只剩 Tauri command 与 `thread::spawn` 入口。队列续接模型同步简化：现状每个队列项经 `continue_background_queue` → `spawn_background_action` 新开一个线程（`commands.rs:347-358` 附近）；改为单线程 `run_background_worker_loop` 循环消费——FIFO 单工语义不变，且 `catch_unwind` 有唯一包裹点。

（a）`runtime_state/action_tasks.rs` 新增框架（追加在现有 `BackgroundHubAction` 与 session 方法之后；`BackgroundHubAction::from_request`（19-30 行）保留为唯一后台判定表，01 计划已落地——其内部经 `HubActionId::from_str` → `from_action_id` 单点映射，本切片不再触碰）：

```rust
// runtime_state/action_tasks.rs 顶部追加 use
use std::panic::{self, AssertUnwindSafe};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::tauri_app::view_model::HubViewModel;

/// 后台任务统一抽象：run 在不持锁的情况下执行外部命令/拷贝。
pub(in crate::tauri_app) trait BackgroundTask: Send + 'static {
    type Output: Send + 'static;
    fn run(&self) -> Result<Self::Output, HubError>;
}

/// emit 抽象为闭包注入：生产侧由 commands.rs 包 app.emit("hub-state-changed")，
/// 测试侧注入收集闭包，使驱动可以脱离 tauri::AppHandle 单测。
pub(in crate::tauri_app) type EmitState<'a> = &'a dyn Fn(&HubViewModel);

/// poisoned 锁统一恢复点：不再静默 return（现状 commands.rs 11 处
/// `let Ok(mut session) = session_handle.lock() else { return; }`，94-349 行间），
/// 改为 into_inner 恢复 + stderr 日志（仓内无 log crate，见风险章节）。
pub(in crate::tauri_app) fn lock_session(
    session_handle: &Arc<Mutex<HubRuntimeSession>>,
) -> MutexGuard<'_, HubRuntimeSession> {
    session_handle.lock().unwrap_or_else(|poisoned| {
        eprintln!("zircon_hub: Hub runtime session lock was poisoned; recovering last state");
        poisoned.into_inner()
    })
}

/// 单一泛型驱动，替代四份 run_background_*_action。
/// prepare/complete 直接传方法路径（如 HubRuntimeSession::prepare_background_project_package），
/// 它们可强转为所需 fn 指针类型。
pub(in crate::tauri_app) fn execute_background_task<T: BackgroundTask>(
    request: &HubActionRequest,
    session_handle: &Arc<Mutex<HubRuntimeSession>>,
    emit_state: EmitState<'_>,
    prepare: fn(&mut HubRuntimeSession) -> Result<Option<T>, HubError>,
    complete: fn(&mut HubRuntimeSession, T, Result<T::Output, HubError>) -> Result<(), HubError>,
) {
    let pending = {
        let mut session = lock_session(session_handle);
        if let Err(error) = session.apply_request_project_target(request) {
            let _ = session.record_background_action_error(request, error.to_string());
            let view_model = session.view_model();
            drop(session);
            emit_state(&view_model);
            return;
        }
        let pending = match prepare(&mut session) {
            Ok(pending) => pending,
            Err(error) => {
                let _ = session.record_background_action_error(request, error.to_string());
                let view_model = session.view_model();
                drop(session);
                emit_state(&view_model);
                return;
            }
        };
        let view_model = session.view_model();
        drop(session);
        emit_state(&view_model);
        pending
    };

    let Some(pending) = pending else {
        let view_model = lock_session(session_handle).view_model();
        emit_state(&view_model);
        return;
    };

    let result = pending.run();
    let mut session = lock_session(session_handle);
    let view_model = match complete(&mut session, pending, result) {
        Ok(()) => session.view_model(),
        Err(error) => {
            let _ = session.record_background_action_error(request, error.to_string());
            session.view_model()
        }
    };
    drop(session);
    emit_state(&view_model);
}

/// 后台请求分发：BackgroundHubAction 是唯一映射表（不复制第四份表）。
/// None 分支保留现状 spawn_background_action 内的 apply_action 回退（commands.rs:94-107），
/// 防御性兜底，队列里只会出现后台 id。
pub(in crate::tauri_app) fn dispatch_background_request(
    request: &HubActionRequest,
    session_handle: &Arc<Mutex<HubRuntimeSession>>,
    emit_state: EmitState<'_>,
) {
    match BackgroundHubAction::from_request(request) {
        Some(BackgroundHubAction::BuildProject) => execute_background_task(
            request,
            session_handle,
            emit_state,
            HubRuntimeSession::prepare_background_editor_runtime_build,
            HubRuntimeSession::complete_background_editor_runtime_build,
        ),
        Some(BackgroundHubAction::PackageProject) => execute_background_task(
            request,
            session_handle,
            emit_state,
            HubRuntimeSession::prepare_background_project_package,
            HubRuntimeSession::complete_background_project_package,
        ),
        Some(BackgroundHubAction::InstallDevice) => execute_background_task(
            request,
            session_handle,
            emit_state,
            HubRuntimeSession::prepare_background_device_install,
            HubRuntimeSession::complete_background_device_install,
        ),
        Some(BackgroundHubAction::OpenEditor) => execute_background_task(
            request,
            session_handle,
            emit_state,
            HubRuntimeSession::prepare_background_editor_launch,
            HubRuntimeSession::complete_background_editor_launch,
        ),
        None => {
            let mut session = lock_session(session_handle);
            let view_model = match session.apply_action(request.clone()) {
                Ok(view_model) => view_model,
                Err(error) => {
                    let _ = session.record_background_action_error(request, error.to_string());
                    session.view_model()
                }
            };
            drop(session);
            emit_state(&view_model);
        }
    }
}

/// worker 主循环：catch_unwind 唯一包裹点。dispatch 作 fn 指针注入，
/// 生产侧传 dispatch_background_request，测试侧注入可控 panic 的分发函数。
pub(in crate::tauri_app) fn run_background_worker_loop(
    first_request: HubActionRequest,
    session_handle: &Arc<Mutex<HubRuntimeSession>>,
    emit_state: EmitState<'_>,
    dispatch: fn(&HubActionRequest, &Arc<Mutex<HubRuntimeSession>>, EmitState<'_>),
) {
    let mut request = first_request;
    loop {
        let outcome = panic::catch_unwind(AssertUnwindSafe(|| {
            dispatch(&request, session_handle, emit_state);
        }));
        if let Err(payload) = outcome {
            let detail = panic_detail(payload.as_ref());
            eprintln!(
                "zircon_hub: background worker panicked while running {}: {detail}",
                request.action_id
            );
            let mut session = lock_session(session_handle);
            session.record_background_worker_panic(&request, &detail);
            let view_model = session.view_model();
            drop(session);
            emit_state(&view_model);
        }
        // take_next_background_action（现 122-126 行附近）已负责复位
        // background_worker_active：队列空时置 false。panic 路径同样走到这里，
        // 因此 worker 不会把 active 卡死在 true。
        let next_request = lock_session(session_handle).take_next_background_action();
        match next_request {
            Some(next_request) => request = next_request,
            None => return,
        }
    }
}

fn panic_detail(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(text) = payload.downcast_ref::<&str>() {
        (*text).to_string()
    } else if let Some(text) = payload.downcast_ref::<String>() {
        text.clone()
    } else {
        "unknown panic payload".to_string()
    }
}

impl HubRuntimeSession {
    pub(in crate::tauri_app) fn record_background_worker_panic(
        &mut self,
        request: &HubActionRequest,
        detail: &str,
    ) {
        let _ = self.record_background_action_error(
            request,
            HubError::status(
                HubMessage::with_params(
                    HubMessageId::Shell(ShellMessageId::BackgroundTaskPanicked),
                    [detail],
                ),
                Some(HubMessage::new(HubMessageId::Shell(
                    ShellMessageId::ReviewActionTarget,
                ))),
            ),
        );
    }
}
```

（b）四个 pending 类型实现 trait（各 owner 文件内落地，文件归属不变）：

```rust
// runtime_state/build_actions.rs —— PendingEditorRuntimeBuild 现无 run()，
// 外部命令由 commands.rs:147 附近调 run_build_command(pending_build.command())；改为：
use super::action_tasks::BackgroundTask;

impl BackgroundTask for PendingEditorRuntimeBuild {
    type Output = BuildExecutionReport;
    fn run(&self) -> Result<BuildExecutionReport, HubError> {
        run_build_command(self.command())
    }
}
// 同文件 build_selected_project_engine（31-38 行）的
// `let result = run_build_command(pending_build.command());` 改
// `let result = pending_build.run();`，前台/后台共用同一 run 路径。

// runtime_state/project_delivery_actions.rs —— 删除 29-45 行附近两个固有 impl 的 run，
// 原函数体平移进 trait impl（硬切换，不留双轨）：
use super::action_tasks::BackgroundTask;

impl BackgroundTask for PendingProjectPackage {
    type Output = ProjectPackageReport;
    fn run(&self) -> Result<ProjectPackageReport, HubError> {
        package_project(&self.request)
    }
}

impl BackgroundTask for PendingDeviceInstall {
    type Output = (ProjectPackageReport, DeviceInstallReport);
    fn run(&self) -> Result<(ProjectPackageReport, DeviceInstallReport), HubError> {
        let package_report = package_project(&self.package_request)?;
        let install_request =
            DeviceInstallRequest::new(package_report.package_dir.clone(), self.device_root.clone());
        let install_report = install_package_to_device(&install_request)?;
        Ok((package_report, install_report))
    }
}

// runtime_state/editor_launch_actions.rs —— 39-50 行固有 run 平移为：
use super::action_tasks::BackgroundTask;

impl BackgroundTask for PendingEditorLaunch {
    type Output = EditorLaunchReport;
    fn run(&self) -> Result<EditorLaunchReport, HubError> {
        // 函数体与现 41-50 行一致（launch_editor / Command::new(executable).spawn()）
    }
}
```

四个 pending 类型均为 `PathBuf`/`String`/`BuildCommand` 纯数据，`Send + 'static` 与 `UnwindSafe` 无障碍（风险章节既有判断维持）。同文件内 `package_recent_project` / `install_recent_project_to_device` / `open_selected_project_or_editor` 的前台同步路径继续调 `pending.run()`，只是经 trait 方法（需 `use super::action_tasks::BackgroundTask;`）。

（c）`commands.rs` 终态（整文件收敛到约 80 行；删除 110-358 附近全部四份执行函数与 `emit_and_continue`/`emit_current_state_and_continue`/`continue_background_queue`、94-107 内联回退、第 6 行 `use crate::build::run_build_command;`）：

```rust
// commands.rs —— spawn_background_action 终态
use super::runtime_state::action_tasks::{dispatch_background_request, run_background_worker_loop};

fn spawn_background_action(
    request: HubActionRequest,
    session_handle: Arc<Mutex<HubRuntimeSession>>,
    app: tauri::AppHandle,
) {
    thread::spawn(move || {
        let emit_state = |view_model: &HubViewModel| {
            let _ = app.emit("hub-state-changed", view_model);
        };
        run_background_worker_loop(
            request,
            &session_handle,
            &emit_state,
            dispatch_background_request,
        );
    });
}
```

配套：`runtime_state.rs:8` 的 `mod action_tasks;` 改 `pub(in crate::tauri_app) mod action_tasks;`，让 `commands.rs` 直接路径引用，不加 re-export 桥。`hub_state`/`hub_action`（36-66 行附近）与 `HubCommandState`（14-34 行附近）不动；前台 `app.emit("hub-state-changed", &view_model)`（58、64 行附近）维持现状。

（d）任务可观测字段链（切片 4）：

```rust
// state/task_status.rs —— 结构体（1-11 行）追加字段 + builder：
pub struct TaskStatus {
    // ……既有 8 个字段不动……
    pub progress_percent: u8,
    /// 单调递增后台任务 id；0 = 当前状态不关联后台任务。
    pub task_id: u64,
}

impl TaskStatus {
    pub fn with_task_id(mut self, task_id: u64) -> Self {
        self.task_id = task_id;
        self
    }
}
// idle()/running()/new() 等构造器（37-112 行）一律补 task_id: 0。

// tauri_app/runtime_state.rs —— session 结构（47-72 行附近）追加：
//   background_task_counter: u64,            // load_from_paths 初始化为 0
// snapshot()（239-266 行附近）追加：
//   queued_background_actions: self.background_action_queue.len(),

// state/hub_snapshot.rs —— HubSnapshot（16-41 行）追加：
//   pub queued_background_actions: usize,

// runtime_state/action_tasks.rs —— start_background_action_status（78-93 行附近）改为：
//   self.background_task_counter += 1;
//   self.task_status = TaskStatus::running_operation(...)
//       .with_task_id(self.background_task_counter);
// record_background_action_error（135-162 行附近）末尾构造 error 状态时补
//   .with_task_id(self.task_status.task_id)   // 保留与失败任务的关联

// tauri_app/view_model.rs —— HubTaskSummary（81-91 行）追加：
//   pub task_id: u64,        // serde camelCase → taskId
//   pub queued: usize,
// task_summary()（301-317 行附近）填充：
//   task_id: snapshot.task_status.task_id,
//   queued: snapshot.queued_background_actions,
```

```ts
// web/src/types/hub.ts —— HubTaskSummary（9-16 行）追加：
export interface HubTaskSummary {
  // ……既有字段……
  progressPercent: number;
  taskId: number;
  queued: number;
}
// web/src/App.tsx 的两个 taskSummary 字面量（68-75、112-119 行）与
// web/src/data/hubData.ts（54-61 行）补 taskId: 0, queued: 0。
// 纯数字字段，无业务文案，不违反「文案归 Rust DTO」约束。
```

#### 文件变更清单

| 路径 | 动作 | 变更内容一句话 |
|------|------|----------------|
| `zircon_hub/src/tauri_app/runtime_state/action_tasks.rs` | 修改 | 落 `BackgroundTask` trait、`EmitState`、`lock_session`、`execute_background_task`、`dispatch_background_request`、`run_background_worker_loop`、`panic_detail`、`record_background_worker_panic`；`start_background_action_status` 接 task_id 计数 |
| `zircon_hub/src/tauri_app/runtime_state.rs` | 修改 | `mod action_tasks;`（8 行）提为 `pub(in crate::tauri_app)`；session 增 `background_task_counter`；`snapshot()` 增 `queued_background_actions` |
| `zircon_hub/src/tauri_app/commands.rs` | 修改 | 删四份 `run_background_*_action`（110-324 附近）、`emit_and_continue`/`emit_current_state_and_continue`/`continue_background_queue`（326-358 附近）、spawn 内联回退（94-107）与 `run_build_command` import；spawn 接 worker loop |
| `zircon_hub/src/tauri_app/runtime_state/build_actions.rs` | 修改 | `PendingEditorRuntimeBuild` 实现 `BackgroundTask`；`build_selected_project_engine` 改 `pending_build.run()` |
| `zircon_hub/src/tauri_app/runtime_state/project_delivery_actions.rs` | 修改 | 两个固有 `run`（29-45 附近）平移为 `BackgroundTask` impl |
| `zircon_hub/src/tauri_app/runtime_state/editor_launch_actions.rs` | 修改 | `PendingEditorLaunch::run`（39-50 附近）平移为 `BackgroundTask` impl |
| `zircon_hub/src/state/task_status.rs` | 修改 | `TaskStatus` 增 `task_id` + `with_task_id`；各构造器补默认 0 |
| `zircon_hub/src/state/hub_snapshot.rs` | 修改 | `HubSnapshot` 增 `queued_background_actions: usize` |
| `zircon_hub/src/tauri_app/view_model.rs` | 修改 | `HubTaskSummary` 增 `task_id`/`queued` 并在 `task_summary()` 填充 |
| `zircon_hub/web/src/types/hub.ts` | 修改 | `HubTaskSummary`（9-16 行）增 `taskId`/`queued` |
| `zircon_hub/web/src/App.tsx`、`zircon_hub/web/src/data/hubData.ts` | 修改 | taskSummary 字面量补 `taskId: 0, queued: 0` |
| `zircon_hub/tests/project_workflow_contract.rs` 等四个契约文件 | 修改 | commands/action_tasks/build_actions snippet 刷新（见契约联动） |

#### 实施步骤

1. `action_tasks.rs` 落框架代码（目标代码形状 a，暂无调用方），`runtime_state.rs:8` 模块声明提可见性。验证：`cargo check -p zircon_hub --locked`。
2. 三个 owner 文件落 trait impl 并删固有 `run`（目标代码形状 b）；`build_selected_project_engine`（`build_actions.rs:31-38`）改 `pending_build.run()`。同步刷新契约联动表第 4-6 行。验证：`cargo check -p zircon_hub --locked`、`cargo test -p zircon_hub --lib project_delivery --locked`、`cargo test -p zircon_hub --lib editor_launch --locked`、`cargo test -p zircon_hub --test project_workflow_contract --test project_quick_actions_contract --locked`。
3. `commands.rs` 硬切换（目标代码形状 c）：删旧四函数与续接函数，spawn 接 `run_background_worker_loop`。同步刷新契约联动表第 1-3、7 行（四个契约文件的 commands 块）。验证：`cargo check -p zircon_hub --locked`、`cargo test -p zircon_hub --test project_workflow_contract --test project_quick_actions_contract --test ui_foundation_contract --test tauri_react_shell_contract --locked`。
4. 补 M1 行为测试（见契约联动「新增测试」），覆盖 prepare/run/complete 失败、panic 续接、FIFO。验证：`cargo test -p zircon_hub --lib action_tasks --locked`。
5. 可观测字段链（目标代码形状 d）：`task_status.rs` → `runtime_state.rs`（计数器 + snapshot）→ `hub_snapshot.rs` → `view_model.rs` → `hub.ts` → `App.tsx`/`hubData.ts`。验证：`cargo test -p zircon_hub --lib --locked`；前端在 `zircon_hub/` 目录（package.json 位于 `zircon_hub/`，非 `web/`）执行 `npm run typecheck`、`npm run build`。
6. 收尾全量：`cargo test -p zircon_hub --locked`、`cargo test -p zircon_hub --lib --locked`、`cargo fmt --all --check`。验收：`rg "run_background_(build|package|install|editor)_action" zircon_hub/src` 零命中；`rg "emit_and_continue|continue_background_queue" zircon_hub/src` 零命中。

#### 契约联动

需同变更刷新的既有断言（原文 → 改为）：

| # | 文件（位置） | 现有断言原文 | 改为 |
|---|------|--------------|------|
| 1 | `project_workflow_contract.rs`（commands 块，398-423 行附近） | `"run_background_build_action(request, session_handle, app);"` 等四条、`"fn run_background_build_action("` 等四条、`"session.prepare_background_editor_runtime_build()"` 等八条 prepare/complete 调用、`"emit_current_state_and_continue(session_handle, app);"`、`"fn emit_and_continue("`、`"fn continue_background_queue("`、`"session.take_next_background_action()"`、`"let view_model = match session.apply_action(request.clone())"`、`"session.record_background_action_error(&request, error.to_string())"`、`"run_build_command(pending_build.command())"` | commands 块收敛为：`"fn spawn_background_action("`、`"thread::spawn(move ||"`（保留）、`"run_background_worker_loop("`、`"dispatch_background_request,"`、`"app.emit(\"hub-state-changed\", &view_model)"`（保留）；prepare/complete/take_next/apply_action 回退/catch_unwind 断言移入 action_tasks 块 |
| 2 | `project_quick_actions_contract.rs`（commands 块，215-255 行附近，四函数条目在 225-246） | 同上一组 | 同上 |
| 3 | `ui_foundation_contract.rs`（commands 块，544-579 行附近） | `"run_background_package_action(request, session_handle, app);"` 等 | 同上 |
| 4 | `project_workflow_contract.rs:237`、`project_quick_actions_contract.rs:160`（build_actions 块） | `"let result = run_build_command(pending_build.command())"` | `"let result = pending_build.run()"` + `"run_build_command(self.command())"`（trait impl 内） |
| 5 | `project_quick_actions_contract.rs:140`（editor_launch 块） | `"pub(in crate::tauri_app) fn run(&self) -> Result<EditorLaunchReport, HubError>"` | `"impl BackgroundTask for PendingEditorLaunch"` + `"fn run(&self) -> Result<EditorLaunchReport, HubError>"` |
| 6 | `tauri_react_shell_contract.rs:209-211、214` | `"run_background_package_action(request, session_handle, app);"` 等三条、`"run_build_command(pending_build.command())"` | `"run_background_worker_loop("`、`"dispatch_background_request"`、`"run_build_command(self.command())"` |
| 7 | action_tasks 块（`project_workflow_contract.rs:212-228`、`project_quick_actions_contract.rs:196-214` 附近、`ui_foundation_contract.rs:640-652` 附近；条目已是 `"HubActionId::BuildProject => Some(Self::BuildProject)"` 形式） | 既有条目不改 | 只增不改：补 `"trait BackgroundTask"`、`"fn execute_background_task"`、`"fn dispatch_background_request"`、`"fn run_background_worker_loop"`、`"catch_unwind"`、`"fn lock_session"` |

注意保留项：`action_tasks.rs` 既有单测 `background_actions_queue_while_worker_is_active_and_dequeue_fifo`（381-427 行附近）断言「排队不得改写运行中状态」（`assert_eq!(session.task_status, running_status, ...)`，406-409 行附近）——切片 4 把队列长度放 snapshot 而非 `TaskStatus`，正是为了让该断言原样存活。

新增测试（均落 `action_tasks.rs` `#[cfg(test)]`，沿用其 `session_with_project`/`temp_test_dir` 既有 fixture；emit 闭包用 `RefCell<Vec<HubViewModel>>` 收集）：
- `execute_background_task_emits_running_then_completion_states_in_order`：对 package 任务传真实 prepare/complete 方法指针；断言第一次 emit 的 `task_summary.label == "Packaging"`（running），末次 emit `"Package created"`，且 `task_summary.task_id > 0`。
- `execute_background_task_records_prepare_target_failure_and_emits_once`：request 带 `target_id: Some("missing-project")`；断言仅 emit 一次、`task_status.severity == Error`、detail 含 `Unknown recent project target`。
- `execute_background_task_surfaces_run_failure_as_recorded_history`：`default_build_output_dir` 置空使 `pending.run()` 失败；断言 history[0] `HubActionStatus::Failed`、`task_status.detail == "Package output root is required"`（既有口径不变）。
- `execute_background_task_surfaces_complete_failure_as_error_state`：完成前把 `session.config_path` 指到「父路径是文件」的非法路径使 `record_action_and_persist` 失败；断言末次 emit 为 error 状态且不 panic。
- `background_worker_dispatch_processes_queue_in_fifo_order`：先 `start_background_action_or_record_error` 接 package，再排队 install；以 `dispatch_background_request` 驱动 `run_background_worker_loop`；断言 history 自顶向下为 InstallProject、PackageProject（install 内联包）、PackageProject（首任务），结束时 `background_worker_active == false`、队列为空。
- `worker_panic_records_error_resets_worker_flag_and_continues_queue`：注入「首个请求 panic、后续请求委托真实分发」的 dispatch fn；预排一个 package 请求；断言 panic 后 `task_status.severity == Error`、detail 含 `"Background task panicked"`、第二个请求正常完成、`background_worker_active == false`。
- `lock_session_recovers_poisoned_session_lock`：在持锁闭包内 panic 使锁中毒，再调 `lock_session` 断言可取回 guard 且 session 状态可读。

panic 诊断由 `state/hub_message/shell.rs::ShellMessageId::BackgroundTaskPanicked` 与 `ReviewActionTarget` 结构化承载；不得在 `localized.rs` 恢复英文 `strip_prefix` 解析。

测试阶段：
- `cargo test -p zircon_hub --locked`（build/delivery/editor-launch 相关契约）。
- 新增测试：队列 FIFO 顺序（连发 build+package+install，断言 history 顺序）；prepare 失败、run 失败、complete 失败三类路径都正确续接队列且 `background_worker_active` 复位。
- 注意：`zircon_hub/Cargo.toml` 的 `[lib] test = false`（10-13 行）使 `cargo test -p zircon_hub --locked` 默认只跑 `tests/` 集成契约；src 内单测必须用 `cargo test -p zircon_hub --lib --locked` 显式选中（见风险章节注记）。

### M2 持久化原子性与单点化

切片：
1. `settings/hub_config.rs` 落原子 save：写 `hub.toml.tmp` 后使用平台原子替换；Windows 走 `ReplaceFileW`，失败时只清理 tmp，绝不先删除 canonical target。
2. 全仓 `rg persist` 盘点 `runtime_state` 内所有落盘点，统一改走 session 上单一 `persist()`（内部带 last-project / editor recent 同步），删除散落变体；persist 错误转 `TaskStatus` error + recovery（"检查磁盘空间/权限后重试"口径，文案归 07）。
3. 为 lock poisoned 分支补 `log`（或现有诊断通道）输出。

#### 目标代码形状

（a）原子落盘——不另起 public `persist_atomic`，直接由唯一入口 `HubConfig::save` 写同目录 tmp。首次创建可用 `fs::rename`；Windows 已存在目标必须调用 `ReplaceFileW(REPLACEFILE_WRITE_THROUGH)`。任何替换失败都保留旧 target 并清理 tmp，不允许 unlink-old + retry：

```rust
// settings/hub_config.rs —— 替换 save（51-60 行）
impl HubConfig {
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), HubError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        let text = toml::to_string_pretty(self)?;
        write_atomic(path, text.as_bytes())
    }
}

/// tmp + rename 原子替换；失败时清理 tmp，不碰既有目标文件。
fn write_atomic(path: &Path, contents: &[u8]) -> Result<(), HubError> {
    let mut tmp_name = path.as_os_str().to_os_string();
    tmp_name.push(".tmp");
    let tmp_path = PathBuf::from(tmp_name);
    fs::write(&tmp_path, contents)?;
    if let Err(error) = replace_file(&tmp_path, path) {
        let _ = fs::remove_file(&tmp_path);
        return Err(error);
    }
    Ok(())
}

fn replace_file(tmp_path: &Path, path: &Path) -> Result<(), HubError> {
    if fs::rename(tmp_path, path).is_ok() {
        return Ok(());
    }
    replace_existing_file(tmp_path, path)
        .map_err(|error| HubError::message(format!("Atomic replace failed: {error}")))
}
```

（b）persist 单点化——现状三个变体（`runtime_state.rs:431-454`）：`persist()`（= with_last_project(None)）、`persist_hub_config()`（只写 hub.toml）、`persist_with_last_project(Option<&Path>)`（hub.toml + editor recent JSON）。收敛为调用方唯一可见的 `persist(last_project_path)`，内部 IO 拆 `persist_unchecked`（非调用方变体，仅供错误拦截分层）；签名 `&self` → `&mut self` 以便失败时写 `task_status`：

```rust
// runtime_state.rs —— 替换 431-454 行三函数
fn persist(&mut self, last_project_path: Option<&Path>) -> Result<(), HubError> {
    let result = self.persist_unchecked(last_project_path);
    if let Err(error) = &result {
        eprintln!("zircon_hub: failed to persist hub state: {error}");
        self.task_status = TaskStatus::error(
            "Save Hub state failed",
            error.to_string(),
            "Check disk space and write permissions for hub.toml, then retry",
        )
        .with_operation(
            TaskOperationKind::Hub,
            self.config_path.to_string_lossy().into_owned(),
        );
    }
    result
}

fn persist_unchecked(&self, last_project_path: Option<&Path>) -> Result<(), HubError> {
    let mut config = self.config.clone();
    config.runtime = self.runtime_state_for_config();
    config.save(&self.config_path)?;
    match last_project_path {
        Some(path) => save_editor_recent_projects_with_last_project(
            &self.editor_config_path,
            &self.config.recent_projects,
            Some(path),
        )?,
        None => {
            save_editor_recent_projects(&self.editor_config_path, &self.config.recent_projects)?
        }
    }
    Ok(())
}
```

行为变化点明示：此前 `persist_hub_config` 不写 editor recent JSON，统一后每次 persist 都同步重写（内容幂等、文件小，写放大可接受，见风险注记）；`search_projects`（295 行）与 `view_all_projects`（377 行）的 `let _ = self.persist_hub_config();` 改 `let _ = self.persist(None);`——结果仍不阻塞 UI 流，但失败不再不可见（`task_status` 已被 persist 内部置为 error）。

调用点盘点（2026-06-12 工作树快照 `rg persist` 实测，机械替换表；01 计划仍在同一工作树推进，实施时需重跑 `rg "persist_hub_config|persist_with_last_project|self\.persist\(\)" zircon_hub/src` 复核行号）：

| 现调用 | 位置 | 改为 |
|--------|------|------|
| `self.persist_hub_config()` | `runtime_state.rs:273、290、295（let _）、310、325、340、367、377（let _）、399`；`action_tasks.rs:92、161`；`learn_actions.rs:85、118`；`new_project_actions.rs:23`；`output_actions.rs:64、135`；`project_actions.rs:202、245、267、483`；`quick_actions.rs:18` | `self.persist(None)` |
| `self.persist()` | `runtime_state.rs:132（session.persist()?）、421` | `self.persist(None)` |
| `self.persist_with_last_project(Some(&path))` | `runtime_state.rs:359`；`editor_launch_actions.rs:332`；`project_actions.rs:89、177` | `self.persist(Some(&path))` |
| `self.persist_with_last_project(None)` | `project_actions.rs:226、291` | `self.persist(None)` |

（c）poisoned / 静默路径补日志（仓内无 `log` crate——`zircon_hub/Cargo.toml` dependencies 仅 serde/serde_json/tauri/thiserror/toml，§3 禁新增依赖，统一用 `eprintln!` 即现有 stderr 诊断通道）：M1 的 `lock_session` 已覆盖后台线程全部锁点；本切片剩余两处——`commands.rs::HubCommandState::session()`（25-29 行附近，保留向 IPC 返回 `Hub runtime state lock is poisoned` 的 `Err`，在 `map_err` 前补一行 `eprintln!`）；以及验收性检查 `rg "let Ok\\(.*session" zircon_hub/src` 不得再出现静默丢弃锁错误的分支。

（d）persist 错误文案由 typed `HubMessage` owner 承载；`localized.rs` 不再解析英文 detail。实施和测试必须断言 message id/rendered text，而不是追加 `status_detail` 字符串分支。

#### 文件变更清单

| 路径 | 动作 | 变更内容一句话 |
|------|------|----------------|
| `zircon_hub/src/settings/hub_config.rs` | 修改 | `save`（51-60）改 tmp+rename 原子写；新增私有 `write_atomic`/`replace_file` 与两个单测 |
| `zircon_hub/src/tauri_app/runtime_state.rs` | 修改 | 431-454 三个 persist 变体收敛为 `persist(&mut self, Option<&Path>)` + `persist_unchecked`；失败写 `TaskStatus` error |
| `zircon_hub/src/tauri_app/runtime_state/{action_tasks,learn_actions,new_project_actions,output_actions,project_actions,quick_actions,editor_launch_actions}.rs` | 修改 | 按盘点表机械替换 22 处调用点 |
| `zircon_hub/src/tauri_app/commands.rs` | 修改 | `HubCommandState::session()` poisoned 分支补 `eprintln!` |
| `zircon_hub/src/state/hub_message/` | 修改 | typed persist failure detail/recovery 与双语模板 |
| `zircon_hub/tests/project_management_contract.rs` | 修改 | 119-125 行三签名断言与 97 行 `session.persist()?;` 刷新 |
| `zircon_hub/tests/project_workflow_contract.rs` | 修改 | 199-202 行 persist 签名断言刷新 |
| `zircon_hub/tests/project_quick_actions_contract.rs` | 修改 | 129 行 `"self.persist_hub_config()"` 刷新 |
| `zircon_hub/tests/ui_foundation_contract.rs` | 修改 | 601 行 `"persist_hub_config"` 刷新 |
| `zircon_hub/tests/ui_shell_navigation_contract.rs` | 修改 | 204 行 `"self.persist_hub_config()"` 刷新 |

#### 实施步骤

1. `hub_config.rs` 改 `save` 为原子写（目标代码形状 a）并补两个单测（见契约联动）。验证：`cargo test -p zircon_hub --lib hub_config --locked`。
2. `runtime_state.rs` 落 `persist`/`persist_unchecked`（目标代码形状 b），按盘点表机械替换全部 22 处调用点；同步刷新五个契约文件的 persist 断言（见契约联动）。验证：`cargo check -p zircon_hub --locked`、`cargo test -p zircon_hub --lib --locked`、`cargo test -p zircon_hub --test project_management_contract --test project_workflow_contract --test project_quick_actions_contract --test ui_foundation_contract --test ui_shell_navigation_contract --locked`。
3. 在 `state/hub_message/shell.rs` 补 typed persist failure/recovery 文案（目标代码形状 d）；`runtime_state.rs` 补 `persist_failure_sets_recoverable_status_and_recovers_after_retry` 单测。验证：`cargo test -p zircon_hub --lib persist_failure --locked`。
4. `commands.rs::session()` 补 poisoned 日志（目标代码形状 c）；验收 `rg "lock()" zircon_hub/src/tauri_app` 确认除 `lock_session` 与 `session()` 外无裸锁分支。验证：`cargo check -p zircon_hub --locked`。
5. 全量回归：`cargo test -p zircon_hub --locked`、`cargo test -p zircon_hub --lib --locked`、`cargo fmt --all --check`。验收：`rg "persist_hub_config|persist_with_last_project" zircon_hub` 仅剩契约测试历史文档（应为零命中）。

#### 契约联动

需同变更刷新的既有断言（原文 → 改为）：

| 文件（位置） | 现有断言原文 | 改为 |
|------|--------------|------|
| `project_management_contract.rs:97` | `"session.persist()?;"` | `"session.persist(None)?;"` |
| `project_management_contract.rs:120-122` | `"fn persist(&self) -> Result<(), HubError>"`、`"fn persist_hub_config(&self) -> Result<(), HubError>"`、`"fn persist_with_last_project(&self, last_project_path: Option<&Path>) -> Result<(), HubError>"` | `"fn persist(&mut self, last_project_path: Option<&Path>) -> Result<(), HubError>"`、`"fn persist_unchecked(&self, last_project_path: Option<&Path>) -> Result<(), HubError>"` |
| `project_workflow_contract.rs:199、201` | `"fn persist_hub_config(&self) -> Result<(), HubError>"`、`"fn persist_with_last_project(&self, last_project_path: Option<&Path>) -> Result<(), HubError>"` | 同上两条新签名（200 行 `"config.runtime = self.runtime_state_for_config();"` 与 202 行 `"save_editor_recent_projects_with_last_project("` 不变） |
| `project_workflow_contract.rs:479`、`ui_selected_project_runtime_contract.rs:120` | `"self.persist_with_last_project(Some(&project.path))"` | `"self.persist(Some(&project.path))"` |
| `project_quick_actions_contract.rs:129` | `"self.persist_hub_config()"` | `"self.persist(None)"` |
| `ui_foundation_contract.rs:601` | `"persist_hub_config"` | `"fn persist("` |
| `ui_shell_navigation_contract.rs:204` | `"self.persist_hub_config()"` | `"self.persist(None)"` |

新增测试（测试函数名 + 断言要点）：
- `hub_config.rs::save_replaces_existing_config_atomically_without_leaving_tmp_file`：同一路径先后 save 两份不同配置；断言重载等于第二份、`hub.toml.tmp` 不存在。
- `hub_config.rs::save_keeps_previous_config_when_tmp_write_is_blocked`：预先在 `hub.toml.tmp` 路径创建同名目录使 `fs::write(tmp)` 失败；断言 `save` 返回 Err、原 `hub.toml` 内容逐字未变（对应切片测试点「persist 目标目录只读时…」的 Windows 可靠替代注入，见风险注记）。
- Windows `hub_config.rs::save_keeps_previous_config_when_atomic_replace_is_denied`：把 existing target 设为只读后保存新配置；断言返回 atomic replace error、旧内容逐字不变且 tmp 已清理。
- `runtime_state.rs::persist_failure_sets_recoverable_status_and_recovers_after_retry`：加载正常 session 后把 `session.config_path` 指向「父路径是一个文件」的非法位置；`apply_action(show-page)` 返回 Err，断言 `task_status.severity == Error`、`label == "Save Hub state failed"`、`recovery` 非空；恢复 `config_path` 后再次 `apply_action` 成功且 `hub.toml` 重载内容正确（内存状态仍可再次 persist）。
- hub-message 测试断言 persist failure/recovery 的 message id、参数数量及稳定中英文渲染。

测试阶段：
- 新增测试:persist 目标目录只读时返回错误且内存状态仍可再次 persist；tmp 文件不残留。
- `cargo test -p zircon_hub --locked` 全量回归。

### M3 交付链路清理与 TOCTOU 修复

切片：
1. `projects/package.rs`：`package_project` 失败路径删除本次创建的 `unique_package_dir`；成功路径不变。
2. `projects/device_install.rs`：`fs::create_dir(&install_dir)` 失败（AlreadyExists）即报"安装已存在"，删除 exists 预检；拷贝失败删除本次 install 目录。
3. 为两者补失败注入测试（用只读目录/不存在的 source 模拟）。

#### 目标代码形状

共享小件落 `projects/local_paths.rs`（两文件已分别 `use super::local_paths::reject_inside_root;`——`package.rs:8`、`device_install.rs:6`，同模块落点自然）：

```rust
// projects/local_paths.rs 追加
use std::fs;
use crate::state::HubMessage;

/// 原子占位：fs::create_dir 一步完成「不存在性检查 + 创建」，消除
/// exists() → create_dir_all 的 TOCTOU；AlreadyExists 用调用方口径报错。
pub(super) fn create_owned_dir(
    path: &Path,
    already_exists_message: impl FnOnce() -> HubMessage,
) -> Result<(), HubError> {
    match fs::create_dir(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            Err(HubError::status(already_exists_message(), None))
        }
        Err(error) => Err(error.into()),
    }
}

/// 只清理「本次由 create_owned_dir 创建」的目录：调用方保证 created_dir
/// 是占位成功后的路径，失败即整目录删除，既有产物绝不会传进来。
pub(super) fn cleanup_dir_on_error<T>(
    created_dir: &Path,
    result: Result<T, HubError>,
) -> Result<T, HubError> {
    if result.is_err() {
        let _ = fs::remove_dir_all(created_dir);
    }
    result
}
```

`package.rs`——现状 `package_project`（55-80 行）创建 `unique_package_dir`（90-96 行，`packages/{basename}-{created_unix_ms}`）后直拷，中断即残留半成品；改为「占位 + 失败整删」：

```rust
// projects/package.rs —— 替换 package_project（55-80 行）
use super::local_paths::{cleanup_dir_on_error, create_owned_dir, reject_inside_root};

pub fn package_project(request: &ProjectPackageRequest) -> Result<ProjectPackageReport, HubError> {
    if request.project_root.as_os_str().is_empty() || !request.project_root.is_dir() {
        return Err(HubError::message(
            "Project root is not available for packaging",
        ));
    }
    if request.output_root.as_os_str().is_empty() {
        return Err(HubError::message("Package output root is required"));
    }

    reject_output_inside_project(&request.project_root, &request.output_root)?;
    fs::create_dir_all(&request.output_root)?;

    let package_dir = unique_package_dir(request);
    if let Some(parent) = package_dir.parent() {
        fs::create_dir_all(parent)?;
    }
    create_owned_dir(&package_dir, || {
        format!(
            "Package directory already exists: {}",
            package_dir.to_string_lossy()
        )
    })?;
    cleanup_dir_on_error(&package_dir, fill_package_dir(request, &package_dir))
}

fn fill_package_dir(
    request: &ProjectPackageRequest,
    package_dir: &Path,
) -> Result<ProjectPackageReport, HubError> {
    let project_dir = package_dir.join(PACKAGE_PROJECT_DIR);
    fs::create_dir_all(&project_dir)?;
    let files_copied = copy_project_tree(&request.project_root, &project_dir)?;
    let manifest_path = package_dir.join(PACKAGE_MANIFEST_FILE);
    write_package_manifest(request, &manifest_path, files_copied)?;
    Ok(ProjectPackageReport {
        package_dir: package_dir.to_path_buf(),
        manifest_path,
        files_copied,
    })
}
```

package/device directory collision 使用 `DeliveryMessageId::PackageDirectoryAlreadyExists` / `DeviceInstallAlreadyExists` typed message 与参数化路径；不得恢复 `localized.rs::status_detail` 的英文前缀解析。

`device_install.rs`——现状 `install_package_to_device`（29-58 行）`install_dir.exists()` 预检（45-50 行）后 `fs::create_dir_all`（51 行）：双检即 TOCTOU，且拷贝失败残留半截目录。改为：

```rust
// projects/device_install.rs —— 替换 install_package_to_device（29-58 行）
use super::local_paths::{cleanup_dir_on_error, create_owned_dir, reject_inside_root};

pub fn install_package_to_device(
    request: &DeviceInstallRequest,
) -> Result<DeviceInstallReport, HubError> {
    if request.package_dir.as_os_str().is_empty() || !request.package_dir.is_dir() {
        return Err(HubError::message("Package directory is not available"));
    }
    if request.device_root.as_os_str().is_empty() {
        return Err(HubError::message("Device install directory is required"));
    }

    reject_device_inside_package(&request.package_dir, &request.device_root)?;
    fs::create_dir_all(&request.device_root)?;

    let install_dir = request
        .device_root
        .join(package_install_name(&request.package_dir));
    // 删除 exists() 预检：create_owned_dir 原子占位，冲突由 typed
    // DeliveryMessageId 承载路径并在显示边界渲染双语文本。
    create_owned_dir(&install_dir, || {
        HubMessage::with_params(
            HubMessageId::Delivery(DeliveryMessageId::DeviceInstallAlreadyExists),
            [install_dir.to_string_lossy().into_owned()],
        )
    })?;
    cleanup_dir_on_error(
        &install_dir,
        copy_directory_tree(&request.package_dir, &install_dir).map(|files_copied| {
            DeviceInstallReport {
                install_dir: install_dir.clone(),
                files_copied,
            }
        }),
    )
}
```

注入策略修正（2026-06-12 核实）：Windows 上目录的只读属性不阻止在目录内创建文件，原切片「用只读目录模拟」不可靠；失败注入改为两类——(1) 占位冲突注入：预建同名目录/文件触发 AlreadyExists，同时断言既有目录内容未被删（锁「只删本次新建」边界）；(2) `cleanup_dir_on_error`/`create_owned_dir` 以 helper 单测直接锁清理与占位语义，配合契约 snippet 断言两个调用点接线。「不存在的 source」注入沿用既有测试（`package_project` 对空/缺失 project_root 在创建任何目录前即失败，`package.rs:56-63`）。

#### 文件变更清单

| 路径 | 动作 | 变更内容一句话 |
|------|------|----------------|
| `zircon_hub/src/projects/local_paths.rs` | 修改 | 追加 `create_owned_dir`、`cleanup_dir_on_error` 两个 `pub(super)` helper 及单测 |
| `zircon_hub/src/projects/package.rs` | 修改 | `package_project` 改「占位 + fill_package_dir + 失败整删」；新增 `fill_package_dir`；新错误消息一条 |
| `zircon_hub/src/projects/device_install.rs` | 修改 | 删 `exists()` 预检与 `create_dir_all(install_dir)`（45-51 行），改 `create_owned_dir` + `cleanup_dir_on_error`；冲突通过 typed delivery message 报告 |
| `zircon_hub/src/state/hub_message/delivery.rs` | 修改 | package/device collision typed message 与双语模板 |

#### 实施步骤

1. `local_paths.rs` 落两个 helper + 单测 `create_owned_dir_rejects_existing_directory_with_caller_message`、`cleanup_dir_on_error_removes_dir_only_on_error`。验证：`cargo test -p zircon_hub --lib local_paths --locked`。
2. `device_install.rs` 切换（目标代码形状），既有三个单测（101-156 行）应原样通过；新增 `install_package_to_device_rejects_existing_install_dir_without_modifying_it`（预建 install_dir + 内放标记文件；断言 typed `DeviceInstallAlreadyExists` 可渲染且标记文件仍在）。验证：`cargo test -p zircon_hub --lib device_install --locked`，以及锁定上层口径的 `cargo test -p zircon_hub --lib install_failure_localizes_duplicate_install_directory --locked`（`project_delivery_actions.rs`，断言中英文渲染均正确）。
3. `package.rs` 切换（目标代码形状），既有三个单测（167-239 行）应原样通过；新增 `package_project_rejects_preexisting_unique_package_dir_without_deleting_it`（固定 `created_unix_ms: 42` 预建 `packages/demo-42` + 标记文件；断言 typed collision message 且标记仍在）。消息模板归 `state/hub_message/delivery.rs`。验证：`cargo test -p zircon_hub --lib package --locked`。
4. 全量回归：`cargo test -p zircon_hub --lib --locked`、`cargo test -p zircon_hub --locked`、`cargo fmt --all --check`。手工验证：跑一次真实 package/install 后中途构造失败（如包目录冲突），确认磁盘上无半成品 `packages/*-{timestamp}` 与半截 install 目录残留。

#### 契约联动

- 契约面是 typed `DeliveryMessageId`、参数和最终双语渲染，不是英文 `detail.strip_prefix(...)`。`project_page_copy_contract.rs` 应断言 typed message 接线；`project_delivery_actions.rs::install_failure_localizes_duplicate_install_directory_summary_and_history` 继续断言中英文渲染结果。
- 集成契约 snippet 无需刷新：`project_workflow_contract.rs`/`ui_foundation_contract.rs` 等对 delivery 的断言面是 `"package_project(&self.request)"`、`"install_package_to_device(&install_request)"` 等调用名，均不变。
- 新增测试（函数名 + 断言要点）：
  - `local_paths.rs::create_owned_dir_rejects_existing_directory_with_caller_message`：对已存在目录返回 Err 且消息为闭包产物；对不存在路径创建成功。
  - `local_paths.rs::cleanup_dir_on_error_removes_dir_only_on_error`：Err 结果时目录被删，Ok 结果时目录保留。
  - `device_install.rs::install_package_to_device_rejects_existing_install_dir_without_modifying_it`：AlreadyExists 路径不触碰既有内容。
  - `package.rs::package_project_rejects_preexisting_unique_package_dir_without_deleting_it`：同上（package 侧）。
  - delivery message 测试断言 `PackageDirectoryAlreadyExists` 携带路径后能渲染稳定中英文。

测试阶段：
- `cargo test -p zircon_hub --lib package --locked`、`cargo test -p zircon_hub --lib device_install --locked`、`cargo test -p zircon_hub --lib project_delivery --locked` 及新增失败路径用例（原文「`cargo test -p zircon_hub project_cloud_local_delivery --locked`」的过滤词经 2026-06-12 全仓检索不存在，已按真实测试名修正，见风险注记）。
- 手工验证：磁盘上无半成品 `packages/*-{timestamp}` 残留。

## 风险与协调

- 依赖 01 计划的 `HubActionId`；若 01 未完成，M1 暂以现有字符串判定接入但不复制第四份表（单点引用 `action_tasks.rs` 现有映射）。【2026-06-12 复核：该前置已解除——01 计划 M1 已在当前工作树落地，`commands.rs::spawn_background_action` 已按 `HubActionId` 匹配，`BackgroundHubAction::from_request` 已经 `HubActionId::from_str` → `from_action_id` 单点映射，相关契约 snippet 已是 `HubActionId` 形式。】
- 【2026-06-12 协调注记】01 计划正在同一未提交工作树上持续落地，本文所有「现状」行号均为 2026-06-12 当日快照（如 `commands.rs` 较初版漂移约 4 行、`action_tasks.rs` 漂移约 5 行）；实施每个切片前先用文中给出的 `rg` 命令复核行号与断言原文，以届时工作树为准。
- `tauri-react-shell.md` 与多个契约测试断言 `commands.rs`/`runtime_state.rs` 的 owner 角色：重构保持文件归属不变（框架落 `action_tasks.rs`，`commands.rs` 仍是 Tauri command + spawn 入口），同变更刷新契约断言文本。
- `catch_unwind` 要求任务类型 `UnwindSafe`：pending 类型均为纯数据（PathBuf/String），预计无障碍；如有内部可变引用，改为线程入口处整体捕获。
- 【2026-06-12 核实修正】`zircon_hub` 无 `log` crate（`Cargo.toml` dependencies 仅 serde / serde_json / tauri / thiserror / toml），且 index.md §3 禁新增第三方依赖：M1/M2 的 poisoned、panic、persist 失败日志统一用 `eprintln!`（stderr 即「现有诊断通道」），不引入 log/env_logger。
- 【2026-08-01 收敛】Rust `std::fs::rename` 在 Windows 不提供本计划所需的“已存在目标原子替换且失败保留旧文件”合同；当前 owner 使用 `ReplaceFileW(REPLACEFILE_WRITE_THROUGH)`，禁止恢复先删旧再 rename 的回退。
- 【2026-06-12 核实修正】M3 测试阶段原写的过滤词 `project_cloud_local_delivery` 在仓内不存在（全仓检索零命中）；已改为真实测试名过滤（`--lib package` / `--lib device_install` / `--lib project_delivery`）。
- 【2026-06-12 核实修正】Windows 上目录只读属性不阻止目录内写入，「只读目录」失败注入不可靠：M2 的 tmp 写失败注入改为「在 `hub.toml.tmp` 路径预建同名目录」，M3 的清理语义改由占位冲突注入 + `cleanup_dir_on_error`/`create_owned_dir` helper 单测锁定。
- 【2026-06-12 核实补充】`zircon_hub/Cargo.toml` 的 `[lib] test = false`（10-13 行）使 `cargo test -p zircon_hub --locked` 默认只跑 `tests/` 下集成契约；src 内单测必须 `cargo test -p zircon_hub --lib --locked` 显式选中。本计划所有单测验证命令均已显式带 `--lib`，里程碑收尾需两条命令都跑。
- 【2026-06-12 核实补充】目标 2 的 task_id / 排队信息原里程碑切片未覆盖，已补为 M1 切片 4；队列长度放 `HubSnapshot.queued_background_actions` 而非 `TaskStatus`，避免破坏 `action_tasks.rs:406-409` 附近「排队不得改写运行中状态」的既有单测断言（`assert_eq!(session.task_status, running_status, ...)`）。
- 【2026-06-12 设计注记】persist 单点化后每次落盘同时重写 editor recent JSON（此前 `persist_hub_config` 不写）：内容幂等、文件小，写放大可接受；`save_editor_recent_projects` 自身的原子化不在本计划范围（hub.toml 原子性是本计划目标）。worker 队列续接由「每队列项新开线程」改为单线程循环消费，FIFO 单工语义不变。

## Code Review 收敛结果（2026-08-01）

- front-matter 已从 `planned` 收敛为 `in_progress`：M1/M2/M3 主实现存在，但不以静态存在代替当前受管行为门。
- 排队动作现在在出队时建立新的 running status，并在 prepared/success/error 全链保留同一非零 task id；新增三个 `background_action_lifecycle_*` 回归。原计划承诺的 worker FIFO、panic continuation、poison recovery 与 prepare-target failure 直接测试仍需补齐。
- Windows config replace 已删除“先删旧再 rename”风险，改为 `ReplaceFileW(REPLACEFILE_WRITE_THROUGH)`；新增 atomic replace denied 测试锁定旧配置保留与 tmp 清理。
- 任务取消没有 action/API/竞争测试，已明确整体留给 v2 决策；不得宣称 v1 支持 queued cancel。
- 受管 `cargo test -p zircon_hub --lib background_action_lifecycle_ --locked` GREEN；受管 `save_keeps_previous_config_when_atomic_replace_is_denied` GREEN。安装收据清理后的 `device_install` 复验连续被 Plugins01、Runtime11 的 CPU reservation 拒绝，未执行；因此只声明上述聚焦门，不声明 Hub 全量 GREEN，也不标 completed。
