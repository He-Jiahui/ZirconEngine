---
title: Native Dynamic Fixture Current-Source Algorithm Performance Review
date: 2026-08-24
status: static_complete_shared_changes_preserved_dynamic_pending
scope:
  - zircon_plugins/native_dynamic_fixture
canonical_owners:
  - docs/plans/optimize/zircon_plugins/20-plugin-sdk-example-native-editor-fixture-test-carrier-artifact-isolation-product-truth-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
  - docs/plans/optimize/zircon_runtime/58-runtime-plugin-interface-bridge-slot-generation-strong-weak-native-vm-lifecycle-diagnostics-product-integration-review.md
  - docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md
  - docs/plans/optimize/zircon_runtime_interface/05-runtime-host-foreign-output-safe-api-ownership-admission-budget-fuse-observability-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/PluginDescriptor.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/ModuleDescriptor.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginManager.cpp
  - dev/UnrealEngine/Engine/Plugins/Tests/TestFramework/TestFramework.uplugin
  - dev/UnrealEngine/Engine/Plugins/Tests/ModularTestFrameworkTests/ModularTestFrameworkTests.uplugin
---

# Native Dynamic Fixture Current-Source Algorithm Performance Review

## 1. Status and frozen scope

The Native Dynamic Fixture completed E3 current-worktree static review over **2/2 Rust files** at revision `7fe97290fd3b0350c2c0f404fd00ad2d18f1335d`:

| Module folder | Files | Physical / non-empty lines | Bytes | Tests / ignored | Current fingerprint |
|---|---:|---:|---:|---:|---|
| `zircon_plugins/native_dynamic_fixture/native` | 2/2 | 882 / 820 | 35,219 | 8 / 1 | `4c31b8c036347f2bee7b3736d62153a8c124264959f0784f6354ed7c4dc5896f` |

The fingerprint is SHA-256 over sorted `repository-relative-path|sha256(file-bytes)` rows joined by LF. Both files pass standalone `rustfmt --check --edition 2021 --config skip_children=true`; package diff check passes.

Shared worktree changes in `native/Cargo.toml`, `native/src/lib.rs` and `native/src/tests.rs` are preserved. They remove ABI v2 fallback, hard-cut callbacks/buffers to V3, change the import response magic to `ZRIMO002`, add `reference_repairs`, and update the benchmark baseline wording. This review binds evidence to those current bytes and does not claim ownership.

Managed Windows Cargo remains unavailable, so the seven normal local tests, one ignored release benchmark and seven real-DLL loader tests in `zircon_runtime/.../real_fixture.rs` were not run. The checked-in manifest still says “ABI v2 fallback coverage” although the Cargo feature and code are gone. No current-source product executable exists for WPR/ETW or RenderDoc.

## 2. Per-file review result

| Module | Reviewed file | Result |
|---|---|---|
| Native ABI, runtime/editor behavior and importer | `native/src/lib.rs` | Declares Runtime+Editor entries, one main-thread Update system, one event, one Data importer, four commands, static state callbacks and failure features. Framing and output are bounded, but system/event/state/editor behavior do not exercise their declared contracts. |
| Unit/performance source | `native/src/tests.rs` | Covers overflow rejection, source budget, response parity/output budget, generated metadata and an ignored alternating P95 encoder benchmark. It does not run the full output sink path or system/event/state lifecycle. |

The package also contains Cargo/manifest and one shader asset pair. It is a normal experimental Standard plugin for Client, Server and Editor, with native-dynamic packaging and no TestFixture role or Shipping exclusion.

## 3. Local algorithm assessment

### Preserved M0 improvements

The current import request decoder is linear and bounded:

- `u64 -> usize` conversion is checked;
- metadata end uses `checked_add`;
- metadata is limited to 64 KiB and source to 256 KiB before JSON parsing;
- response writes use checked length addition and a host-declared maximum;
- typed `serde_json::to_writer` removes the baseline's owned response metadata tree and intermediate metadata Vec.

These close the prior panic/wrap and unbounded-input findings at the fixture boundary. They are source-reviewed only because Cargo did not run.

### Remaining buffer and measurement problem

`encode_import_response` still parses a full `serde_json::Value`, serializes the entire response into a plugin-owned `Vec`, and then `fixture_import_data_json` passes that Vec to `output.write`, which copies into the host-owned output buffer. At the 1 MiB command limit the dynamic path can retain approximately two full response buffers at once, plus the 256 KiB input and JSON tree overhead.

The ignored benchmark measures `encode_import_response` only and prints `bounded_full_response_buffers=1`; it excludes `output.write` and therefore cannot support an end-to-end one-buffer claim. Its allocation byte figures are modeled, not allocator measurements. The alternating 21-pair P95 timing is useful as a microbenchmark shape, but remains ignored, release-only and unexecuted.

Plugins01/Runtime Interface05 must either provide a reserve/commit or seekable/chunked host output contract, or use a two-pass length/count plus direct sink writer with explicit CPU tradeoff. Qualification measures plugin and host allocation high-water together. Until then the bounded encoder is a compatible local win, not the final transfer algorithm.

## 4. Structural performance findings

### P0: a fault-injection fixture can enter normal product and Shipping graphs

The package defaults to Standard, supports Client/Server/Editor and exports panic, overflow, missing-descriptor, missing-entry, unknown-ABI and missing-capability variants under the same package/crate identity. Repository scan and dist CI do not establish a TestFixture/negative-artifact boundary.

Plugins20 M0/M1 must add TestFixture role, hidden/disabled/explicit-load policy, Shipping denial and variant-qualified artifact receipts. Positive and each negative variant have distinct identity, source digest, feature set, expected failure stage and storage/catalog eligibility. Default MVP and Shipping graphs contain zero fixture module, DLL or entry symbol.

### P1: every activated frame can perform useless main-thread native work

The runtime registration declares `Update`, `main-thread-only`, `write:world` and a native `tick` bridge. `fixture_runtime_tick_bridge` only returns OK and writes no host-owned state. If this fixture is accidentally activated, every update pays scheduling, access arbitration and an FFI call on the main thread for zero useful work. The theoretical call rate equals the engine update rate; no current-source measurement exists.

The positive fixture must increment a host-owned generation-qualified test counter and the real-DLL test must verify stage, ordering, affinity and declared access. Negative/no-op variants must not register a per-frame system. WPR/ETW acceptance reports calls, main-thread CPU and useful state changes, with exactly zero default-product calls.

### P1: event capability has no emit algorithm

`native_dynamic_fixture.echoed` appears in registration and an ad-hoc event manifest, but no host event callback or emit path exists. Real fixture tests only search the manifest string. Event schema, ordering, backpressure, loss and unload fencing are untested.

Plugins01/Runtime58 must expose a bounded host-owned event API. Echo emits one typed event correlated to its command receipt; flood tests exercise queue item/byte/age limits, loss accounting and unload rejection.

### P1: state save/restore does not own state

Save always allocates the same constant blob; restore only compares bytes; commands/tick never mutate state; unload releases nothing. The tests prove buffer ownership and callback dispatch, but not state migration, quiescence or lifecycle cleanup.

The positive variant owns a small host-observable state record mutated by tick/command. Save captures its schema/generation, restore affects later behavior, and unload joins callbacks and revokes state. Reload rejects stale generation and verifies migration or explicit incompatibility.

### P1: Editor entry is an empty success path

Editor capability and entry are published, but native editor registration contains zero extensions, empty command/event manifests and an always-DENIED callback. Host-ready only logs/records one diagnostic. Product discovery can pay DLL entry and status work without receiving Editor behavior.

Either add one minimal real Editor contribution/lifecycle test or remove Editor target/capability from this fixture. Empty entry success cannot count as capability readiness.

### P1: output budgets are bounded but errors are not diagnosable

Malformed framing, budget failure, invalid UTF-8/JSON and response overflow all collapse to the same static `asset import request was malformed` status. Metadata strings can consume the shared 64 KiB budget and are copied into summary/diagnostic response text. The request includes a machine `source_path`, which is echoed without redaction.

Return stable error code/stage/limit/observed values without exposing raw local paths. Host and plugin independently enforce metadata/source/response/temporary budgets and emit per-stage allocation/latency metrics.

### P1: manifest and current artifact contract have drifted

The shared hard cut removes `abi_v2_only`, but generated `plugin.toml` still describes ABI v2 fallback. The package declaration says ABI v3. Hand-editing the generated file would hide the broken generation path.

Regenerate all carrier metadata from one typed definition and compare descriptor symbols, ABI versions, feature variants and target policies before build. Current mismatch fails fixture qualification but does not justify reverting the V3 hard cut.

### P1: real-DLL tests are strong but omit the performance-critical claims

`real_fixture.rs` source builds/loads the cdylib and exercises descriptor/entries, echo, panic containment, output overflow, constant state, unload, negative features and Data import. It does not invoke the registered tick through the scheduler, observe an emitted event, validate Editor contributions, distinguish artifact variants or measure buffers/CPU/wakeups. None ran in this review.

Extend the existing real artifact harness rather than create a parallel loader. Bind every result to BuildSet/artifact variant and keep negative artifacts outside product/export catalogs.

## 5. Unreal evidence and adopted policy

Unreal's current source makes test/developer load policy explicit:

- `PluginDescriptor.h:131-163` separates default enablement, hidden and explicit-load policy from discovery.
- `ModuleDescriptor.h:102-107,163,187,236` expresses DeveloperTool, loading phase, configuration deny and compile eligibility.
- `PluginManager.cpp:2185-2191` checks target/configuration module eligibility; lines 2909-2917 skip explicitly loaded plugins during normal phase loading; lines 3351-3718 own explicit mount/unmount.
- `TestFramework.uplugin` is disabled by default and uses DeveloperTool. `ModularTestFrameworkTests.uplugin` is disabled, tagged as a test plugin and denied in Shipping.

Zircon should adopt the separation rather than the names: production discovery, positive fixture qualification and negative artifact qualification are distinct resolved graphs. A fixture is valuable only when its declared system/event/import/state/editor behavior really executes and cleanup is observed.

## 6. Required optimization sequence

| Milestone | Required result | Acceptance gate |
|---|---|---|
| M0 Preserve boundary hardening | Keep checked framing, 64/256 KiB input budgets, bounded writer and typed streaming serialization; add typed error corpus. | Integer/length/JSON fuzz has zero panic and never exceeds declared input/output/temporary limits. |
| M1 Fixture/variant isolation | TestFixture role, explicit load, Shipping denial, positive and negative artifact identities. | Default MVP/Shipping have zero fixture work; every negative artifact is selectable only by its expected-failure test. |
| M2 Real system/event/state | Host-owned counter mutation, typed event emission/backpressure and mutable save/restore/unload generation. | Scheduler and event harness observe useful effects, ordering, limits and stale-generation rejection. |
| M3 Import transfer contract | Remove plugin-to-host full-response copy or document bounded tradeoff; typed redacted failures and allocator metrics. | Real FFI import stays within input/output/temporary budgets and reports plugin+host allocation high-water and P50/P95/P99 latency. |
| M4 Editor truth and target matrix | Real minimal Editor contribution or remove Editor entry; Client/Server/Editor behavior matrix. | No target reports capability without executable behavior; unsupported combinations fail before load. |
| M5 Real artifact lifecycle | Current V3 cdylib build/load, command/import/system/event/state/unload/reload plus negative variants. | Cross-platform results or policy skips bind ABI, feature set, artifact digest and failure stage. |
| M6 Dynamic performance qualification | Default-excluded and explicit positive/fault workloads under WPR/ETW. | Default overhead zero; publish DLL bytes/load, per-frame bridge CPU/useful changes, event loss, import CPU/RSS/allocation/I/O, wakeups and energy. |

## 7. Direct-fix decision and dynamic status

The simple framing/encoder hardening is already present in current source. Removing the no-op system, adding fake event/state side effects or hand-editing the generated manifest before fixture-role and host-owned contracts exist would weaken the test or hide drift. This review preserves shared changes and makes no further source edit.

Static review is complete. Cargo/local/real-DLL tests, scheduler/event behavior, artifact isolation, WPR/ETW, RenderDoc and power acceptance remain pending. No Git milestone commit or quantified WeCom notification is warranted.
