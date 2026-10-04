---
title: Runtime04 Pack Export CLI Product Baseline
category: zircon_runtime
report_id: Runtime04-pack-export-cli-product-baseline-2026-09-26
date: 2026-09-26
implementation_status: fixture_complete
validation_status: managed_validation_pending
performance_status: product_samples_pending
---

# Runtime04 pack export CLI product baseline

Runtime04 M0 calls for reproducible 1K and 100K asset workloads, source-bound
measurements, raw latency percentiles, and memory evidence. The previous writer
and delta benchmarks isolate helpers. The ignored Windows Release integration
fixture now invokes the real `zircon_export_pack` executable for each sample.
It does not copy the writer or export implementation into the test.

| Workload | Input and oracle | Timed operation |
| --- | --- | --- |
| `pack_1k` | 1,000 assets, 16 KiB each, 900 unique payloads and 100 duplicates | Fresh CLI process, manifest load, trim, source-file reads, pack construction and write, report write |
| `pack_100k` | 100,000 assets, 256 bytes each, 90,000 unique payloads and 10,000 duplicates | Same full CLI path at 100K asset scale |
| `delta_1k` | 1,000-asset base; target has 750 changed/new, 250 reused and 250 removed paths | Full target export, delta creation, on-disk delta verification and application |

The test generates source files and manifests before timing. Inputs use relative
source paths and deterministic payload bytes. It verifies report counts,
deduplication counts, delta reconciliation and byte-identical old/new pack and
delta output hashes. The marker prints the target manifest and executable
BLAKE3 hashes. Input construction, output hashing and JSON report parsing are
outside the wall timer; process startup, CLI work, file output and 1 ms memory
sampling are inside it. Each sample reuses its variant's output directory and
removes its previous outputs before timing. Temporary corpus cleanup is
best-effort after the test or a preparation failure; filesystem cleanup errors
do not convert a valid performance sample into a failure.

Two current-binary tests use Cargo's `CARGO_BIN_EXE_zircon_export_pack` without
extra environment variables. They perform five warmups and 31 measured runs,
printing all raw wall, peak working-set and peak pagefile-commit samples plus
nearest-rank P50/P95/P99 for wall time and working set. The marker is
`RUNTIME04_PACK_EXPORT_CLI_CURRENT_V1`. On a managed Windows Release lane, run
both baseline scales in one Cargo invocation:

```text
cargo test --locked -p zircon_runtime --release --test runtime04_pack_export_cli_performance runtime04_pack_export_cli_current -- --ignored --nocapture --test-threads=1
```

The 100K corpus has 90,000 source files and 100,000 asset entries. Each CLI
invocation opens a source path for every asset and rereads unique payloads;
the suite performs 36 full CLI invocations. It is an expensive scale run;
the managed lane must reserve
up to 30 minutes for its samples after corpus setup. Each 100K child is capped
at five minutes. The 1K current suite is capped at ten minutes with a
90-second child cap. Comparison suites are capped at 60 minutes for 100K and
20 minutes for 1K/delta; delta children have a three-minute cap. Timed-out
children are sent a kill request and polled for reap for at most five seconds;
failure remains visible with stderr. The suite limit is checked before each
child, so the last admitted child can finish up to its own timeout after that
limit. The delta base-pack preparation also has a three-minute child timeout,
is outside measured samples, and attempts temporary-corpus cleanup on failure.

The opt-in `RUNTIME04_PACK_EXPORT_CLI_PRODUCT_V1` comparison requires
`ZR_RUNTIME04_PACK_BASELINE_EXE`, `ZR_RUNTIME04_PACK_CANDIDATE_EXE`, and
`ZR_RUNTIME04_PACK_MATCHED_SOURCE_ID`. Missing or identical binaries fail
before sampling. A matching source ID is an operator claim; the managed receipt
must independently seal each executable's source snapshot, compiler, target
profile and hash. The test uses five warmup pairs and 31 alternating AB/BA
measured pairs for each selected workload. It prints both raw series and
P50/P95/P99, and checks pack/delta hash parity. Provisional regression ceilings
are candidate wall P95 <= 105%, wall P99 <= 110%, and peak working-set P95
<= 110% of baseline. These are not speedup claims or the Runtime04 product
target. They have not been measured or accepted yet. The writer helper's
separate <= 95% hash-admission P95 target remains unchanged. With 31 measured
samples, nearest-rank P99 is the maximum sample.

When the managed lane can seal and inject all three required variables, run
the selected comparison cases serially in one Release invocation:

```text
cargo test --locked -p zircon_runtime --release --test runtime04_pack_export_cli_performance runtime04_pack_export_cli_product -- --ignored --nocapture --test-threads=1
```

Working set is the Windows process-memory proxy for RSS. The fixture samples
`K32GetProcessMemoryInfo` during execution and requires a final peak counter
query before accepting memory data. Windows documentation specifies the peak
counters but does not promise a successful post-exit query; if it fails, the
printed observed peak is only a lower bound and the memory gate fails closed.
It prints peak pagefile commit separately. The [Microsoft process memory
documentation](https://learn.microsoft.com/en-us/windows/win32/psapi/process-memory-usage-information)
defines these counters.
The tests print OS, architecture, package version, logical worker count,
Windows CPU identifier and temporary storage root. The five warmups make this
a warm-cache workload, but OS cache and other machine activity are not
controlled. The managed result must attach source fingerprint, rustc/build
profile, CPU and physical storage details, cache/power state, raw output and
the exact executable hashes. The current-binary marker alone is a descriptive
baseline; it does not prove a before/after improvement.

No Cargo or product measurement ran in this source slice. Runtime04 M0 pack
export evidence and the opt-in matched-binary comparison remain pending managed
execution. The fixture does not cover runtime pack mount, random reads,
1/10/100 GiB bundles, async asset loading, or editor frame stall. Those gates
stay open under Runtime04 M3/M7/M8 and the broader performance matrix.
