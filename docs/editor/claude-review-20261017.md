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
