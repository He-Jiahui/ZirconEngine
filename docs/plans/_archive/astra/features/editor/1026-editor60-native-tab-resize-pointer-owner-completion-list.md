---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/button_dispatch/entry/sequence/entry.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/move_dispatch/entry.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_interaction/drag.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_interaction/resize.rs
  - zircon_editor/src/tests/host/retained_window/native_host_contract/native_capture_pointer_owner.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events/focus.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/test_support.rs
  - zircon_editor/src/ui/retained_host/app/workspace_docking/drawer_resize/movement.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/host_shell/drag_resize.rs
  - zircon_editor/src/ui/retained_host/app/tests/close_prompt.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/lifecycle.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/60/2026-09-27-native-tab-resize-pointer-owner.md
status: implemented_pending_validation
validation_status: programmatic_hide_static_review_passed_managed_validation_pending
---

# Editor1026 / Editor60 native tab and resize pointer owner completion list

| Plan item | Source result | Evidence and remaining gate | Status |
| --- | --- | --- | --- |
| ED60-G02 native capture ownership | Translated `UiPointerId` reaches native button/move dispatch. Tab and resize arm/move/finish retain and compare the owner; foreign press cannot replace or switch an active capture; foreign move/up cannot mutate or consume it. | Real root and floating child `UiHostWindow` tests cover both capture kinds, foreign and ineligible input, true-owner completion, and later independent foreign gestures. Managed tests and OS capture acknowledgement/loss remain pending. | implemented_pending_validation |
| ED60-G03 terminal correctness | Owner Up preserves existing callback/redraw finish, while foreign Up leaves the owner capture active. The Editor1025 hierarchy regression now requires owner Reparent once and foreign tab finish separately. | Exactly-once hierarchy journal/history/terminal assertions and existing cancellation tests; platform cancel/focus/capture-loss delivery remains pending. | implemented_pending_validation |
| ED60-G02/G03 FocusLost cancellation | `Focused(false)` retires native tab/resize capture and emits explicit Cancel. App resize Cancel releases the shell pointer capture, removes active and transient extent, and does not dispatch a resize; drag Cancel does not dispatch a drop. Late Up is inert and a new pointer can arm. | Root/child native host regressions and a real Workbench resize/journal rollback regression are added; v2 independent static review is complete, and managed Editor `--lib` remains pending. | implemented_pending_validation |
| ED60-G03 CloseRequested HideWindow terminal | After the close callback returns `HideWindow`, the native event loop retires tab/resize capture through the FocusLost Cancel path before hiding. `KeepWindowShown` leaves capture active for close prompts. | Root/child native response-handler tests cover noncommitting Cancel, late Up and restart; Workbench tests cover clean child close without tab detach, dirty prompt preservation, and root resize transient rollback with unchanged journal. Independent static review is complete; managed Editor `--lib` and OS close delivery remain pending. | implemented_pending_validation |
| ED60-G03 programmatic child hide retirement | `UiHostWindow::hide()` clears only its native capture without calling into the mutably borrowed app host. A child resize owner removed from runtime layout is canceled before recompute invalidation and geometry snapshots, restoring the committed extent in that recompute. | `programmatic_hide_retires_only_local_tab_capture_without_callback`, `programmatic_child_hide_retires_only_its_native_tab_capture`, and `removed_child_resize_owner_rolls_back_before_recompute_and_preserves_other_window_capture` cover late Up/restart, per-window capture isolation, source-owner filtering, same-recompute geometry rollback, and no `SetDrawerRegionExtent`; existing `keep_shown_close_preserves_resize_capture_until_owner_release` and dirty child prompt tests preserve capture on `KeepWindowShown`. Independent static review is complete; managed Editor `--lib` and OS delivery remain pending. | implemented_pending_validation |
| ED60-G03 cross-window leave | `PointerLeft` remains nonterminal for native tab/resize so owner Up beyond the window can resolve to `DetachToWindow`. Touch leave cannot cancel another pointer's mouse capture. | Translated mouse/touch leave host tests preserve capture then permit owner Up; actual OS delivery and detach product acceptance remain pending. | implemented_pending_validation |
| ED60-G36/G38–G40 | Host state remains per `UiHostWindow`; capture checks add only constant-time ID comparisons in source. | Native multi-window isolation, stage/latency/alloc metrics, true Editor feedback, and same-workload reference benchmarks are pending product gates. No measured performance result is claimed. | product_gate_pending |

V1 and v2 exact preimages, predecessor overlap hashes, scoped static checks, inverse actions, and independent reviews remain anchored in their frozen manifests. The CloseRequested and programmatic-hide successors have separate source changes and completed independent static reviews. The programmatic-hide receipt is `.codex/state/session-coordinator/async-validation-batches/2026-09-28-editor1026-programmatic-hide-resize-successor-independent-static-review.json` (SHA-256 `eceec8ed4b2f5244712d7566d3b59cfa35fd0f5f616220f4b99e3206410ebced`). Programmatic hide retires local native state without app reentry; child resize layout retirement cancels at recompute entry. Managed Editor `--lib`, OS capture acknowledgment/loss, cross-window detach commit, visual acceptance, and performance evidence remain pending. No direct Cargo run or platform acceptance is claimed.
