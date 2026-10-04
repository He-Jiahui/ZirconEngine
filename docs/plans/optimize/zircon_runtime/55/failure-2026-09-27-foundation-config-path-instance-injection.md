---
handoff_kind: failure
status: open
created_at: 2026-09-27
summary_slug: foundation-config-path-instance-injection
origin_plan: docs/plans/optimize/zircon_app/08-product-host-bootstrap-loop-dynamic-runtime-shutdown-current-source-review.md
fixing_plan: docs/plans/optimize/zircon_runtime/55-runtime-foundation-module-config-event-service-driver-manager-persistence-lifecycle-product-integration-review.md
origin_child_dir: docs/plans/optimize/zircon_app/08
fixing_child_dir: docs/plans/optimize/zircon_runtime/55
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/foundation/module.rs
  - zircon_runtime/src/foundation/runtime/config_manager.rs
  - zircon_runtime/src/foundation/mod.rs
  - zircon_runtime/src/foundation/tests.rs
tests:
  - cargo +1.94.1 check -p zircon_runtime --no-default-features --features target-server --lib --locked
  - cargo +1.94.1 test -p zircon_runtime --no-default-features --features target-server --lib --locked foundation:: -- --test-threads=2
  - tools/dev/dev-fast-build.ps1 -Action test -Package zircon_app -FeatureOverride target-server -LibTests -TestFilter profile_bootstrap -LinkMode static
  - tools/dev/dev-fast-build.ps1 -Action test -Package zircon_app -FeatureOverride target-server -LibTests -TestFilter profile_bootstrap -LinkMode dev-dynamic
---

# Runtime55: Foundation config persistence lacks per-instance host file authority

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_app/08-product-host-bootstrap-loop-dynamic-runtime-shutdown-current-source-review.md`
- 来源执行切片：Runtime55 IO-owner handoff upward App08 profile bootstrap replay.
- 修复责任计划：`docs/plans/optimize/zircon_runtime/55-runtime-foundation-module-config-event-service-driver-manager-persistence-lifecycle-product-integration-review.md`
- 交接原因：Foundation owns the normal persistence manager factory; App owns composition inputs and fixtures.

## 失败现象与复现证据

At main `bc02eefafead65dbf5050482110e8175250a5e77` the call chain is App ProductCompositionRequest ->
BuiltinEngineEntry::bootstrap -> normal Foundation descriptor ->
DefaultConfigManager::new -> production config_file_path selector. The latter
uses the same process/user file for all instances. Runtime's private cfg(test)
path override is absent when zircon_runtime is App's normal dependency.
ConfigCommitFence correctly rejects replacement while an admitted same-path
commit is running; that safety rule must remain.

The existing [asset IO handoff](failure-2026-09-07-asset-module-runtime-io-pool.md)
documents a historical profile batch of 4 passed / 8 failed, including config
collisions. Its exact job/log are no longer available; those counts are historical
record evidence, not a current dynamic result. The IO selector source is already
repaired and remains outside this new root cause. Current source diagnosis is
saved in `.codex/tmp/failure-roll-01a0df1a-app08-bootstrap-fixture-triage.json`.

No new Cargo reproduction or pass is claimed. The added two behavioral regressions
must execute at the managed boundary, together with the unchanged in-flight commit
regression and original App static/development-DLL profile acceptance.

## 最低共享层根因

The normal Foundation factory did not accept immutable per-runtime file authority.
ServiceFactory already supports an owned Send/Sync closure. Reuse that contract;
the path must not enter Core's persisted JSON, become user-editable state, or be
selected by a process-global test environment mutation.

## 架构修复验收

- Bind an absolute path to the normal descriptor before registration. Preserve
  its identity, init level, lifecycle, dependencies and canonical manager metadata.
- Reject empty/relative paths, another module, missing or duplicate canonical
  ConfigManager declarations before changing the descriptor; no fallback.
- Construct the real DefaultConfigManager with that path, preserving CoreWeak,
  recovery, atomic writer, debounce, bounded shutdown and ConfigCommitFence.
- Concurrent real descriptor activations must load and persist distinct owner
  values in distinct files, reopen them correctly, and release their Core roots.
- Keep `replacement_activation_fails_fast_while_an_admitted_commit_is_still_running`
  intact and execute it. No suite serialization replaces the isolation contract.
- App08's separate owner passes paths through ProductCompositionRequest and
  BuiltinEngineEntry and uses unique fixture files. The original profile batch
  remains pending until its independent native fixture/admission failures pass.

## 禁止临时方案

- No alternate/fake ConfigManager, JSON control key, global env override, fence
  weakening, suite-wide lock, default selector change or test-only injection API.
- No source/queue/zero-test evidence as dynamic acceptance; no foreign commit scope.

## 修复结果与回传

Open: `producer_and_consumer_source_present_managed_validation_pending`.
Runtime55 exports `bind_config_file_path` and the real manager's
`new_with_file_path`. Explicit authority is nonempty and absolute; default
construction is byte-preserved. Two lower regressions are added. Stable Session:
`failure-roll-01a0df1a-runtime55-config-path-r1`, epoch 628. Prior archived attribution was transferred by fingerprint
`47c021dff07666b96332cf4576580dca271990a79ee4ac6b629fde504b15ff0d`.
Current source hashes:

- `zircon_runtime/src/foundation/module.rs`: `cab0c330558fce82d8ea212e628d169503a4dd85005146f9ff69570d316f9ca1`.
- `zircon_runtime/src/foundation/runtime/config_manager.rs`: `9d9c705836818b291662876b6b20419e93e67cd2499cd1a9f9af03a6c289324e`.
- `zircon_runtime/src/foundation/mod.rs`: `487cd1e080c0231d175621b64256bbdfe7c24d556025d101dbd3830f3e4899f4`.
- `zircon_runtime/src/foundation/tests.rs`: `ac39f15117f57234e4614d7e92c08d30bdc25068d18b86fa5eaad6dfaa880f99`.


The unchanged existing fence-test file SHA256 is `f7ebe78f99f46084ddba8be0a255b711bf50a9015661bed6bf9021a02ea391e9`.
Preimages are retained under `.codex/tmp/failure-roll-01a0df1a-runtime55-config-path-before-*`.
App08 consumer changes are present under its separate active Session
`failure-roll-01a0df1a-app08-config-path-r1`: all six current source hashes
match epoch-628 attribution, and its local formatting and independent review
passed C0/I0/M0. Those source checks do not establish dynamic acceptance.
Managed admission still needs a valid full external inventory/archive;
no ticket, Cargo pass, return, closeout, commit or WeCom result is claimed.
All compiler products/caches must physically stay under coordinator-selected
drive-root `D:/cargo-targets`, `E:/cargo-targets` or `F:/cargo-targets`.
