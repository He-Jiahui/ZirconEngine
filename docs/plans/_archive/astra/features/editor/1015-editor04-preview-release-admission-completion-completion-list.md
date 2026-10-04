---
related_code:
  - zircon_editor/src/ui/host/editor_asset_manager/manager/preview_refresh/request_preview_refresh.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/preview_refresh/admission_tests.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/04/2026-09-27-preview-release-admission-completion.md
tests:
  - zircon_editor/src/ui/host/editor_asset_manager/manager/preview_refresh/admission_tests.rs
doc_type: milestone-detail
status: implemented_pending_validation
---

# Editor04 Release preview admission completion list

| Work | Source evidence | Remaining acceptance |
| --- | --- | --- |
| Release admission during actual Ready/Error publication. | `complete_refresh` is evaluated outside `debug_assert!`; publication and drop ordering remain in place. | Grouped managed Debug and Release compilation/tests on the sealed source. |
| Verify real publication and 64-slot refill. | Actual preview jobs publish catalog changes through the existing job system; two regressions check Ready/Error, publish identity, token release, preservation of 63 current tokens, waiting-UUID refill and cap retention. | Run `editor04_preview_publication_` and the ignored Release diagnostic `editor04_preview_release_admission_refill_diagnostic`; no Debug-only acceptance. |

No Cargo or Release execution has run locally. The diagnostic marker
`EDITOR04_PREVIEW_RELEASE_ADMISSION_REFILL_V1` is behavioral evidence, not a
speed benchmark or a product frame/memory qualification.
