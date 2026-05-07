---
related_code:
  - zircon_runtime/src/ui/text/edit_state.rs
  - zircon_runtime/src/ui/tests/text_layout.rs
  - zircon_runtime_interface/src/ui/surface/render/editable_text.rs
implementation_files:
  - zircon_runtime/src/ui/text/edit_state.rs
  - zircon_runtime_interface/src/ui/surface/render/editable_text.rs
plan_sources:
  - user: 2026-05-06 restore preedit text when composition is canceled
tests:
  - zircon_runtime/src/ui/tests/text_layout.rs
  - cargo test -p zircon_runtime --lib editable_text_state --locked --target-dir C:\Users\HeJiahui\AppData\Local\Temp\opencode\zircon-ime-cancel-target --message-format short --color never
doc_type: module-detail
---

# Runtime UI Text Edit State

`edit_state.rs` applies shared `UiTextEditAction` values to `UiEditableTextState`. It owns byte-boundary clamping, selection replacement, caret movement, delete/backspace behavior, and the provisional IME composition state used by runtime text layout tests.

## Composition Lifecycle

`SetComposition` makes preedit text visible immediately by replacing the requested byte range in `state.text`. Existing non-empty composition ranges keep the current visible-footprint behavior, while an empty range acts as pure provisional insertion at the caret/range start. Because that replacement is provisional, the helper also stores the text that occupied the replaced range in `UiTextComposition::restore_text`.

Before accepting a new preedit update, the helper restores any active composition snapshot and then applies the new range against the restored text. This preserves the previous commit-time behavior where composition ranges are interpreted against pre-composition text rather than earlier provisional text.

`CommitComposition` takes the active composition and leaves the visible text in place. It only moves the caret to the composition end, clears selection, and clears the composition record, so the preedit text is not inserted twice.

`CancelComposition` takes the active composition, replaces the visible provisional range with `restore_text`, clears the composition record, and leaves normal caret/selection normalization to the same range-replacement helper used by other text edits. This returns the field to its pre-composition contents instead of retaining canceled preedit text.

## Tests

`editable_text_state_applies_selection_and_composition_actions` covers selection replacement, visible preedit insertion, and commit behavior.

`editable_text_state_restores_preedit_text_when_composition_is_canceled` covers the IME cancel regression where `SetComposition` had already written provisional text into `state.text` and `CancelComposition` previously cleared only `state.composition`.

`editable_text_state_updates_composition_against_preedit_base_text` covers repeated preedit updates where the active visible footprint must be restored before the next range is applied, including a replacement text that is longer than the explicit IME range.

`editable_text_state_inserts_preedit_without_consuming_text_for_empty_range` covers IME engines that send an empty source range with longer preedit text, ensuring provisional insertion does not delete following non-composition bytes and cancel still restores the original field contents.
