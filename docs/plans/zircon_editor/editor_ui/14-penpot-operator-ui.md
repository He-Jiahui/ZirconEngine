# Plan 14: Penpot operator UI

Owner: `zircon_editor::ui::operator`. Authorized by John He's 2026-10-17 review,
section 5. This plan supersedes design plan 01 §1.3 **only for the operator shell**.

## Authority and lifecycle

Penpot files own operator visuals, layout, components, tokens and icons. Reviewed
normalized bundles under `tools/penpot/designs/zircon-operator-ui/bundle` are the
input of record. Generated `.zui` stays the runtime input, with deterministic
provenance and a bundle/output hash lock; it is never edited by hand.
`assets/ui/operator/contract/roles.contract.toml` owns behavior. The plugin and
compiler validate annotations against it; Rust validates every required role,
screen, command and opaque control identity before binding. A generated role
sidecar avoids extending the existing `.zui` schema or encoding legacy control IDs.
The existing reversible bridge retains authority for non-operator UI.

The plugin reads selected fresh Penpot boards without requiring imported `.zui`
metadata. `dev.zircon.zop/binding` stores role/command/slot annotations. A verified
manifest identifies file IDs/revisions, plugin/compiler versions and every input
hash. Export is a deterministic ZIP with normalized JSON and actual Penpot PNGs.
Compilation is pure until the CLI output step. Verify compares all generated
bytes and rejects stale files; apply only replaces paths owned by the prior lock,
preserving the handwritten contract and unowned files. Artifact writes use real
drive-root `D/E/F:\cargo-targets` storage with no aliases or reparse points.

Runtime ownership stays unchanged: Runtime owns worlds, GPU devices and products;
Editor owns authoring selection/history, docking, commands and view projections.
Generated visuals cannot create a new command executor or runtime authority.
The future host loads the role sidecar and `.zui` together, validates them, then
binds roles to existing models/commands with effective keymap/WhenClause checks.
Missing/duplicate roles or unknown commands fail startup with the board path;
hot reload must keep the previous coherent template on failure.

## Layout and interaction contract

Use section 5's S1–S11 state/breakpoint matrix. S2 has a menu bar, toolbar, viewport
dock, right Outliner/Details dock, collapsed bottom Content/Log/Messages dock and
status bar. These become `WorkbenchLayout` nodes. At 1920×1080 the viewport keeps
at least 62% width; the right dock starts at 360 px, minimum 280. Lists scroll in
their own panes, the Content Drawer overlays the viewport, and the viewport keeps
focus for scoped shortcuts. Validate 1280×720, 1920×1080 and 2560×1440 at 100/150% DPI.
Text fields consume text; menus restore focus; drags/scrubs use one history
transaction with Esc cancellation. Preview features are excluded by default.
Unsupported Pause/Step/Eject/authoring verbs stay absent until their executors land.

Primary reference: [App Shell 4](https://pro.reactbits.dev/docs/app-ui/app-shell/app-shell-4)
for content/context relationships. Supporting references:
[Navbar](https://pro.reactbits.dev/docs/app-ui/navbar) for chrome,
[Command Menu](https://pro.reactbits.dev/docs/app-ui/command-menu) for commands,
[File Manager](https://pro.reactbits.dev/docs/app-ui/file-manager) for assets,
[List](https://pro.reactbits.dev/docs/app-ui/list) for virtual collections,
[Agent Activity 5](https://pro.reactbits.dev/docs/app-ui/agent-activity/agent-activity-5)
for log filtering/follow,
[Settings Form](https://pro.reactbits.dev/docs/app-ui/settings-form) and
[App Dialog](https://pro.reactbits.dev/docs/app-ui/app-dialog) for settings/feedback.
These map information architecture only; no licensed source/assets are copied.
No new visual screen is accepted here; rendered reference inspection and real
Penpot/engine parity remain prerequisites for visual implementation.

The initial compiler supports a strict subset of ZUI v2 and fails closed on
unsupported shadows/blur, multiple fills, nonuniform grids, unimplemented
alignment/wrapping, nonlibrary instances/images, unnamed interactive semantics,
or unrecognized state expressions. Errors identify the board/node path. Tokens
map DTCG values to `$editor.*`; no measured status may come from a design sample.
Further layout support requires both runtime parity and compiler tests.

## Milestones and acceptance

| Milestone | Deliverable | Acceptance / present state |
| --- | --- | --- |
| O0 | C1/C2, H1/H2 | Repair code and startup smoke landed; Windows compilation, first frame, backend matrix and PR gate **pending**. |
| O1 | Roles, command menus, keymap, truthful status, perf namespaces | Role validation/menus/keymap/preview feedback implemented; Rust execution and full status projection **pending**. |
| O2 | Penpot kit, Annotate, ZOP Export, compiler/goldens | Tooling scaffold only; actual Foundations/Components files, engine-loaded fixtures and managed validation **pending**. |
| O3 | Generated S2/S3/S6/S7, role binders, shell switch | Blocked on real O0 acceptance; no operator shell is selected or advertised as launchable. |
| O4 | Content/log/PIE, async Play, shared GPU product, authoring verbs | Pending contracts, automation and timing evidence. |
| O5 | Project/settings/asset editors, DPI, default switch | Pending O3/O4 plus parity at 100/150% DPI and F4 restart test. |
| O6 | Remove legacy shell/mocks/control-ID painters | Last: all editor tests, automation and parity must pass before deletion. |

Require compiler byte determinism, manifest integrity and role/command tests before
applying a real bundle. O3 requires L1 create/select/move/undo/redo/duplicate/rename/
delete/save/reopen. O4 requires L2 asset placement/undo, L3 Play/Pause/Step/Stop and
L4 log filtering/registered commands. Verify dock persistence across restart.

Performance budgets at 1920×1080 on the reference Windows GPU host: idle has no
unneeded frame updates or polling; hover p95 <=4 ms; 10k Outliner scroll <=8 ms;
drag/scrub/camera/resize p95 <=16.7 ms, p99 <=33 ms; 30 s PIE <=1 full chrome redraw/s
and zero preview readback bytes with a shared GPU product; Play-start frame gap
<=100 ms. A scenario name alone is not timing evidence.

See [implementation evidence](../../../editor/claude-review-20261017.md) for exact
Windows commands and limitations. No synthetic fixture is Penpot design evidence.
