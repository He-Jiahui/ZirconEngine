# Runtime10 M2 Project Startup Prepared Owner And Empty Host-Request Acceptance

Plan: docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
Milestone: M2
Status: completed
Files: ["docs/plans/performance/01/fixed-2026-07-17-empty-host-request-batch.md", "docs/plans/zircon_runtime/runtime/10/2026-07-17-empty-host-request-batch-return.md", "docs/plans/zircon_runtime/runtime/10/2026-07-17-m2-project-startup-prepared-owner-acceptance.md", "docs/zircon_runtime/asset/project-manifest.md", "docs/zircon_runtime/scene/level_system.md", "zircon_runtime/src/asset/pipeline/manager/asset_manager/asset_manager.rs", "zircon_runtime/src/asset/pipeline/manager/project_asset_manager/mod.rs", "zircon_runtime/src/asset/pipeline/manager/project_asset_manager/open_project.rs", "zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs", "zircon_runtime/src/dynamic_api/session/construction.rs", "zircon_runtime/src/dynamic_api/session/error.rs", "zircon_runtime/src/dynamic_api/session/ffi.rs", "zircon_runtime/src/dynamic_api/session/project.rs", "zircon_runtime/src/dynamic_api/tests/host_requests.rs", "zircon_runtime/src/dynamic_api/tests/session_profiles.rs", "zircon_runtime/src/scene/module/mod.rs", "zircon_runtime/src/tests/runtime_absorption/code_review_findings/late_api_cleanup/f18_asset_manager_resolution.rs"]

> Machine milestone `M2` is the coordinator late-adoption node for the parent plan's ABI-safe structure and session-boundary work. It does not claim completion of the human-authored UI mirror milestone that is also titled `M2` in the older prose numbering.

## Scope delivered

- `RuntimeProjectConfig::prepare` opens and scans the project once before plugin selection. `RuntimePreparedProject` owns that exact `ProjectManager` and immutable startup manifest snapshot until the dynamic session transfers it through the abstract `AssetManager::open_prepared_project` contract.
- `ProjectAssetManager` remains the concrete activation owner. Path-based `AssetManager::open_project` delegates to the same activation implementation instead of maintaining a second state transition.
- Scene default-level loading clones the already active project snapshot under a short read lock, releases the lock, and performs scene I/O without reopening or rescanning the project and without executing external I/O under the manager lock.
- Empty host-request drains return canonical `ZrOwnedByteBuffer::empty()` at the Runtime10 FFI owner. Non-empty JSON schema, allocation ownership, and release behavior remain unchanged.
- Source guards reject a concrete Runtime04 resolver in the dynamic session, a second project open/scan, and reintroduction of the old resolver ownership path. No compatibility module, alias, shim, or fallback owner was added.

## Fresh testing evidence

| Gate | Coordinator evidence | Result |
| --- | --- | --- |
| Runtime production compile | job `cc5e2e75a7c447c092b6afe72954e42d` / run `f24239e0baf34216bea0ed95d955bf56`; `cargo check -p zircon_runtime --lib --locked` | exit 0; 5m17s |
| Prepared-project snapshot behavior | job `1c78ff96a67d44f4ad80f47704e720fc` / run `7878fa0242cd48b5a67eee0f0b6e62bb`; `cargo test -p zircon_runtime --lib --locked project_startup_snapshot_survives_disk_manifest_rewrite -- --test-threads=1` | both named targets executed once; 2 passed, 0 failed, 8185 filtered; exit 0 |
| Empty host-request ABI behavior | job `b0ea82ad0943466794e3af3c5333816b` / run `4b9e4151d39f4cd9b95de28b2c0ee261`; `cargo test -p zircon_runtime --lib --locked dynamic_session_drains_runtime_ime_cursor_area_and_surrounding_text_requests_once -- --test-threads=1` | target executed once; 1 passed, 0 failed, 8190 filtered; exit 0 |
| Static hygiene | exact-file `rustfmt --check`; scoped `git diff --check`; final source-contract script | passed |

## Review

- Performance01 `empty-host-request-batch` was returned through the coordinator as [`../../../performance/01/fixed-2026-07-17-empty-host-request-batch.md`](../../../performance/01/fixed-2026-07-17-empty-host-request-batch.md); the Runtime10 return record is [`2026-07-17-empty-host-request-batch-return.md`](2026-07-17-empty-host-request-batch-return.md).
- Independent specification review: accepted, Critical 0 / Important 0.
- Independent quality review: Critical 0 / Important 0. It specifically confirmed that the snapshot clone is performed under a short lock while external I/O occurs after release, and that the behavior tests exercise real prepared-owner activation rather than a copied manifest-only surrogate.
- `docs/zircon_runtime/dynamic_api/session.md` is deliberately excluded because it is under a live Render01 F2 lease; the two owned subsystem documents record the current project and scene contracts without absorbing that foreign scope.

## Boundary of this acceptance

This record accepts only the exact manifest above. The parent Runtime10 plan remains `in_progress`; UI mirror convergence, cdylib reload failure injection, full Runtime MVP product startup, save/reopen, editor round-trip, and wave-wide regression remain separate work and are not inferred from these focused gates.
