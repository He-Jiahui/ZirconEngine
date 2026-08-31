# M1 Durable I/O Artifact Identity Exhaustion Preflight

Date: 2026-08-31  
Plan: Frameworks 01  
Session: `frameworks01-shader-invocation-hard-cut-r12-1b2684b4-20260825`  
Status: `production_implemented / static_green / profile_harness_ready / managed_build_green /
full_test_and_profile_pending`

## Objective

Hard-cut the remaining wrapping process-local identifiers in Resource durable I/O. Atomic sibling
files and durable transaction artifacts must never reuse an identity within one process lifetime,
must fail closed at identifier exhaustion, and must not let one journal owner delete another
owner's stale or live artifacts.

This preflight follows the required order: whole-module code review, reference-engine comparison,
algorithm and invariant selection, then implementation. It does not authorize a local counter-only
patch and it does not claim a performance result. No production source listed here was edited by
this preflight.

## Admission and evidence rules

- Run the unchanged current-source ResourceManagement/readiness release profile before production
  mutation. The canonical managed-sccache Failure remains open until that origin command produces
  profile artifacts.
- Use the same current-source durable transaction tests before and after the hard cut. Record exact
  source hashes and managed job/run receipts.
- Do not replace `u64` with a larger wrapping integer, random retry loop, timestamp, or unchecked
  UUID helper. Exhaustion is an explicit terminal state.
- Durable journal schema is version-only. Version 6 and the old `{pid}-{sequence}` transaction ID
  wire are rejected after the cut; there is no compatibility reader or migration fallback.
- Existing `create_new` publication, WAL ordering, target validation, owner lock, and fail-closed
  recovery remain mandatory. A namespace token augments these authorities; it does not replace
  filesystem exclusivity or `PathIdentity`.
- Performance acceptance requires measured allocation and elapsed profile artifacts. Static
  operation counts and collision reasoning are not reported as speedup, power reduction, or proof
  of optimal scale.

## Current-source defect map

### Atomic sibling allocation

`atomic_file/pathing.rs::unique_sibling_path` obtains `NEXT_ATOMIC_FILE_ID` with
`fetch_add(1, Relaxed)`, combines it with the process ID, checks that the path is missing, and
returns it. The later staging open correctly uses `create_new`, but the counter silently wraps to
zero. After wrap, a stale sibling from the same PID namespace can be selected again. The existence
probe avoids an immediately visible collision; it does not establish a non-repeating identity
contract and it cannot distinguish exhaustion from ordinary collision search.

The parser in `is_atomic_write_transaction_path` also accepts sequence zero. Zero therefore cannot
currently serve as a terminal sentinel without a hard cut.

Current hashes:

- `io/atomic_file/mod.rs` `88b80eae9ad92d04dfb17cf949c150acb8f415cfd4ba64e0b041540577539fb9`;
- `io/atomic_file/pathing.rs` `0965d058dfcfc8f7f174eebaf26bf80db1195ccb36eead66b6ad3a65b7979358`;
- `io/atomic_file/transaction.rs` `b017dacd800b050b53d65aa07ef438c3533546fe8664463941397563d30699f1`.

### Durable transaction allocation

`transaction/pathing.rs::next_transaction_id` has the same wrapping allocator and emits only
`{pid}-{sequence}`. The ID is persisted in the journal and embedded in staging, backup, rollback,
and journal paths. `valid_transaction_id` accepts any parseable `u64`, including zero.

The transaction owner lock is scoped to the canonical journal directory. It correctly serializes
writers for one journal owner and `reject_pending_recovery` prevents a second transaction from
starting while that owner has pending evidence. It does not serialize two different journal
directories that target the same file.

That distinction is currently unsafe because `stage_file` calls `remove_reserved_if_exists` on
artifact paths before opening them with `create_new`. Two different journal owners can generate
the same `{pid}-{sequence}` after process restart or PID reuse and can target the same file. The
later owner can then delete the first owner's artifact because the artifact name contains no
journal-owner namespace. This is a structural ownership defect, not a retry-count detail.

Current hashes:

- `io/transaction/pathing.rs` `ee5eadbd55dafb10f098d1db102c3cf1443bc7e8c6a29dc9964e43835c107afd`;
- `io/transaction/engine.rs` `8516e63b0ee05a208cbecc8f17efeec8bf77f70e56225c1c2973c3fb033c4890`;
- `io/transaction/stage.rs` `721e6913f3ffac0a4f39ec0b523c458a88e88c5aa0a6d59d4372079e1a15df14`;
- journal schema version is currently 6.

### Scope boundary

This slice closes artifact identity and exhaustion. It does not silently introduce a global lock
for every target path. Cross-owner concurrent writes to the same target are a separate admission
and conflict-policy problem. After this cut, different owners cannot delete one another's
artifacts; existing atomic replace and recovery semantics continue to decide the target result.

## Reference-engine review

### Unreal Engine, primary reference

`FPaths::CreateTempFilename` uses `FGuid::NewGuid` and still checks for an existing candidate in a
loop. `FFileManagerGeneric` separately exposes no-replace file creation behavior. The useful
architectural lesson is layered uniqueness plus filesystem exclusivity: an identifier narrows the
namespace, while the filesystem remains the final admission authority.

Reviewed local sources:

- `dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/Paths.cpp`
  `57f7bb3adab5f0ad888d1e3b8723959404236b82907e73a7e1525bd0606ba0db`;
- `dev/UnrealEngine/Engine/Source/Runtime/Core/Private/HAL/FileManagerGeneric.cpp`
  `d40fd61f7ece87024c308ec3096ccaf106952651200048443daf352eb5608242`.

Zircon retains its stronger `create_new` and WAL behavior. It does not copy a UUID call whose local
implementation can panic when OS randomness fails, because durable I/O needs an explicit failure
path and adding a new manifest dependency is not justified for a monotonic process-local
authority.

### Godot and Bevy, secondary checks

Godot temp-file creation combines datetime, ticks, and an incremented suffix before ordinary open.
That is useful as a portability check but is weaker than Zircon's current create-new/WAL boundary
and is not adopted. The routed local Bevy snapshot was also searched for an equivalent durable
multi-file journal/owner namespace; no matching authority was found, so Bevy is not used as
positive evidence for this slice.

Reviewed local source:

- Godot `core/io/file_access.cpp`
  `1b47370e0d34ea506cc8d467e43c2d9143ed88b354bd8c51a171e1e17a14a343`.

The reference decision is therefore: keep Zircon's filesystem exclusivity and durable journal,
add a checked monotonic terminal protocol, and partition transaction artifacts by journal owner.

## Selected architecture

### Shared checked sequence authority

Add one private durable-I/O sequence primitive used by both atomic-file and durable-transaction
allocators. It receives an `AtomicU64`, returns `NonZeroU64`, and uses a compare-exchange loop:

1. values `1..u64::MAX - 1` advance to their checked successor;
2. `u64::MAX` is issued exactly once and atomically transitions the counter to zero;
3. zero is terminal and returns a typed `ArtifactIdentityExhausted` error;
4. no reset API exists in production;
5. ordering remains `Relaxed` because the atomic only allocates uniqueness and does not publish
   payload memory.

The compare-exchange protocol is O(1) expected time, O(C) retries under C-way contention, and O(1)
space. It avoids the ambiguous `fetch_add` result where zero can mean either initial state or wrap.
Tests inject a private counter near the boundary; production statics remain write-inaccessible.

Atomic-file APIs continue to expose their existing I/O domain boundary. The typed exhaustion value
is retained as the inner error of `io::Error`, so callers do not receive a fabricated collision or
retry forever. Durable transactions add an explicit `DurableTransactionError` exhaustion variant.

### Atomic sibling identity

The atomic sibling wire remains `{target-token, role, pid, nonzero-sequence}`. On collision,
allocation proceeds to the next checked sequence. `create_new` remains the final race authority.
Sequence zero is rejected by the recognizer. If the last issuable sequence collides, the next
allocation returns exhaustion; it never wraps or selects an older sibling.

This keeps leaf names short and avoids adding a journal concept to standalone atomic writes.

### Journal-owner transaction namespace

The durable transaction ID hard-cuts to:

`{journal-owner-token}-{pid}-{nonzero-sequence}`

`journal-owner-token` is the full lowercase BLAKE3 hex digest of the canonical journal-directory
operation-path encoding already established by `PathIdentity`. On Windows it hashes the canonical
UTF-16 units; on Unix it hashes the canonical raw bytes. Recovery recomputes the token from the
canonical directory containing the discovered journal and requires an exact match.

The token is a namespace discriminator, not a path equality authority. Windows case-insensitive
identity, ancestor checks, symlink handling, and verbatim operation-path validation remain owned by
`PathIdentity`. The owner lock remains the concurrency authority for one journal directory.

The current basename token is already 64 hex characters and tag length is bounded to 32. With a
64-hex owner token, maximum PID and sequence, and the longest current role/suffix, the leaf remains
below 255 bytes. No digest truncation is needed.

Because different journal directories produce different artifact names, one owner cannot reach
another owner's staging/backup paths through `remove_reserved_if_exists`. Same-owner restart reuse
remains governed by the owner lock, pending-recovery gate, and journal cleanup.

### Schema and validation hard cut

Bump the journal schema from version 6 to version 7. Version 7 validation requires exactly three
transaction-ID components, a 64-character lowercase hexadecimal owner token, a valid PID, and a
nonzero sequence. It then recomputes the owner token and all expected artifact/journal paths.

Version 6 and every two-component transaction ID are rejected as unsupported/invalid before any
artifact mutation. There is no dual parser, best-effort recovery, zero sentinel compatibility, or
legacy filename deletion path.

## Required RED/GREEN matrix

### Checked sequence

- a counter at `u64::MAX - 1` issues `MAX - 1`, then `MAX`, then typed exhaustion;
- repeated terminal calls remain exhausted and never return zero or one;
- concurrent callers around the boundary receive each available value exactly once, with every
  remaining caller receiving exhaustion;
- static guards reject `wrapping_add`, `fetch_add` identity allocation, and production reset APIs in
  the durable-I/O owners.

### Atomic files

- a stale candidate collision advances to the next checked nonzero sequence;
- collision at the final issuable sequence terminates with the typed inner error;
- recognizer rejects sequence zero and malformed suffixes;
- existing create-new race, replace, backup recovery, directory sync, and symlink rejection cases
  remain GREEN.

### Durable transactions

- two canonical journal directories targeting the same file produce different owner tokens and
  different staging/backup/journal paths;
- staging for owner B cannot delete or overwrite an artifact named for owner A;
- same journal owner produces a stable token across planning and recovery;
- version 6 and `{pid}-{sequence}` journals fail closed before cleanup or target mutation;
- owner-token mismatch, uppercase/short digest, zero sequence, extra components, and artifact path
  mismatch all fail closed;
- commit, rollback, crash recovery, torn-tail handling, pending-owner rejection, and multi-file
  durability remain GREEN under version 7.

## Performance and validation plan

The checked allocator adds a compare-exchange loop; owner hashing adds O(D) work once per durable
transaction for D path-encoding units. Artifact planning remains O(W) for W writes and disk I/O
dominates expected production cost. This is only a complexity argument.

Acceptance requires:

1. rustfmt and scoped diff/static checks;
2. focused boundary and cross-owner tests;
3. the complete managed `zr_resource --lib` suite on an approved non-C target;
4. the unchanged current-source ResourceManagement/readiness profile before and after the cut;
5. an ignored durable-I/O microprofile with fixed 1/16/256-write batches reporting p50, p95, MAD,
   allocations, bytes, peak live bytes, and exact source hashes;
6. separate RSS/I/O/power evidence before any power or bottleneck-elimination statement.

An allocator boundary test is not a benchmark. A modeled BLAKE3 byte count is not an elapsed-time
result. The algorithm is accepted only if the measured profile shows no material regression and
the identity/cross-owner RED matrix is GREEN.

## Ownership and execution gate

Coordinator read-only matrices show the atomic-file module, transaction pathing, and transaction
engine attributed to the current Frameworks 01 lifecycle with stale attribution and no live foreign
lease. Exact request evidence:

- atomic-file prefix matrix `ea9e9f1cb78f471ca0c1ca30f758112a`;
- transaction pathing matrix `e3a6e35f62e745f58d791c91e40dce50`;
- transaction engine matrix `114541c7714144229a5d4ec8570a0a64`.

The implementation owner must fresh-claim every exact source/test path before mutation and
baseline-attribute its current hash. The shared sequence owner and journal schema/recovery owners
must be included in the immutable write scope; no mixed foreign blob is absorbed.

Production remains held until the unchanged exact-origin profile returns its artifacts. If the
shared Cargo window is occupied or an orphaned foreign job has not been officially released, work
continues on static architecture/test planning rather than bypassing FIFO or starting unmanaged
Cargo.

## Current status

I3 production is implemented under the selected architecture:

- `ArtifactSequence` issues nonzero identities through checked compare-exchange, publishes
  `u64::MAX` once, then returns typed `ArtifactIdentityExhausted` forever;
- atomic sibling collision search rejects zero and returns the typed exhaustion value as the
  `io::Error` source instead of wrapping;
- durable transaction IDs are `{full-owner-blake3}-{pid}-{nonzero-sequence}` and recovery validates
  the token against the canonical journal directory before artifact mutation;
- journal schema is hard-cut from version 6 to 7; legacy two-component IDs, zero, malformed digest,
  owner mismatch and old version evidence fail closed;
- different journal owners cannot name or delete one another's staging/backup artifacts, while
  create-new, WAL, owner lock and `PathIdentity` remain the admission authorities.

Current SHA-256:

- `io/artifact_identity.rs`
  `2e399e4a9d5c46fe02080e824ee0fa7e85bb285b0e94bea5fdc5e9514bed94fa`;
- `io/atomic_file/pathing.rs`
  `7c7bf55cbf6fc68878668194eec3139dd5f639fcb4990f7898765114acacf771`;
- `io/transaction/pathing.rs`
  `2af0baf1e6e5f5769e398d5537dffb9850fa2cdffa7a728e5de4ea6626856445`;
- `io/transaction/engine.rs`
  `b1505ad0e7dc926e7009bd03ac9c7c2616b7c20d5a46562874acde792cedcd1a`;
- `io/transaction/schema.rs`
  `d769d17ce813267909acca34e97aff190c43b1d6a0911e5aac24aa45cd4453f0`;
- `io/transaction/recovery/validation.rs`
  `739326bf7b3284796d19d683b1831bb1b23fa96dc9a6277c4dcf593355c77d6b`.

The required ignored release microprofile is materialized at
`engine/tests/durable_io_profile.rs`, SHA-256
`8cb2a4eed733b6e8800df8921a35cd8637f3025a6d3502bc5d408292011629dc`. It runs real complete durable
commits at fixed 1/16/256-write batches, at least 31 measured samples plus 3 warmups, and writes
raw/summary/metadata artifacts only to an explicit non-C directory. The measured interval excludes
fixture construction, validation reads and cleanup. Reports include p50, p95, MAD, allocation
count, requested bytes, peak live bytes and BLAKE3 source fingerprints; RSS/I/O counters/power stay
explicitly unavailable until external evidence exists.

Frameworks resource boundary guards are GREEN `14/14`; rustfmt and scoped diff-check are GREEN.
After the foreign RuntimeInterface03 E0277 was fixed, managed job
`a31f2a72cba34ced8b5dce40854359de` reached owned test compilation and exposed a journal helper
visibility error. `frame_codec::encode_frame` is now visible only inside
`crate::io::transaction`, SHA-256
`B800148F8E147869260307E413F6A9B21F726C84D4B655D0A13C40586627B1CD`; it is not a public Resource
capability. Follow-up job `6486a2ea6b664b2ba0130ab61193090b` completed a GREEN production build,
linked the current lib-test binary, and every executed durable transaction test passed. Its sole
suite failure was an unrelated stale source-contract spelling, now fixed. A later rebuilt suite
exposed an I2 blocking-receiver lifetime deadlock before reaching durable tests; that owned fix is
implemented and awaits focused/full Cargo validation behind foreign job
`42419d28b8254edf816b6b125bfa3eeb`.

The durable release profile has not run and has produced no sample artifact. I3 is source-complete,
not accepted: managed full-suite correctness, measured profile, independent review, milestone
commit and WeCom completion notification remain pending, and no speedup, bottleneck-elimination,
engine parity or power claim is made.

### 2026-08-31 current-source behavior closure

The first retained-output full managed rerun, job `338da8564d6d4a8eab23b2b73968c76d`, made the
previous opaque Cargo 101 actionable: the only real failure was
`split_at_deepest_existing_ancestor_preserves_parent_components`. The fixture used chained
`PathBuf::join("missing").join("..")`; Windows collapsed that pair before the production resolver
received it. The resolver also used normalized `Path::components`, so an externally supplied raw
lexical parent component could not be retained independently from Win32 metadata normalization.

`split_at_deepest_existing_ancestor` now separates those authorities. Platform-native raw UTF-16
on Windows and raw bytes on Unix identify the lexical tail, while filesystem metadata only decides
whether the current physical candidate exists. The regression fixture constructs the exact raw
`missing/../asset.zmeta` input through `OsString::push`, without Unicode conversion or a normalized
`PathBuf::join` round trip. The leaf-to-root probe order and one final canonicalization remain; the
function does not return to root-to-leaf probing. Current `io/transaction/pathing.rs` SHA-256 is
`2af0baf1e6e5f5769e398d5537dffb9850fa2cdffa7a728e5de4ea6626856445`.

Validation evidence on the exact current source is:

- latest managed-storage helper/test SHA-256 values are
  `7d1eb4fe2bad2fb7bc124efcac272c187226b9a6f52dbdf9c86e4cd5342f74d9` and
  `4798293a9503186b1917aa5dc5074bbbc005dacd866868366f4eb529d1502cc9`;
- artifact audits `e70da9153ec842c694629a7c65647200` and
  `a52f5b4be1354507b81e488af0e46018` both returned `unmanaged=[]`;
- Resource static boundary tests are GREEN `14/14`; rustfmt and scoped diff-check are GREEN;
- focused managed job `8bdf433f83b9403ebfc590c70f9f9c4a` completed build and the exact parent-component
  regression with exit 0;
- the linked current binary passed five default-parallel full runs, each
  `218 passed / 0 failed / 11 ignored`, in 1.77--2.84 seconds;
- full managed job `e0005cfdfca4412db14497195d7a52cc` completed build and
  `cargo test -p zr_resource --locked --lib` with exit 0. The visible poison panics are deliberate
  lock-recovery probes and the harness result is GREEN.

The foreign unmanaged
`cargo +1.94.1 test -p zircon_runtime --lib structure_convention --locked --jobs 1` process was
allowed to finish naturally. With `cargo=0` and `rustc=0`, artifact audit request
`32286d5a17f54e368eca884f86cfdeb7` returned `unmanaged=[]`, after which the exact single-threaded
Windows release profile ran as managed job `04e2338eebc74091a4827eaedae49d98`. The job started at
`2026-08-31T14:39:37+08:00`, finished at `14:47:22`, released at `14:47:26`, and returned exit 0.
Its target and all report artifacts are under
`E:\cargo-targets\frameworks01-durable-io-profile`; no C-drive artifact was requested.

The current-source release results are:

| writes/transaction | p50 | p95 | MAD | p50 writes/s | allocations/write | requested bytes/write | p50 peak live |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 49.1682 ms | 74.8593 ms | 1.9993 ms | 20.34 | 699.00 | 111,767.00 | 8,783 B |
| 16 | 269.5669 ms | 335.7085 ms | 22.8331 ms | 59.35 | 382.25 | 63,240.62 | 77,781 B |
| 256 | 3,594.3007 ms | 3,996.5201 ms | 136.6521 ms | 71.22 | 362.10 | 60,503.50 | 1,227,408 B |

Each row contains 31 measured samples after 3 warmups with 128 payload bytes per write. The raw
CSV has 94 lines (header plus 93 samples), SHA-256
`b78344e1921432570b94da9ed726a6587d91dfcf6571284e85e0103f18a8d0c1`; the summary SHA-256 is
`bfb2a48dd6fc40b4f1e139436c06fae4e2c726af67bfc453577be085c09c3ee9`; metadata SHA-256 is
`51a5ced6fcd92a534914de0a0f2afa6524dc51648167c0063c09981c24d90e1b`.

The release binary fingerprints its compile-time current sources through `include_str!`. Recorded
BLAKE3 values are artifact identity `ac57ee14...aecfc7`, transaction pathing
`b9a49801...a2cee`, transaction engine `b5fa6578...47726`, and journal schema
`766dbc0f...0eba`. Within this one current-source build, batching from 1 to 256 writes amortizes
p50 from 49.1682 to 14.0402 ms/write and increases p50 throughput from 20.34 to 71.22 writes/s;
allocations remain approximately O(W) at 362.10 allocations/write for the 256-write case. This is
a scaling observation, not a before/after speedup. Without a profiler split between serialization,
journal/rename/directory durability and allocator work, it does not justify a structural
optimization yet.

I3 is now `source_complete / managed_behavior_green / release_profile_green /
independent_review_pending`. RSS, OS I/O counters and power remain explicitly unavailable, so no
bottleneck-elimination, engine-parity or power claim is made. Independent review, milestone commit
and WeCom notification remain pending.
