# Runtime NumberField invariant edit-session implementation record (2026-08-29)

## Status

`mvp_product_slice_implemented_unvalidated / managed_validation_pending /
platform_wgpu_profile_power_pending`

## Structural correction

The previous Surface path tried to project every text edit into the widget's canonical value
property. That is invalid for `NumberField`: the canonical value is `Float`, while an active editor
must preserve incomplete strings such as `-`, `.`, or `1e`.

The product path now has one retained metadata authority with distinct roles:

- `value: Float` is the committed model value;
- `value_text: String` is the active edit buffer;
- `number_edit_active: bool` selects buffer or canonical display for both input and render;
- the existing editable property transaction remains the only metadata/component/binding/dirty
  mutation gateway.

No second document store, binding registry, renderer state, numeric model, or format cache was added.
The parser and policy decision live in the folder-backed `surface/input/number_field.rs`; route code
only selects edit, submit, blur, and cancel behavior. Touched production owners remain below the
repository's 800-line review threshold.

## Unreal alignment

Primary references remain `SSpinBox.cpp` (`TextField_OnTextChanged`,
`TextField_OnTextCommitted`, `CommitValue`) and `SNumericEntryBox.h` (`SendChangesFromText`). The
implemented MVP preserves the same structural rules: editable string and typed value are separate,
per-key publication is conditional, and commit owns parse plus numeric policy before typed event
publication.

`SSpinBox::OnKeyDown` is also the reference for outer numeric keyboard routing. Zircon now handles
unmodified Up/Down before generic single-line caret movement, steps from the canonical Float rather
than an incomplete edit buffer, and exits edit mode after an accepted step. Left/Right remain text
caret commands in Zircon's current single-focus-node model; matching Unreal's separate outer/inner
focus targets is explicit post-MVP work rather than an implicit key remap.

Zircon deliberately limits V1 to invariant ASCII with `.` and optional `e/E`. Locale, numbering
system, precision, rounding, and edit/display formatter identity remain explicit future type-interface
work; no platform locale guessing is permitted.

## Behavior completed

- Versioned, content-free `UiNumberInputReceiptV1` reports format, parse status, commit method, and
  commit status through dispatch diagnostics.
- Default character, IME, clipboard, and accessibility text edits update only the edit buffer.
- Explicit `number_publish_per_key` publishes a typed Float Change only for complete, finite,
  range-valid text.
- Enter parses, clamps, optionally snaps to a positive step, formats, closes the edit buffer, and
  publishes typed Float Commit.
- Empty/intermediate/invalid Enter is rejected while preserving the active buffer.
- Focus loss commits valid text and restores canonical text for invalid input.
- Escape cancels and restores canonical text without publishing a commit.
- NumberField defaults to single-line number filtering; scientific notation admits `e/E`.
- The invariant edit buffer has a named 128-byte MVP hard limit. Node-level max-grapheme settings
  may lower but cannot raise it; direct parser calls return typed `TooLong` before parsing.
- IME commit ends preedit and remains an edit operation, not a field submit.
- Accessibility `SetValue` is an explicit typed model action: it applies the same parse/clamp/snap
  decision and atomically commits Float plus canonical text; invalid input performs no partial write.
- Up/Down and key repeat use a typed `KeyboardStep` decision and the same atomic property transaction.
  A successful step discards the transient buffer and canonicalizes display; invalid step/policy or
  non-finite arithmetic preserves the buffer and performs no write.

## Evidence and open work

Source regressions cover Float preservation, out-of-range edit receipt, clamped Enter commit,
intermediate Enter rejection, Escape cancellation, invalid blur restoration, canonical keyboard
step, invalid-step zero-write, descriptor state, DTO serde/versioning, and legacy diagnostics
defaults. Scoped Rustfmt and `git diff --check` pass.

The managed Windows `zircon_runtime` build plus `number_field` lib-test batch entered the managed
validator but produced no output or terminal result for about two minutes; the local wait was ended
without polling or retrying the coordinator. This record therefore does not claim managed Cargo,
real platform IME/accessibility/clipboard, WGPU/PNG,
allocator/RSS/latency, power, or matched-Unreal acceptance. Performance optimization remains gated on
the planned per-key and commit profiles. Focused external numeric refresh/revision conflict now has a
separate revision-qualified Float gateway and source regressions, but remains dynamically unvalidated.
Locale type interface, keyed stable formatting cache, and real platform accessibility acceptance
remain open.

The hot path exposes eight fixed profile counters for parse count/input bytes, edit/commit decisions,
typed publication, clamp, snap, and keyboard step. One transaction receipt carries the already-computed edit
decision to diagnostics/event projection, so an ordinary numeric edit invokes the parser once.
No latency, allocation, RSS, or power numbers have been collected yet, and no format cache or other
profile-gated optimization has started.

The existing `UiTextModelUpdateRequest` gateway is intentionally String-only: its admission requires
the canonical metadata property to be a TOML String. An actual NumberField therefore returns
`InvalidTarget` before stale-revision comparison or retention, and neither Float `value` nor
`value_text` changes. A source regression now locks this boundary. Focused numeric refresh uses the
separate versioned Float request and edit-base revision contract recorded in
`2026-08-29-number-field-focused-model-refresh-architecture.md`; widening the text request remains
forbidden.
