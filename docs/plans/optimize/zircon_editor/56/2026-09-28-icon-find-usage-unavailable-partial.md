---
doc_type: optimization-implementation
status: partial_implemented_managed_validation_pending
editor: Editor56
plan_item: ED56-P0-02
related_code:
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/componentized_window.rs
tests:
  - icon_library_find_usage_click_reports_unavailable_without_fake_results
---

# ED56-P0-02 partial: Icon Find Usage reports unavailable

**Status:** partial; managed validation pending.

The installed Icon Library Find Usage button Click previously entered the generic extension feedback map, which always wrote "Icon usage search queued" and "14 references". The current route has no Search Operation, icon reference scanner, or result set, so that feedback claimed a search and count that never happened.

This slice corrects capability truth for Icon Find Usage only:

- The Click route now reports a localized unavailable state and says that no reference scanner is connected.
- English and Simplified Chinese text comes from the existing locale catalogs; the production action does not add hard-coded visible text.
- EditorContext passes its shared EditorI18nService into the Workbench bridge, which translates against the active locale when clicked.
- A focused bridge regression dispatches the installed Click binding and checks English and Simplified Chinese status/output, including the absence of "queued" and "14 references".

**Still open:** Icon Usage still has no Search Operation, scanner/provider, query, or real results. Gameplay Tags Reference Scan retains its existing static route. This is a partial ED56-P0-02 correction, not completion, and it carries no performance acceptance.

**Validation boundary:** Rustfmt 1.94.1, TOML parsing and locale-key symmetry, direct byte newline/whitespace checks, and scoped diff checks are static checks. Cargo was not run, so the behavioral regression has not executed. OS/winit validation and performance measurement were also not performed.
