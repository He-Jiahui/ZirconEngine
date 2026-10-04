---
related_code:
  - zircon_runtime/src/text/document
  - zircon_runtime/src/ui/text/layout_engine
  - zircon_runtime/src/ui/text/measure_cache.rs
  - zircon_runtime/src/ui/text/resolved_layout.rs
  - zircon_runtime/src/ui/surface/input
related_plans:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/zircon_runtime/text/09/2026-08-09-text-pipeline-performance-architecture-and-profiling-plan.md
  - docs/plans/zircon_runtime/text/09/2026-08-30-grapheme-index-incremental-review-plan.md
  - docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md
write_scope: []
status: pending
---

# Runtime text document and layout closure

This is a current-source static closure of the canonical text-document storage,
index and UI layout owners. It remains pending: the crate does not build, no
focused Rust test reached execution and no current product executable exists for
UI frame, memory, power or pixel evidence. No Rust source was changed.

## Scope and source state

- `zircon_runtime/src/text/document/**`: 10 Rust files, 3,715 physical lines,
  3,374 nonempty lines, 128,748 bytes and 52 tests; no ignored tests or include
  sites. Sorted raw-content SHA256:
  `c8ff76d75ab0136e4cf7d5833040271384c611ed6af015fedd3e5c27dba849e1`.
- `zircon_runtime/src/ui/text/layout_engine/**`: 60 Rust files, 14,357 physical
  lines, 13,291 nonempty lines, 511,276 bytes, 213 tests, three ignored manual
  performance tests and one include site. Sorted raw-content SHA256:
  `42465f68949f595e23bbe08d68bd7b681012fcfec8022ed59fc56357644c7728`.
- All 70 files pass isolated `rustfmt --check --edition 2024 --config
  skip_children=true`. The document files are foreign staged additions and the
  layout owner is broad foreign modified/added/deleted work. Existing changes
  were preserved; no reconciliation edit was attempted.
- `cargo +1.94.1 check -p zircon_runtime --lib --message-format=short` used
  `E:/Git/ZirconEngine/target/codex/runtime-text-layout-check`. The prior
  document `Vec<TextDocumentPiece>` inference, overflow outcome and layout
  constraint iterator blockers are closed in current source. The crate instead
  fails with 214 broad foreign integration errors across graphics, runtime
  contracts, scene, input and adjacent owners. The focused document owner emits
  warnings in the captured output, not a new focused compiler blocker.

## Positive work to preserve

- `TextDocument` uses immutable `Arc<str>` original storage, one append-only
  addition buffer and piece rows. It is not publicly cloneable. Edits carry
  checked document key/revision/range and UTF-8 boundaries through a prepared
  transaction; dropping a prepared edit publishes nothing.
- `TextDocumentStore` has explicit document, current/total bytes, replacement,
  retained source, piece, snapshot-lease count and snapshot-byte limits.
  Snapshot lease release is terminal-on-drop and residency is maintained
  incrementally rather than recalculated from every document.
- The hard-line model reparses the affected line envelope and preserves stable
  line identities. The grapheme index has a conservative ASCII/no-CRLF
  incremental splice; Unicode, emoji and CRLF contexts fail closed to the full
  Unicode segmentation owner.
- `SharedTextLayoutSession` has bounded frame/persistent shape, layout and
  measure caches. Non-ready results do not enter caches and font generations
  are checked again before publication.
- Plain horizontal, no-wrap, clipped document layouts can retain a bounded
  visible hard-line window with overscan. Arabic tatweel probing and table
  numeric geometry are bounded, paragraph overrides use indexed spans, and
  profile labels are static.
- The 265 focused Rust tests cover edit transactions, budgets, snapshots, line
  and grapheme correctness, cache behavior, bidi/wrapping/overflow, tables and
  viewport layout. The three ignored tests are manual 31-sample wall-clock
  probes, not accepted product evidence.

## Retained findings

1. **Editable text has split content authority (P0).** Product input still
   materializes a complete editable `String` from template metadata, applies
   mutations to that value and clones it into property/event/history outputs.
   `UiTextDocumentSession` mirrors the same content into the piece store, and a
   source-epoch change creates another `Arc<str>` from the current text. The
   piece document is therefore not yet the one canonical editable generation.
   One `TextContentGeneration` must feed mutation, undo, IME, component state,
   accessibility, layout and events; compatibility strings are bounded export
   views, not a peer retained owner.
2. **Replacement admission follows candidate allocation (P0/P1).** Document
   replacement calls `pieces_split_at` twice. Each call scans the retained piece
   sequence and builds before/after vectors; the edit then coalesces through a
   third vector. Store admission runs only after `document.prepare_replace`, so
   piece/replacement cap+1 can pay broad scans and candidate allocations before
   rejection. Compile a checked edit proposal from source ranges, replacement
   bytes, projected pieces/index rows and retained peak before materialization.
3. **Line and grapheme materialization remain broad (P0/P1).** Hard-line lookup
   and source-offset lookup are linear in line count. A large affected hard line
   can materialize most of the old and new document envelope. Full grapheme
   rebuild flattens the document and allocates the boundary vector; even the
   accepted ASCII splice builds a successor boundary vector proportional to the
   retained suffix. Current store budgets do not reserve hard-line/index
   candidate bytes as one edit peak.
4. **Layout cache hits deep-copy resolved layouts (P0).** A
   `UiTextLayoutResolution` owns line strings, glyph advance vectors, run
   vectors/run strings, boxes and editable text. A frame-dedup hit calls
   `.cloned()`. A persistent hit clones for the return value and again for frame
   dedup; a miss retains one generation and makes return/frame copies. Render
   commands then own the resolution. Stable layout identity must return an
   `Arc<TextLayoutGeneration>` plus a compact placement/edit overlay; cache hits
   and repeated consumers must copy zero payload rows.
5. **Artifact publication duplicates retained content (P0/P1).** Rich layout
   registration creates an `Arc` from `layout.lines.clone()`, so the artifact
   and resolved layout retain separate line generations. Partial plain layout
   creates an `Arc<str>` from parsed text, and visible-line assembly clones each
   line string and run vector. The layout heap estimate excludes artifact-owned
   allocations. One generation must own source, lines, runs and glyph artifact
   rows with exact inclusive residency accounting.
6. **Rich-table layout performs two complete cell layouts (P1).** The preferred
   extent pass lays out every cell, drops the result, and the final pass lays
   out every cell again. It also clones the resolved style per cell. The current
   geometry budget guards finite axis values, not row/cell counts, text bytes,
   glyph work, candidate peak or deadline. Compile one table proposal, retain
   intrinsic cell generations and consume them during final placement.
7. **Viewport materialization is deliberately narrow (P1).** The retained
   visible-window route applies to plain horizontal no-wrap clipped documents
   with a document key and no preedit. Wrapped, rich, vertical and table content
   still materialize the complete layout. This is a correctness-preserving
   first slice, not proof that large-document UI is generally bounded.
8. **Stable identity still rebuilds owned keys (P1).** Style cache keys clone
   font-family and language strings. Requests without an exact document key
   hash text; the retained plain document cache compares complete text because
   it receives `&str` rather than a source lease/generation. Pass an immutable
   document/style/font/layout generation through the entire request chain.

## Architecture handoff

1. Compile one immutable `TextContentGeneration` per editable owner with exact
   document/source, line, grapheme, undo, IME and component generations. Input
   and accessibility actions submit checked edit proposals against it; commit
   atomically publishes the successor and terminal event receipt.
2. Pre-admit `TextEditProposal` count/bytes/pieces/line-envelope/index candidate
   and retained peak before any split, flatten or boundary allocation. Maintain
   indexed line starts and a chunked boundary owner so local edits do not
   require complete suffix-vector copying.
3. Compile an Arc-backed `TextLayoutGeneration` keyed by exact text, style,
   font, viewport and device/raster generations. Lines, runs, glyph artifacts,
   hit/caret geometry and render consumers borrow the same immutable rows.
4. Separate logical line models from visible line views. Extend viewport
   materialization to admitted wrapped/rich/vertical domains only after
   semantic parity tests; offscreen content retains estimates and dirty state,
   not complete line/run/glyph payloads.
5. Compile a `RichTableLayoutProposal` from cell count/text bytes, row/column
   recipes, intrinsic/final glyph work, workspace/current/candidate peak and
   deadline. Intrinsic cell results are reused by final placement.
6. Diagnostics `Disabled/Counters/Sampled/Full` borrows compiled IDs and labels.
   Disabled allocates no profile/artifact-management projection. Explicit owned
   exports are count/byte/deadline admitted and never become the cache ABI.

## Evidence and acceptance gates

Unreal Slate `TextLayout.cpp` keeps line models separate from line views,
returns from already-generated model views, and uses `UpperBoundBy` over
estimated vertical offsets to begin visible-range generation. Its
`TextLayoutTest_LazyGeneration.cpp` verifies that estimates can be built before
visible views and that views materialize as the visible window changes. This is
evidence for retained logical lines and lazy view publication, not Zircon's ABI
or budgets.

Eleven selected repository source-contract suites pass 115/115 tests. The text
subset covers incremental index admission and the broader text infrastructure;
the layout subset covers incremental snapshots, edge evidence, generation-owned
order, rebuild budgets, report segmentation, indexed slots, materialized layout
projection, Taffy parent work and surface materialization. These are static
contracts and pressure models, not Rust execution or product timing.

M0 adds exact allocation/copy/visit counters and RED tests for split editable
authority, replacement cap+1, line/index candidate peaks, cache-hit clones,
artifact residency and two-pass table cells. M1-M3 establish content/edit/layout
generations and pre-admission. M4-M6 extend viewport domains and collect managed
F4/product evidence only after Cargo is green.

Acceptance covers documents 0/1 B/1 KiB/1 MiB/16 MiB/cap+1; pieces, lines,
graphemes, cells and runs 0/1/64/1K/64K/cap+1; edits at beginning/middle/end and
ASCII/combining/emoji/CRLF; plain/rich/table, horizontal/vertical, wrap/no-wrap,
viewport/preedit; cache cold/hit/evict; diagnostics Disabled/Counters/Full; and
stale revision, font change, cancellation and shutdown. Report proposal/edit,
line/index/layout/table/artifact latency, source/piece/line/glyph visits,
String/Vec/Arc allocations/clones/bytes, current/candidate/retained peaks, cache
outcomes and terminal generations.

Hard gates: current source builds; one content generation feeds every editor and
layout consumer; cap+1 allocates no broad candidate; local admitted edits do not
flatten the document; stable layout hits clone zero payload; artifacts and
layout share one row owner; table cells are shaped once per accepted generation;
offscreen work is domain-bounded; diagnostics match actual work. No benchmark
artifact or micro-fix is warranted before these ownership corrections.
