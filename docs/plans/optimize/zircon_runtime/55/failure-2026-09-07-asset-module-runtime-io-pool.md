---
handoff_kind: failure
status: open
created_at: 2026-09-07
summary_slug: asset-module-runtime-io-pool
origin_plan: docs/plans/optimize/zircon_app/08-product-host-bootstrap-loop-dynamic-runtime-shutdown-current-source-review.md
fixing_plan: docs/plans/optimize/zircon_runtime/55-runtime-foundation-module-config-event-service-driver-manager-persistence-lifecycle-product-integration-review.md
origin_child_dir: docs/plans/optimize/zircon_app/08
fixing_child_dir: docs/plans/optimize/zircon_runtime/55
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/module.rs
  - zircon_runtime/src/asset/tests/module_lifecycle.rs
tests:
  - tools/dev/dev-fast-build.ps1 -Action test -Package zircon_runtime -FeatureOverride target-server -LibTests -TestFilter asset_module_manager_uses_the_activating_runtime_io_owner -LinkMode static
  - tools/dev/dev-fast-build.ps1 -Action test -Package zircon_app -FeatureOverride target-server -LibTests -TestFilter headless_bootstrap_stores_headless_platform_config -LinkMode static
  - tools/dev/dev-fast-build.ps1 -Action test -Package zircon_app -FeatureOverride target-server -LibTests -TestFilter headless_bootstrap_stores_headless_platform_config -LinkMode dev-dynamic
---

# Runtime55: AssetModule selects the wrong runtime task pool

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_app/08-product-host-bootstrap-loop-dynamic-runtime-shutdown-current-source-review.md`
- 来源执行切片：App08 managed server validation and static/development-DLL equivalence.
- 修复责任计划：`docs/plans/optimize/zircon_runtime/55-runtime-foundation-module-config-event-service-driver-manager-persistence-lifecycle-product-integration-review.md`
- 交接原因：Runtime55 owns the AssetModule service factory. Current attribution is
  `claude-runtime55-foundation-m1-20260902`; its dirty content must be preserved.
- Client and Editor compiler failures remain under their existing handoffs. This separate
  server bootstrap failure must not be counted as a cache or linker failure.

## 失败现象与复现证据

Managed job `78159eadffa5401da1902ea46df3bdd8` checked and executed the App `target-server`
library on sealed input `d576d039ef2a9a78375c43e1eff486669c901417bcb42dd50ec5cce7932a7594`.
The suite reported 182 passed, 36 failed and 2 ignored. Six bootstrap tests reached
`ProjectAssetManager::new` and failed with `ProjectAssetManager requires the runtime IO task pool`,
`left: Compute`, `right: Io`, then `ServiceFactoryPanicked` for
`AssetModule.Manager.ProjectAssetManager`.

The affected tests include `headless_bootstrap_stores_headless_platform_config`,
`headless_bootstrap_stores_absent_primary_window_descriptor`,
`runtime_bootstrap_stores_primary_window_descriptor`,
`runtime_bootstrap_stores_default_render_profile_bundle`,
`entry_config_can_select_headless_render_profile_bundle`, and
`runtime_plugin_bootstrap_installs_neutral_module_lifecycle_observer`.
Other suite failures are not attributed to this handoff.

The log is retained at
`E:/cargo-targets/zircon-engine/cache/build-benchmarks/app08-server-lib-contracts-20260907/results/library-dynamic.log`.
The current and sealed `zircon_runtime/src/asset/module.rs` SHA-256 both equal
`ad2e56fd2cdb1d51268758aaff63387bb485356e7d99553c4a1c1bf7b61f5537`.

## 最低共享层根因

The inspected candidates were the App bootstrap configuration, module dependency ordering,
AssetModule service factory, EngineTaskGraph domain selector, and ProjectAssetManager constructor.
`EngineTaskGraph::worker_pool()` explicitly returns `TaskPoolKind::Compute`.
The AssetModule factory calls `ProjectAssetManager::new(core.task_graph().worker_pool().clone())`,
while the constructor requires `TaskPoolKind::Io`. The runtime already exposes
`task_pool(TaskPoolKind::Io)`; the integration factory is the lowest mismatched owner.

## 架构修复验收

- Route the asset service through the existing runtime-owned IO domain and preserve its
  lifecycle, registration, importer and service ownership contracts.
- Add a focused lower-layer service activation regression proving the selected pool is IO
  and shares the runtime's task-pool owner.
- Re-run the headless bootstrap reproduction through the managed validator in static and
  development dynamic modes. Report other initialization failures separately.
- Re-run affected profile/bootstrap tests before returning this handoff.

## 禁止临时方案

- Do not remove or weaken the IO-kind assertion, allocate a separate default pool, or
  special-case App test initialization to bypass the normal module factory.
- Do not alter client/Editor features or claim failed suites as successful timing samples.

## 修复结果与回传

Open: Runtime55 source snapshot 2975 is repaired and independently reviewed.
The lower IO-owner regression and original App headless reproduction now pass in
static and development dynamic modes. The complete affected bootstrap batch,
closeout evidence binding, return and commit remain pending as detailed below.

## Runtime55 source repair

Fixing Session `failure-roll-01a07160-runtime55` retained pre-edit snapshot 2974.
The two source paths were transferred from archived owners through fingerprint
`09a9b46864791fb9a6c0f0f5d554f38dc16d53ad5846e5728a4024c7bd540a9d`
and apply request `0d778ede8f384f9883a3bc745ca5f51f`. Prior Foundation dependency removal
in the module descriptor is preserved.

The service factory now clones the activating runtime's existing `TaskPoolKind::Io`
pool. The original named IO-owner regression previously compared against Compute
and omitted its TasksModule dependency. It now registers that real dependency,
activates the unchanged production module factory, and checks IO kind, exact IO
execution owner, and separation from the Compute owner with a three-worker budget.

The source scope lists only the two changed owners. The unchanged lower constructor
`zircon_runtime/src/asset/pipeline/manager/project_asset_manager/construction.rs`,
selector `zircon_runtime/src/core/runtime/tasks/task_graph/engine_task_graph.rs`, and
App consumer `zircon_app/src/entry/tests/profile_bootstrap.rs` remain the direct
support and acceptance chain; they are not included as unrelated commit paths.
Independent reviewer `01a07160-5337-7570-a507-ed6decf2d32b` reviewed snapshot
2974 -> 2975 with Critical=0, Important=0, Moderate=0. Before/after source hashes
were identical: module `847276e5fec1bae87551197a96d3ca13fd538de7f364823721d8c36a03321b5f`
and regression `e9f54b585d3489eef1baed981adf25daa4444c62e7f2479b4eab0fc4c231f032`.
This is source review evidence only.

The initial managed-ticket submission
`failure-roll-20260907-asset-module-io-owner-2975-r1` was rejected before creating
a ticket with `validation_ticket_external_worktree_dirty` for the live external
`zr_vm` checkout. That checkout remains excluded from this execution.

A separate managed Cargo job `a1ae48838780446aa1b50615c045d72b` used a newly sealed
derived snapshot at
`E:/cargo-targets/zircon-engine/cache/build-benchmarks/runtime55-asset-io-2975-20260907`.
Its 10,708-file input manifest is
`6ff43218043a99aae891c011b6a6a9b70f85d7f6fc67af3d48455bfdae28f791`;
the unchanged App/server parent is
`4f436be400521f50968a65de9c55bf8bdd3e30ef9d416d1fc2cbfe384613eb82`.
Only the two reviewed asset source files differ from that parent. External inputs
were copied from the sealed parent, without reading the live `zr_vm` worktree.

The command was `tools/dev/dev-fast-build.ps1 -Action test -Package zircon_runtime
-FeatureOverride target-server -LibTests
-TestFilter asset_module_manager_uses_the_activating_runtime_io_owner -LinkMode static`
with the derived `-SourceSnapshot` and full `-SourceSnapshotDigest`. Cargo retained
`--locked`. The job reached terminal failure: check 315.026 seconds, Cargo exit 101,
1,042 compiler errors, and zero tests executed. The App-only parent lacks Runtime
`cfg(test)` document/fixture inputs, beginning with the scene-reflection v0 JSON
and the runtime-absorption document includes; subsequent type errors are not an
IO-owner assertion result. The prerequisite input closure must be repaired before
retrying this unit-test command. The pool and job were released.

Full log and embedded managed receipt:
`E:/cargo-targets/zircon-engine/cache/build-benchmarks/runtime55-asset-io-2975-20260907/results/runtime-asset-io-static.log`.
Lower regression, original App static/dynamic bootstrap acceptance, failure return,
Git closeout, and WeCom commit notification remain pending. No failed run or static
review is presented as dynamic acceptance.

## 2026-09-08 input-closure correction

Derived input `runtime55-asset-io-test-inputs-20260908`, digest
`f6b0656e9394066fb5aedecb51187ed8b6ba0ceefd857fbdf6cbfd3f04d951ba`,
added the 241 baseline document/fixture/source includes omitted by the App-only
parent. It also overlaid two separately owned support files: importer contract
`2e9b07a772a510852c63d6260bd5eed64b673e57be290b4aba1295bbe338fccd`
and plugin package tests
`adf3b2b41b15dc7f4a3c6dee1ef3f35869044e07199f3afdfff33ef4b8714619`.
Those two broad overlays are not the Runtime55 IO fix.

Managed Cargo job `94924f658d164ab5a299830cd6aa7b11` reached compilation using
the unchanged IO test filter and `--locked`. Its log contains 21 compiler errors,
beginning with missing `AssetImportBuildContext`, `AssetImportBuildIdentity`,
`AssetImportRecipe` and `AssetImportRecipeValue` exports. Old importer consumers
also access the now-private `import_settings` field. The previous missing include
errors are absent, but the new importer overlay requires a larger migration than
the original reproduction. The IO test did not execute.

The entry process ended without a final managed receipt or exit code. The
coordinator marked this exact job `orphaned` at `2026-09-07T16:52:16.192401Z`
with an empty process tree. Its original `validate-matrix:failure-roll-01a07160-runtime55`
owner released it through request `9fa877d07bb04a71aa032cadec19c638`; no successful
result or synthetic exit code was recorded. The compiler log remains at
`E:/cargo-targets/zircon-engine/cache/build-benchmarks/runtime55-asset-io-test-inputs-20260908/results/runtime-asset-io-static-r2.log`.

The next reproduction starts from the original sealed App input plus the two
reviewed IO files and the 241 missing baseline test inputs. It preserves the
original importer and plugin contracts; the unrelated full migration overlays
above will not be reused as standalone support. Baseline input contents are
checked against commit `585b031793088c28feef357488211f290927ec50`, accounting
explicitly for Windows CRLF materialization, before a new complete manifest is
sealed. No current worktree source or external `zr_vm` files are reverted or read
into that reproduction.

The corrected input is sealed at
`E:/cargo-targets/zircon-engine/cache/build-benchmarks/runtime55-asset-io-original-contract-20260908`,
digest `5dba8bc7d7ed12b0c2d688e4400110b233fc88ab2c10499ce59d66300f717096`,
with 10,949 files. Full manifest equality was checked against the original
10,708-file IO-repair snapshot plus exactly those 241 baseline inputs. No importer
or plugin support overlay is present.

Managed job `8aa14660d4a546729e25710aa056fe1f` passed the unchanged lower command
on that exact input: `asset::tests::module_lifecycle::asset_module_manager_uses_the_activating_runtime_io_owner`
actually ran, with 1 passed / 0 failed / 0 ignored and 6,922 tests filtered out.
Both Cargo check and test succeeded; the entry and embedded receipt report exit 0.
Queue time was 8.329 seconds, source synchronization 23.549 seconds, check 138.941
seconds, compile/link 224.042 seconds, and test stage 5.162 seconds. The harness
itself reported 0.01 seconds. These distinct timing scopes are not interchanged.
The post-run source manifest was reverified, and the two owned worktree hashes
still match reviewed snapshot 2975.

The log and self-contained managed receipt are retained at
`E:/cargo-targets/zircon-engine/cache/build-benchmarks/runtime55-asset-io-original-contract-20260908/results/runtime-asset-io-static-r3.log`;
the checked sample record is in the adjacent `runtime-asset-io-static-r3.json`.
This establishes the lower IO-owner regression. App results on exactly the same
sealed input are recorded below; no worktree or external dependency overlays were
added between these commands.

## 2026-09-08 original App reproduction and remaining upper failures

The original command with `-Package zircon_app -FeatureOverride target-server
-LibTests -TestFilter headless_bootstrap_stores_headless_platform_config` passed
in both required link modes. Static job `dde3fdbbad754c2cbd2618154179a355` and
development dynamic job `752f2bdfb3c046b28c8e6ed719b006f1` each executed 1 passing
test, 0 failed / 0 ignored, with 219 filtered out. Both entry and managed receipt
report exit 0; each harness reported 0.01 seconds. Dynamic mode used its distinct
coordinator compatibility pool and compiled/linked for 158.427 seconds. Static
compile/link took 2.336 seconds. Full source manifests were checked after both runs.

Logs and self-contained managed receipts under the corrected input directory:

- `results/app-headless-original-static.log`, with adjacent checked sample JSON.
- `results/app-headless-original-dev-dynamic.log`, with adjacent checked sample JSON.

The affected `-TestFilter entry::tests::profile_bootstrap -LinkMode static` batch
also ran, in job `2bdb2f9df8404674b8bafb208d1f914b`: 4 passed / 8 failed /
0 ignored, 208 filtered out. Cargo check passed; the entry reports exit 1. The
log is `results/app-profile-bootstrap-static.log`. No failed test reports the
original Compute-versus-IO assertion, but this whole batch is not accepted.

Seven tests fail earlier in `FoundationModule.Manager.ConfigManager` because
concurrent bootstrap instances select the same default config path and its
filesystem commit is already in progress:

- `entry_config_can_select_headless_render_profile_bundle`
- `runtime_bootstrap_stores_default_render_profile_bundle`
- `headless_bootstrap_stores_headless_platform_config`
- `headless_bootstrap_stores_absent_primary_window_descriptor`
- `runtime_bootstrap_stores_primary_window_descriptor`
- `runtime_bootstrap_excludes_editor_module`
- `runtime_plugin_bootstrap_installs_neutral_module_lifecycle_observer`

The inspected lower path gate in
`zircon_runtime/src/foundation/runtime/config_manager/commit_fence/registry.rs`
deliberately rejects competing commits to one canonical path. Its existing
regressions assert that behavior. The App fixture boundary must provide isolated
configuration ownership; removing the fence, editing the user's configuration,
or serializing the entire acceptance suite would not repair that boundary.

The eighth test,
`bootstrap_accepts_required_native_dynamic_plugin_from_export_load_manifest`,
has a separate native-plugin fixture failure: its `virtual_geometry` Runtime DLL
is absent, and the fixture manifest lacks current capability/packaging/target
contracts. These App fixture findings remain with the original App08 acceptance
owner; they are not claimed as Runtime55 production fixes or passing results.

The unchanged owned hashes still match independently reviewed snapshot 2975.
The successful validator jobs belong to operational Session
`validate-matrix:failure-roll-01a07160-runtime55`; their real execution evidence
must not be relabeled as fixing-Session `cargo_job_runs` or validation tickets.
The current failure remains open until separate upper failures are routed,
required independent closeout review and coordinator identity binding complete,
and the canonical return plus `fix(failure): asset-module-runtime-io-pool` commit
and SHA-deduplicated WeCom result are recorded.
