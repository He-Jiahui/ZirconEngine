# Runtime82 document store incremental residency accounting

Date: 2026-08-28

Status: `multi_document_baseline_and_post_complete / 12_lanes_31_samples_each /
1024_document_baseline_and_post_reproduction_complete / linear_aggregate_scan_eliminated /
incremental_residency_accounting_implemented / 53_direct_source_tests_green /
managed_runtime_power_wgpu_pending`

## Decision

`TextDocumentStore::prepare_admitted_replace`, `open`, `snapshot`, and the diagnostic `report`
currently obtain aggregate visible, retained-source, and flattened-snapshot bytes by scanning every
live document. The document lookup remains `O(log D)`, and the retained local edit is bounded by the
edited region, but admission therefore adds `O(D)` work to each accepted or rejected edit.

The measured cost scales with live document count and dominates the 1024-document store/direct edit
difference. Replace the repeated aggregate scan with store-owned incremental residency counters.
Keep document lookup, edit preparation, explicit limits, snapshot lease accounting, and
prepare-before-commit behavior unchanged. This evidence does not authorize a `HashMap`, another
document index, a guessed compaction threshold, or weaker admission checks.

## Source review and reference boundary

The current production path is
`zircon_runtime/src/text/document/store.rs::prepare_admitted_replace -> report ->
TextDocument::storage_report` for every document. `open` repeats the same scan for total admission;
`snapshot` scans before checking flattened-snapshot capacity. `close` currently drops the document
without a retained aggregate to repair.

Unreal Slate scopes retained text ownership to each `FSlateEditableTextLayout`, then applies explicit
local edits to its `FTextLayout` line model. `SlateEditableTextLayout.cpp:2138-2153` establishes the
scoped edit transaction and `TextLayout.cpp:2336-2399` repairs the target line/run ranges. That
reference supports keeping mutation and accounting beside the retained owner; it does not provide a
process-global aggregate or justify copying Unreal's string storage. Zircon's explicit session-wide
admission limits are an additional containment contract, so their totals must be maintained by the
same store transaction rather than reconstructed by unrelated consumers.

## Reproducible baseline

The optimized standalone Windows profile includes the current production
`text/document/{store,storage,edit,hard_line_model,index,report}.rs` sources directly. Build output is
under `E:/ZirconBuilds`; accepted evidence is under `docs/tests/runtime/text`. No Cargo target or
artifact was written to C or repository `target` directories.

Each lane uses a 4096-byte ASCII document, 100 alternating one-byte replacements at the beginning,
middle, or end, three warm-ups, and 31 measured samples. Live-document cardinalities are 1, 16, 256,
and 1024. Each sample separately measures:

- `TextDocumentStore::replace`, including aggregate admission accounting and `BTreeMap` lookup;
- 100 standalone `TextDocumentStore::report` calls;
- 100 direct `TextDocument::replace` calls against the same source and range;
- counted allocation/reallocation bytes and process working-set/pagefile observations.

Accepted matrix:

- `docs/tests/runtime/text/runtime_text_document_store_accounting_baseline_20260828.jsonl`:
  396 lines, SHA-256
  `34C85E516AD1F722309B81D87E5F4F04D3A6919693D37A27B31A5CC501DA5F63`.

The initial 1024-middle lane was affected by a machine-wide transient: direct editing rose from the
other lanes' roughly 58-67 microseconds per 100 operations to 159.6 microseconds, and report latency
rose at the same time. It is retained in the accepted matrix rather than silently removed. All three
1024 lanes were independently rerun with ten warm-ups and 31 samples:

- beginning: SHA-256
  `115E566DC2F752FC25BD93C520F89AAD034C00632792534AD90675C4C102E41B`;
- middle: SHA-256
  `D9AE1589D85E77D38E512FFA311DEBEAD46DF31AD6A006FAD2C8983110E0EFFC`;
- end: SHA-256
  `380D2093063344F78884E812B84C36F04D29F84567A246E656D40F27CA72671C`.

## Baseline results

Times are p50 for 100 operations.

| Documents | Store edit | Aggregate report | Direct edit | Report per operation |
|---:|---:|---:|---:|---:|
| 1 | 63.8-64.9 us | 1.7-1.8 us | 54.9-56.8 us | 0.017-0.018 us |
| 16 | 74.9-78.9 us | 8.9-9.6 us | 56.3-57.7 us | 0.089-0.096 us |
| 256 | 217.6-240.4 us | 137.3-141.5 us | 63.7-65.9 us | 1.373-1.415 us |
| 1024 reproduced | 475.8-707.2 us | 373.6-556.4 us | 40.3-60.4 us | 3.736-5.564 us |

The standalone report measurement grows by roughly 220-327 times from one to 1024 documents. At
1024 documents it accounts for most of the 4.35-6.47 microsecond per-edit gap between store and
direct mutation. The scan performs no counted allocation, so this is traversal/accounting work, not
an allocator artifact. Position changes document-local constants but does not change the `O(D)`
store term.

The absolute 1024-document latency is below a frame budget, but it is paid synchronously per edit,
scales independently of edited range, and consumes most of this boundary's measured CPU time. The
algorithm is therefore structurally wrong for the admitted multi-owner session even though the MVP
policy normally expects fewer active editable owners.

## Authorized implementation

1. Add one private residency value owned by `TextDocumentStore` for current document bytes, retained
   source bytes, and current flattened-snapshot bytes.
2. Update it only at successful `open`, changed edit commit, first snapshot materialization, and
   `close`. A rejected prepare, dropped prepared edit, no-op, stale revision, or denied snapshot must
   leave it unchanged.
3. Keep active snapshot lease count/bytes in the existing shared lease usage owner because leases can
   outlive a mutable store borrow and release on `Drop`.
4. Make `report` an `O(1)` projection of document count, residency, and lease usage. Keep its output
   schema and content-free diagnostics unchanged.
5. Add focused lifecycle regressions for snapshot replacement, close, rejection, dropped prepare,
   and no-op counter stability. Rerun the identical 12-lane matrix before claiming the scan removed.

## Acceptance gate

- `report` p50 must no longer grow linearly from 1 to 1024 documents within the resolution/noise of
  this harness;
- 1024-document store edit p50 must approach the direct edit lane plus `BTreeMap` lookup and fixed
  counter work, rather than include the pre-change 3.7-5.6 microseconds per edit scan;
- allocations must not increase materially;
- every existing admission/revision/snapshot test and the new counter lifecycle tests must pass;
- managed Runtime, power, and product WGPU evidence remain separate acceptance work and must not be
  inferred from this direct profile.

## Implemented change and post evidence

`TextDocumentStore` now owns one private `TextDocumentStoreResidency`. Successful open adds current
and retained-source bytes. Changed edit preparation computes the exact next residency value beside
the prepared document change, and only the infallible commit publishes both. First snapshot
materialization adds current snapshot bytes; the next changed edit removes that revision's snapshot
residency. Close subtracts the removed document's exact storage report before releasing the owner.
No-op, rejected prepare, invalid receipt projection, dropped prepared edit, stale revision, denied
snapshot, and unknown close leave counters unchanged.

Active snapshot lease count/bytes remain in their existing shared usage owner because a lease may
outlive a mutable store borrow and must release usage on `Drop`. Public report and error types,
limits, document lookup, source storage, and edit algorithms did not change. `report` is now an
`O(1)` projection plus the existing lease-usage lock.

The identical 12-lane matrix was rerun against the changed production source:

- `docs/tests/runtime/text/runtime_text_document_store_accounting_post_20260828.jsonl`:
  396 lines, SHA-256
  `7BA7171389FD043FF7D3F35C609C6264648361D82C02761403C6C58BB44AB22E`.

| Documents | Baseline report p50 | Post report p50 | Baseline store edit p50 | Post store edit p50 |
|---:|---:|---:|---:|---:|
| 1 | 1.7-1.8 us | 0.5-0.6 us | 63.8-64.9 us | 33.7-41.8 us |
| 16 | 8.9-9.6 us | 0.6 us | 74.9-78.9 us | 33.2-36.0 us |
| 256 | 137.3-141.5 us | 0.6-0.7 us | 217.6-240.4 us | 36.1-37.8 us |
| 1024 | 628.6-786.2 us | 0.7-0.8 us | 755.8-868.0 us | 36.5-39.8 us |

The post `report` range is flat within sub-microsecond timer noise from 1 through 1024 documents.
At 1024 documents the stable beginning/end lanes improve store p50 by 20.37-23.78 times. The initial
baseline middle lane's machine transient makes its 89.81x ratio unsuitable as the primary number,
but the post middle result converges with the other post lanes rather than preserving an `O(D)`
term. Counted store-edit allocation is unchanged for matched edit positions.

All three 1024-document lanes were independently rerun post-change with ten warm-ups and 31 samples.
The machine ran at a lower absolute frequency in that pass, but store/direct deltas remained fixed:
store p50 65.9-68.7 microseconds versus direct p50 61.5-63.2 microseconds per 100 edits; report p50
was 1.0 microsecond per 100 calls. Evidence hashes are:

- beginning: `BB18D0CC936422B7961B66B66F4E5357B593911F393A51315232E96AEFA1E99B`;
- middle: `479307D65835A1D2A44F3FBADC145C90DD9222E2793D04023ABBC7F4C620F0E3`;
- end: `8A34D8C5EABF6250E4ED8BF6D697237FEF14756B8EAACFEC65A8504405A6C6F9`.

The direct current-source harness passes 53/53 tests. The new lifecycle test covers two opens,
changed edit, first snapshot materialization, snapshot invalidation by another edit, and both closes;
the dropped-prepared-edit test now asserts the complete report remains unchanged. This closes the
measured store-accounting bottleneck only. Managed Runtime integration, power, product WGPU render,
and Unreal matched-runtime measurements remain pending.
