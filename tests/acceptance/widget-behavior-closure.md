---
related_code:
  - zircon_runtime_interface/src/ui/dispatch/pointer/effect.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/invocation.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/result.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/component_event.rs
  - zircon_runtime/src/ui/dispatch/pointer/dispatcher.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/tree/node/interaction.rs
  - zircon_runtime/src/ui/template/build/interaction.rs
  - zircon_runtime/src/ui/tests/event_routing.rs
  - zircon_runtime/src/ui/tests/hit_grid.rs
  - zircon_runtime/src/ui/tests/template.rs
implementation_files:
  - zircon_runtime_interface/src/ui/dispatch/pointer/effect.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/invocation.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/result.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/component_event.rs
  - zircon_runtime/src/ui/dispatch/pointer/dispatcher.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/tree/node/interaction.rs
  - zircon_runtime/src/ui/template/build/interaction.rs
plan_sources:
  - user: 2026-05-06 集中完善 widget 组件行为闭环，参照 dev 下虚化源码
  - docs/superpowers/specs/2026-05-06-widget-behavior-closure-design.md
  - docs/superpowers/plans/2026-05-06-widget-behavior-closure.md
  - .codex/plans/Zircon UI 与 Unreal Slate 差异审计及后续里程碑.md
  - .codex/plans/Material UI + .ui.toml 全链路 UI 系统推进计划.md
  - .codex/plans/Shared Slate-Style UI Layout, Render, And Hit Framework.md
tests:
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime/src/ui/tests/event_routing.rs
  - zircon_runtime/src/ui/tests/hit_grid.rs
  - zircon_runtime/src/ui/tests/template.rs
doc_type: testing-guide
---

# Widget Behavior Closure

## Scope

- Shared Runtime UI widget/component pointer behavior closure for hover, press, focus, capture, release-inside click, scroll fallback, diagnostics, and metadata-driven interaction inference.
- The accepted layer is `zircon_runtime_interface::ui` pointer DTOs plus `zircon_runtime::ui` shared dispatch/surface/template behavior.
- Editor host, painter, render command, Material layout, popup/menu, drag/drop, IME, and full keyboard text editing cutovers remain outside this acceptance slice.

## Reference Evidence

- Unreal Slate reference: `SWidget`, `FReply`, `FSlateApplication`, `FHittestGrid`, and `FWidgetPath` files listed in `docs/superpowers/specs/2026-05-06-widget-behavior-closure-design.md`.
- Slint Material reference: state-layer, button, text-field, menu, dialog, and navigation files listed in the same design spec.

## Test Inventory

- `zircon_runtime_interface` pointer reply/effect contract tests cover constructors, new result defaults, diagnostics, component event reasons, and serde-default compatibility.
- `zircon_runtime::ui::tests::event_routing` covers release-inside click, release-outside rejection diagnostics, physical hit-path click resolution under capture, same-target hover idle diagnostics, focus/blur envelopes, capture release reply effects, captured-dispatch exclusivity, scroll fallback diagnostics including no-op and ancestor fallback boundaries, dispatch reply effects, shared input routing, and template-built custom click envelopes.
- `zircon_runtime::ui::tests::hit_grid` remains the focused regression set for visibility/clip/disabled hit-grid behavior that pointer routing depends on.
- `zircon_runtime::ui::tests::template` covers binding/metadata-driven interaction inference, including custom bound components, event-kind capability precision for scroll/focus/hover/click bindings, explicit focusable metadata, unbound visual non-interaction, and explicit opt-out from the temporary legacy button fallback.

## Validation Log

- 2026-05-06 10:01 +08:00: `cargo clean --target-dir "E:\zircon-build\targets\widget-behavior-closure"`: removed the scoped target because the `E:` drive had less than the repository policy's 50 GB free-space threshold before the final Cargo gate.
- 2026-05-06 10:01 +08:00: `rustfmt --edition 2021 --check "zircon_runtime_interface/src/ui/dispatch/pointer/effect.rs" "zircon_runtime_interface/src/ui/dispatch/pointer/invocation.rs" "zircon_runtime_interface/src/ui/dispatch/pointer/result.rs" "zircon_runtime_interface/src/ui/dispatch/pointer/component_event.rs" "zircon_runtime_interface/src/ui/dispatch/pointer/mod.rs" "zircon_runtime_interface/src/tests/contracts.rs" "zircon_runtime/src/ui/dispatch/pointer/dispatcher.rs" "zircon_runtime/src/ui/surface/surface.rs" "zircon_runtime/src/ui/template/build/interaction.rs" "zircon_runtime/src/ui/template/build/tree_builder.rs" "zircon_runtime/src/ui/tests/event_routing.rs" "zircon_runtime/src/ui/tests/hit_grid.rs" "zircon_runtime/src/ui/tests/shared_core.rs" "zircon_runtime/src/ui/tests/template.rs" "zircon_runtime/src/tests/ui_boundary/runtime_host.rs"`: passed with no output.
- 2026-05-06 10:03 +08:00: `$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo test -p zircon_runtime_interface --lib contracts --locked --jobs 1 --target-dir "E:\zircon-build\targets\widget-behavior-closure" --message-format short --color never -- --nocapture`: passed, `45 passed; 0 failed; 3 filtered out`.
- 2026-05-06 10:04 +08:00: `$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo check -p zircon_runtime_interface --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\widget-behavior-closure" --message-format short --color never`: passed.
- 2026-05-06 10:26 +08:00: `$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir "E:\zircon-build\targets\widget-behavior-closure" --message-format short --color never -- --nocapture`: the first post-clean attempt timed out after 10 minutes while compiling dependencies and produced no test result; warmed rerun passed, `16 passed; 0 failed; 860 filtered out`, with existing runtime warning noise.
- 2026-05-06 10:28 +08:00: `$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo test -p zircon_runtime --lib hit_grid --locked --jobs 1 --target-dir "E:\zircon-build\targets\widget-behavior-closure" --message-format short --color never -- --nocapture`: passed, `11 passed; 0 failed; 865 filtered out`, with existing runtime warning noise.
- 2026-05-06 10:28 +08:00: `$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo test -p zircon_runtime --lib ui::tests::template --locked --jobs 1 --target-dir "E:\zircon-build\targets\widget-behavior-closure" --message-format short --color never -- --nocapture`: passed, `12 passed; 0 failed; 864 filtered out`, with existing runtime warning noise.
- 2026-05-06 10:28 +08:00: `$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo test -p zircon_runtime --lib shared_core --locked --jobs 1 --target-dir "E:\zircon-build\targets\widget-behavior-closure" --message-format short --color never -- --nocapture`: passed, `36 passed; 0 failed; 840 filtered out`, with existing runtime warning noise.
- 2026-05-06 10:36 +08:00: `$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\widget-behavior-closure" --message-format short --color never`: passed with existing runtime warning noise.
- 2026-05-06 10:44 +08:00: `$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\widget-behavior-closure" --message-format short --color never`: passed with existing runtime/editor warning noise.
- 2026-05-06 10:53 +08:00: final review identified capture fallthrough and scroll-fallback no-op/ancestor risks. New `event_routing` regressions failed before the fix: captured dispatch still invoked node `2` while node `3` was captured, zero scroll was marked handled, and clamped inner scroll consumed before the outer scrollable.
- 2026-05-06 10:56 +08:00: `rustfmt --edition 2021 --check "zircon_runtime/src/ui/dispatch/pointer/dispatcher.rs" "zircon_runtime/src/ui/surface/surface.rs" "zircon_runtime/src/ui/tree/node/interaction.rs" "zircon_runtime/src/ui/tests/event_routing.rs"`: passed with no output after the review fixes.
- 2026-05-06 10:57 +08:00: `$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir "E:\zircon-build\targets\widget-behavior-closure" --message-format short --color never -- --nocapture`: passed, `18 passed; 0 failed; 861 filtered out`, with existing runtime warning noise.
- 2026-05-06 10:59 +08:00: `rustfmt --edition 2021 --check "zircon_runtime/src/ui/dispatch/pointer/dispatcher.rs" "zircon_runtime/src/ui/surface/surface.rs" "zircon_runtime/src/ui/tree/node/interaction.rs" "zircon_runtime/src/ui/tests/event_routing.rs" "zircon_runtime/src/ui/tests/shared_core.rs" "zircon_runtime/src/ui/tests/hit_grid.rs" "zircon_runtime/src/ui/tests/template.rs"`: passed with no output.
- 2026-05-06 11:00 +08:00: `$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo test -p zircon_runtime --lib hit_grid --locked --jobs 1 --target-dir "E:\zircon-build\targets\widget-behavior-closure" --message-format short --color never -- --nocapture`: passed, `11 passed; 0 failed; 868 filtered out`, with existing runtime warning noise.
- 2026-05-06 11:00 +08:00: `$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo test -p zircon_runtime --lib shared_core --locked --jobs 1 --target-dir "E:\zircon-build\targets\widget-behavior-closure" --message-format short --color never -- --nocapture`: passed, `36 passed; 0 failed; 843 filtered out`, with existing runtime warning noise.
- 2026-05-06 11:00 +08:00: `$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo test -p zircon_runtime --lib ui::tests::template --locked --jobs 1 --target-dir "E:\zircon-build\targets\widget-behavior-closure" --message-format short --color never -- --nocapture`: passed, `12 passed; 0 failed; 867 filtered out`, with existing runtime warning noise.
- 2026-05-06 11:01 +08:00: `$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\widget-behavior-closure" --message-format short --color never`: passed with existing runtime warning noise.
- 2026-05-06 11:02 +08:00: `$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\widget-behavior-closure" --message-format short --color never`: passed with existing runtime/editor warning noise.
- 2026-05-06 10:46 +08:00: `git diff --check`: no whitespace errors; output contained LF-to-CRLF warnings only for existing dirty files and touched docs/Rust files.
- 2026-05-06 11:45 +08:00: `git diff --check`: rerun after the review-fix closeout inspection reported no whitespace errors; output contained LF-to-CRLF warnings only across the existing dirty worktree.
- 2026-05-06 12:44 +08:00: `rustfmt --edition 2021 --check "zircon_runtime/src/ui/template/build/interaction.rs" "zircon_runtime/src/ui/tests/template.rs"`: initially reported a formatting-only diff in `interaction.rs`; after `rustfmt --edition 2021 "zircon_runtime/src/ui/template/build/interaction.rs" "zircon_runtime/src/ui/tests/template.rs"`, the same check passed with no output.
- 2026-05-06 12:58 +08:00: `$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo test -p zircon_runtime --lib ui::tests::template --locked --jobs 1 --target-dir "E:\zircon-build\targets\widget-behavior-closure" --message-format short --color never -- --nocapture`: the first attempt timed out after 10 minutes while compiling dependencies and produced no test result; warmed rerun passed, `16 passed; 0 failed; 871 filtered out`, with existing runtime warning noise.
- 2026-05-06 13:04 +08:00: `$env:TMP="E:\tmp\cargo-tmp"; $env:TEMP="E:\tmp\cargo-tmp"; cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir "E:\zircon-build\targets\widget-behavior-closure" --message-format short --color never`: passed with existing runtime warning noise.
- 2026-05-06 13:14 +08:00: `git diff --check`: rerun after the binding-precision validation reported no whitespace errors; output contained LF-to-CRLF warnings only across the existing dirty worktree.

## Known Boundaries

- The broad `cargo test -p zircon_runtime --lib template ...` filter also matches unrelated plugin-extension template tests and previously failed outside the touched UI files at `zircon_runtime/src/tests/plugin_extensions/export_build_plan.rs`; acceptance uses the narrower `ui::tests::template` filter for the runtime UI template tests.
- Validation is scoped to the widget behavior closure files and crate-local runtime/interface checks. No workspace-wide green claim is made.
- Existing unused/dead-code warning noise remains in runtime math/graphics support modules; warning cleanup was not part of this slice.

## Acceptance Decision

- Accepted for the scoped shared Runtime UI widget behavior closure covered by the commands above.
- Remaining behavior backlog: full keyboard/text/IME, drag/drop operation lifetime, popup/menu outside-click semantics, multi-pointer/multi-window capture, and editor host full cutover to consume all returned damage/component envelopes.
