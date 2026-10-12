# Runtime dependency boundaries

The portable runtime core, native host, asset codecs and plugin workers have
explicit boundaries. These are build and persistence contracts; changing a
dependency must keep the applicable contract and validation intact.

## Portable core and native host

`zircon_runtime/core-min` contains the core and shared framework contracts.
Asset importing, filesystem watching, dynamic plugin hosting, scenes and the
native platform owner require `zircon_runtime/native-host`. Native client,
server, rendering, text and application entry presets select that feature.
`zircon_app/core-min` is a native application entry; it also selects
`zircon_runtime/native-host`.

`libloading`, `notify`, `meshopt`, `zstd`, `ring` and `goblin` are optional
dependencies under the non-Wasm target condition. `windows-sys` is under the
Windows target condition. Native command-line binaries require `native-host`.
The portable core must resolve without these dependencies and compile against
the real `wasm32-unknown-unknown` standard library. The full client, text,
rendering and Jolt worlds are outside this portable-core acceptance scope.

Run the resolved dependency guard and actual target check from the repository
root, with the Rust and Cargo homes and temporary directory under an admitted
physical `D:/cargo-targets`, `E:/cargo-targets` or `F:/cargo-targets` root:

```powershell
python -B tools/audits/check_runtime_dependency_boundaries.py --text
./tools/dev/local-cargo.ps1 -TargetDir D:/cargo-targets/zircon-local/wasm/local/target check -p zircon_runtime --lib --no-default-features --features core-min --target wasm32-unknown-unknown --locked
```

The `portable-runtime-core` GitHub Actions job executes both checks. It needs a
self-hosted Windows X64 runner with a physical `D:` build root and at least
35 GiB free, as required by local-cargo. A local pass does not establish that
this runner is configured or that a remote CI run has passed.

## YAML imports and persisted binary data

The data importer uses `serde-saphyr` with deserialization only. YAML remains a
supported asset format, with raw source text and extracted asset dependencies
preserved. Its explicit options keep YAML 1.2 boolean inference, treat `<<` as
an ordinary key and retain the last duplicate key. Filesystem includes remain
disabled. See the [parser documentation](https://docs.rs/serde-saphyr/1.3.0/serde_saphyr/).

`zircon_runtime_interface::serialization::PositionalEncoding` owns all runtime
positional serialization through `bincode-next`'s Serde adapter. The codec
configuration is explicit, following its
[compatibility migration guide](https://github.com/Apich-Organization/bincode/blob/main/docs/migration_guide.md).

| Persisted use | Encoding and compatibility |
| --- | --- |
| Interface binary v1 envelope | Little endian, variable integers; magic, wire version and payload validation limits retained |
| Animation binaries | Little endian, fixed integers; existing header/version handling and prefix decoding retained |
| Navigation artifacts | Little endian, fixed integers; current version and v1 migration retained |
| Cooked asset and render manifests | Little endian, fixed integers; existing schemas, chunk hashes, raw-size checks and version headers retained |
| UI compiler cache and generic payload disk envelope | Little endian, fixed integers; v1 disk records and serialized cache-key inputs retained |
| V2 UI file-cache body | Self-describing JSON v6; prior v5 entries rebuild from source through a distinct schema key |

The codec streams encoding once. A bounded writer rejects an output before
crossing the format's byte cap. Slice decoding rejects trailing bytes except
at explicit prefix-decoding call sites. Stream decoding caps the bytes read and
rejects oversized string/byte-buffer claims before allocating their contents.
Artifact readers separately check stream exhaustion and the declared raw size.
Its hard claim ceiling matches the existing 2 GiB cooked-artifact range;
individual formats retain their smaller caps. Variable integer claim
accounting also admits small wide integers that occupy only one wire byte.

Frozen v1 byte fixtures test both slice and stream paths. Malformed length
tests cover impossible and over-budget claims, and navigation/animation/asset
tests cover their existing version and corruption paths. Changes to payload
structure still require a format version or migration; replacing the codec
does not authorize an unversioned schema change.

UI cache records also have a frozen byte fixture produced by bincode 1.3.3.
The cache test checks identical writes, reopening the historical record and
eviction by its original asset identity. Cooked-artifact tests retain the
current v7 header and earlier-version rebuild boundaries. The scene document
bridge restores opaque JSON component values from their authoring TOML string
representation, including nested nulls and scalar strings.

V2 UI authoring and compiled DTOs contain dynamic TOML values and optional
fields omitted during serialization. Direct positional decoding cannot
reliably recover these shapes; bincode 1.3.3 rejects dynamic values with
`DeserializeAnyNotSupported`. The v6 file-cache body uses JSON with named
fields, retaining dynamic values and TOML's date-time representation. The v1
outer disk envelope remains byte-compatible. A v5 cache is rebuilt from its
source files, then the v6 record can be reused by another cache instance.

The dependency guard rejects `bincode` and `serde_yaml` in the portable,
native-text and data-importer graphs. Cargo lockfiles for both workspaces must
be updated together.

## Shared thread plan

`EngineThreadBudget::process_default()` creates one immutable plan from the
process's available parallelism. Default runtime task pools, Net services and
Jolt backends use that same plan. Hosts may supply `for_parallelism(...)` to
the explicit constructors and derive task-pool options from it.

| Owner | Planned threads |
| --- | --- |
| Frame/update caller | 1 |
| Net shared Tokio runtime | 1 async worker, at most 1 blocking worker |
| Net TCP/UDP socket owner | 1 |
| Jolt world | `min(saturating_sub(hardware, 7), 4)` workers; zero uses the update caller |
| Runtime task pools | `max(hardware - 4 - Jolt workers, 3)` workers across three domains |

The sum is `max(hardware, 7)`. Hosts below seven logical CPUs retain the three
independent task domains and parked Net owners, so the physical thread count
can exceed the logical CPU count. The plan reports that count explicitly.
Audio callbacks, lifecycle timers, additional Net services, additional Jolt
worlds and explicitly created task pools have separate owners; the table is
not a process-wide reservation allocator.

Net's socket worker stores a handle to the manager-owned Tokio runtime. It no
longer creates a second runtime. The socket worker stops before the shared
driver is dropped. Jolt receives an explicit worker count through
`JPC_JobSystemThreadPool_new3`, including zero for the small-host plan.

The opt-in `jolt_budgeted_physics_step_timing_sample` test measures 512 moving
bodies after 30 warmup steps, collecting 240 step samples for zero and four
Jolt workers. It prints p50, p95 and maximum microseconds and checks that all
bodies remain active and finite. The four/sixteen hardware plans are inputs
on the same machine; this is a physics-step sample, not a full-frame benchmark
or a timing pass threshold.

The local 2026-10-09 sample reported p50/p95/max of 195/766/19584 µs with zero
workers and 187/217/400 µs with four workers. Each configuration used the same
512-body fixture and 240 measured steps. These short samples describe the
observed physics step and can include scheduling outliers; they establish no
whole-frame performance improvement.

## Text dependency consolidation

Runtime SDF measurements use a borrowed `ttf-parser::Face` owned with shared
font bytes through `self_cell`. `fontsdf` and its extra `fontdue` parser and
`glam` version are removed from the runtime graph. Actual SDF/MSDF/MTSDF
rasterization continues through `fdsm`; bitmap rasterization uses `swash`.
The pre-generation SDF padding rule is retained, with recorded size, bearing,
advance and baseline fixtures from the former implementation. Real WOFF2
decoding and nonzero TTC-face rasterization remain covered.

The root runtime-text and plugin UI-importer graphs must each resolve one
version of `glam`, `ttf-parser`, `read-fonts`, `skrifa` and `swash`. Compatible
`swash` and cosmic-text dependency constraints share the same `skrifa` /
`read-fonts` versions. The resolved guard detects divergence in either lockfile.
Other duplicate package families need separate call-chain review; this
boundary does not require merging unrelated platform or tooling versions.

SDF tests explicitly admit packaged fonts into an owned collection. Windows
CJK tests explicitly enable system-font discovery and verify character
coverage before registering handles; a family query falling back to the
packaged ASCII font cannot serve as evidence of CJK rasterization.

Atlas admission refreshes the preceding frame's visible glyphs before font
handle resolution can enforce the LRU limit. This retains visible glyphs when
a newly admitted font causes cache pressure.

## Local verification, 2026-10-09

The real Wasm core check, native default Runtime check, and native App minimum
check passed. Focused tests passed: 84 Interface serialization tests, 165
Runtime asset/font/thread tests, 131 importer/Net/Physics/provider tests, and
8 UI persistence tests, plus one opt-in Jolt timing sample. The resolved graph
guard, manifest/preset parsing, CI YAML structure, and scoped formatting checks
passed. Local command evidence and failure details are recorded in the generated
`.codex/outbox/dependency-fix-20261009/review.md` artifact for this checkout.

Larger text and UI batches recorded additional failures outside the repaired
paths. Their logs remain in that record; this focused verification does not
establish a full-suite pass or remote CI/product acceptance.
