---
handoff_kind: failure
status: open
created_at: 2026-08-12
summary_slug: thread-ownership-guard-test-scope
origin_plan: docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
fixing_plan: docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
origin_child_dir: docs/plans/zircon_editor/editor/14
fixing_child_dir: docs/plans/zircon_editor/editor/14
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/jobs/tests/thread_ownership_contract.rs
  - zircon_editor/src/core/jobs/tests/thread_ownership_contract/scanner.rs
  - zircon_editor/src/core/export/stages/compile_host.rs
  - zircon_editor/src/core/play/process_backend/output.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/export.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/environment.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/environment/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/window.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/lifecycle.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/redraw/present.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/with_viewport.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/live_host/development_watch.rs
  - docs/plans/zircon_runtime/runtime/11/failure-2026-08-10-blocking-io-process-output-budget.md
  - docs/plans/zircon_editor/editor/12/failure-2026-08-12-native-plugin-development-watcher-job-ownership.md
tests:
  - cargo test -p zircon_editor --lib bare_thread_guard_ --locked --jobs 1 -- --test-threads=1
  - cargo test -p zircon_editor --lib editor_production_sources_do_not_create_bare_threads --locked --jobs 1 -- --test-threads=1
  - cargo test -p zircon_editor --lib present_artifact_export_runs_as_an_injected_export_job --locked --jobs 1 -- --test-threads=1
  - cargo test -p zircon_editor --lib profile_artifact_admission_reservation_bounds_capture_before_materialization --locked --jobs 1 -- --test-threads=1
  - cargo test -p zircon_editor --lib profile_artifact_rejection_precedes_export_materialization --locked --jobs 1 -- --test-threads=1
  - cargo test -p zircon_editor --lib rejected_profile_artifact_submission_records_a_host_warning --locked --jobs 1 -- --test-threads=1
  - cargo test -p zircon_editor --lib profile_artifact_job_is_cancelled_when_the_final_host_owner_drops --locked --jobs 1 -- --test-threads=1
  - cargo test -p zircon_editor --lib profile_output_root_requires_an_absolute_non_c_drive_path --locked --jobs 1 -- --test-threads=1
  - cargo test -p zircon_editor --lib invalid_profile_output_root_precedes_export_materialization --locked --jobs 1 -- --test-threads=1
  - cargo test -p zircon_editor --lib invalid_profile_output_root_records_a_host_warning --locked --jobs 1 -- --test-threads=1
  - Runtime11 bounded stream/process-output validation before compile-host and Play reader migrations
  - Editor12 native plugin development watcher job-ownership return before the crate-wide thread gate can pass
---

# Editor14: M2 bare-thread guard test scope and production worker hard cut

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md`
- 来源执行切片：M2 bare-thread ownership hard cut audit
- 修复责任计划：`docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md`
- 交接原因：Editor14 owns the only editor job admission boundary and its hard-cut guard. Runtime11 owns the reusable bounded blocking-I/O stream required by the external process reader.

## 失败现象与复现证据

`editor_production_sources_do_not_create_bare_threads` reads every non-`tests` Rust source file as one text
stream. Inline `#[cfg(test)] mod tests { ... }` fixtures therefore reach the scanner and are reported as
production owners even though the compiler excludes them from production. The guard needed a syntax-token
test-only module boundary, not a path allowlist.

The initial audit found the compile-host readers and retained profiling writer. The subsequent complete
production-source audit records four raw-worker owners:

- `core/export/stages/compile_host.rs` starts one raw thread for each stdout/stderr pipe reader.
- `core/play/process_backend/output.rs` starts one raw thread for each Play stdout/stderr reader.
- `ui/retained_host/host_contract/profiling_artifacts/export.rs` previously started a process-lifetime raw writer
  through `OnceLock<SyncSender<_>>`; this failure now contains its injected-job forward repair.
- `ui/retained_host/app/module_plugin_actions/live_host/development_watch.rs` starts a debouncing hot-reload
  worker through `thread::Builder`, a private `JoinHandle`, and a synchronous channel.

Runtime11 owns the shared bounded stream replacement for both process-output families. Editor12 owns the native
plugin watcher lifecycle and has its own forward failure for the final owner. Removing these calls from the guard,
adding a whitelist, or moving them behind an alias would only hide obsolete ownership.

## 最低共享层根因

The immediate support defect is the guard's token scope: it recognizes filesystem test paths but not an inline
`#[cfg(test)]` module. The remaining production defect has two different lower owners:

1. Export and Play process stdout/stderr need Runtime11's single bounded blocking-I/O stream owner. Replacing the
   readers with `Command::output` would lose bounded tail/logging behavior and can retain unbounded output in
   memory.
2. Retained profiling export needs an injected EditorJobSystem-backed artifact job with lifecycle, cancellation,
   bounded admission, and shutdown ownership. That forward repair is implemented in this failure scope.
3. Native plugin development watching needs Editor12 to own the event/debounce/reload lifecycle through the
   injected job system, rather than a retained-host private thread/channel/join protocol.

## 架构修复验收

- The scanner excludes only bodies of direct `#[cfg(test)] mod` items and still reports a production thread
  that follows a test module.
- No production Rust source creates `std::thread`, `std::thread::Builder`, scoped threads, or aliases after
  M2 migration; tests remain free to create deterministic fixture threads.
- Runtime11's bounded stream ticket is consumed by the export compile host before its reader ownership is
  removed; stdout/stderr identity, bounded tails, persistent logs, cancellation, and terminal errors remain
  observable.
- Runtime11's bounded stream ticket is likewise consumed by the Play output backend before its reader ownership
  is removed.
- Editor12 replaces the native plugin development watcher worker with an injected, cancellable job-system
  lifecycle; it retains coalescing/debounce and hot-reload diagnostics without a direct thread, private channel,
  or UI-thread join.
- Profiling artifacts are submitted through the injected EditorJobSystem and have no retained-host private
  worker, queue, or shutdown path. An explicitly configured `C:` or relative artifact root is rejected with a
  typed host warning before presentation materialization; no profile artifact may be written to the system drive,
  including through a verbatim device-path alias.
- The focused guard then runs through the coordinator-managed Cargo gate before any fixed return.

## 禁止临时方案

- Do not whitelist source paths, comments, module names, or aliases in the guard.
- Do not keep raw readers behind an Editor14 facade, use `Command::output` as an unbounded substitute, or add a
  second background queue.
- Do not let retained UI own a scheduler, worker thread, or a process-lifetime channel service.
- Do not preserve the native plugin watcher worker behind an adapter or treat `notify`'s callback as authority to
  create a separate editor worker.

## 修复结果与回传

Open state: the scanner test-scope repair is implemented with focused regression coverage. The retained profiling
export has been hard-cut to an injected
EditorJobSystem `Export` job: it atomically reserves bounded shared admission before materializing a presentation or capturing a screenshot, reports typed
worker failure through the canonical job lifecycle, retains no retained-host worker/channel/queue, and keeps the
submitted JobId in the shared UiHostWindow owner so the final window reference requests cooperative cancellation.
Admission, reservation-commit, or explicit invalid-output-root rejection is retained as a host warning instead of
being silently consumed.
The existing app-root autosave closeout delegates to that same EditorJobSystem shutdown deadline. Its composition
binding occurs in the clean startup composition leaf before `RetainedEditorHost` assembly. The remaining Runtime11
compile-host and Play pipe-reader migrations are still open. Editor12 owns the separately recorded native plugin
development-watcher worker migration. No Cargo, fixed return, or milestone acceptance is claimed.

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 证据 |
|---|---|---|---|
| 2026-08-12 | `open / guard-test-scope-forward-repair` | scanner now removes only direct `#[cfg(test)] mod` bodies before thread-owner analysis; regressions prove test fixture threads are ignored while a subsequent production owner remains visible. | `thread_ownership_contract.rs` and its scanner leaf have scoped `rustfmt --check` and `git diff --check` evidence. Managed Rust test evidence is still required. |
| 2026-08-12 | `open / production-worker-hardcut-routed` | recorded the real raw export pipe readers and retained profiling artifact writer separately from the test-scope scanner fix. | `compile_host.rs:199-200` and profiling `export.rs:70-76`; Runtime11 bounded process-output failure is the lower shared prerequisite for the former. No path allowlist, compatibility facade, or private replacement queue was added. |
| 2026-08-13 | `open / profile-artifact-system-drive-policy-forward-repair` | profile export uses an injected `Export` job and rejects an explicitly configured `C:`, relative, or device-namespace output root before frame materialization; omitted root still means no capture, and normal UNC roots remain valid. | `profiling_artifacts/{environment,export}.rs` plus focused root-policy, pre-materialization, and host-warning tests are listed for managed validation. No Cargo or fixed return is claimed. |
