# Runtime Text rich-source contract validation

## Scope

Close the remaining fail-open path where malformed `RichTextLayoutSource` run metadata could be
skipped by `RichAdvanceIndex::source_spans` and later published as partial or empty rich geometry.

## Structural correction

`text/layout/rich_source.rs` now owns `validate_rich_text_layout_source`. The validator checks that
each indexed run is present, has a non-sentinel strictly increasing parent source index, is non-empty,
non-overlapping, numerically representable, within the source text, and sliceable at UTF-8 boundaries.
Legal gaps remain supported because `RichAdvanceIndex`
continues to fill them with the base style. `source_spans` is now the typed `Result` owner, so index,
rich artifact, and UI prewarm consumers cannot bypass validation. Layout/source failures return
`TextLayoutError::LayoutFailed`; shaping-generation changes retain `Deferred` for the normal retry
path, while malformed prewarm input is discarded.

## 已完成项目

- Rich source validator is shared by index, artifact, and prewarm consumers.
- Rich advance-index append stages preserve `Deferred` generation outcomes instead of collapsing them
  into `Failed`, keeping generation retry semantics intact.
- Horizontal and VerticalRl forced-line/range conversion now shares the checked source-range owner;
  `usize::MAX`, `u32::MAX`, original-start recovery, and unchecked chunk offset arithmetic are no
  longer used as successful rich layout ranges.
- Forced hard-line ranges now use canonical `hard_line_count` preallocation plus
  `visit_hard_lines` filling, avoiding an intermediate `HardLine` vector while preserving canonical
  separators. The two bounded scans are intentional: the first obtains capacity, the second writes
  the final compact ranges.
- Rich horizontal/VerticalRl range and UI item projections fail closed on malformed source data.
- Rich table source ranges, local projection, top-level overlap, and hard-line delimiter handling are
  checked without silent clamping.
- Measurement shaped-geometry projection validates the exact shaped source snapshot and glyph/line
  UTF-8 ranges before producing grapheme advances; malformed cached/compatibility runs propagate
  typed failure instead of being clamped to width one.
- VerticalRl rich column projection now validates every numeric conversion and generated range
  before advance measurement; sentinel recovery and failed leading-space slices return
  `LayoutFailed`.
- `RichParseBudget` now owns source/visible-output bytes, recognized token count, per-token bytes,
  per-token attribute count/bytes, and the shared HTML/BBCode active-tag depth. Source rejection occurs
  before global cache lookup/copy, emoji expansion checks the remaining output budget while
  materializing, and public parser entrypoints return `RichTextParseError` rather than publishing a
  saturated artifact. Defaults are 65,536 recognized tokens, 64 KiB per token, 64 attributes/16 KiB
  attribute bytes per token, and 128 active tags. Token/attribute limits fail before name/value
  allocation; depth `max + 1` fails before stack growth.
- `CompiledRichText` construction is fallible and routes visible byte length, run/paragraph/table
  counts, grapheme ranges, and table-cell projection indices through one checked `usize -> u32`
  owner. Parser/layout production files no longer use `u32::MAX` as a successful rich identity.
- UI layout maps parser capacity rejection to stable low-cardinality
  `TextLayoutError::RichTextBudgetExceeded`; failed cache single-flight results are shared with
  current waiters and then removed from residency instead of pinning invalid source.
- BBCode paragraph list-prefix layout now reuses `checked_source_range`, rejects reversed,
  out-of-bounds, and non-UTF-8 prefix metadata, and propagates an unrepresentable `usize -> u32`
  prefix index as `RichTextBudgetExceeded` instead of saturating or silently applying zero inset.
- Static regression, Rust formatting, Python compilation, and scoped diff checks are green.

## Evidence

- Rust source regression covers empty and partially covered valid input.
- Rust source regression rejects overlapping, empty, out-of-bounds, duplicate, descending, and sentinel source indices/ranges.
- Rich horizontal glyph/word and VerticalRl word range extraction now propagates an invalid
  hard-line UTF-8 slice as `LayoutFailed` instead of dropping that line; the static contract covers
  all three owners.
- Rich UI horizontal and VerticalRl item projection also reject malformed run identity/ranges as
  `LayoutFailed`, instead of publishing a successful zero-advance item or silently declining the
  complete rich route.
- Rich table layout now checks absolute table/cell ranges against the projected source and parent
  table with checked arithmetic and UTF-8 slicing. Legal empty cells remain accepted; reversed,
  clipped, cross-projection, or overlapping/descending cell ranges fail closed instead of being
  clamped into empty cells. Legal gaps between cells remain accepted.
- `UiParsedText::project_range` is now the single checked projection owner: reversed,
  out-of-bounds, non-UTF-8, and checked absolute-offset failures return `Result` before slicing;
  `rich_table/source_slice.rs` only propagates the typed error. Delimiter trimming reuses the
  canonical hard-line separator owner instead of a private LF-only check.
- Static contract `test_runtime_text_rich_source_contract.py` passes 13/13, including malformed
  measurement geometry and VerticalRl column admission.
- The combined text static suite passes 33/33, including incremental document and pointer-index
  contracts.
- Targeted Rustfmt and scoped diff checks pass.
- A second managed `cargo check --manifest-path zircon_runtime/Cargo.toml --lib --offline --locked`
  using the independent E: target also timed out after 184s without diagnostics while the workspace
  continued compiling other crates; source-only validation remains green and no process was stopped.
- A managed `cargo check -p zircon_runtime --lib --offline --locked` attempt timed out after 184s
  without diagnostics while unrelated workspace cargo/rustc processes held the shared target; no
  process was interrupted and this is not treated as a source failure.
- A third bounded `cargo check --manifest-path zircon_runtime/Cargo.toml --lib --offline --locked`
  attempt used `target/codex_text_check_final` on E: and timed out after 304s without diagnostics.
  Its `zircon_runtime` rustc process was still active afterward and was left running; this remains
  incomplete validation, not a pass.
- The later incremental output file records existing workspace errors outside the touched
  rich-source/layout files (Rust 2024-only syntax, missing `zr_contracts`, graphics export/privacy
  mismatches, and unrelated text font/glyph modules). No diagnostic points at this slice's rich
  source, advance-index, forced-range, or VerticalRl files; the workspace still has no clean Cargo
  gate and this evidence is not promoted to a product-validation pass.
- After the text-infrastructure fixes, the next incremental Cargo output no longer contains
  diagnostics for `text/glyph_artifact`, `text/font`, `text/sdf/font_bake`, or
  `text/shaping/cosmic`. Remaining errors are confined to untouched graphics/core/plugin/
  dynamic-api modules, so the text-module compile diagnostics addressed by this slice are closed;
  the workspace Cargo gate remains red for unrelated reasons.
- The added static infrastructure compile-contract suite passes 15/15: crate-local glyph projection
  ownership, optional-hash branch typing, typed family dedupe storage, SDF face-cache call shape,
  scoped default-face visibility, Cosmic snapshot cloning/revision access, paragraph-prefix checked
  slicing/index conversion, parser/cache admission ordering, total/per-token/attribute tokenizer
  budgets, bounded active-tag depth, no saturated compiled identity, and typed UI failure routing.
- Two current-source Cargo checks used `target/codex_text_admission_check` on E: and each reached the
  184s hard timeout. The latest compiler fingerprint contains existing graphics/core dependency and
  export errors; filtering primary error spans reports 0 errors under `text/**`, `ui/text/**`, and
  `core/framework/text/**`. This is diagnostic evidence only, not a clean Cargo pass. Timed-out Cargo
  child processes were identified by exact command line and stopped after the bounded check ended.
- The next bounded incremental Cargo check failed after 54s before entering `zircon_runtime`, at an
  unrelated `zircon_runtime_interface/runtime_api/session` import of missing
  `ZrRuntimeTranslatedEventV1`. It is a red workspace gate, not owned Text validation or a pass.
- Standalone `rustc` compilation of `text/rich/admission.rs` succeeds; targeted rustfmt, file-budget
  review (`parser.rs` 763 lines plus `parser/builder.rs` 86 lines), static tests, and scoped diff
  checks pass.
- The existing 5,000-depth active-tag release benchmark now requires an explicit request budget.
  Default 10,000-depth hostile inputs return `ActiveTagDepthBudgetExceeded`; delta-style clone/allocation
  optimization remains profile-gated and has not been claimed or implemented.
- Managed Cargo, real WGPU/PNG under `docs/tests/runtime/text`, profile/RSS/power, and Unreal-matched
  product validation remain pending; no screenshot is produced by this source-only change.

Status: `rich_source_contract_fail_closed_static_implemented /
rich_forced_range_owner_shared / rich_advance_index_result_owner_preserves_deferred /
base_style_gap_fill_preserved / text_module_compile_diagnostics_closed /
rich_parser_typed_byte_admission_implemented / rich_compiled_index_saturation_removed /
rich_active_tag_depth_admission_implemented /
rich_tokenizer_count_and_materialization_budget_implemented /
managed_validation_pending`.
