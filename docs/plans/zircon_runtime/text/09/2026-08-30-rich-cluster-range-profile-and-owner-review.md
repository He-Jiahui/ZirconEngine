# Rich compiled cluster-range profile and owner review

Date: 2026-08-30

Status: `baseline_profile_complete / structural_cutover_implemented_static /
managed_product_validation_pending`

## Scope

This review covers `CompiledRichText::cluster_ranges` only. It does not claim end-to-end parser,
shaping, layout, renderer, frame-time, RSS, or package-power acceptance.

The current rich compiler materializes one `(u32, u32)` entry for every grapheme in the complete
visible text. Production source search found no consumer outside `CompiledRichText` construction,
identity/byte accounting, and its public accessor; the only external call is a test assertion.
Shaping and layout already own the grapheme/cluster data they actually consume.

Local Unreal comparison:

- `IRichTextMarkupParser::Process` publishes line/run parse ranges plus stripped output.
- `FTextRunParseResults` carries original/content ranges and metadata.
- `FDefaultRichTextMarkupParser::ParseLineRanges` builds line/run ranges; it does not publish a
  document-sized grapheme vector into the rich parser artifact.
- Character/grapheme breaking remains a text-layout/shaping responsibility.

The Zircon field therefore crosses the same parser/marshaller boundary that the target Unreal-style
pipeline keeps separate.

## Baseline method

An isolated release benchmark was compiled and run entirely under
`E:/Git/ZirconEngine/target/codex_text_profile`. It executes the current production expression:

```rust
text.grapheme_indices(true)
    .map(|(start, grapheme)| (start as u32, (start + grapheme.len()) as u32))
    .collect::<Vec<_>>()
```

Input is ASCII `a`, the maximum-grapheme case per encoded byte. Each lane uses 31 samples. Payload
bytes are the exact vector capacity multiplied by `size_of::<(u32, u32)>()`; working-set values come
from Windows `GetProcessMemoryInfo`. This isolates the suspected owner and is not an end-to-end
product benchmark.

## Baseline results

| source bytes | ranges | payload bytes | first working-set delta | p50 | p95 | p99 |
|---:|---:|---:|---:|---:|---:|---:|
| 1,048,576 | 1,048,576 | 8,388,608 | 9,478,144 | 65,236 us | 108,201 us | 127,439 us |
| 8,388,608 | 8,388,608 | 67,108,864 | 68,182,016 | 736,093 us | 967,395 us | 1,031,905 us |
| 33,554,432 | 33,554,432 | 268,435,456 | 269,508,608 | 3,074,179 us | 5,128,054 us | 6,087,184 us |

Raw 1 MiB microseconds:

```text
108201,81725,77566,72003,76587,67334,78940,77509,81712,67545,127439,94304,77034,63596,65561,65236,64673,59925,66019,64106,61920,61899,62528,65042,64425,52476,58275,56832,50003,54363,51883
```

Raw 8 MiB microseconds:

```text
566371,600073,733376,562271,648183,553640,531510,613045,748964,587473,736093,591150,560486,548845,643379,763462,687020,839295,902300,817301,774670,967395,780123,899223,880019,965122,1031905,908503,687871,791501,933429
```

Raw 32 MiB microseconds:

```text
2536445,3592787,3056154,2959203,2606347,3038393,3094433,3092790,3365761,3372751,4710776,4420164,4728757,6087184,5128054,4013176,3054791,3137437,2961163,3059040,2630162,2748360,2836970,3089137,2640021,2886123,2613408,3074179,3137462,2980808,3186791
```

## Structural decision

The index has linear time and memory cost but no production consumer. A higher count budget would
only hide the duplicate owner behind a smaller maximum and would preserve the wrong architecture.
The planned hard cutover is:

1. Remove `cluster_ranges` storage, construction, equality, byte accounting, and accessor from
   `CompiledRichText`.
2. Remove the test that treats a parser-owned grapheme vector as a public contract; retain tests for
   canonical text and run ranges.
3. Do not add a compatibility accessor, lazy cache, or second cluster index.
4. Keep cluster/grapheme work in shaping/layout owners that have an actual consumer and request
   context.
5. Add a static contract preventing document-sized `grapheme_indices` materialization from returning
   to `rich/compiled.rs`.

For this isolated owner the post-cutover payload is exactly zero bytes and its build pass is removed,
changing this stage from `O(G)` time/`O(G)` memory to no stage. This does not prove end-to-end speed or
power improvement. Managed Cargo, full rich parse/layout profiles, RSS/power, matched Unreal load,
WGPU, and product PNG remain required before broader claims or milestone acceptance.

## Post-cutover evidence

`CompiledRichText` no longer imports `UnicodeSegmentation`, stores or compares `cluster_ranges`, walks
the visible text with `grapheme_indices(true)`, accounts a duplicate range vector, or exposes a public
cluster-range accessor. The only external caller was a UI test; it now asserts the canonical compiled
run byte ranges `(0, 4)` and `(4, 7)` for the link plus inline-object fixture.

The isolated owner therefore has exact post-cutover payload `0` bytes and no build pass. This is a
structural removal, so reporting a synthetic near-zero timing loop would be misleading; the removed
stage has no replacement to benchmark. The static regression explicitly rejects the field, accessor,
grapheme materialization, and segmentation import. The combined Runtime Text static suite passes
34/34, targeted rustfmt and scoped diff checks pass, and source search finds no remaining
`cluster_ranges` reference. Managed Cargo remains blocked before Runtime Text by the unrelated runtime
interface session export, so end-to-end latency, allocation/RSS, package power, WGPU, product PNG, and
matched Unreal-load evidence remain pending.
