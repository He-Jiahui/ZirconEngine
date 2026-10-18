# Editor review implementation evidence (2026-10-17)

Branch: `fix/claude-review-20261017`. Input: John He's English review
`editor-penpot-ue-20261017.md`.

## Critical batch

C1: Record the virtual-geometry indirect counter upload before borrowing the
native encoder. The upload remains ordered before dispatch. Remove unnecessary
mutable context bindings in both affected executors.

Decision: keep advanced renderer plugins in `target-editor-host` for this repair.
This preserves the existing editor feature closure and project/plugin support.
Acceptance must check that complete closure and the plugin workspace, rather
than hiding failures by removing dependencies.

C2 is deferred: `.github/**` is read-only under the project write policy. No
workflow, branch-protection or permission changes are made by this task. The
review's PR editor compile gate remains required external work.

Linux verification: `git diff --check`; Rust parser/format inspection when
available. This host has Rust 1.85.1 (project: 1.94.1), no physical Windows
`D/E/F:\cargo-targets` storage and no activated Jenkins configuration. No Cargo
compile, Rust test, GUI startup or performance pass is claimed.

Required Windows batch (independent local evidence; not Jenkins acceptance):

```powershell
$t = 'E:\cargo-targets\zircon-local\claude-review-20261017'
& tools/dev/local-cargo.ps1 -TargetDir $t check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_virtual_geometry_runtime --all-targets --locked
& tools/dev/local-cargo.ps1 -TargetDir $t check -p zircon_app --bin zircon_editor --no-default-features --features target-editor-host --locked
& tools/dev/local-cargo.ps1 -TargetDir $t check --manifest-path zircon_plugins/Cargo.toml --workspace --all-targets --locked
& tools/dev/local-cargo.ps1 -TargetDir $t test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_virtual_geometry_runtime indirect --locked
```

## Commands and popup menus (H8, H6, L5)

The shipped keymap now uses Alt+P/Esc and includes Save All. A regression test
compares every descriptor default to the shipped preset and checks predicate-aware
conflicts. The palette's cached search catalog now uses the effective keymap,
including overrides and explicit unbinding. Existing workbench menu projection
already applied effective shortcuts; that behavior is retained.

Legacy shell popup contents are projected from that menu model using ordered
command IDs in `unreal/menus.contract.toml`. Dispatch enters the command boundary
and rechecks enablement. Add exposes all seven implemented node constructors,
which now require a project. Empty, unsupported popups and the dead Pause button
are hidden. Pause/Step and additional authoring/navigation shortcuts remain pending;
no unsupported command is advertised as working.

Linux checks: default-keymap TOML consistency, popup/trigger contract consistency,
`git diff --check`, and Rust **unexpanded AST parsing only** using the installed
compiler (`RUSTC_BOOTSTRAP=1 rustc --edition=2021 -Zunpretty=ast-tree <file>`).
AST output is discarded; no compilation products are created. Rust type checking
and the added behavior tests have not run on this host.

Required focused Windows tests, in addition to the Critical batch:

```powershell
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_editor --lib core::commands --locked
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_editor --lib main_menu_items --locked
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_editor --lib command_palette --locked
```

## Startup batch (H1, H2, L3)

Remove the app's late process-environment mutation. Backend selection belongs to
Runtime: enumerate adapters once per device startup, prefer qualified DX12 then
Vulkan on Windows, and retain an explicit `WGPU_BACKEND` restriction. The shared
chrome presenter reuses the runtime's device; this change does not extend the
dynamic-session ABI. Log the selected API/adapter/type and declare the override
in help and the MVP child-environment allowlist. Standalone chrome device choice
and real shared-presenter startup still require Windows acceptance.

`build_editor.py` now performs a bounded RenderableEmpty GUI launch after its help
precheck, validates the captured RGBA PNG (usable dimensions, luma deviation > 8),
and rejects software fallback or a missing backend diagnostic. A failed startup
keeps a failed smoke receipt and logs. No package success receipt is issued.
The explicit-Python preview launcher and [local instructions](run-editor-locally.md)
retain the BuildSet boundary and approved output-path checks.

Linux verification: `TMPDIR=/workspace/ZirconEngine/tools/tests python3 -B -m unittest
tools.tests.test_editor_startup_smoke tools.tests.test_build_editor_storage_mode_contract
-q`: **24 tests, 9 passed, 15 Windows packaging tests skipped**. These exercise
image validation, launch arguments/environment and failure receipts with mocked
processes, not a real window. Rust AST parsing and `git diff --check` passed.

Required Windows checks: the Critical editor closure plus
`test -p zircon_runtime --lib select_adapter --locked`, followed by the preview
command in the local instructions with `WGPU_BACKEND` unset and then `dx12` and
`vulkan` where available. Capture at 100%/150% DPI. O0 remains **unaccepted** until
real compilation, first-frame and PR-gate evidence exists.
