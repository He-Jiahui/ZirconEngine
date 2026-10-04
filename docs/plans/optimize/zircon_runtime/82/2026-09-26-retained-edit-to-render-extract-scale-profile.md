---
title: Runtime82 Retained Edit to Render Extract Scale Profile
category: zircon_runtime
report_id: Runtime82-retained-edit-to-render-extract-scale-profile-2026-09-26
date: 2026-09-26
implementation_status: fixture_complete
validation_status: managed_validation_pending
performance_status: scale_samples_pending
---

# Runtime82 retained edit to render extract scale profile

## Product-path gap

The existing grapheme and edit-session Release comparisons time local helpers.
They cannot show the cost of an actual keyboard edit passing through
`UiInputManager`, retained document receipt publication, `UiSurface` rebuild,
and a visible text render command. Runtime82 still records full-string product
copies and leaves million-character editing latency, allocation, and RSS open.

This slice adds a test fixture to the existing widget keyboard test module. It
uses its actual text-input surface and manager, makes the fixture root visible,
and selects the existing `InputField` renderer component. The regular regression
sends Backspace through the manager, checks the retained edit receipt and exact
Unicode content in the render extract, then undoes and checks the restored text.

Two ignored Windows Release profiles time the same route at 1, 100, 1,000,
10,000, and 1,000,000 ASCII graphemes. Each case starts with a trailing marker,
warms five edit/undo cycles, then measures 31 more cycles. The timer covers
Backspace dispatch, the full `surface.rebuild()` through CPU render extraction,
and their total; the rebuild stage also includes layout and hit-test work;
result validation and undo restoration occur outside the timed interval. Every
sample requires a typed retained edit receipt, exact edited and undone text in
both the document property and render command, the expected caret offset, and
advancing text layout revision. The printed marker
`RUNTIME82_EDIT_TO_RENDER_EXTRACT_PROFILE_V1` includes p50/p95/p99, raw sample
order, OS, architecture, and package version for each stage and scale.

## Acceptance boundary

The million-character case is an ignored diagnostic because the current widget
authority still keeps and publishes whole strings. It has no numeric pass
threshold until a same-hardware product budget and matched Unreal workload are
frozen. The fixture measures a retained in-crate UI surface and CPU render
extract. It does not measure App/Editor host ingress, WGPU submission or present,
allocation, RSS, IME, clipboard, or multi-window lifecycle. `RTE-GATE-016` and
`RTE-GATE-047` remain open.

Run the regular regression with grouped managed Runtime lib tests. Run both
ignored profiles in the grouped Windows Release lane and retain their raw
samples. Static formatting and diff checks alone do not establish a pass.
