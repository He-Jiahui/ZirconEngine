---
related_code:
  - zircon_runtime/src/diagnostic_log/mod.rs
  - zircon_runtime/src/diagnostic_log/level.rs
  - zircon_runtime/src/diagnostic_log/platform.rs
  - zircon_runtime/src/diagnostic_log/sink.rs
  - zircon_runtime/src/diagnostic_log/timestamp.rs
  - zircon_app/src/entry/entry_runner/diagnostic_log_args.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_app/src/entry/entry_runner/runtime.rs
  - zircon_app/src/bin/runtime_preview.rs
  - zircon_runtime/src/asset/runtime_asset_path.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
  - zircon_editor/src/ui/template_runtime/runtime/build_session.rs
  - zircon_editor/src/ui/slint_host/host_contract/window.rs
  - zircon_editor/src/ui/slint_host/host_contract/presenter.rs
implementation_files:
  - zircon_runtime/src/diagnostic_log/mod.rs
  - zircon_runtime/src/diagnostic_log/level.rs
  - zircon_runtime/src/diagnostic_log/platform.rs
  - zircon_runtime/src/diagnostic_log/sink.rs
  - zircon_runtime/src/diagnostic_log/timestamp.rs
  - zircon_app/src/entry/entry_runner/diagnostic_log_args.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_app/src/entry/entry_runner/runtime.rs
  - zircon_app/src/bin/runtime_preview.rs
plan_sources:
  - user: 2026-05-04 add console and file-backed startup diagnostics for exported editor/runtime diagnosis
  - user: 2026-05-04 keep verbose diagnostics on in debug builds and off by default in release builds while preserving log/warn/error
  - docs/superpowers/plans/2026-05-04-build-asset-staging.md
tests:
  - cargo test -p zircon_runtime --lib diagnostic_log --locked --jobs 1 --target-dir E:\zircon-build\targets\diagnostic-log-level-runtime-test --message-format short --color never
  - cargo test -p zircon_app diagnostic_log_startup_args --no-default-features --features target-editor-host --locked --jobs 1 --target-dir E:\zircon-build\targets\diagnostic-log-level-app-test --message-format short --color never
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\zircon-build\targets\diagnostic-log-runtime-check --message-format short --color never
  - cargo check -p zircon_app --bin zircon_editor --no-default-features --features target-editor-host --target-dir E:\zircon-build\targets\diagnostic-log-editor-app-check --locked --jobs 1 --message-format short --color never
  - E:\zircon-build\ZirconEngine\zircon_editor.exe smoke run with E:\zircon-build\ZirconEngine\logs\2026-05-04-15-35-18\editor.log
  - E:\zircon-build\ZirconEngine\zircon_runtime.exe smoke run with C:\Users\HeJiahui\AppData\LocalLow\ZirconEngine\ZirconEngine\logs\2026-05-04-15-57-47\runtime.log
doc_type: module-detail
---

# Diagnostic Log

`zircon_runtime::diagnostic_log` is the lightweight process-startup diagnostic sink used by exported editor/runtime binaries. It mirrors each accepted diagnostic line to stderr and, when a writable directory is available, appends the same line to a per-run file under `logs/<yyyy-MM-dd-hh-mm-ss>/<channel>.log`.

The sink is intentionally small and process-local. It is for startup, asset-resolution, and presentation-boundary evidence; it is not a replacement for structured telemetry or the editor runtime diagnostics pane.

## Log Directory Policy

`initialize_process_log("editor")` uses `DiagnosticLogLocation::LocalFirst`. The candidate order is:

1. `ZIRCON_LOG_ROOT/<timestamp>` when `ZIRCON_LOG_ROOT` is set.
2. `<current executable directory>/logs/<timestamp>`.
3. `<current working directory>/logs/<timestamp>` when it differs from the executable directory candidate.
4. The Unity-compatible user log directory as a fallback.

`initialize_unity_process_log("runtime")` uses `DiagnosticLogLocation::UnityCompatibleFirst`. It still honors `ZIRCON_LOG_ROOT` first, then prefers the Unity-compatible user log directory before local executable/current-directory folders.

The Unity-compatible roots are:

1. Windows: `%USERPROFILE%\AppData\LocalLow\ZirconEngine\ZirconEngine\logs\<timestamp>`.
2. macOS: `$HOME/Library/Logs/ZirconEngine/ZirconEngine/logs/<timestamp>`.
3. Linux/Unix: `$HOME/.config/unity3d/ZirconEngine/ZirconEngine/logs/<timestamp>`.

## File And Console Sink

`initialize_process_log_with_location(...)` sanitizes the channel name, opens the first writable candidate as `<channel>.log`, selects the default or overridden level filter, and records which directory was selected. `write_diagnostic_log(scope, message)` is the verbose compatibility helper for detailed startup diagnostics. Explicit calls can use `write_debug_log`, `write_log`, `write_warn`, `write_error`, or `write_diagnostic_log_at` when the call site owns the severity.

Accepted lines are written in this form:

```text
[2026-05-04-15-35-25] [verbose] [editor_host_presenter] present frame=1 frame_size=1280x720 ...
```

Each write replaces embedded newlines with `\n`, writes to stderr, writes to the file when available, and flushes immediately. The immediate flush is deliberate because these diagnostics are primarily used around startup failures, forced smoke-run termination, and crash-adjacent investigations.

## Level Filter Policy

`DiagnosticLogLevel` has five severities: `verbose`, `debug`, `log`, `warn`, and `error`. `DiagnosticLogFilter` either disables the sink with `off` or accepts a minimum level.

The default filter depends on the Rust build profile:

1. Debug builds enable `verbose` and above, so the current asset, template, native host, and presenter diagnostic details are always available during development.
2. Release builds enable `log` and above, so `verbose` and `debug` details are suppressed by default while normal startup log lines, warnings, and errors remain visible.

The global runtime override is `ZIRCON_LOG_LEVEL=verbose|debug|log|warn|error|off`. The `trace`, `info`, `warning`, `err`, `none`, and `quiet` aliases are accepted for command-line friendliness, but documentation and scripts should prefer the canonical names.

The exported editor and runtime binaries also accept `--log-level <level>` and `--log-level=<level>`. Startup arguments take precedence over `ZIRCON_LOG_LEVEL`. The editor strips `--log-level` before parsing headless operation-control arguments, so commands such as `--operation ... --headless --log-level warn` keep their existing operation semantics. The runtime binary currently accepts only diagnostic startup arguments; any remaining argument after diagnostic parsing is rejected as an unknown runtime argument.

## Current Diagnostic Producers

The editor and runtime entry runners initialize the sink before loading the runtime library or constructing the editor host.

The built-in asset resolver logs every root candidate set, selected root, final path, and file-existence result at `verbose`. This is what proves whether exported binaries are reading staged `ZirconEngine/assets`, a development checkout fallback, or a user override from `ZIRCON_ASSET_ROOT` during debug builds or explicit verbose sessions.

The editor template runtime logs built-in `.ui.toml` registration and `res://...` import resolution at `verbose`. The native host window and presenter log `HostWindowPresentationData` arrival, native window creation, frame size, tab counts, pane kinds, and selected frame rectangles at `verbose`. Native window, softbuffer presenter, resize, and present failures are `error` so they still appear under the default release filter.

## Exported Editor Evidence

The 2026-05-04 exported editor smoke run from `E:\zircon-build\ZirconEngine` wrote `E:\zircon-build\ZirconEngine\logs\2026-05-04-15-35-18\editor.log`. That log shows:

1. No `exists=false` or `path_exists=false` diagnostics for the inspected asset/import path.
2. Built-in editor templates and imports selected `E:\zircon-build\ZirconEngine\assets`.
3. `editor_host_window` received populated presentation data before creating the native window.
4. `editor_host_presenter` presented 1280x720 frames with `page_tabs=1`, `document_tabs=2`, and `document_pane_kind=Scene`.

That evidence rules out staged asset-root failure, missing built-in imports, empty/default presentation data, and zero-sized presenter frames for this run. If the exported UI still looks skeletal, the remaining boundary is the current CPU painter in `zircon_editor/src/ui/slint_host/host_contract/painter.rs`, which intentionally draws only minimal chrome markers rather than the full authored workbench template scene.

## Exported Runtime Evidence

The 2026-05-04 exported runtime smoke run from `E:\zircon-build\ZirconEngine` wrote `C:\Users\HeJiahui\AppData\LocalLow\ZirconEngine\ZirconEngine\logs\2026-05-04-15-57-47\runtime.log`. That log selected `source=unity-compatible-user-log-directory` and mirrored the same startup lines to stderr.

The runtime process was still running when stopped by the smoke harness, so the forced `-1` exit code only records test termination. The log location evidence confirms that standalone runtime startup uses the Unity-compatible user log path unless `ZIRCON_LOG_ROOT` overrides it.
