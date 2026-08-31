---
title: Editor UI Profile Source Binding and Allocation Evidence Audit
category: zircon_editor
report_id: Editor01-profile-source-binding-allocation-audit-2026-08-25
date: 2026-08-25
implementation_status: product_directory_bridge_and_fail_closed_preflight_static_validated
validation_status: product_capture_blocked
---

# Editor UI Profile Source Binding and Allocation Evidence Audit

## Decision

The current UI profiler has useful process, latency and subsystem-counter coverage, but its source
identity is not strong enough to call a capture `current-source`. It also has no general allocator
evidence. Product p95/CPU/RSS results must remain historical or provisional until the profile
manifest binds the exact managed build closure to the launched binaries and required CPU/heap
collectors fail closed.

The original review was read-only. The 2026-08-31 follow-up adds a narrow product-directory bridge
without absorbing the external scenario, WPR or counter work already present in
`tools/profile-capture-manifest.ps1` and `tools/ui-profile-capture.ps1`.

## Confirmed Coverage

The current toolchain already records several important authorities:

- `ui-profile-native-resize.ps1` samples one launched process ID and records elapsed time, total
  processor-time delta, average core/system utilization, start/end/peak working set, start/end/peak
  private bytes and post-interaction quiescent memory.
- `ui-profile-process-evidence.ps1` rejects missing or internally inconsistent PID, CPU, working-set,
  private-byte and quiescence evidence and supports CPU-per-operation budgets.
- `ui-profile-latency-evidence.ps1` computes nearest-rank p50/p95/p99/max and validates typed input
  outcomes against input-to-damage and damage-to-submit samples and present-batch membership.
- `ui-profile-counter-evidence.ps1` checks retained presentation, resize, visual-asset, SVG-tree and
  GPU image cache counters, including cache hits/misses, source visits, texture writes and resource
  generation behavior.
- `profile-capture-manifest.ps1` fingerprints the Editor executable, Runtime library, capture tools,
  selected source files, Git HEAD and the input fixture.

These are valuable components. The issue is not that the profiler has no evidence; it is that the
evidence does not yet prove the launched binary corresponds to the complete source under review.

## P0: Dirty Source Identity Is Path-only

`Get-ZirconProfileGitMetadata` hashes the lines returned by `git status --porcelain=v1`. Those lines
encode status and path, not file contents. Editing the contents of an already modified file while
leaving its status unchanged produces the same `dirty_tree_sha256`.

This creates a concrete false-acceptance case:

1. Build Editor from version A of an already modified UI file.
2. Change that same file to version B without changing its Git status category.
3. Launch the version-A binary and export a profile manifest.
4. HEAD, dirty-entry count and dirty-status hash can remain identical even though the worktree source
   no longer matches the binary.

The status hash is useful workspace metadata, but it is not a source-closure identity.

## P0: Critical-source Timestamp Gate Is Incomplete

`Export-ZirconProfileCaptureManifest` fingerprints only `Get-ZirconProfileCriticalSourcePaths` and
compares binary timestamps with the newest file in that curated list. The current list does not
contain this slice's pointer route, rich-link, editable-text pointer or command-palette renderer
files. An unlisted source can therefore change after the binary while the timestamp gate still
passes.

Timestamps are also not proof of provenance: copied or restored binaries can have newer timestamps
without containing the current source. Binary SHA-256 is necessary, but it becomes authoritative
only when a managed build receipt binds that SHA-256 to an exact source-closure hash.

## P1: Allocator Evidence Is Missing

Working set and private bytes reveal retention and gross growth, but they do not measure allocation
churn. A pointer path can allocate and free thousands of short-lived strings or vectors while
ending with flat RSS. Current `image_cache_key_allocation_count` counters cover one graphics-domain
allocation, not process-wide or UI-phase allocator activity.

The acceptance request therefore still lacks:

- allocation/deallocation call counts and bytes for the measured interaction window;
- peak live allocated bytes and post-quiescence live bytes;
- allocation deltas attributed to input routing, layout, render extraction, host-scene conversion,
  SVG parse/raster and GPU image preparation;
- a stable session/PID/source identity shared with latency and process evidence.

The original audit found that WPR started only the built-in `CPU` profile and could fail open. That
specific gap is now closed in the current tooling. `ui-profile-wpr.ps1` records the launched product
PID, executable fingerprint and process lifetime, then exports both a system sampled profile and a
PID/lifetime-filtered sampled-stack product through xperf. It sets `is_product_timing` only when the
filtered product exists. `ui-profile-preflight.ps1` also fails closed when WPR/xperf or the required
system-profile privilege is missing.

The 2026-08-31 focused WPR/preflight/machine-manifest contracts pass 18/18. The source-bound
preflight artifact is
`E:/zircon-profiles/ui-profile-preflight-20260831-r15.json` (SHA-256
`33A6B752FDB8427684FB921E07113613DC1305B9BA8A61679F4CF5C83E681073`). It binds 276 critical source
files and reports exactly three current blockers: missing managed Editor binary, missing managed
Runtime library, and missing WPR system-profile privilege. This is tooling/static evidence, not a
product CPU or allocation result. A general heap/allocation collector remains absent.

The managed build-to-capture directory gap is now closed statically. `tools/build-editor.ps1`
publishes `zircon_editor.exe` and `zircon_runtime.dll` directly beneath an approved
`D:/ZirconBuilds`, `E:/ZirconBuilds` or `F:/ZirconBuilds` child directory. The capture launcher now
accepts that exact directory through `-ProductDirectory`; when it is absent, the existing managed
`CARGO_TARGET_DIR/profiling` layout remains the fallback. `ui-profile-product-directory.ps1`
normalizes the path before authority checks, rejects root-only, unmanaged and traversal-escaped
paths, and applies the shared reparse-point guard to existing path components. The helper is part of
the capture-tool fingerprint closure.

The focused product-directory contract passes 5/5. The complete output-contract suite reaches
47/48: every directory, manifest, binary fingerprint and missing-tool case relevant to this bridge
passes. The one remaining failure is an independent current-source drift in the workbench pointer
owner: `pointer.rs` plus `pointer_feedback.rs` no longer contains the statically required
`ui.workbench.pointer.feedback_refresh_count`. It is not treated as bridge evidence and was not
repaired from this slice. No Cargo command or product capture was run.

## Target Evidence Contract

### Managed Build Receipt

The profiler must consume a managed build receipt containing at least:

- managed run/job/ticket ID and target triple/profile/features;
- Git HEAD plus the exact attributed overlay/source manifest;
- sorted compile-input closure entries `(relative_path, byte_length, sha256)`;
- a closure SHA-256 derived from those sorted entries;
- Editor executable and Runtime library SHA-256 values;
- build completion time as diagnostic metadata only.

The profile launcher must recompute both binary hashes and reject any mismatch. A dirty shared
checkout should be profiled only through an isolated managed source copy whose overlay manifest is
part of the receipt. `git status` remains descriptive, not authoritative.

### Profile Manifest Schema v3

The profile manifest should record:

- build receipt identity and compile-input closure hash;
- capture-tool closure hash;
- launched PID, executable path and executable/runtime hashes;
- scenario fixture hash and exact interaction counts;
- collector requirements and actual collector receipts;
- output root validation proving all durable artifacts are on the approved E/D/F roots;
- warmup/measured ordinal and trial-group identity.

The binary/source timestamp comparison can remain as a fast diagnostic, but it must not decide
source validity.

### Allocation Collector

Use either a Windows ETW heap profile bound to the launched PID or an instrumented allocator enabled
only by the profiling feature. The normalized artifact must expose:

- allocation/deallocation count and bytes;
- peak and quiescent live bytes;
- samples grouped by named UI phase and, when available, stack;
- lost-event/sample-completeness counters;
- collector start/stop timestamps and PID;
- linkage to the same manifest and scenario ID as latency/process evidence.

Acceptance runs must fail when the requested collector is absent, incomplete, PID-mismatched or
reports lost evidence above its explicit threshold. A `counter-only` mode may remain fail-open, but
it must not be accepted as CPU-stack or allocator proof.

## Required Regression Matrix

1. Two different contents at the same already-dirty path produce different source-closure hashes.
2. Editing an unlisted Rust/TOML/ZUI/shader/build input invalidates the receipt.
3. A binary with a newer timestamp but the wrong SHA-256 is rejected.
4. A receipt from another target/profile/feature set is rejected.
5. A PID mismatch between interaction, latency, WPR/heap and manifest artifacts is rejected.
6. Missing WPR or heap evidence fails a run that requested those collectors.
7. Lost or truncated latency/heap samples fail completeness gates.
8. Output paths on C: are rejected before collector start; durable artifacts remain on E/D/F.
9. Three measured trials after warmup retain distinct identities and aggregate p50/p95/p99/max
   without mixing historical or different-source samples.

## Product Acceptance Impact

No current-source `zircon_editor.exe` or `zircon_runtime.dll` is available in the approved managed
target pool, so this audit does not publish new CPU, RSS, allocator or latency numbers. A future
managed bundle can now be selected directly, but directory selection is not a build receipt and
does not close source-provenance acceptance. The current Windows token also cannot start the
required WPR system sampler. Once a managed source-closure receipt, current binaries and an elevated
capture token exist, the first accepted matrix remains idle hover, button click, editable-text drag,
200-step native resize, 10K command palette and visible SVG resize. Each must carry the v3 source
identity before its values can be compared with the UI budgets.
