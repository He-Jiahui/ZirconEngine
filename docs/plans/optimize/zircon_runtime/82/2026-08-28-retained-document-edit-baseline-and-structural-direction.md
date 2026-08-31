# Runtime82 retained document edit baseline and structural direction

Date: 2026-08-28

Status: `baseline_and_post_wall_allocation_rss_matrices_complete / 17_scenarios_31_samples_each /
append_only_addition_source_implemented / separator_neutral_local_hard_line_edit_implemented /
52_direct_source_tests_green / structural_hotspots_eliminated /
wpr_cpu_stack_capture_blocked_by_windows_policy / power_capture_pending /
unreal_matched_runtime_pending`

## Decision

The current retained document has two structural edit costs. They are not micro-implementation noise:

1. every non-empty replacement creates a distinct immutable addition chunk, while
   `prepare_replace` calls `pieces_split_at` twice and then coalesces through another full vector;
   repeated tail inserts therefore rebuild an ever-growing piece list;
2. `prepare_hard_line_edit` flattens the complete reanalysis envelope into `old_source`, constructs a
   complete `new_source`, reparses both, and reconciles them even when a separator-free edit changes
   only the content length of one retained hard line.

The first defect makes sequential tail typing quadratic in piece metadata work. The second makes a
single-character edit linear in the byte length of a long source hard line. The measured data
selected, and the post-change matrix now validates, one append-only addition buffer plus a
separator-neutral local hard-line mutation path. It does not authorize an arbitrary compaction
threshold, rope, gap buffer, tree-backed piece table, or full document rewrite.

## Reproducible profile

The harness references the current `text/document/{storage,edit,hard_line_model,index,report}.rs` and
`text/hard_line.rs` production sources directly. It is compiled as an optimized standalone Windows
binary so the workspace's unrelated compile failures and target-directory cleanup do not enter the
sample. Only minimal neutral receipt DTOs and the upstream `unicode-segmentation` source are adapted;
the measured storage/edit/hard-line implementations are the repository sources.

Machine and toolchain:

- Windows 11 Pro 10.0.26200, AMD Ryzen 7 5800H, 8 cores / 16 logical processors, 41,791,548 KiB
  visible memory;
- `rustc 1.94.1 (e408947bf 2026-03-25)`, x86_64-pc-windows-msvc, LLVM 21.1.8;
- optimized build, debug assertions disabled;
- baseline repository HEAD `681588f7a1cbfaae3147e8b93e1be6705d810f21` with the current in-progress document sources;
- three explicit warm-ups except the costly 10k/one-million operation lanes, which use one; every
  scenario has 31 measured samples;
- the allocator wrapper counts successful allocation/reallocation calls and requested bytes during
  the edit, snapshot, and source-index phases independently; `GetProcessMemoryInfo` records process
  working set and pagefile observations after each sample.

Accepted evidence:

- `docs/tests/runtime/text/runtime_text_document_storage_baseline_20260828.jsonl`: 561 JSONL records,
  SHA-256 `0FFA75F91F2636D35E0BBBD769A1DAAE73ADACF13E717A07B3FC2DCD36366301`;
- `docs/tests/runtime/text/runtime_text_document_storage_baseline_summary_20260828.csv`: header plus
  17 summaries, SHA-256 `87594092B487333774E7AB8D99539A3C03348004DDBDE66DBA7A87716D4BA592`.

The corpus covers ASCII, CJK, combining graphemes, ZWJ emoji, tail/middle insert, tail replace,
middle delete, undo-shaped insert/delete pairs, explicit flattened snapshot, explicit grapheme index,
1/100/1k/10k edit counts, and a one-million-grapheme base document.

## Results

All times below are whole operation-stream wall times, not per-operation values.

| Lane | Edit p50 / p95 | Counted allocation | Final structure |
|---|---:|---:|---:|
| 1 tail insert, 1k ASCII base | 0.007 / 0.009 ms | 12 calls / 2.8 KB | 1 chunk, 2 pieces |
| 100 tail inserts, 1k ASCII base | 1.150 / 1.252 ms | 1,106 calls / 1.08 MB | 100 chunks, 101 pieces |
| 1k tail inserts, 1k ASCII base | 25.586 / 57.317 ms | 11,009 calls / 83.68 MB | 1,000 chunks, 1,001 pieces |
| 10k tail inserts, 1k ASCII base | 1,710.706 / 2,329.812 ms | 110,013 calls / 8.127 GB | 10,000 chunks, 10,001 pieces |
| 1k middle inserts, 10k ASCII base | 83.780 / 170.786 ms | 11,010 calls / 101.84 MB | 1,000 chunks, 1,002 pieces |
| 1k tail replacements, 10k ASCII base | 72.223 / 149.105 ms | 11,009 calls / 20.89 MB | 1,000 chunks, 2 pieces |
| 1k middle deletes, 10k ASCII base | 66.681 / 145.192 ms | 10,000 calls / 19.83 MB | 0 chunks, 2 pieces |
| 1k undo-shaped edits, 10k ASCII base | 56.752 / 74.728 ms | 10,509 calls / 21.10 MB | 500 chunks, 3 pieces |
| 1 tail insert, 1M ASCII base | 6.700 / 7.460 ms | 12 calls / 2.00 MB | 1 chunk, 2 pieces |
| 100 tail inserts, 1M ASCII base | 711.913 / 955.835 ms | 1,106 calls / 200.88 MB | 100 chunks, 101 pieces |
| 100 middle inserts, 1M ASCII base | 799.927 / 1,947.978 ms | 1,107 calls / 200.89 MB | 100 chunks, 102 pieces |

The 10k tail lane allocates 8.127 GB through the edit stream while its final content is 11 KB and
the content-free retained lower bound is about 593 KB. This is direct evidence that temporary piece
list construction, not retained content, dominates that lane. A linear scale-up from the 1k lane's
83.68 MB would be about 0.837 GB; the observed allocation is 9.71 times that value.

The one-million base lane is independently decisive: one tail character costs a p50 of 6.7 ms and
about 2 MB of allocation even though the piece list starts with one piece. One hundred edits allocate
about 201 MB. This matches the source-level two-full-string hard-line preparation path and cannot be
fixed by piece compaction alone.

Unicode changes constants but not the structural result. At 1k tail inserts, p50 is 31.095 ms for
CJK, 40.788 ms for combining graphemes, and 55.313 ms for ZWJ emoji; all three still publish exactly
1,000 addition chunks and 1,001 pieces.

Snapshot/index work is not the primary 2 KB-document hotspot. After 1k tail edits, flattening has a
10.2 microsecond p50 and 4,016 counted bytes; building 2,001 grapheme boundaries has an 84.7
microsecond p50 and 36,752 counted bytes when it also performs the initial flatten. These costs remain
important at document scale, but they do not explain 83.68 MB of edit-phase allocation.

## Implemented change and post-change evidence

`TextDocument` now owns one append-only addition `String`. Prepared edits reserve logical ranges in
that source and commit appends atomically after the revision check; old snapshot leases remain exact.
Adjacent sequential addition ranges therefore coalesce into one piece. `PreparedHardLineEdit`
separately represents a separator-neutral edit wholly inside one retained line's content as a local
content-length update. It preserves that line's stable ID and publishes only its model range. Edits
that touch or introduce CR/LF, including an insertion between CR and LF, continue through the
separator-aware reparse path.

The exact baseline executable, compiler mode, 17 scenarios, warm-up policy, and 31-sample count were
rerun against the changed production sources. Accepted post-change evidence is:

- `docs/tests/runtime/text/runtime_text_document_storage_post_20260828.jsonl`: 561 JSONL records,
  SHA-256 `DAE7E5C6111C0CD8BAEF261F4354B4D3665A8B616308EFE5203D9A304EBDA3BA`;
- `docs/tests/runtime/text/runtime_text_document_storage_post_summary_20260828.csv`: header plus 17
  summaries, SHA-256 `B34C8CF4A9B251F2F591C94158E9E3A377F26B7E366838DDAD43F20C7C53B122`;
- `docs/tests/runtime/text/runtime_text_document_storage_comparison_20260828.csv`: header plus 17
  baseline/post comparisons, SHA-256
  `5208B3050135A2065CE0300B45409B0F5A9348089125FC9421E1650F287D1D02`.

| Lane | Baseline p50 | Post p50 | p50 speedup | Allocation before / after | Allocation reduction | Final post structure |
|---|---:|---:|---:|---:|---:|---:|
| 1k tail inserts, 1k ASCII base | 25.586 ms | 0.504 ms | 50.79x | 83.68 MB / 0.363 MB | 230.58x | 1 source, 2 pieces |
| 10k tail inserts, 1k ASCII base | 1,710.706 ms | 4.508 ms | 379.46x | 8.127 GB / 3.643 MB | 2,231.08x | 1 source, 2 pieces |
| 1k middle inserts, 10k ASCII base | 83.780 ms | 0.513 ms | 163.28x | 101.84 MB / 0.483 MB | 210.89x | 1 source, 3 pieces |
| 1 tail insert, 1M ASCII base | 6.700 ms | 0.006 ms | 1,080.71x | 2.001 MB / 249 B | 8,035.31x | 1 source, 2 pieces |
| 100 tail inserts, 1M ASCII base | 711.913 ms | 0.061 ms | 11,767.16x | 200.88 MB / 0.036 MB | 5,544.85x | 1 source, 2 pieces |
| 100 middle inserts, 1M ASCII base | 799.927 ms | 0.034 ms | 23,807.34x | 200.89 MB / 0.048 MB | 4,165.52x | 1 source, 3 pieces |

The 10k tail lane no longer grows one source/piece per operation, and the one-million-character lanes
no longer copy or parse the full hard line for separator-neutral edits. Unicode lanes retain the same
bounded structure: 1k CJK, combining, and ZWJ-emoji tail streams have post p50 values of 0.473,
0.512, and 0.488 ms respectively, each with one addition source and two pieces. Snapshot p50 after
1k edits is about 0.002 ms; source-index p50 is 0.064-0.067 ms. The original two structural
bottlenecks are therefore absent from the managed wall/allocation matrix rather than merely shifted
into snapshot or source-index construction.

Focused regression evidence is 52/52 green direct-source tests: 49 production document,
hard-line, and store tests plus three structural harness guards. They cover prepare/commit revision
fences, snapshot leases, stable line IDs, separator edits, CRLF interior fallback, local dirty
receipts, addition-source/piece limits, and content-free reporting. Scoped formatting is also green.

## Profiler and power qualification

Windows Performance Recorder 10.0.26100, WPA, and xperf are installed. Starting the built-in `CPU`
profile failed before workload execution with `0xc5585011` (`Failed to enable the policy to profile
system performance`). WPR status afterward confirmed that no recording remained active. No ETL was
created. No installed `samply`, Cargo flamegraph, PerfView, VSPerfCmd, or Tracy profiler was found.

The current Windows session therefore cannot produce sampled CPU stacks or a qualified WPR power
capture without additional system performance authority. Wall/allocation/RSS evidence is accepted
for selecting the structural implementation, but sampled stacks, energy, package power, and matched
Unreal runtime numbers remain explicitly pending. No claim about power parity or superiority to
Unreal is made.

## Unreal reference and target structure

Unreal's `FSlateEditableTextLayout::InsertTextAtCursorImpl` splits admitted input into source lines and
routes ordinary text to the retained layout. `FTextLayout::InsertAt` mutates only the target
`FLineModel.Text`, marks that model dirty, removes its generated line views, and repairs run ranges.
`RemoveAt`, `SplitLineAt`, and `JoinLineWithNextLine` keep separator-changing structure distinct from
ordinary in-line text changes. This is evidence for Zircon's retained-line/local-dirty boundary; it is
not evidence that Zircon must copy Unreal's `FString` storage.

The selected Zircon change is:

1. replace one immutable `Arc<str>` per replacement with one document-owned append-only addition
   `String`; pieces reference stable byte ranges in that logical source, so adjacent sequential
   additions can coalesce without copying old addition bytes;
2. represent a separator-neutral edit within one hard-line content range as a direct prepared length
   update preserving the stable hard-line ID; retain the existing bounded reparse path for edits that
   insert, remove, split, or join CR/LF separators;
3. preserve expected document identity/revision, prepare-before-commit, exact dirty ranges, hard-line
   receipts, snapshot leases, and content-free admission/reporting;
4. do not introduce a guessed compaction interval. Fragmented random edits remain measurable after
   this change and can justify a tree-backed piece index only if the same post-change matrix proves it.

## Post-change acceptance

The exact 17-scenario matrix was rerun. Acceptance state is:

- complete: sequential tail insertion keeps one addition source and a bounded piece count;
- complete: 10k tail allocation no longer grows quadratically with operation count;
- complete: separator-neutral 1M tail and middle edits no longer allocate/copy the full hard-line
  source per operation;
- complete: separator insertion/removal, CRLF edits, stable hard-line IDs, revision receipts,
  snapshot leases, and admission failures retain focused regression coverage;
- complete: 52 direct-source tests are green and post-change JSONL/CSV evidence is captured beside
  this baseline;
- sampled stacks, WPR power, and matched Unreal measurements stay open until the environment can run
  them; they are not silently replaced by wall time.
