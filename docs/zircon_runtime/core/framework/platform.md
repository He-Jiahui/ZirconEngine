---
related_code:
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/core/framework/platform/mod.rs
  - zircon_runtime/src/core/framework/platform/module_identity.rs
  - zircon_runtime/src/core/framework/platform/runtime_target_mode.rs
  - zircon_runtime/src/core/framework/platform/preferences/mod.rs
  - zircon_runtime/src/core/manager/resolver.rs
  - zircon_runtime/src/platform/module.rs
  - zircon_runtime/src/platform/preferences/backend.rs
  - zircon_runtime/src/platform/preferences/atomic_file.rs
  - zircon_app/src/entry/platform_preferences.rs
  - zircon_runtime/src/input/module/descriptor.rs
  - zircon_runtime/src/graphics/runtime_builtin_graphics/host/module_host/module_registration/module_descriptor.rs
  - zircon_runtime/src/platform/config.rs
  - zircon_runtime/src/platform/capability/report.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/plugin/runtime_plugin/descriptor.rs
implementation_files:
  - zircon_runtime/src/core/framework/platform/mod.rs
  - zircon_runtime/src/core/framework/platform/module_identity.rs
  - zircon_runtime/src/core/framework/platform/runtime_target_mode.rs
  - zircon_runtime/src/core/framework/platform/preferences/mod.rs
  - zircon_runtime/src/core/framework/platform/preferences/backend_kind.rs
  - zircon_runtime/src/core/framework/platform/preferences/error.rs
  - zircon_runtime/src/core/framework/platform/preferences/key.rs
  - zircon_runtime/src/core/framework/platform/preferences/storage.rs
  - zircon_runtime/src/platform/preferences/backend.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
  - docs/plans/zircon_runtime/frameworks/05/failure-2026-07-13-core-contract-reverse-dependencies.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - tools/tests/test_frameworks_05_layer_direction.py::Frameworks05LayerDirectionTests::test_runtime_target_mode_has_one_neutral_owner
  - tools/tests/test_frameworks_05_layer_direction.py::Frameworks05LayerDirectionTests::test_platform_module_identity_has_one_neutral_contract_owner
  - python tools/runtime_domain_dependency_audit.py --pretty --output .codex/tmp/frameworks05-runtime-target-mode-neutral-owner.json
  - python tools/runtime_domain_dependency_audit.py --pretty --output .codex/tmp/frameworks05-platform-current-final.json
  - python -m unittest tools.tests.test_frameworks_03_contract_feature_boundary tools.tests.test_runtime_domain_dependency_audit
  - managed Windows cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked (pending until the repository Cargo testing stage is available)
doc_type: module-detail
---

# Platform Framework Contracts

## Purpose

`zircon_runtime::core::framework::platform` owns platform vocabulary that must be shared by runtime assembly, capability policy, plugin manifests, application hosts, the editor, and external plugins without depending on any of those concrete domains. It owns the canonical `RuntimeTargetMode`, runtime platform module identity, and the namespaced persistent-preference contract.

## Ownership

`RuntimeTargetMode` selects the client-runtime, server-runtime, or editor-host family. The enum is a serialized contract, not builtin-module assembly behavior. Its unique declaration therefore lives in `core/framework/platform/runtime_target_mode.rs`; concrete target manifests and module lists remain in `builtin`, capability/config evaluation remains in `platform`, and plugin availability/validation remains in `plugin`.

The former `builtin/runtime_modules/ids/target_mode.rs` owner and `zircon_runtime::builtin::RuntimeTargetMode` re-export were deleted in one hard cut. All Runtime, App, Editor, plugin SDK, and first-party plugin call sites use `zircon_runtime::core::framework::platform::RuntimeTargetMode`. No alias, compatibility module, prelude projection, or duplicate declaration remains.

`PLATFORM_MODULE_NAME` is module dependency vocabulary consumed by platform assembly, input, graphics, App profile tests, and the Runtime prelude. Its single declaration lives in `core/framework/platform/module_identity.rs`. The concrete `platform::module` implementation and all internal cross-domain consumers read that contract directly. The public `zircon_runtime::platform` facade preserves its established path through a direct curated re-export from the neutral owner; it does not forward through the concrete module implementation or duplicate the declaration.

`PreferenceStorage` is the versioned manager contract for namespace/key byte values. `PreferenceStorageBackend` is the host implementation boundary; errors distinguish unavailable, denied, capacity, corrupt-backend, and transient-I/O failures and preserve backend sources. `PlatformDriver` accepts exactly one non-unavailable backend, while callers resolve `ManagerServiceHandle<dyn PreferenceStorage>`. Desktop app bootstrap selects an approved local user-data root and installs the atomic-file backend after Platform activation. Mobile and browser hosts inject sandbox-native backends through the same driver contract; headless remains explicitly unavailable unless its host opts in. No process-memory persistence fallback or Editor/WOC special case exists.

## Dependency Direction

The neutral owner makes the intended direction explicit: concrete platform, input, graphics, builtin assembly, plugin, editor, and application domains may depend on `core/framework/platform`; the framework layer never depends on those facades. Runtime module dependency declarations therefore do not create `input→platform` or `graphics→platform` implementation-root edges. Preference consumers depend on the neutral trait and manager handle, while only the process host sees the concrete backend. External App/Editor/plugin callers remain behind the `zircon_runtime` facade as required by Frameworks01. This removes all seven production `platform→builtin` references recorded by the Frameworks05 failure handoff and the two internal platform module-identity edges while preserving the public contract value and path.

## Validation State

The `RuntimeTargetMode` focused owner guard moved from RED to GREEN during its hard cut. The platform module-identity guard was likewise observed RED before the neutral owner existed and is GREEN after the hard cut; it scans Runtime, App, Editor, and plugin Rust sources for duplicate declarations and retired root paths. The atomic Foundation successor preserves the result with the full Frameworks05 layer suite at 28/28 GREEN and the current production audit at 2,401 references / 74 edges; `graphics→platform` and `input→platform` remain 0. Managed Windows Cargo validation remains pending until the coordinator can create an immutable full-compile-input validation copy.
