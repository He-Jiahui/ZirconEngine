---
title: Plugin Net Content Download Current-Source Algorithm Performance Review
date: 2026-08-24
status: static_complete_m0_arithmetic_guard_implemented_dynamic_pending
scope:
  - zircon_plugins/net/features/content_download/runtime
canonical_owners:
  - docs/plans/optimize/zircon_runtime/08e-network-runtime-review.md
  - docs/plans/optimize/zircon_runtime/25-filesystem-path-uri-vfs-mount-watch-sandbox-atomic-io-review.md
  - docs/plans/optimize/zircon_plugins/10-first-party-network-source-runtime-editor-dist-catalog-transport-rpc-replication-product-integration-review.md
  - docs/plans/optimize/zircon_hub/03-marketplace-account-auth-organization-cloud-repository-provider-review.md
  - docs/plans/optimize/zircon_tooling/09-release-channel-artifact-repository-install-update-rollback-operations-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Online/BuildPatchServices/Public/Interfaces/IBuildInstaller.h
  - dev/UnrealEngine/Engine/Source/Runtime/Online/BuildPatchServices/Public/Interfaces/IBuildManifest.h
  - dev/UnrealEngine/Engine/Source/Runtime/Online/BuildPatchServices/Private/Installer/DownloadService.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Online/BuildPatchServices/Private/Installer/CloudChunkSource.h
  - dev/UnrealEngine/Engine/Source/Runtime/Online/BuildPatchServices/Private/Installer/MemoryChunkStore.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Online/BuildPatchServices/Private/BuildPatchFileConstructor.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Online/BuildPatchServices/Private/Installer/Verifier.cpp
---

# Plugin Net Content Download Current-Source Algorithm Performance Review

## 1. Status and frozen scope

The Content Download feature completed E3 current-source static review over **20/20 Rust files** at revision `080fefe6acd449beded4497dee4a474b9e1f7383`:

| Module folder | Files | Physical / non-empty lines | Bytes | Tests / ignored | Current fingerprint |
|---|---:|---:|---:|---:|---|
| `zircon_plugins/net/features/content_download/runtime` | 20/20 | 2,210 / 2,001 | 80,751 | 25 / 4 | `689c3eb501d27ac91b3f092ec4d23b648806abbe75a15c0f79b7d78dc7a0ab75` |

The fingerprint is SHA-256 over sorted `repository-relative-path|sha256(file-bytes)` rows joined by LF. Shared changes in `manager.rs`, `manager/{attempts,bitmap,resume,state}.rs` were preserved. All 20 files pass `rustfmt --check --edition 2021 --config skip_children=true` and the scope passes `git diff --check`.

Managed Windows Cargo is unavailable, so none of the 25 tests ran. Current product-call search across 16,753 Rust files in App, Editor, Hub, Runtime and Plugins outside this feature found zero manager, queue, fetch or resume caller. The production factory resolves canonical NetManager, while the HTTP feature currently installs a separate private HTTP-enabled manager; the required feature dependency therefore does not prove an HTTP-capable production handle. There is no current-source App/Hub download executable for WPR/ETW. RenderDoc is irrelevant until an installed asset is loaded into a rendered frame.

M0 added checked total-byte accumulation and checked per-chunk range ends in `manager/manifest.rs`, plus two overflow regressions in `tests/manifest.rs`. This closes debug panic/release wrap at manifest admission but is statically verified only.

This feature is a synchronous in-memory algorithm model, not an accepted downloader, cache or installer.

## 2. Per-file review result

| Module | Reviewed files | Result |
|---|---|---|
| Feature/package surface | `capability.rs`, `feature.rs`, `lib.rs`, `plugin.rs` | Registers a client-only facade and canonical manager handle, but no App/Hub/Editor consumer or product composition test exists. |
| Manager facade/state | `manager.rs`, `manager/state.rs` | One mutex owns every manifest, progress vector, attempt, diagnostic, full partial/completed chunk and bool bitmap; no ticket generation, filesystem, cache or task owner exists. |
| Manifest/attempts | `manager/{manifest,attempts}.rs` | Free-form unsigned URLs and strings drive sequential mirror selection; arithmetic/layout, host trust and replacement generation are incomplete. |
| HTTP/fetch/resume | `manager/{http_fetch,resume}.rs` | Synchronously buffers response and prefix, hashes on the caller, retains the complete chunk in RAM and forces development security. |
| Progress/bitmap | `manager/{progress,bitmap}.rs` | Caller-provided cache hits/bools mark chunks complete without content verification; cancel only changes an enum. |
| Tests | `tests/{attempts,feature_registration,http_fetch,manifest,progress,resume,support}.rs` plus inline modules | Single-process private-manager loopback and in-memory examples; no catalog composition, real cache/install, crash, disk fault, cancel race, hostile manifest, large artifact or cross-process resume. |

The framework DTO in `zircon_runtime/src/core/framework/net/download.rs` was reviewed with the feature. It has chunk URL/offset/length/hash, free-form mirrors and a progress enum, but no manifest version/provenance/signature/root digest, target artifact identity, dependency closure, cache key, security profile, staging/install path, generation, space policy or terminal receipt.

## 3. Current-source local optimization assessment

Four local improvements are present but unexecuted:

- Selected mirror lookup no longer constructs every candidate URL. The ignored 256-mirror/128-lookup fixture models 33,024 legacy URL string allocations plus 128 vector allocations versus 128 selected strings and no URL vector.
- Completion bitmap generation switches from nested string scans to a borrowed HashSet above eight completed chunks. The ignored 4,096-chunk fixture models 4,096 hash lookups instead of millions of comparisons.
- Applying a 512-chunk resume bitmap uses one state lock and one progress clone instead of the modeled 514 locks and 513 clones.
- Partial chunks use a nested download/chunk table, so the ignored 8,192-lookup fixture removes a 1,024-byte key allocation per lookup.

These are compatible local wins but do not fix the product algorithm. Resume apply still performs linear membership scans for each completed chunk, and each normal completion scans the manifest to find a bitmap index. Attempt and failure keys still allocate chunk strings. More importantly, all stores are unbounded and full payload bytes dominate allocation, hashing, mutex lifetime and retained RSS.

## 4. Structural algorithm findings

### P1: the feature cannot compose with production HTTP

The factory resolves canonical NetManager, but HTTP capability is owned by another private manager. Tests use `cfg(test)` injection of `http_runtime_manager()`, bypassing real catalog/Core composition. The feature has zero product caller.

Runtime08E must own one NetworkRuntimeInstance. HTTP registers an activation-generation backend lease into that owner, and Content Download receives an async HTTP facade from the same instance. A real catalog composition test must start App/Hub/exported dist and complete a loopback artifact without test-only constructors.

### P1: manifest is neither trusted nor layout-safe

Manifest identity is only process-local `NetDownloadId`; content source is free-form strings. There is no signature/provenance/version/root digest, target artifact key, dependency closure, allowed host/security profile, maximum chunks/bytes or rollback policy. M0 now rejects total-byte and per-chunk end overflow, but offset order, overlap/gap and URL/range semantics remain undefined. Per-chunk URLs suggest independent objects while nonzero offsets and range headers suggest pack-relative ranges; mirror URLs instead append chunk ID.

Compile a signed versioned artifact manifest before network or allocation. Validate checked counts/sizes/ranges, unique stable chunk IDs/content hashes, artifact/dependency closure, target variant, allowed origins, security profile and install transaction. Freeze whether offsets are artifact-relative or source-object-relative and encode that in the schema. M0 only hardens arithmetic; it does not establish trust.

### P1: synchronous full-memory fetch creates main-thread stalls and multiple payload copies

`fetch_next_chunk` performs a synchronous request with a 30-second timeout, then hashes the complete vector on the caller. A resumed attempt simultaneously retains the original prefix in state, a prefix clone, the response body and the combined vector; success leaves the complete chunk in `partial_chunks` indefinitely. Size rejection occurs after the HTTP backend already buffered the body. Request IDs repeat as `attempt_index + 1` across downloads and chunks.

Replace this API with a generation-qualified async ticket. Stream bounded blocks into Runtime25 staging while updating the content hash; reserve response/disk/cache bytes before I/O. Worker/executor policy owns hashing and disk work. Main-thread consumers receive bounded progress deltas only. Correlation IDs are globally unique within the runtime generation.

### P1: resume and cache completion are untrusted bools

`store_resume_bitmap` accepts arbitrary booleans and length. `apply_resume_bitmap` zips with the current manifest and calls those bits cache hits without reading bytes or checking hashes. Re-queueing the same download ID overwrites only manifest/progress and leaves old bitmap, partial bytes, attempt indices, diagnostics and cache hits, so a new manifest can inherit stale completion state by index/name.

Persistent resume metadata must be namespaced by immutable artifact/manifest digest and record staged byte ranges plus incremental-hash/checkpoint identity. Restart verifies file size/range/hash before reuse. Replacing a ticket generation atomically retires all prior state. A cache hit is a content-addressed verified lease, never a caller assertion.

### P1: corrupt resume data cannot self-repair

On final hash mismatch, the partial prefix is deliberately retained and the next mirror uses the same static range. If corruption is in the prefix, every mirror repeats failure; the system never invalidates the prefix and retries the full chunk. `resume_from_byte` is static manifest data rather than durable progress, and an end-exclusive resume offset is accepted by manifest validation but rejected later by range construction.

Range resume must use persisted staged length plus validator metadata such as artifact digest/ETag where available. A resumed hash mismatch invalidates the suspect checkpoint and schedules a bounded full refetch before terminal failure. Mirror health/backoff is independent from local corruption. Every transition emits a typed repair reason.

### P1: cancel, terminal state and concurrency are cosmetic

Cancel only sets `Cancelled`; it does not cancel an HTTP operation, prevent later fetch, release payloads or block completion. Fetch does not check current status. Concurrent calls for the same chunk can duplicate network/hash work, race failure indices and skip mirrors. Failed/complete/cancelled rows and diagnostics remain forever.

The ticket owner provides one atomic state machine with in-flight deduplication, cancel token, deadline and exactly one terminal receipt. Terminalization stops admission, cancels/joins requests, closes staging writers and cleans or retains artifacts by policy. Late callbacks are fenced by ticket generation.

### P1: scheduling, mirrors and resource budgets are absent

Callers manually select a chunk and invoke a blocking fetch. There is no dependency/order scheduler, concurrency limit, bandwidth or disk-pressure controller, priority/fairness, prefetch window, retry backoff, `Retry-After`, origin health/quarantine, space reservation or eviction. Failed diagnostic vectors and all maps are unbounded.

Use an install graph with bounded ready/in-flight/verified queues. Scheduler policy considers construct order, current/install reuse, origin health, retry class, bandwidth, staging space and cache pressure. It adapts concurrency within declared limits and never performs unbounded work because a manifest is large.

### P1: there is no stage, verify, publish, rollback or repair transaction

Successful chunks remain memory values. No file is constructed, flushed, fsynced, atomically published, mounted or associated with last-good state. `Verifying` is never a real stage. There is no disk-full/read-only/permission/partial-write/crash recovery or install lock.

Plugins10 M7 consumes Runtime25 atomic I/O and Tooling09/Hub ownership. Required states are admitted -> downloading -> staged -> chunk-verified -> artifact-constructed -> artifact-verified -> publish-prepared -> active, with cancelled/failed/rollback/repair terminals. Publication uses an owner lock, durable journal, same-volume staging, flush/fsync, atomic active-generation switch and deterministic restart recovery.

### P1: observation cannot quantify useful download work

Progress contains downloaded/total bytes and free-form diagnostic only. It cannot distinguish network bytes, reused cache bytes, written/verified/published bytes, retries, wasted/corrupt bytes, queue time, origin health, memory/disk high-water, cancellation latency, CPU hash time, wakeups or energy. No common diagnostics producer or Editor/Hub consumer exists.

Publish bounded generation-qualified ticket/origin/chunk metrics with sensitive URL/token redaction. Dynamic qualification reports useful installed bytes separately from transfer/retry/write/read/hash work.

## 5. Unreal evidence and adopted policy

Unreal BuildPatchServices is the primary structural reference:

- `IBuildInstaller.h:22-65,141-338,365+` separates typed install errors, control/state/progress and detailed download/install/verify statistics. Zircon's one progress DTO cannot represent an installer terminal contract.
- `DownloadService.cpp:176-192,194-220,256-269` assigns service request IDs, tracks active requests and provides per-request/all-request cancellation instead of changing a progress enum.
- `CloudChunkSource.h:108-158,172-198` configures retry count, bounded prefetch, bandwidth limit, health thresholds and approximately optimal simultaneous connections; it also delays cloud work until resume/source selection has run.
- `MemoryChunkStore.cpp:188-231` invokes an explicit capacity/eviction policy and moves booted chunks to an overflow store or a lost-chunk callback. Payload retention is governed rather than an unbounded HashMap.
- `BuildPatchFileConstructor.cpp:90-145` computes peak disk-space requirement before construction. Lines 315-399 bind resume data to install resume IDs and a staging directory instead of a process-local bool vector.
- `BuildPatchFileConstructor.cpp:463-535` defines memory and disk backing-store limits, headroom, fixed allocation spans and usage counters for chunks that must outlive their source.
- `Verifier.cpp:274-324,339-430` separates verification work, uses worker threads after construction threads return, selects staged paths, checks size and incrementally hashes files with pause/abort checks.
- `IBuildManifest.h:280-370` exposes file/chunk hashes, chunk requirements, sizes, feature/version and encryption identity as installer metadata rather than only a URL list.

Zircon should adopt these responsibility boundaries, not copy Unreal classes or historical hash choices. The minimum viable pipeline is trusted manifest -> checked admission/space plan -> bounded source scheduler -> staged incremental write/hash -> persistent content-addressed cache -> artifact construction -> independent verify -> atomic publish/last-good -> terminal receipt and telemetry.

## 6. Required optimization sequence

| Milestone | Required result | Acceptance gate |
|---|---|---|
| M0 Truth and arithmetic guard | Keep product capability unavailable; checked manifest byte/range arithmetic and regression tests; preserve compatible local wins. | **Implemented, dynamically pending:** two overflow regressions exist and 20/20 files pass standalone rustfmt; Cargo execution remains unavailable. Additional RED contracts must expose stale-ID inheritance, bool cache trust, cancel-after-fetch and corrupt-prefix mirror loops. |
| M1 Product composition and identity | One canonical HTTP/network owner plus generation-qualified ticket/artifact/manifest identities. | Real catalog App/Hub/export/native composition works with no test-only manager; stale callbacks cannot enter replacement tickets. |
| M2 Trusted manifest and plan | Signed/versioned manifest, stable chunks/dependencies/targets/origins, checked layout and space/budget plan. | Hostile/oversize/overlap/version/key corpus is rejected before I/O/allocation; plan is deterministic across restart. |
| M3 Bounded source scheduler | Async requests, cancel/deadline, in-flight dedup, origin health/backoff, adaptive bounded concurrency and bandwidth/space/cache limits. | Flood/slow origin cannot block main thread or exceed declared item/byte/age/in-flight limits; no chunk starves. |
| M4 Staging, resume and cache | Runtime25 staged block writes, incremental hash, manifest-bound resume journal and verified content-addressed cache/eviction. | Restart resumes only verified ranges; corrupt prefix triggers full repair; artifact larger than RAM completes within memory budget. |
| M5 Construct, verify and publish | Artifact/file construction, independent verifier, fsync/atomic activation, last-good rollback and repair. | Kill-point/disk-full/read-only/partial-write matrix yields active-old, active-new or typed recoverable state, never a mixed install. |
| M6 Product consumers and observation | App/Hub progress/cancel/retry UI plus common redacted download/install metrics and Editor diagnostics. | Consumers survive restart/reconnect; terminal receipt and high-water/retry/waste metrics agree with files and network traces. |
| M7 Dynamic qualification | Current-source catalog processes, local/remote origins, 1/10/100 GiB artifacts, warm/cold cache, fault/soak and WPR/ETW/power capture. | Publish BuildSet-bound P50/P95/P99 queue/chunk/install latency, throughput, CPU, RSS, disk/network bytes, wakeups and joules per useful installed GiB. |

Static current-source review and M0 source implementation are complete. Test execution and product/dynamic acceptance remain pending. No Git milestone commit or quantified WeCom notification is warranted.
