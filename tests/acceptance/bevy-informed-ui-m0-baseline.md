---
related_code:
  - docs/ui-and-layout/bevy-informed-ui-m0-gap-audit.md
  - dev/bevy/crates/bevy_window/src/event.rs
  - dev/bevy/crates/bevy_ui/src/lib.rs
  - dev/bevy/crates/bevy_ui/src/layout/convert.rs
  - dev/bevy/crates/bevy_ui/src/focus.rs
  - dev/bevy/crates/bevy_ui_widgets/src/lib.rs
  - dev/bevy/crates/bevy_ui_render/src/lib.rs
  - zircon_runtime_interface/src/ui/dispatch/input/event.rs
  - zircon_runtime_interface/src/ui/surface/diagnostics.rs
  - zircon_runtime_interface/src/ui/surface/render/batch.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/layout/pass/measure.rs
  - zircon_runtime/src/ui/layout/pass/arrange.rs
  - zircon_runtime/src/ui/surface/render/cache.rs
  - zircon_runtime/src/ui/tests/event_routing.rs
  - zircon_runtime/src/ui/tests/surface_dirty_domains.rs
  - zircon_editor/src/ui/slint_host/app/invalidation.rs
  - zircon_editor/src/tests/host/slint_window/native_host_contract.rs
implementation_files:
  - docs/ui-and-layout/bevy-informed-ui-m0-gap-audit.md
  - tests/acceptance/bevy-informed-ui-m0-baseline.md
plan_sources:
  - .codex/plans/Bevy-Informed Zircon UI 架构优化里程碑计划.md
  - .codex/plans/Bevy 对齐的 Zircon UI Text Widgets Focus A11y 里程碑计划.md
tests:
  - python .opencode\skills\zircon-project-skills\zr-language-feature-design\scripts\search_feature_evidence.py "UiSystems|RenderUiSystems|CursorMoved|WindowScaleFactorChanged|FocusPolicy|UiWidgetsPlugins|from_node|taffy" --languages bevy --code-only --max-count 40
  - git diff --check -- docs/ui-and-layout/bevy-informed-ui-m0-gap-audit.md tests/acceptance/bevy-informed-ui-m0-baseline.md .codex/sessions/archive/20260508-0034-bevy-informed-ui-m0.md
doc_type: testing-guide
---

# Bevy-Informed UI M0 Baseline Acceptance

## Scope

This acceptance record covers M0 Baseline Audit for `.codex/plans/Bevy-Informed Zircon UI 架构优化里程碑计划.md`. It records evidence and acceptance targets only; no runtime/editor implementation source is modified by this slice.

## Evidence Collected

- Coordination: refreshed active `.codex/plans/` and `.codex/sessions/` within the 4-hour lookback before writing the audit.
- Branch: `git branch --show-current` returned `main`.
- Dirty-worktree context: existing unrelated runtime/editor/UI changes were present before this M0 documentation slice; they were not reverted or edited.
- Bevy reference scan: ran the `search_feature_evidence.py` command recorded in the frontmatter and used the returned `bevy_window`, `bevy_winit`, `bevy_ui`, `bevy_ui_widgets`, and `bevy_ui_render` hits as reproducible anchors.
- Source inspection: read Bevy window events, UI schedule, taffy conversion, focus interaction, widget behavior, and render extract references.
- Zircon inspection: read current shared input DTOs, runtime surface rebuild path, recursive measure/arrange layout passes, render batch DTOs, render cache, diagnostics DTOs, editor host invalidation, runtime UI tests, and editor native host tests.

## Baseline Result

M0 is accepted as an audit/evidence slice when the repository contains:

- `docs/ui-and-layout/bevy-informed-ui-m0-gap-audit.md` with Bevy reference anchors, Zircon current-state anchors, gap matrix, preserved baseline tests, M1-M9 acceptance checklist, and open risks.
- `tests/acceptance/bevy-informed-ui-m0-baseline.md` with the evidence commands, scope, validation status, and non-acceptance boundaries.

## Non-Acceptance Boundaries

- This M0 slice does not claim `cargo check --workspace --locked`, `cargo build --workspace --locked --verbose`, or `cargo test --workspace --locked --verbose` passed.
- This M0 slice does not claim M1 window/input pump convergence, M2 pipeline/schedule implementation, M3 taffy layout, M4 headless widget registry, M5 backend render statistics, M6 incremental cache/pool closure, M7 editor UX cutover, M8 accessibility/focus/navigation, or M9 CI gates are implemented.
- Full Cargo validation is deferred to milestone testing stages because this is docs/acceptance-only and active sibling sessions own overlapping runtime/editor source.

## Future Test Gates

- Runtime repeated hover: `cargo test -p zircon_runtime --lib repeated_same_target_mouse_moves_do_not_dirty_or_rebuild_surface --locked`.
- Runtime dirty domains: `cargo test -p zircon_runtime --lib surface_dirty_domains --locked`.
- Runtime click cancellation: `cargo test -p zircon_runtime --lib primary_release_outside_pressed_target_does_not_mark_click_target --locked` and `cargo test -p zircon_runtime --lib captured_release_uses_hit_path_not_capture_target_for_click_target --locked`.
- Editor native hover: `cargo test -p zircon_editor --lib native_host_repeated_hierarchy_hover_moves_do_not_rebuild_presentation --locked`.
- Editor native text input: `cargo test -p zircon_editor --lib native_host_welcome_material_text_field_accepts_keyboard_input --locked`.
- Milestone CI parity when source milestones settle: `cargo build --workspace --locked --verbose`, `cargo test --workspace --locked --verbose`, `cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --locked --all-targets --verbose`, `cargo build --manifest-path zircon_plugins/Cargo.toml --workspace --locked --verbose`, and `cargo test --manifest-path zircon_plugins/Cargo.toml --workspace --locked --verbose`.

## Validation Log

- 2026-05-08: `python .opencode\skills\zircon-project-skills\zr-language-feature-design\scripts\search_feature_evidence.py "UiSystems|RenderUiSystems|CursorMoved|WindowScaleFactorChanged|FocusPolicy|UiWidgetsPlugins|from_node|taffy" --languages bevy --code-only --max-count 40` completed and returned Bevy source hits for window events, winit translation, UI schedule, taffy dependency, UI widgets, focus policy, and render systems.
- 2026-05-08: `git diff --check -- "docs/ui-and-layout/bevy-informed-ui-m0-gap-audit.md" "tests/acceptance/bevy-informed-ui-m0-baseline.md" ".codex/sessions/20260508-0034-bevy-informed-ui-m0.md" ".codex/sessions/archive/20260508-0034-bevy-informed-ui-m0.md"` passed with no output after archiving the active session note.
