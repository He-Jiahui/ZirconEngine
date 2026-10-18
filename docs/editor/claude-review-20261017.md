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

## Runtime scheduling and shell-lock scope (H3, partial H5)

Add an explicit runtime-tick redraw and callback. Continuous and timed runtime
demand use it; runtime pumps, Play inspection and viewport image polls are split
from asset/job/settings maintenance. Viewport damage merged with a runtime tick
retains regional presentation without becoming full chrome maintenance. Real
maintenance requests still win and include the runtime pump. Add the PlayInEditor,
GizmoDrag, CameraFly and PlayStart profiling namespaces (the latter three need
automation producers before budgets can be measured).

Release the shell mutex around Play backend start/stop/retirement, including start
failure rollback; preserve the existing cleanup branches. Consumer shutdown on
Stop also runs before taking the shell lock. **This is not asynchronous startup**:
Starting/Stopping worker ownership, cancellation and progress are still required
for H5's <=100 ms inter-frame-gap target.

H4 remains open. The current scene product path uses renderer-owned products,
whereas Play is owned by the dynamically loaded runtime session. Its native
surface `present_viewport` API is not a compositable product export and cannot
simply be substituted for a Game-view image without covering chrome or violating
device/resource ownership. No readback counter is suppressed; a versioned shared
product/device-lifetime contract and Windows GPU tests remain necessary.

Linux verification: Rust AST parsing and diff checks; no Rust type check or timing
pass. Required Windows batch:

```powershell
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_editor --lib retained --locked
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_editor --lib core::play --locked
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_app --test editor_mvp_authoring --no-default-features --features target-editor-host f4_project_authoring_survives_full_application_restart --locked
```

Run 30 s PIE with profiling, `ZIRCON_EDITOR_TRACE_SLOW_REDRAW=1` and
`ZIRCON_EDITOR_TRACE_SLOW_TICK=1`: require <=1 full redraw/s, p95 <=16.7 ms,
p99 <=33 ms. H4 additionally requires `play.preview.copy_bytes=0` on a real shared
GPU path; H5 requires maximum Play-start frame gap <=100 ms. These are pending.

## Preview capabilities and truthful feedback (H7)

Add `editor.ui.show_preview_features` as a persisted User boolean, default false.
Mock module/extension entry points and direct workspace mounts honor it;
responsive toolbar updates preserve the gate. Opt-in navigation is marked Preview.
Turning the setting off retires the mock workspace and restores Scene. Existing
mock-workspace fixtures now opt in explicitly. Working local selection, menu and
search interactions retain their behavior even where the legacy preview-ID list
also classifies them as previews.

Replace extension feedback table receipts and mock module compile/save/simulation
claims with `Not implemented (preview)` or clearly labeled input summaries. The
unimplemented generated-panel Pin action also reports its capability honestly.
A source lint covers 218 canned status entries, rejecting queued/applied/compiled/
saved without a job/receipt placeholder. New Rust behavior tests cover default
visibility, direct admission, setting changes and opt-in badges.

Linux verification: the receipt lint, localization TOML parsing, Rust unexpanded
AST parsing and `git diff --check` pass. Rust behavior/type checking and GUI
visibility acceptance remain pending. Required Windows tests:

```powershell
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_editor --lib core::settings --locked
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_editor --lib preview_features --locked
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_editor --lib module_workspace --locked
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_editor --lib module_tools --locked
```

## Penpot operator contract and compiler scaffold (H9, partial O1/O2)

[Plan 14](../plans/zircon_editor/editor_ui/14-penpot-operator-ui.md) records split
visual/runtime/semantic authority, UI reference mapping, native ownership,
milestones and acceptance budgets. It scopes the exception to plan 01 §1.3 to
the operator shell and retains plan 13's unresolved native parity limits.

Add a 17-role engineering contract and `ui::operator` registry with behavior
tests for required/duplicate/unknown roles, command validation and designer node
renaming. No legacy control-ID literals or source-text guard tests are used in
this module; warnings are denied. Generated role sidecars keep the existing ZUI
schema intact. No operator host/binder/shell-switch stub is installed.

The plugin gains Annotate and ZOP Export for fresh boards, actual local library
foundation tokens, a revision-stable Penpot PNG and a hash manifest in a
deterministic ZIP. The new dependency-free compiler package checks semantics,
layout/paint restrictions, i18n keys and manifest integrity; it emits native ZUI
v2 documents, role sidecars and a provenance/output lock. Compile/verify/apply
have bounded canonical landing paths, storage checks and ownership/hash checks
before replacing or retiring generated files. Preview output cannot be applied.
The managed Penpot validator now includes its tests and existing-bridge schema
parity checks.

Linux verification: **25 Node tests passed**, including determinism, golden tokens,
ZIP CRC/hash verification by Python's standard ZIP reader, fresh-board
normalization, role/command errors, preview exclusion, valid TOML generation,
unsupported-feature rejection, path guards and apply ownership. All compile/export
fixtures stay in memory; the checked-in fixture/golden is explicitly synthetic.
Rust unexpanded AST/TOML parsing and diff checks passed. The Angular/plugin build,
TypeScript checks, managed bridge-parity test, Rust behavior tests and real Penpot
host exports have **not** run here (no installed Penpot dependencies).

No actual Foundations/Components/Screens kit or real bundle was available or
fabricated. SVG/component-state export, connected-library revision manifests,
repeaters and state-expression binders remain unsupported and fail explicitly.
No product assets were generated/applied, O2 is unaccepted, and O3–O6 stay gated
on O0 plus real Penpot/native acceptance. Next commands:

```powershell
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_editor --lib ui::operator --locked
& tools/penpot/tools/scripts/run-validation.ps1 -ArtifactRoot E:\cargo-targets\zircon-local\penpot\zop-<unique>
```

## Controller-derived run controls (partial M8)

The run toolbar now uses the same controller-derived command snapshot as command
admission. Project navigation no longer carries a separate Playing value; old
serialized Playing navigation values reopen as Project. The editor's restoration
checkpoint remains responsible for restoring selection/gizmos, not runtime state.
Building shows disabled Play; CleanupFailed keeps an enabled Retry Stop slot.
Play without a project is visibly disabled. No Pause/Step/Eject backend is added.

Linux verification: changed Rust files parse as unexpanded ASTs; diff checks pass.
Behavior tests cover every existing controller state, no-project admission,
legacy navigation deserialization and retained Stop visibility after cleanup
failure. They await the Windows toolchain, along with the existing play lifecycle
tests. Required commands:

```powershell
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_editor --lib toolbar_run --locked
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_editor --lib core::commands --locked
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_editor --lib core::play --locked
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_editor --lib play_mode --locked
```
