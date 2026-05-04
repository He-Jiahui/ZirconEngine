---
related_code:
  - tools/zircon_build.py
  - Cargo.toml
  - zircon_app/Cargo.toml
  - zircon_runtime/Cargo.toml
  - zircon_runtime/src/asset/runtime_asset_path.rs
  - zircon_runtime/src/diagnostic_log/mod.rs
  - zircon_runtime/src/diagnostic_log/platform.rs
  - zircon_runtime/src/diagnostic_log/sink.rs
  - zircon_runtime/src/diagnostic_log/timestamp.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_app/src/entry/entry_runner/runtime.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_fixture.rs
  - zircon_editor/src/ui/asset_editor/node_projection.rs
  - zircon_editor/src/ui/layouts/views/view_projection.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
  - zircon_editor/src/ui/template_runtime/runtime/build_session.rs
  - zircon_plugins/Cargo.toml
  - zircon_plugins/native_dynamic_fixture/plugin.toml
  - zircon_plugins/native_dynamic_fixture/native/Cargo.toml
  - zircon_runtime/src/plugin/native_plugin_loader/mod.rs
  - zircon_runtime/src/plugin/native_plugin_loader/candidate_from_manifest.rs
  - zircon_runtime/src/plugin/native_plugin_loader/discover_load_manifest.rs
implementation_files:
  - tools/zircon_build.py
  - zircon_runtime/src/asset/runtime_asset_path.rs
  - zircon_runtime/src/diagnostic_log/mod.rs
  - zircon_runtime/src/diagnostic_log/platform.rs
  - zircon_runtime/src/diagnostic_log/sink.rs
  - zircon_runtime/src/diagnostic_log/timestamp.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_fixture.rs
  - zircon_editor/src/ui/asset_editor/node_projection.rs
  - zircon_editor/src/ui/layouts/views/view_projection.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
  - zircon_editor/src/ui/template_runtime/runtime/build_session.rs
plan_sources:
  - user: 2026-05-03 add tools/zircon_build.py for staged editor/runtime/plugin builds
  - user: 2026-05-04 confirm editor/runtime asset staging and exported lookup support
  - user: 2026-05-04 add file-backed exported editor/runtime diagnostics
  - docs/engine-architecture/runtime-editor-pluginized-export.md
  - docs/superpowers/plans/2026-05-01-runtime-interface-cdylib-loader.md
tests:
  - python -m py_compile tools/zircon_build.py
  - python tools/zircon_build.py --help
  - python tools/zircon_build.py --list-plugins
  - python tools/zircon_build.py --targets editor,runtime --out <dir> --mode debug --dry-run
  - python tools/zircon_build.py --targets plugins --plugins native_dynamic_fixture --out <dir> --mode debug --dry-run
  - python tools/zircon_build.py --targets editor,runtime --out E:\zircon-build --mode debug
  - E:\zircon-build\ZirconEngine\zircon_editor.exe smoke run with E:\zircon-build\ZirconEngine\logs\2026-05-04-15-35-18\editor.log
doc_type: workflow-detail
---

# Zircon Build Tool

`tools/zircon_build.py` is the staged local build entry point for producing a
runnable `ZirconEngine` directory from the repository checkout. It builds editor,
runtime, and selected plugins into separate Cargo target directories, then copies
only deployable runtime artifacts into the staged engine directory.

## Output Layout

Given `--out E:\builds\zircon`, the tool writes:

```text
E:\builds\zircon\
  ZirconEngine\
    zircon_editor.exe
    zircon_runtime.exe
    zircon_runtime.dll
    assets\
      ui\
      fonts\
      icons\
      viewport_gizmos\
    plugins\
      native_plugins.toml
      <plugin-id>\
        plugin.toml
        native\
          <native-plugin-dylib>
  targets\
    editor\
    runtime\
    plugins\
      <plugin-id>\
```

`ZirconEngine` is the runnable/staged payload. `targets` contains Cargo
intermediate artifacts and stays outside the runtime payload. This split prevents
Cargo `debug/deps` layout details from leaking into the final engine directory.

The `assets` directory is a merged engine asset root. The build tool stages
`zircon_editor/assets` and `zircon_runtime/assets` into the same payload root so
authored `res://ui/...`, runtime fixture, icon, font, and viewport-gizmo paths
work from the exported directory. If both crate asset roots provide the same
relative file with different bytes, staging fails instead of silently choosing
one copy; identical duplicates are treated as idempotent.

The editor target also stages a sibling `zircon_runtime.dll`/`so`/`dylib`, because
`zircon_editor` resolves the runtime library from `ZIRCON_RUNTIME_LIBRARY` or the
current executable directory. Keeping the library beside the executable fixes the
common local Cargo layout issue where the dynamic library remains under
`debug/deps` and `LoadLibraryExW` cannot find it.

Built-in engine asset lookup follows the staged layout. Runtime/editor code first
honors `ZIRCON_ASSET_ROOT`, then checks `assets` beside the current executable,
then `assets` under the current working directory, and finally falls back to the
crate-local `assets` directory for `cargo run` and unit-test workflows. Editor
call sites pass `zircon_editor/assets` as their development fallback so editor
templates still resolve from source when no staged payload exists.

The lookup returns the first candidate root that contains the requested relative
file. It only falls back to the first existing root when no candidate contains the
file, which keeps a staged payload from masking editor-only development assets and
keeps missing built-ins visible in diagnostics.

## Startup Logs

Exported editor/runtime binaries initialize a lightweight startup diagnostic sink
before their main host work. Each line is mirrored to stderr and, when possible,
to a per-run file named `logs/<yyyy-MM-dd-hh-mm-ss>/<channel>.log`.

Editor logs prefer the staged executable directory. For example, running
`E:\zircon-build\ZirconEngine\zircon_editor.exe` from the staged payload writes a
file such as:

```text
E:\zircon-build\ZirconEngine\logs\2026-05-04-15-35-18\editor.log
```

Standalone runtime logs still honor `ZIRCON_LOG_ROOT` first, then prefer a
Unity-compatible user log directory before local fallbacks:

```text
Windows: %USERPROFILE%\AppData\LocalLow\ZirconEngine\ZirconEngine\logs\<timestamp>\runtime.log
macOS:   $HOME/Library/Logs/ZirconEngine/ZirconEngine/logs/<timestamp>/runtime.log
Linux:   $HOME/.config/unity3d/ZirconEngine/ZirconEngine/logs/<timestamp>/runtime.log
```

Set `ZIRCON_LOG_ROOT` to force both editor and runtime logs under a known root.
The asset resolver, editor template loader, native host window, and native
presenter currently write startup diagnostics there, so exported-display issues
can be classified without guessing whether the failure is asset staging,
presentation data, window creation, or rendering.

## CLI And Interactive Use

The three required decisions are build targets, output directory, and mode:

```powershell
python tools/zircon_build.py --targets editor,runtime --out E:\builds\zircon --mode debug
```

`--targets` accepts `editor`, `runtime`, `plugins`, or comma-separated
combinations. `--mode` is `debug` or `release`. If any required value is missing
and stdin is interactive, the tool prompts for the missing selection; if stdin is
not interactive, it exits with a clear error.

Plugin builds add a fourth selection:

```powershell
python tools/zircon_build.py --targets plugins --plugins native_dynamic_fixture --out E:\builds\zircon --mode debug
```

`--plugins` accepts plugin ids, menu numbers, ranges, `all`, `native`, or `rlib`.
`--plugin-carrier native_dynamic` or `--plugin-carrier rlib_static` filters the
plugin catalog before selection. `--list-plugins` prints the discovered catalog
and carrier classification without building.

Plugin builds require a real staged host. If the same invocation does not build
`editor`, an existing `zircon_editor` executable must already be present under
`<out>/ZirconEngine`; if the invocation builds neither `editor` nor `runtime`, the
runtime library must already be present too. This keeps incremental plugin
compilation tied to a real engine payload instead of producing detached plugin
artifacts with no host to load or link them.

## NativeDynamic Boundary

`native_dynamic` plugins are Rust `cdylib` crates selected through a package
`plugin.toml` and copied into the runtime payload:

```text
ZirconEngine/plugins/<plugin-id>/plugin.toml
ZirconEngine/plugins/<plugin-id>/native/<crate>.dll|so|dylib
ZirconEngine/plugins/native_plugins.toml
```

These libraries are valid dynamic plugin artifacts because they export the native
plugin ABI symbols consumed by `NativePluginLoader`. The loader expects the
package manifest at `plugins/<plugin-id>/plugin.toml` and the dynamic library
under `plugins/<plugin-id>/native/` using the crate name declared by the runtime
or editor module.

The current native ABI is intentionally byte-oriented and manifest-oriented. It
can report package manifests, entry diagnostics, capability negotiation, command
metadata, serialized command callbacks, state callbacks, unload callbacks, and
plugin-owned buffers. It does not pass Rust trait objects, editor state, ECS
objects, `wgpu` objects, or borrowed runtime references across the dynamic
boundary.

## rlib Static Boundary

Most `zircon_plugins/*/{runtime,editor}` crates intentionally build as rlib
crates. They are Rust static-link plugin packages, not dynamic plugin payloads.
Their real behavior enters the engine through `LibraryEmbed` or `SourceTemplate`
builds that call crate functions such as `plugin_registration()` and merge the
resulting registration reports into runtime/editor registries.

The build tool can compile these crates ahead of time into:

```text
<out>/targets/plugins/<plugin-id>/
```

That proves the selected rlib crates and their dependencies are valid static-link
inputs. The tool does not copy rlib outputs into `ZirconEngine/plugins`, does not
generate fake dynamic libraries for them, and does not claim they are loadable by
`NativePluginLoader`.

Turning an rlib plugin into an independently loadable plugin requires a real ABI
adapter milestone first. That adapter must convert the plugin's runtime/editor
registration data into stable DTOs or C ABI records and must not move Rust-only
types, references, or host-owned objects across a dynamic library boundary.

## Validation Scope

Use these fast checks for script changes:

```powershell
python -m py_compile tools/zircon_build.py
python tools/zircon_build.py --help
python tools/zircon_build.py --list-plugins
python tools/zircon_build.py --targets editor,runtime --out E:\builds\zircon-smoke --mode debug --dry-run
python tools/zircon_build.py --targets plugins --plugins native_dynamic_fixture --out E:\builds\zircon-smoke --mode debug --dry-run
```

Use a real build when validating executable staging or NativeDynamic publishing:

```powershell
python tools/zircon_build.py --targets editor,runtime --out E:\builds\zircon-smoke --mode debug
python tools/zircon_build.py --targets plugins --plugins native_dynamic_fixture --out E:\builds\zircon-smoke --mode debug
```

The first command should leave `zircon_editor` and the platform runtime library
as siblings under `ZirconEngine`. The second command should leave a
`plugins/native_plugins.toml` file and a copied native dynamic library under the
selected plugin package.
