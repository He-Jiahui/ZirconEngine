---
handoff_kind: failure
status: open
failure_scope: cross_plan
plan_link_mode: child_record_only
created_at: 2026-09-08
summary_slug: editor-palette-cascade-expectation-drift
origin_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
fixing_plan: docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md
origin_child_dir: docs/plans/optimize/zircon_runtime_interface/03
fixing_child_dir: docs/plans/zircon_editor/editor_ui/12
related_code:
  - zircon_runtime_interface/src/tests/editor_design_tokens.rs
  - zircon_runtime_interface/src/ui/design_tokens/cascade_registry.rs
tests:
  - managed Windows static/no-default/locked zircon_runtime_interface --lib tests::editor_design_tokens
  - managed Windows static/no-default/locked zircon_runtime_interface --lib ui::design_tokens::cascade_registry
---

# EditorUI12: cascade assertions retain the superseded palette

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md`
- 来源执行切片：Current-source RuntimeInterface library regression.
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md`
- 交接原因：EditorUI12 owns the approved palette, canonical token registry and product theme parity.

## 失败现象与复现证据

The immutable `interface-project-identity-3170-20260908` library run reports
754 passed, 11 failed, 101 ignored in `results/interface-library-3170.log`.
Two failures share this palette expectation root:

- `tests::editor_design_tokens::editor_design_tokens_register_canonical_and_css_custom_property_values`
  expects `editor.surface.1` to be `#171a1d`; actual is `#242424`.
- `ui::design_tokens::cascade_registry::tests::workbench_dark_palette_matches_component_prototype_baseline`
  expects `editor.surface.0` to be `#111416`; actual is `#151515`.

The current tests match the immutable input. Current SHA-256 values are
`00271b9911aa1d4b7930a5388b44ff7ff184d53fed794e249379689c64ce85cf`
for `tests/editor_design_tokens.rs` and
`9bb87f4fc6dfe3e243c2daa1cec3296ebd198905f67c5c99c8135dfe8e6bda7e`
for `ui/design_tokens/cascade_registry.rs`.

## 最低共享层根因

EditorUI12's 2026-08-27 palette correction explicitly replaced the blue-gray
prototype palette with neutral surface levels `#151515`, `#242424`, `#2f2f2f`,
`#383838`, plus distinct interaction and semantic colors. The current canonical
theme `zircon_editor/assets/ui/editor/theme/editor_tokens.zui` and the existing
direct `EditorPaletteTokens` assertions use that corrected palette. These two
cascade tests retained the old literal expectations. The registry still needs
independent exact-value and canonical-reference assertions after correction.

Ownership is unresolved for the first test file: its last attribution belongs
to archived `editor-ui12-m2-scope-hold-v2-20260812` but records hash
`64579fbf715856a2a97caa1accf1b528da909eae4a69b873849297143882602b`,
which differs from the current hash. Its worktree also includes earlier palette,
density and typography changes. Those changes have not been adopted or edited.
The cascade file has matching attribution to archived
`root-runtime-interface03-activate-link-failure-20260831`, including existing
alias and color-encoding performance changes outside this failure's test delta.

## 架构修复验收

- Legally reconcile exact current source ownership before applying the two
  assertion updates; preserve the existing density, typography and registry work.
- Retain exact literal palette values in the cascade test, sourced from the
  approved theme; retain CSS custom-property references to canonical token names.
- Execute both complete test batches above on frozen current source, including
  color encoding, registry aliases and theme projection consumers.
- Keep product UI acceptance separate; complete formal binding, independent
  C0/I0/M0 review, canonical return and coordinator closeout for this failure.

## 禁止临时方案

- Do not restore the superseded production palette to satisfy old expectations.
- Do not derive every expected value from the same implementation under test.
- Do not overwrite stale attribution, adopt unknown source changes, disable
  assertions or count unrelated passing tests as acceptance.
- Do not modify or revalidate external zr_vm.

## 修复结果与回传

### 2026-09-11 滚动修复记录

- Stable fixing session: `failure-roll-01a084c8-editorui12-palette-cascade`.
  Pre-edit snapshot `3392` preserves the unresolved-attribution inputs. Both
  target paths were unowned because their archived attributions were stale;
  the existing palette/density/typography test edits and registry
  alias/color-encoding performance edits were retained without modification.
- `tests/editor_design_tokens.rs` now expects `editor.surface.1` to be
  `#242424` while preserving the independent
  `--editor-surface-1 -> $editor.surface.1` canonical-reference assertion.
  `cascade_registry.rs` now has exact approved-theme literals for all 17
  cascade palette entries and names the assertion after the approved editor
  theme rather than the superseded component prototype.
- Source snapshot `3393` seals
  `749837a211f6fe5efb1d35b73834d9b21a77bb083dc195ec551f898ed3d0c0ca`
  for `tests/editor_design_tokens.rs` and
  `857340f7f74c435b01058447bc12e75d0eba90b95848fabf9be1ef2951c58f17`
  for `ui/design_tokens/cascade_registry.rs`. Parse-only `rustfmt +1.94.1
  --edition 2024 --config skip_children=true --emit stdout` succeeded for
  both files; `git diff --check` reported no whitespace error; a static
  cross-check matched all 17 literals to `editor_tokens.zui` and preserved
  the canonical CSS alias. These checks are not dynamic acceptance.
- Managed Windows Cargo request
  `editorui12-palette-cascade-cargo-3393-r1` requested `cargo +1.94.1 test
  -p zircon_runtime_interface --no-default-features --locked --lib
  design_tokens -- --nocapture --test-threads=1`, covering both declared
  `design_tokens` test modules. The coordinator rejected it during admission
  with `validation_ticket_external_worktree_dirty`: `E:\Git\zr_vm` must be
  clean before an immutable validation copy can be sealed. No validation
  ticket was created and no dynamic test is claimed as executed.

Open state: `source-fixed-validation-blocked`.
Resume with a fresh managed Cargo submission only after the external worktree
owner supplies a clean revision. No fixed return, independent C0/I0/M0 review,
closeout, commit, or notification is claimed.
The independent button primary/focus test repair proceeds under its own
canonical failure and source snapshot 3228.
