# Runtime 15 Source-Cubemap Projection Owner Split

status: implementation_static_reviewed_compile_blocked_foreign_current_source
date: 2026-07-22
fixing_plan: `docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md`
priority_sources:

- `docs/plans/engine-code-structure-convention.md` R1.4 / R4.2
- `docs/plans/engine-code-review-findings-2026-06.md`

## Scope

This slice removes the current source-cubemap production/test line-budget violations without changing the public `environment::source_cubemap` API or discarding the existing Runtime11 parallel-executor/PMREM and Shader06 immutable-`Arc` storage work.

Exact implementation manifest:

- `zircon_runtime/src/core/framework/render/environment/source_cubemap.rs`
- `zircon_runtime/src/core/framework/render/environment/source_cubemap/projection.rs`
- `zircon_runtime/src/core/framework/render/environment/source_cubemap/tests.rs`
- `zircon_runtime/src/core/framework/render/environment/source_cubemap/tests/projection.rs`

## Completed Items

- The pre-edit current-source audit established the RED: production root `1022 > 1000`; test root `871 > 800`.
- Equirectangular face-size selection, public builder, serial/parallel projection, and the three `SourceCubemapMipChain::from_equirect*` constructors now live in the production `projection` child.
- The root exposes the two free functions through direct `pub use`; it contains no forwarding wrapper or compatibility alias. Associated constructors remain public inherent methods on `SourceCubemapMipChain`.
- Projection/layout/executor/UV behavior tests and their counting executor now live in the folder-backed `tests/projection.rs` owner.
- The Runtime11 parity guard again requires at least one executor dispatch per PMREM mip; the transient `call_count() > 0` weakening was not retained.
- Post-format line counts are production root/child `792 / 248` and test root/child `707 / 167`.

## Current Evidence

- Pre-edit snapshot: `847` exact2, preserving the existing dirty source/test hashes before the mechanical move.
- Post-edit snapshot: `853` exact4; source-manifest fingerprint `3b9b6e4423e9893f1f185f6aadc46f1c657bc98c1ac1a9fc4fc481a6f172379d`.
- `rustfmt +1.94.1 --check`, scoped `git diff --check`, public-symbol exact-once inventory, and seven moved-test exact-once inventory passed.
- Independent final review: `Critical 0 / Important 0 / Minor 0`.
- Source-bound focused reservation: `0d3cb6094a3540879eff8920347dbbba`, command `cargo +1.94.1 test -p zircon_runtime --lib source_cubemap --locked --jobs 1 --message-format short --color never -- --nocapture --test-threads=1`.

The reservation was consumed as job `c758755c6b6443ab8d20801513aeb774`
/ run `f37d74af6599498a99d183f70d35d64c`. It naturally terminaled and
released after 30m44s with exit 101, no live PID, stdout 0 bytes, and zero
target tests executed. Rustc reported 37 foreign current-source errors and 377
warnings; none of the 37 error headers names an exact4 source-cubemap path.
This is compile-blocker evidence, not a source-cubemap red or green result.

The independently checked owner map is complete and mutually exclusive:

| Errors | Lowest owner | Current route |
|---:|---|---|
| 8 | Plugins09 export validation projection | `plugins09-export-validation-projection-r3-20260722`; `docs/plans/zircon_plugins/09/failure-2026-07-17-export-profile-validation-quadratic-scans.md` |
| 17 | Plugins01 native-system access/affinity | `plugins01-native-system-access-affinity-r1-20260722`; `docs/plans/zircon_plugins/01/failure-2026-07-17-native-systems-conservative-world-writer-serialization.md` |
| 8 | Plugins01 event-mirror authority | `plugins01-plugin-event-mirror-authority-r1-20260722`; `docs/plans/zircon_plugins/01/failure-2026-07-22-plugin-event-drain-frame-budget.md` |
| 1 | Performance01 plugin-catalog registration | `plugin/extension_registry/access.rs` E0373; `docs/plans/performance/01/2026-07-22-runtime-plugin-catalog-registration-static-review.md` has no confirmed canonical open failure |
| 1 | Runtime08 ECS resource fixture | `scene/ecs/resource/registry.rs` must use the canonical `SceneResource` marker; do not add a second trait/alias |
| 1 | Editor Layout19 focus contract | `editor-layout19-focus-contract-s1-r1-20260722`; `UiNavigationContract` fixture is missing `boundary` |
| 1 | Runtime08 ECS event-reader contract | `scene/ecs/system/events.rs` has the separate unnumbered lifetime error and must not be absorbed by event-mirror |

Owner-map review: `Critical 0 / Important 0 / Minor 0`. No Cargo green,
fixed return, commit, or parent-plan completion is claimed.

## Remaining Acceptance

1. Wait for the seven lower owner groups above to restore current-source lib-test compilation; Runtime15 must not edit or cfg-gate those paths.
2. Recheck exact4 hashes, create a fresh source-bound focused reservation, and require raw source-cubemap test execution evidence.
3. Run the Runtime 15 production/test line-budget guards against the same current source.
4. Recheck the exact manifest, complete independent acceptance review, then use a managed scoped commit.
5. Only after those gates may the parent Runtime 15 status record this slice as completed.
