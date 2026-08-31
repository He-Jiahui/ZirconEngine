# Rich owned parse clone profile and hard cutover

Date: 2026-08-30

Status: `consumer_review_complete / baseline_profile_complete /
immutable_artifact_hard_cut_implemented_static / managed_product_validation_pending`

## Scope

This review covers `RichTextParser::parse()` returning an owned `RichParseResult` by compiling through
the canonical cache and then cloning `compiled.parsed()`. It does not change parser grammar, cache
single-flight, layout, renderer, WGPU/PNG, package power, or public serialization of the neutral model.

## Current-source consumer review

All production Runtime/UI consumers already enter through `RichTextParser::compile`,
`compile_rich_text`, or lookup of an existing `Arc<CompiledRichText>`. Workspace search found no
production caller of `RichTextParser::parse`. The crate-internal `parse_rich_text` bridge is consumed
only by `#[cfg(test)]` modules. One direct `parser.parse` behavior assertion is also test-only.

The current public method nevertheless does:

```rust
self.compile(markup, format)
    .map(|compiled| compiled.parsed().clone())
```

`RichParseResult.text` is an `Arc<str>`, but the runs, paragraph array, tables, table columns/cells,
and every dynamic style family/feature/link/icon-font payload are cloned. The owned result is detached
from parser/decorator generation and the compiled projection/resource indices, so it is both an
allocation-heavy duplicate and a weaker artifact identity.

## Unreal reference boundary

Local Unreal `FRichTextLayoutMarshaller::SetText` invokes `Parser->Process` once to produce the
processed string and `FTextLineParseResults`, then immediately constructs the layout's model string and
runs from those results. The marshaller/layout owns the output used by rendering; it does not insert a
canonical compiled object into a cache and then expose a second API that deep-copies only part of it.
`URichTextBlock` creates the parser/writer/decorators and supplies the marshaller, preserving one
widget-owned compilation/layout path.

Zircon's equivalent stable boundary is the existing `Arc<CompiledRichText>`: it owns source, format,
parser/decorator/emoji generation, parsed model, dependencies, and cell projections. A borrowed
`compiled.parsed()` view is sufficient for neutral consumers. Returning a detached owned parse result
would move away from, not toward, that ownership model.

## Baseline method

An isolated optimized Rust benchmark was compiled and run entirely under
`E:/Git/ZirconEngine/target/codex_text_profile`; compiler temporary files were redirected to the same
E-drive directory. The fixture mirrors `RichParseResult` clone ownership: shared `Arc<str>`, deep-cloned
runs with dynamic family/four features/link, paragraphs, and tables with columns/four cells. It profiles
4,096, 32,768, and the default maximum 131,072 runs; paragraph/table counts are proportional. Each lane
uses 31 samples. A counting global allocator reports exact requested allocations/bytes for the clone
expression; fixture construction and destruction are outside the timed/allocation deltas.

## Baseline results

| runs | paragraphs | tables | allocations/clone | bytes/clone | first working-set delta | p50 | p95 | p99 |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 4,096 | 256 | 32 | 12,355 | 1,014,784 | 1,335,296 | 2,454 us | 5,677 us | 10,592 us |
| 32,768 | 2,048 | 256 | 98,819 | 8,118,272 | 10,113,024 | 22,059 us | 68,484 us | 101,118 us |
| 131,072 | 8,192 | 1,024 | 395,267 | 32,473,088 | 40,169,472 | 111,366 us | 232,754 us | 331,802 us |

Raw 4,096-run microseconds:

```text
4091,2273,2440,2376,2481,10592,5677,2431,2263,2969,3135,4008,3289,2548,2348,2454,2261,2186,2594,2303,2280,2436,2426,2398,2502,4488,2608,2323,2651,4838,2191
```

Raw 32,768-run microseconds:

```text
24206,23225,19941,21006,23033,21137,21824,19525,21413,24482,22059,23816,19489,19566,19099,19053,18058,19479,19449,21805,21324,47033,30177,38731,61111,24772,40062,49880,101118,68484,35247
```

Raw 131,072-run microseconds:

```text
163687,225639,331802,177811,166484,125311,87054,72717,78890,93455,75146,88025,86324,89687,109481,184510,82256,111366,109955,109686,163387,232754,136577,86946,103928,82552,132976,145380,114847,148615,133639
```

Allocation counts/bytes are exact and identical across all 31 samples within each lane. From 4,096 to
131,072 runs, object count grows 32x, requested clone bytes grow 32x, allocation count grows 31.99x,
and p50 grows 45.38x. The cost is a structural `O(artifact dynamic owners)` copy after canonical compile,
not parser tokenization or cache lookup.

## Hard-cut decision

1. Remove the production `RichTextParser::parse -> RichParseResult` API. `compile` remains the single
   public compilation entry and returns `Arc<CompiledRichText>`.
2. Consumers read `compiled.parsed()` or more specific compiled views while retaining the parent Arc;
   no public owned-clone compatibility alias is added.
3. Remove the production crate-internal `parse_rich_text` bridge. A `#[cfg(test)]` helper may keep
   legacy parser-focused assertions readable, but production builds must contain no compiled-artifact
   clone expression.
4. Keep `RichParseResult` as the neutral internal model and serializable DTO where directly constructed
   by layout/cache tests; this cut changes ownership, not the parser's intermediate representation.
5. Add a static gate that rejects a production `parsed().clone()` API and requires the test helper to
   be explicitly cfg-gated.

After hard cutover the removed production stage has zero allocations, zero bytes, and no latency
because it no longer exists. That exact statement is narrower than end-to-end compile/layout/frame
performance. Managed Cargo, product migration outside the tracked workspace, allocation/RSS/power,
WGPU framebuffer, and new PNG evidence remain required before milestone acceptance.

## Implementation evidence

The production `RichTextParser::parse` method has been removed. `compile` is now the only public parser
materialization entry and returns the canonical `Arc<CompiledRichText>`. The crate-internal owned parse
bridge and method are both explicitly `#[cfg(test)]`; they keep parser corpus assertions concise but do
not enter production builds. The only former direct behavior assertion now calls `compile` and checks
the same typed capacity error.

No production compatibility alias, owned snapshot wrapper, alternate cache, or partial artifact type
was added. `RichParseResult` remains the neutral model inside the canonical compiled owner and for
direct fixture construction. Production consumers continue to borrow `compiled.parsed()` while
retaining the parent Arc and its parser/decorator/emoji generation.

The removed production stage now has exact post-cutover allocation count 0, requested bytes 0, and no
latency stage. The current reproducible Runtime Text static suite passes 34/34 and targeted Rust 2024 formatting
passes. `parser_registry.rs` is 155 lines. Managed Cargo and downstream external-source compatibility
have not run; end-to-end compile/layout/frame, RSS/power, WGPU framebuffer, and product PNG remain open.
