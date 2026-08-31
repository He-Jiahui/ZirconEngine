# Rich inline image semantic fallback owner review

Date: 2026-08-30

Status: `RRT-P1-029_inline_image_semantic_fallback_static_complete /
RRT-P1-040_qualified_inline_children_and_managed_validation_pending`

## Scope

This infrastructure slice gives compiled HTML/BBCode inline images a canonical alternative-text and
tooltip fallback, then makes rich accessibility text consume one retained semantic product. It does not
claim resource readiness/error state, source region, relative units, tint, virtual accessibility child
identity, image actions, icon/widget alternatives, or managed renderer acceptance.

## Current-source finding

`InlineObjectRef::Image` retained texture, size, and baseline only. HTML tokenization already admitted
bounded attributes, but `alt` and `title` were classified as unsupported and discarded. BBCode supported
only the positional `[img=locator]` form. `CompiledRichText::text()` therefore retained U+FFFC as the
only image representation, and `RichSemanticProjection` exposed that raw placeholder to accessibility.

Adding a string replacement in the accessibility module would create a second run walker on every
snapshot and would not participate in parser metadata/cache residency budgets. It would also diverge
from layout/paint generation ownership.

## Reference boundary

Local Unreal keeps `FRunInfo`, shared text/ranges, and `FSlateImageRun`/`FSlateWidgetRun` under the same
rich marshaller and `FSlateTextBlockLayout` owner used by desired size, children, arrange, and paint.
`SRichTextBlock` does not ask an accessibility adapter to parse markup or reconstruct image runs. Zircon
uses that retained-run principle, while keeping its own versioned HTML/BBCode syntax and typed budgets;
it does not copy Unreal's image data-table schema.

## Implemented contract

1. `InlineObjectRef::Image` now stores `alternative_text: Option<String>` and
   `tooltip: Option<String>`. `Some("")` is distinct from absence and explicitly marks a decorative image.
2. `HtmlSubsetV1` admits `alt` and `title`. `BbCodeV1` keeps `[img=locator]` and adds the attribute form
   `[img src="locator" alt="..." title="..."]` without an alias parser.
3. Alternative and tooltip bytes count toward the existing per-run/decorator metadata admission and the
   compiled cache residency estimate.
4. `RichParseBudget::max_semantic_text_bytes` is a separate request-local unit. It defaults to 32 MiB,
   follows `max_output_bytes` in `RichParseBudget::new`, has an explicit builder, and fails before cache
   publication with `SemanticTextByteBudgetExceeded`.
5. `CompiledRichText` builds one semantic `Arc<str>` after inline-run indexing. Sources without inline
   objects share the existing visible-text Arc and allocate no second string. Inline sources replace each
   admitted placeholder exactly once, including multiple placeholders merged into one identical run.
6. Explicit alternative text wins, including empty text; tooltip is used only when alt is absent. Missing
   image fallback and unqualified icon/widget placeholders are omitted rather than exposed as U+FFFC.
7. `RichSemanticProjection::visible_text()` is now an O(1) read of the compiled semantic product. The
   accessibility module still owns no parser, inline walker, cache, or markup heuristic.

Malformed, overlapping, empty, non-UTF-8-boundary, or non-placeholder inline ranges fail artifact
construction through a typed source-range error. No partial semantic string is published.

## Algorithm and performance boundary

Compilation performs one ordered pass over the existing inline-run index and each inline placeholder,
`O(V + I + S)` for visible bytes, inline placeholders, and semantic output bytes. It does not scan every
run when no inline index exists; that path checks the semantic budget and shares the visible Arc. The
inline path allocates exactly one retained semantic string, accounted in `estimated_bytes`. Accessibility
reads it in O(1) before its required DTO `String` clone.

No timing, allocation, RSS, power, or cross-engine improvement is claimed. The ownership change removes
per-snapshot reconstruction as a future design option; matched E-drive profiling remains required before
any further cache or representation optimization.

## Evidence and remaining gates

- A failing-first static contract required image fallback fields, HTML attributes, metadata accounting,
  a dedicated semantic budget, retained compiled semantic text, and projection reuse.
- Rust parser tests cover HTML metrics plus alt/title and the BBCode attribute form while retaining the
  positional form and locator security policy.
- Rust semantic tests cover alt replacement, decorative empty alt, merged adjacent images, source/format
  generation checks, and hidden Surface-owner projection.
- Rust admission tests cover semantic expansion rejection before cache publication; direct artifact tests
  reject an empty inline range. Cache residency tests count alternative-text storage.
- Rustfmt and scoped diff-check pass. The complete Runtime Text static suite passes 56/56 in 0.141 s.
- Production owners remain below the module budget: `compiled.rs` 799 lines,
  `compiled/semantic_text.rs` 159 lines, `admission.rs` 527, `html_subset.rs` 654, and
  `model/rich.rs` 323. The large shared test root was split to 939 lines with a 132-line inline child.

Managed Cargo/Rust behavior, AccessKit/screen-reader inspection, real WGPU/PNG, allocation/RSS/power,
milestone commit, and WeCom remain open. This non-visual slice creates no strategy screenshot.
