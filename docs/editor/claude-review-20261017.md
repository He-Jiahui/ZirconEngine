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

C2's original permission deferral was superseded by John He's explicit
2026-10-18 authorization. The hosted PR gate, runner signal, timeouts and branch
protection are implemented; see the C2 follow-up below for evidence and remaining
native acceptance.

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

## Status and snap authority (M2)

Status signals now come from context-owned log and notification services. Log
severity totals count accepted records since the last clear, survive retention
eviction and do not depend on console filters. Messages count pending decisions,
live toasts and active progress notifications; a task projection alone is not a
message. Snapshot fixtures explicitly provide empty signals.

Show T/R/S enabled states and step values independently of grid visibility.
The status magnet toggles translate snapping and reflects its checked state even
through the incremental chrome patch. Default scale snapping is off in both the
settings authority and its standalone value projection; existing saved overrides
remain effective. The legacy combined grid command explicitly writes the per-kind
settings, rather than serving as a second snap authority. Remove the fabricated
100% zoom control and give snap values a wider status slot.

Linux verification: 44 changed/new Rust files passed unexpanded AST syntax
inspection; status ZUI TOML/node references and diff checks passed. Behavior tests
were added for log eviction/clear, real warning emission, toast counts, per-kind
snapping and incremental checked-state updates. Rust tests and rendered spacing
acceptance remain pending:

```powershell
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_editor --lib core::logging --locked
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_editor --lib core::notifications --locked
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_editor --lib status_bar --locked
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_editor --lib unreal_snap --locked
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_editor --lib viewport_chrome --locked
```

## Stable presenter startup and degradation notice (M3, partial M4)

The native event-loop owner now resolves runtime presenter readiness before
creating the window/surface. Poll at 50 ms for at most 2 s (40 polls); use the
shared presenter when ready, otherwise select standalone once for this window.
The event loop remains nonblocking during readiness and removes the readiness
wake deadline after selection. Delete the post-startup surface replacement and
its obsolete source-guard tests. Surface creation itself still runs on the
window thread; its driver latency needs Windows measurement.

Creation errors retain their actual cause through shared → standalone GPU →
softbuffer fallback. Both CPU viewport capture and software presentation now
produce warning diagnostics, fallback counters and a persistent non-modal banner
in the legacy shell. The banner explicitly warns about viewport/PIE performance;
its tooltip preserves the full diagnostic. Adapter selection details remain in
the runtime log, rather than being fabricated by the UI host. Per-Play-start
toast/log notification and inline adapter-name projection remain M4 debt.

Linux verification: 14 changed/new Rust files passed unexpanded AST syntax
inspection, banner ZUI TOML/node references and diff checks passed. Added behavior
tests cover the timeout/poll cap, no wakes after selection, burst input, immutable
selection, and persistent banner projection. They require Windows execution:

```powershell
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_editor --lib presenter_startup --locked
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_editor --lib graphics_degradation --locked
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_editor --lib contract::window --locked
```

Capture normal/shared and `ZIRCON_PROFILE_FORCE_SOFTBUFFER=1` profiling builds;
verify the banner is readable, `presenter_selected` occurs once, no upgrade
markers occur, and readiness causes no 20 Hz idle wakes after selection. These
startup, rendering and idle-CPU checks are pending real-host acceptance.

## Shipping startup diagnostics (M11)

Shipping Windows editor builds use the windows subsystem; development builds
keep the console. Runner failures are written to the process log before flush,
including argument failures that precede logger initialization. A native shipping
GUI failure dialog shows the actual diagnostic/recovery text and log filename
(or explicitly says the file is unavailable). Terminal exit classification is
unchanged. Help, commandlets, first-frame automation and an explicit
`ZIRCON_EDITOR_STARTUP_DIALOG=0` remain non-modal; the failure-path integration
test sets that override.

Linux verification: binary/helper/test Rust AST syntax and Cargo TOML parsing
passed; Windows linking and the native dialog have not run. Remaining commands:

```powershell
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_app --bin zircon_editor --no-default-features --features target-editor-host --locked
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_app --test f0_editor_product_startup_diagnostics --no-default-features --features target-editor-host --locked
& tools/dev/local-cargo.ps1 -TargetDir $t check -p zircon_app --bin zircon_editor --no-default-features --features shipping-editor --locked
```

Stage a shipping editor, remove its runtime manifest in an isolated staged test
copy, launch from Explorer, and verify the visible recovery/log path and original
exit code. Do not use the native dialog as evidence of a successful startup.

## Pointer origin and wheel routing (L2)

Normalized button coordinates are authoritative, including `(0,0)`. Native wheel
events without a known cursor position are rejected instead of being delivered
to the top-left control. Cursor leave and focus loss clear the cached position.
The synthetic normalized-pointer test entry explicitly seeds the position supplied
by its caller; native wheel handling still requires a real move/button position.

Linux verification: four Rust files passed unexpanded AST syntax checks, the test
module path resolves, and diff checks passed. Added behavior tests exercise an
origin click after a different previous position, wheel-before-move rejection,
known-position wheel routing, and wheel rejection after focus loss. Execute:

```powershell
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_editor --lib event_loop::events::pointer --locked
```

## First-frame presenter evidence follow-up (H2, M3/M4)

The smoke now requires exactly one final `presenter_selected` startup marker and
requires its backend to be GPU. A runtime GPU adapter diagnostic alone cannot
prove that the window used GPU presentation: forced softbuffer can coexist with
that diagnostic. Missing/repeated selections and forced software presentation
fail even without a fallback warning. The receipt records the final backend and
whether it actually shared the runtime device; standalone GPU startup does not
claim shared presentation or zero PIE copies.

Linux verification: focused Python startup-smoke behavior tests passed, including
forced software, missing/repeated selection, standalone GPU and failed receipts.
These are mocked launches; the real Windows first-frame gate remains pending.

## Windowing version boundary (L4)

Pin the existing `winit` dependency exactly to `=0.31.0-beta.2`. Both workspace
lockfiles already resolve that version, so no dependency update or windowing API
migration is included. TOML parsing and manifest/lockfile agreement passed.

The future stable upgrade is a separate window-lifecycle batch: check the editor
feature closure, run `contract::window` tests, and verify initial creation,
resize/minimize/restore, focus, pointer origin/wheel, keyboard/IME and 100/150% DPI
on the real Windows host. Capture a first frame using the preview command after
that upgrade. No beta-to-stable migration or native acceptance is claimed here.

## Closeout and remaining acceptance

The source batches are committed and published on `fix/claude-review-20261017`.
The pre-existing dirty Linux `.codex/config.toml` overlay is preserved byte for
byte and excluded from every commit. No compiler output was created on this
Linux host. Its permission profile is disabled; repository write-policy
compliance here is not evidence of sandbox enforcement.

| Finding | Landed source / remaining work |
| --- | --- |
| C1 | Borrow ordering repaired; advanced plugins retained. Full editor/plugin compilation and any downstream failures remain unverified. |
| C2, M1, L7 | Original permission checkpoint superseded by the 2026-10-18 authorization; see the authorized follow-ups below. |
| H1, H2, L3 | Backend policy, launch tooling, capture validator and diagnostics implemented. Real backend selection/window/capture acceptance pending. |
| H3 | Runtime redraw/maintenance split and perf namespaces implemented. Continuous PIE timing/idle behavior pending. |
| H4 | Open: dynamically owned PIE product needs a compositable shared GPU/device-lifetime contract; CPU readback remains. |
| H5 | Shell-lock scope repaired. Open: worker-owned Starting/Stopping, progress, cancellation and <=100 ms frame-gap acceptance. |
| H6, L5 | Command-driven popup contents/admission and project predicates implemented; unsupported Pause hidden. Rust/GUI interaction acceptance pending. |
| H7 | Preview opt-in and truthful feedback implemented; lint passes. Real visibility/dispatch tests pending. |
| H8 | Alt+P/Esc/Save All and effective-keymap palette/menu projection implemented. Follow-up adds F/Ctrl+Space and Q/W/E/R commands; additional authoring/navigation shortcuts remain deferred below. |
| H9 | Plan, role contract, Annotate/export/compiler scaffold and synthetic goldens landed. No real Penpot kit/export or generated operator shell accepted. |
| M2 | Real severity/notification counts and per-kind snap projection implemented. Rust tests/rendered spacing pending. |
| M3 | Bounded readiness and immutable startup presenter selection implemented. Real driver latency/idle/presenter tests pending. |
| M4 | Degradation warning/banner/counters and per-Play-start warning implemented. Inline adapter projection remains deferred below. |
| M5 | Q/W/E/R commands added. Fly/Alt navigation and speed controls explicitly deferred below. |
| M6 | Clipboard/duplicate/visibility/lock, asset placement and generalized Details explicitly deferred below; hierarchy-local rename remains. |
| M7 | Main-window docking migration explicitly deferred to O3 after O0 acceptance. |
| M8 | Controller-derived run admission/state implemented. Pause/Step/Eject and asynchronous state UI remain open. |
| M9 | Deferred at the report's `zircon_editor/build.rs` authorization checkpoint. |
| M10 | New operator tests use behavior, and obsolete presenter guards were removed. Startup happy-path tooling exists; Rust tests, real automation and CI wiring are pending. |
| M11 | Shipping subsystem/log/dialog source implemented. Windows linking and Explorer failure-path acceptance pending. |
| L1 | New operator module denies warnings; some touched unused bindings removed. Broad warning/clippy debt remains. |
| L2 | Pointer-origin/unknown-wheel behavior repaired; Rust regression execution pending. |
| L4 | Existing beta pinned exactly. Stable migration and native lifecycle acceptance pending. |
| L6 | Added actual-painter U+0020 regression coverage and traced asset-summary counts. Rust execution and current-shell 100/150% DPI captures remain pending. |

Reusable local evidence: the Penpot compiler/export batch has **27 passing Node
tests**. The final Python startup/storage batch ran **25 tests: 10 passed and 15
Windows-only skipped**. Preview-feedback lint and structural TOML/AST/diff
checks passed in the relevant batches. Rust AST inspection is syntax evidence,
not type checking, compilation or execution.

O0 and O2 are **unaccepted**. O1 is partial; O3–O6 remain gated. The exact scoped
Windows commands are recorded above, and the staged preview/capture command is
in [the local launch guide](run-editor-locally.md). Compile, successful native
startup, continuous-runtime/PIE jank budgets and real operator UI parity remain
required acceptance work; this closeout does not mark those goals complete.

## Main merge and command follow-up (2026-10-18)

Merge `5f8a8ca92` combines the branch's counter reset before the native borrow
with main's scoped factory/encoder borrows. It retains the source buffer, bind
group and pipeline cache guard through dispatch, then releases the native borrow
before recording the dispatch. `.codex/config.toml` has no change relative to the
first parent. The working overlay still matches `stash@{0}`; the stash is retained.

H8 / partial M5: Frame Selection (F), Content Drawer (Ctrl+Space), and Select /
Translate / Rotate / Scale (Q/W/E/R) now use registered commands and the effective
keymap. Remove the two native shortcut bypasses. Scene commands require a project
and Scene document focus; their event resolves the focused live Scene leaf at
execution, preserving multiple-view ownership and rejecting stale/missing targets.
The drawer uses a transient host intent, including effect coalescing and focus
restoration. Both shipped locales include the command and menu labels.

Added behavior tests cover descriptor/event mapping, keymap overrides/unbinding,
focus/no-project/headless admission, multiple Scene targets, event serialization
and drawer-toggle parity. Linux verification: changed Rust files passed unexpanded
AST parsing; changed TOML parsed; `git diff --check` passed. The startup/storage
Python batch ran **25 tests: 10 passed, 15 Windows-only skipped**:

```sh
TMPDIR=/workspace/ZirconEngine/tools/tests python3 -B -m unittest tools.tests.test_editor_startup_smoke tools.tests.test_build_editor_storage_mode_contract -q
```

Rust compilation/tests remain unexecuted: installed Rust is 1.85.1 against the
project's 1.94.1; this host has neither physical D/E/F cargo-targets storage nor
an activated Jenkins coordinator. Do not create Linux lookalike output roots or
treat AST parsing as a compile pass. In addition to the Critical Windows batch:

```powershell
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_editor --lib core::commands --locked
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_editor --lib target_serialization_tests --locked
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_editor --lib editor_event_execution::viewport_event --locked
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_editor --lib event_bridge --locked
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_editor --lib dispatch::common::effects --locked
```

Native acceptance: rebind/unbind F and Ctrl+Space, focus each of two Scene views,
exercise Q/W/E/R, type those letters in text fields, and verify the focused Game
receives its gameplay keys while Esc stops Play. Menu/palette shortcuts must show
the overrides. These checks remain pending.

H4 remains explicitly deferred: the App-owned dynamically loaded Play session
exports CPU frames and a native surface, not a texture lease compositable on the
chrome device. A repair needs a versioned shared-device/product lease covering
submission, resize, device loss and session retirement across the ABI, plus real
GPU tests. The existing Scene renderer's product handles cannot safely stand in
for those leases. CPU capture and its counters remain truthful.

H5 remains explicitly deferred: `request_play` currently combines activation,
edit protection, backend start, gateway attachment and rollback under its transition
gate. The UI subsequently attaches consumers and the selection domain. Moving the
whole call to a thread would publish Playing before UI completion, race Stop and
shutdown, and leave cancellation/retirement unspecified. A complete change needs
Starting/Stopping ownership, a cancel/completion generation, UI-thread commit and
all rollback tests before native frame-gap acceptance. Shell-lock release remains
landed; no asynchronous-start claim is made. M8 Pause/Step/Eject depends on that
lifecycle and an authoritative runtime stepping/possession contract.

H8's Space tool cycling, Ctrl+D/F2 authoring commands, G/F11/End and additional redo
binding remain deferred until their focused-view or undoable executors exist.
Ctrl+Shift+Z remains the existing rebindable redo default; no second command or
hardcoded Ctrl+Y path is introduced. F2's existing hierarchy-local rename remains
available; migrating it requires preserving its draft/commit/cancel lifecycle.

At this earlier command batch, C2 was still permission-deferred and no protected
CI paths were edited. The later authorized C2 follow-up supersedes that checkpoint.
O0 is unaccepted, so O3–O6 remain gated; the command batch advances O1 source only,
pending its Rust and native checks.

## Penpot compiler follow-up (H9, 2026-10-18)

The compiler previously accepted a command role on a Label/Panel, emitted a Click
event, then disabled input on the same node. It now rejects such an annotation
with the board path; supported command controls remain Button/IconButton. Both
new regressions failed before the fix and pass afterward. The full in-memory
compiler/export batch now passes **27 Node tests**, including the existing golden,
determinism, ZIP integrity, fresh-board and apply-ownership checks:

```sh
node --test --test-reporter=spec tools/penpot/apps/zircon-zop-compiler/src/compiler.test.mjs tools/penpot/apps/zircon-zop-compiler/src/export.test.mjs
```

No real Penpot document/bundle or installed Angular/TypeScript dependencies are
present. O2 therefore remains unaccepted: Node fixtures prove the supported
compiler subset only. The Windows managed validator and Rust `ui::operator`
commands above remain required, followed by actual kit export and engine loading.

## Degraded Play notification (M4, 2026-10-18)

Successful UI Play/Simulate startup emits a committed-start effect only after the
live backend and consumer/selection setup complete. The retained host uses the
existing graphics degradation reason to write a warning to stderr and the editor
log, and to publish a warning toast. Its identity includes the event sequence,
so an earlier live warning cannot suppress a later Play start. The persistent
banner retains the full diagnostic; long Unicode toast text is bounded safely.
Normal graphics, unchanged requests and non-live states produce no start warning.

Added tests cover start-effect admission, event-sequence propagation, unique notice
identities, normal/degraded paths and long Unicode diagnostics. Changed Rust AST,
locale TOML and whitespace checks passed on Linux; behavior execution is pending:

```powershell
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_editor --lib play_graphics_warning --locked
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_editor --lib play_start_notice --locked
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_editor --lib workbench_notifications --locked
```

Force `ZIRCON_PROFILE_FORCE_SOFTBUFFER=1` in a profiling build and start/stop Play
twice before the first toast expires. Each successful start must have a warning
log and a distinct notice; the banner must remain. Repeat normal shared startup
and require no degradation notice. These native checks remain pending.

Inline adapter/backend projection is deferred: the host's presenter factory and
Editor gateway have no cheap immutable selected-device summary. Profiling RPC is
not an always-on UI adapter authority, and standalone chrome can select a different
device from Runtime. Runtime logs keep the actual adapter evidence; a proper UI
summary must identify both devices without guessing from the requested backend.

## Remaining Medium/Low dispositions (2026-10-18)

These are explicit deferrals, not completed features or additional permission
requests. The later authorized follow-ups supersede the C2/M1/L7/M9 permission
deferrals.

- **M5:** defer held-key fly, Alt navigation and camera speed. `ViewportInput`
  currently has pointer/resize events only; Scene Modes run before camera input,
  and the Game view has a separate runtime keyboard/focus path. A coherent change
  must route key down/up/modifiers to the correct Scene, clear held state on
  capture/focus loss, tick movement with elapsed time, and preserve SIE and
  text-field routing. It needs executable controller/focus tests and the real
  CameraFly timing scenario, unavailable on this host. The registered Q/W/E/R
  tools are the landed subset; no unused fly-input variants or speed UI are added.
- **M6:** defer the remaining authoring expansion. `EditorIntent` supports
  create/delete/rename/reparent/transform/inspector edits, but has no clipboard,
  duplicate or visibility/lock operations. Duplication and asset placement need
  stable node/component identity and resource-reference remapping, atomic history
  replay and selection restoration. Generalized Details also needs reflected
  property writes, defaults and multi-selection transactions. The existing
  hierarchy rename draft lifecycle remains available. Adding menu bindings alone
  would advertise missing executors; land each real operation with executable
  undo/redo tests and L1/L2 automation before exposing it.
- **M7:** defer the main-window migration to O3. The legacy `main_band`/`right_dock`
  is still a fixed template; existing `WorkbenchLayout` tab/split/persistence
  machinery does not make those panes movable by itself. It needs role-bound
  panel identities, drag/floating ownership and persisted-layout restoration in
  the generated shell. O0 is unaccepted and no real Penpot bundle is available,
  so a parallel handwritten operator shell would bypass the agreed migration.
- **M8:** the controller-derived admission/projection repair is landed; Pause,
  Step, Eject/Possess and asynchronous state UI stay deferred with H5's lifecycle
  and runtime time/possession contracts.
- **M10:** new operator tests exercise parsed contracts and command state; there
  are **zero Rust-source `include_str!` guards** under `ui/operator`. Preserve
  legacy guards until their code is actually replaced. The first-frame smoke
  exists, but real GUI editing automation and its MVP-lane run are deferred:
  this host cannot launch the Windows product, and lane wiring is protected.
- **L1:** `ui/operator` denies warnings and the previously identified unused
  bindings were removed. Broad warning burn-down stays deferred until current
  Rust 1.94.1 diagnostics are available; the old warning totals are not a current
  baseline. Do not suppress warnings or remove apparently unused feature-gated
  code without checking the supported feature closure.
- **L2–L5:** source repairs remain as recorded above. Rust/native acceptance,
  especially a future winit stable migration, remains pending.

Windows follow-up for M5/M6/M7/L1, after the Critical batch:

```powershell
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_editor --lib scene::viewport::controller --locked
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_editor --lib editing --locked
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_editor --lib ui::workbench::layout --locked
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_editor --lib ui::operator --locked
& tools/dev/local-cargo.ps1 -TargetDir $t clippy -p zircon_editor --lib --locked -- -D warnings
```

Passing existing tests would establish a baseline, not acceptance of the deferred
features. After implementing them, run the review's CameraFly and L1–L4 scenarios;
drag Details below the viewport, restart and verify layout persistence.

## Text spacing and asset-summary investigation (L6, 2026-10-18)

The new `retained_text_label_preserves_u0020_advance_in_painted_pixels` regression
uses the actual retained painter and in-memory RGBA frames at 100%/150% label
sizes. It checks that one and two U+0020 spaces move the following glyph by their
measured advances without moving the first glyph. It does not export a synthetic
frame as current-shell evidence. Both changed Rust files passed AST parsing and
Change Guard content scan with no findings; the Rust test has **not executed**.

The old literal `4 assets in ui` is absent from the current asset browser. The
current `asset_browser.rs` catalog summary uses the visible folder/asset counts.
Its selected-folder header instead uses `selected_folder_breadcrumb` and the
catalog's recursive asset count. `AssetWorkspaceState::build_snapshot` filters
the visible list independently, so the folder total need not equal a filtered
list or the number of materialized tiles. Existing breadcrumb behavior tests
cover the catalog count and parent-chain precedence. This trace does not reproduce
or establish the cause of the historical screenshot mismatch; keep the visual
count check pending rather than change a count authority speculatively.

```powershell
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_editor --lib retained_text_label_preserves_u0020 --locked
& tools/dev/local-cargo.ps1 -TargetDir $t test -p zircon_editor --lib selected_folder_breadcrumb --locked
```

Current-shell captures remain deferred to a real Windows GPU/DPI host. After the
staged preview in [the launch guide](run-editor-locally.md), use the existing
source-bound visual runner. Populate `$visualArgs` from the actual managed build
and prepared normal-frame case inputs: `BundleDirectory`, `ProjectRoot`,
`ExpectedEditorSha256`, `ExpectedRuntimeSha256`, `ExpectedSourceSha256`,
`NormalFrameCaseProducerConfiguration` and `NormalFrameContractConfiguration`.
The expected source hash must be captured **before** that build. Run each command
on a display at its corresponding real Windows scale, using fresh output paths:

```powershell
& tools/analysis/visual/capture-editor-ui-visual.ps1 @visualArgs -DpiProfile '100' -OutputDirectory 'E:\cargo-targets\zircon-local\claude-review-20261017-dpi100'
& tools/analysis/visual/capture-editor-ui-visual.ps1 @visualArgs -DpiProfile '150' -OutputDirectory 'E:\cargo-targets\zircon-local\claude-review-20261017-dpi150'
```

Review spacing, hierarchy overlap/truncation and unfiltered/filtered folder counts
against the published snapshot; retain source/binary hashes and pixel-association
receipts with dated PNGs. No new screenshots, Rust execution, O0 acceptance or
Penpot milestone acceptance are claimed by this Linux follow-up.

Final scoped static review: Change Guard `scan` passed on all **24 changed Rust
files since the main merge**, including the new test, with zero errors, warnings
or suppressions. This content/reachability scan omits write-grant validation;
the Linux config stub has no `zircon-strict` profile, so it cannot supply a full
agent patch-mode permission check. Documentation links/fences/finding coverage
and `git diff --check` also passed. The original stash and idle PID 100961 remain
preserved.

## Authorized PR compile gate (C2, 2026-10-18)

John He explicitly authorized `.github/**` and the related protected CI/build
findings, incremental commits and pushes on this branch. No allowlist or Linux
config-stub change is included. `editor-compile-gate` now runs on PRs using
GitHub-hosted `windows-2022`, with approved physical D: roots, Rust 1.94.1, a
60-minute timeout and the exact locked editor/plugin checks plus command tests
from C2. The existing wrapper rejects vacuous filtered test passes.

Every persistent Windows job in CI and Profile Feature Contract now has a
120-minute timeout and depends on a three-minute hosted runner-availability
probe. Its inventory handling is behavior-tested, including pagination and
missing credentials. Repository API inspection found **zero registered runners**
and **no repository secrets**. Provisioning a managed Windows X64 runner and a
fine-grained `RUNNER_STATUS_TOKEN` with Administration:read remains operator work;
missing either now yields a visible failure before persistent jobs are queued.

The GitHub protection API accepted and read back an up-to-date-branch requirement
for the `editor-compile-gate` check, bound to GitHub Actions app 15368, on `main`.
No review-count, deployment or administrator-enforcement requirement was added.

Local validation: all **20** `.github/tests` tests and the existing focused
CI/toolchain contract pass; `git diff --check` passes. The portable commands,
full Windows reproduction, hosted inspection and runner setup are documented in
[Editor CI validation](ci-validation.md). Rust compilation, the positive hosted
PR run and an actual E0499 negative compilation probe are not claimed here.
The review's throwaway broken-branch probe cannot be performed while staying
exclusively on the requested branch with coherent verified pushes. O0 remains
unaccepted pending real Windows compilation and first-frame evidence.

## MVP job boundaries and cache reuse (partial M1, 2026-10-18)

The MVP workflow now separates `compile-gate` (60 minutes), `focused-tests`
(F1–F4, 90 minutes) and `mvp-editor-windows` (staged F5 acceptance, 120 minutes).
Dependencies stop later jobs after earlier failures. All three use
`windows-2022`, set `WGPU_BACKEND=dx12`, upload failure diagnostics and retain
nonzero/exact-test admission. The staged job collects the preceding diagnostics.
Targets are stable per toolchain, lockfile/config hash and job under physical
`D:\cargo-targets\zircon-local\ci\cache`; run-bound Cargo/rustup/temp directories
remain separate. Cache identity is resolved **after checkout**, before any Cargo
operation. The guard checks approved roots before restore and rejects reparse
ancestors/descendants after restore. Cache bytes are compiler reuse, never a
passing test, product BuildSet or acceptance receipt.

M1 remains **partial and blocked at product publication/receipt transfer**.
`Build-MvpProductInputs.ps1` calls `tools/dev/zircon-session.ps1`, which exits 3
because the local coordinator is retired, then would invoke the retired
`validate-matrix.ps1`, which throws. No active Jenkins deployment exists here or
on the GitHub-hosted image. Those publishers must be migrated to the independent
source/BuildSet owner before a real built-product artifact can cross jobs. The
current versioned F5 registry also requires source-fingerprint-bound workspace
build/test summaries. Removing its workspace test or substituting a green CI
status would weaken acceptance, so the existing F5 requirement remains intact.
The trusted self-hosted CI `rust` job already owns the wave workspace test, but
its result currently has no compatible F5 receipt handoff. Single-unification
focused executables and cross-job product reuse remain part of that migration;
the existing focused target/filter invocations are preserved here.

Validation: **22** CI/runner tests pass. A portable check reproduces all **108**
single-quoted Pester workflow pattern assertions successfully; actual Pester
execution is pending Windows. SHA-256-verified actionlint 1.7.12 accepts all four
workflows. No Rust, GPU, cache-hit timing or F5 acceptance is claimed.

Draft PR #22 is open for the authorized review follow-up. Repository inspection
then found CI, MVP and Profile workflows manually disabled. The repaired **CI**
workflow has been enabled so the new required PR gate can execute on the next
push. MVP/Profile activation and native acceptance remain pending their actual
runner/publisher prerequisites. This changes workflow activation, not runner
registration or retired-service state.

## Build-script inputs (M9, 2026-10-18)

`zircon_editor/build.rs` no longer recursively tracks `assets` or the plugin
source tree. It tracks the six production-embedded TOML resources, the two
explicit gizmo PBMs, plugin catalog manifests and `zircon_plugins/Cargo.toml`
for package membership. Every current top-level plugin-manifest owner is
represented in that workspace. Add/remove catalog packages through that membership
manifest; metadata changes continue to invalidate their individual inputs. Runtime
ZUI loading and the existing asset watcher keep visual edits out of this generator.

All three new invalidation regressions failed on the prior build script and pass
with the repair. The focused existing generated-catalog and role-admission
contracts also pass (**5 focused Python tests** total). Rust unexpanded AST parsing,
Change Guard scan (zero findings) and whitespace checks pass. Two broader legacy
source-contract modules exposed four preexisting stale-path errors outside these
focused build checks; these are recorded separately rather than counted as passes.
Actual Cargo freshness/hot reload remains pending Windows:

```powershell
$t = 'D:\cargo-targets\zircon-local\claude-review-20261017-m9'
& tools/dev/local-cargo.ps1 -TargetDir $t build -p zircon_editor --lib --locked -v
if ($LASTEXITCODE -ne 0) { throw 'Initial editor build failed' }
(Get-Item zircon_editor/assets/ui/editor/unreal/level_editor_window.zui).LastWriteTimeUtc = [DateTime]::UtcNow
& tools/dev/local-cargo.ps1 -TargetDir $t build -p zircon_editor --lib --locked -v
if ($LASTEXITCODE -ne 0) { throw 'Freshness check failed' }
```

Require `Fresh zircon_editor` after the ZUI timestamp change. Touch an embedded
TOML/PBM and a catalog `plugin.toml` in turn and require invalidation. Test targets
that embed a ZUI may rebuild themselves; the non-test editor library is the M9
contract. Use the staged preview from the launch guide to verify actual ZUI hot
reload; no build/GUI execution is claimed on this Linux host.
