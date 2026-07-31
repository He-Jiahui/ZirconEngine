---
related_code:
  - zircon_runtime/src/core/framework/project/export_profile.rs
  - zircon_runtime/src/plugin/export_build_plan/default_profile.rs
  - zircon_runtime/src/plugin/export_build_plan/from_project_manifest/profile.rs
  - zircon_runtime/src/plugin/export_build_plan/export_profile_validation.rs
  - zircon_runtime/src/plugin/export_build_plan/from_project_manifest.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/profiles.rs
implementation_files:
  - zircon_runtime/src/core/framework/project/export_profile.rs
  - zircon_runtime/src/plugin/export_build_plan/default_profile.rs
  - zircon_runtime/src/plugin/export_build_plan/from_project_manifest/profile.rs
  - zircon_runtime/src/plugin/export_build_plan/export_profile_validation.rs
  - zircon_runtime/src/plugin/export_build_plan/from_project_manifest.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan_profile.rs
  - zircon_app/src/entry/tests/export_bootstrap.rs
  - zircon_editor/src/tests/host/manager/minimal_host_contract/native_plugins.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/pipeline_handoff_tests.rs
  - zircon_editor/src/ui/retained_host/app/build_export_actions/profiles.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
tests:
  - export_profile_deserialization_preserves_missing_runtime_profile_id_for_validation
  - export_plan_uses_declared_runtime_profile_id_for_availability_projection
  - export_profile_runtime_profile_selection_has_no_name_or_target_fallback
  - scoped rustfmt --edition 2021
  - scoped git diff --check
doc_type: milestone-detail
---

# Frameworks03 Export Profile Runtime Profile Explicit Hard Cut

Plan: docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
Milestone: M5
Status: implemented_runtime_accepted_app_editor_pending
Date: 2026-07-17
Files: ["docs/plans/performance/01/fixed-2026-07-22-export-profile-validation-quadratic-scans.md", "docs/plans/zircon_plugins/09/2026-07-22-export-profile-validation-quadratic-scans-return.md", "docs/plans/zircon_plugins/09/failure-2026-07-17-export-profile-validation-quadratic-scans.md", "docs/plans/zircon_runtime/frameworks/03/2026-07-17-export-profile-runtime-profile-explicit-hardcut.md", "docs/zircon_runtime/plugin/export_build_plan.md", "zircon_app/src/entry/tests/export_bootstrap.rs", "zircon_editor/src/tests/host/manager/minimal_host_contract/native_plugins.rs", "zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/pipeline_handoff_tests.rs", "zircon_editor/src/ui/retained_host/app/build_export_actions/profiles.rs", "zircon_runtime/src/asset/tests/project/manifest.rs", "zircon_runtime/src/core/framework/project/export_profile.rs", "zircon_runtime/src/plugin/export_build_plan/default_profile.rs", "zircon_runtime/src/plugin/export_build_plan/export_profile_validation.rs", "zircon_runtime/src/plugin/export_build_plan/from_project_manifest.rs", "zircon_runtime/src/plugin/export_build_plan/from_project_manifest/feature_selection.rs", "zircon_runtime/src/plugin/export_build_plan/from_project_manifest/profile.rs", "zircon_runtime/src/plugin/export_build_plan/from_project_manifest/profile_projection.rs", "zircon_runtime/src/plugin/export_build_plan/project_manifest_validation.rs", "zircon_runtime/src/plugin/export_build_plan/project_manifest_validation/duplicates.rs", "zircon_runtime/src/plugin/export_build_plan/project_manifest_validation/identity.rs", "zircon_runtime/src/plugin/export_build_plan/project_manifest_validation/projection.rs", "zircon_runtime/src/plugin/export_build_plan/project_manifest_validation/provider.rs", "zircon_runtime/src/tests/plugin_extensions/export_build_plan.rs", "zircon_runtime/src/tests/plugin_extensions/export_build_plan/catalog_projection.rs", "zircon_runtime/src/tests/plugin_extensions/export_build_plan/profile_feature_matrix.rs", "zircon_runtime/src/tests/plugin_extensions/export_build_plan_feature_provider.rs", "zircon_runtime/src/tests/plugin_extensions/export_build_plan_native_dynamic.rs", "zircon_runtime/src/tests/plugin_extensions/export_build_plan_platform.rs", "zircon_runtime/src/tests/plugin_extensions/export_build_plan_platform/browser_hosts.rs", "zircon_runtime/src/tests/plugin_extensions/export_build_plan_platform/release_adapters.rs", "zircon_runtime/src/tests/plugin_extensions/export_build_plan_profile.rs", "zircon_runtime/src/tests/plugin_extensions/export_build_plan_provider_package.rs", "zircon_runtime/tests/export_build_plan_contract.rs"]

## Scope Delivered

| Slice | Status | Evidence |
|---|---|---|
| Explicit Rust construction | implemented | `ExportProfile::new` requires `RuntimeProfileId`; all 76 repository call sites use four arguments. |
| Old builder removal | implemented | `.with_runtime_profile_id` has zero Rust source matches. |
| Name/target inference removal | implemented | profile selection reads only `profile.runtime_profile_id`; `contains("3d")` and `RuntimeTargetMode` fallback matches are zero in the owner. |
| Untrusted manifest boundary | implemented | serde may retain `None`, but build planning emits a fatal diagnostic and an empty availability report. |
| App/Editor consumers | implemented | bootstrap tests, native-plugin fixtures, export wizard handoff, and production desktop export profiles declare canonical identities. |
| Managed Cargo acceptance | partial | Runtime production-path gate is source-attested GREEN; App/Editor consumer gates remain pending and this record does not claim M5 completed. |

## Architecture Decision

Runtime profile identity is project contract data, not a value inferred by plugin packaging. The
constructor therefore cannot create an incomplete profile. The optional serialized field is kept
only to diagnose malformed or older project data; it is not a compatibility success path.

Missing identity does not select Client2d, Client3d, Editor, or Server temporarily. It records
`export profile <name> must declare runtime_profile_id explicitly`, keeps availability empty, and
allows the existing fatal-plan boundary to stop materialization. Explicit identity/target mismatch
continues to use the existing fatal diagnostic.

## Static Evidence

- 76/76 `ExportProfile::new` call sites have four arguments; untracked Rust call sites are 0.
- `.with_runtime_profile_id` Rust matches: 0.
- profile-name `contains("3d")` inference matches: 0.
- scoped `rustfmt --edition 2021` passed.
- scoped `git diff --check` passed.

## Managed Runtime Evidence

- The first exact31 managed Runtime gate (`f942c000761f44a5996f9b694bff96e0` /
  `afb1e414102d4771918e3452bba85f96`) compiled the hard cut and passed the profile/scale
  regressions, but correctly remained RED because the production-path source guard counted its own
  `RuntimePluginCatalog::builtin_shared()` test literal. It finished 156 passed / 16 failed; the
  owner failure was `export_generation_builds_each_manifest_validation_view_once`, left 2 / right 1.
- The guard now constructs the shared-catalog needle from fragments, so it cannot match itself and
  still independently requires one direct catalog call, the shared catalog API, one runtime catalog
  build, and two explicit manifest-topology projections.
- Current-source successor reservation `cf1c8620d64f4ac88b05057618f83265`, job
  `0d99e095587b4e62a563d0a15441e8c3`, run `35163227122e4922ace66cbbdd28a5fc`
  completed and released with exit 0 and no live PIDs. The exact production-path test passed 1/1
  with 4,378 filtered tests; the managed build completed in 12m05s.
- Full Runtime compile-input attestation covered 10,570 files. Two consecutive pre hashes and the
  post-terminal hash were identical:
  `362f488cbcf0a3d81db4f28e4de3ae29174086f72825592bafbb4d52eef3796a`.
- Exact31 source snapshot 974 fingerprint
  `71eec57a13b9b2f0f6ae76fc9a806ae05ef2ec1d96202298104a57e2b14ac3ae`
  received independent review C0/I0/M0; current exact31 drift is 0.

## Remaining Acceptance

The Runtime export-plan portion is accepted and may be committed atomically with the Plugins09
projection failure return. The implementation is not marked M5 completed until managed App/Editor
consumer gates compile and pass. Frameworks03 M1 and M2 also remain open under the parent plan; this
record does not claim the whole plan complete.
