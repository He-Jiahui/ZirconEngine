---
related_code:
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/action_ids/parser.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/action_ids/tests.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/rows/features/action.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/rows/tests.rs
  - docs/crates/zircon_editor/ui/retained_host/app/module_plugin_actions.md
plan_sources:
  - docs/plans/optimize/zircon_editor/06/2026-09-28-qualified-plugin-feature-action-ids.md
status: candidate_static_complete
validation_status: managed_cargo_pending
---

# Editor1041 / Editor06 qualified feature action IDs completion list

| Plan item | Candidate source result | Remaining gate | Status |
| --- | --- | --- | --- |
| E-PLUGIN-UX-P1-13 qualified package routing | The pane emits an explicit byte-length owner field for dotted package IDs; the parser keeps borrowed owner and feature slices. Simple package action strings remain unchanged. | Managed Cargo regressions and retained-host callback proof are pending. | candidate_static_complete |
| Malformed route admission | Malformed widths, missing separators, empty feature IDs, and unknown action kinds return no action. | Dynamic execution remains pending. | candidate_static_complete |
| Product and performance qualification | No wider action payload or project-generation protocol changed. | Generation/admission, end-to-end click, and performance gates remain open. | product_gate_pending |

No Cargo validation or performance measurement was run for this slice. Grouped
coordinator source attribution completed under request
`9006f4b3eef74e2a837772ecf0341d68`; grouped acceptance remains pending.
