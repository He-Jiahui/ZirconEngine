---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/window/metadata.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer/input_owner.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer/events/drag.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer/events/motion.rs
  - zircon_editor/src/ui/retained_host/app/tests/hierarchy_native_capture.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/60/2026-09-27-hierarchy-native-input-owner.md
status: implemented_pending_validation
---

# Editor1025 / Editor60 hierarchy native input owner completion list

| Plan item | Source result | Evidence and remaining gate | Status |
| --- | --- | --- | --- |
| ED60-G02 host pointer ownership | Native root/child `UiWindowId` metadata is distinct; callback `MainPageId` stays separate. An active reparent records source window, physical pointer ID, Primary button, and checked generation. A pre-dispatch Move observer identifies the moving pointer before the hierarchy gesture callback and marks touch-like/untranslated moves ineligible; the original post-dispatch tooltip target remains available. A post-dispatch owner Primary Up fallback retires a gesture whose pane callback was consumed by native chrome capture. The native observer cancels on same-pointer foreign-window activity or source-pane exit; different pointers cannot supersede or activate the hierarchy gesture. | Real two-`UiHostWindow` native dispatch tests cover A Down/Move, B Up, late A Up; same-window outside Up/Move; foreign-pointer Down/Move/Up, touch-like/untranslated Move, and a real floating tab capture; successful owner Up and duplicate release. Managed tests and OS capture acknowledgement/loss remain pending. | implemented_pending_validation |
| ED60-G03 terminal cleanup | Native Cancel, Secondary press, Escape, source-window focus loss, child close, and main close all retire the same hierarchy gesture state. | Real host World/history/journal tests assert no Reparent after cancellation and one terminal despite late Up. Platform delivery and native drag metrics remain pending. | implemented_pending_validation |
| ED60-G04/G05/G06 predecessor | Editor1022 world/document/history/window authority and Editor1021 4px threshold remain in the Reparent path. Callback-free hierarchy refresh checks World/document authority without losing child-window press identity. | Existing frozen R tests plus a child-window refresh and real owner success regression. Grouped managed validation remains pending. | implemented_pending_validation |
| ED60-G30/G34/G35/G36/G38–G40 | No performance or OS-level acceptance is inferred from the host harness. | Bounded large Reparent, 100K/1M hierarchy, 1% churn, native multi-window capture and isolation, latency/native feedback, and cross-engine benchmarking remain product gates. | product_gate_pending |

The T Editor1025 source manifest and exact preimage inverse record the R successor edits. This is static evidence only; no Cargo execution or measured performance pass is claimed.

The new weak source handle does not add a host-state reference cycle; existing pane-surface callback captures are outside this slice, so whole-window drop is not an accepted claim.

Native tab/resize capture does not yet store physical pointer identity. A foreign tab capture can consume the hierarchy owner's Up; Editor1025 ends the hierarchy gesture once without Reparent, while independent completion of both gestures remains pending a separate lower-layer capture change.
