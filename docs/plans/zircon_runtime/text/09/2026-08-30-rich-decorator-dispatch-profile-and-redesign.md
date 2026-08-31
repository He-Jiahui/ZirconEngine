# Rich decorator dispatch profile and structural redesign

Date: 2026-08-30

Status: `baseline_profile_complete / exact_tag_hash_dispatch_implemented_static /
isolated_post_profile_complete / managed_product_validation_pending`

## Scope

This review covers parser-local BBCode decorator lookup and registration. The dispatch optimization
does not itself close decorator panic/deadline/cancellation isolation, provider owner/revoke leases,
source diagnostics, parser cache qualification, layout, rendering, WGPU/PNG, RSS, or package power.

## Current-source and Unreal review

Zircon's `RichTextDecorator` contract assigns one normalized exact tag to each decorator. Registration
rejects parser-reserved or duplicate tags, and a successful registration advances
`decorator_generation`, which is part of the compiled-rich cache key. Runtime dispatch nevertheless
stores every decorator in a `Vec` and performs `.iter().find(tag)` for each candidate open token. With
`T` candidate tokens and `D` decorators, late hits and misses perform `T * D` string comparisons.

Local Unreal source separates parser output from decorator-created runs. `URichTextBlock` creates
widget-owned decorator instances and gives them to `FRichTextLayoutMarshaller`; the marshaller's
`TryGetDecorator` scans `InlineDecorators` and then `Decorators`, calling the arbitrary predicate
`ITextDecorator::Supports(TextRun, Line)`. That ordered scan is required by Unreal's predicate-based
contract. Zircon does not expose `Supports`: because exact normalized tag ownership is unique at
registration, preserving a predicate scan would copy Unreal's implementation without its semantic
reason. The Unreal-aligned boundary to preserve is widget/parser-owned decorator lifetime and explicit
marshaller dispatch, not linear lookup for an exact key.

## Baseline method

An isolated release Rust benchmark was compiled and run entirely in
`E:/Git/ZirconEngine/target/codex_text_profile`, with compiler temporary files redirected to the same
E-drive directory. Registry construction is outside the timing boundary. Each sample performs 4,096
lookups of the final exact tag, matching the current worst-case successful `.iter().find` path. Lanes
contain 16, 256, or 4,096 decorators and use 31 samples each. The checksum requires every lookup to
resolve the expected final identity.

## Baseline results

| decorators | dispatches | exact comparisons/sample | first working-set delta | p50 | p95 | p99 |
|---:|---:|---:|---:|---:|---:|---:|
| 16 | 4,096 | 65,536 | 12,288 bytes | 517 us | 567 us | 705 us |
| 256 | 4,096 | 1,048,576 | 12,288 bytes | 7,381 us | 10,436 us | 11,026 us |
| 4,096 | 4,096 | 16,777,216 | 12,288 bytes | 116,314 us | 157,050 us | 207,933 us |

Raw 16-decorator microseconds:

```text
501,468,535,567,549,536,563,556,525,547,554,509,517,452,469,453,462,457,516,504,705,526,506,452,490,539,521,532,495,499,539
```

Raw 256-decorator microseconds:

```text
7381,7247,7420,10436,11026,8494,7348,7371,7565,7477,7411,7714,9010,6976,7902,7366,7883,8772,7189,6774,8959,7737,7076,6395,7868,6965,6883,6871,7241,6878,7129
```

Raw 4,096-decorator microseconds:

```text
111077,114203,104960,122412,207933,122992,108538,90211,71414,108896,99889,116314,125292,108009,116463,124546,115776,119386,111070,139872,157050,121348,105000,112595,110278,107265,117259,117170,118118,121890,123485
```

Decorator count increases 256x from 16 to 4,096 while p50 increases 224.98x. No output allocation
occurs in the timed loop. This isolates exact-tag registry scanning as the dominant dispatch cost at
large provider counts rather than parser text materialization or decorator callback work.

## Structural decision

1. Make a request/parser-owned `HashMap<String, Box<dyn RichTextDecorator>>` the sole exact-tag
   dispatch owner. Default `RandomState` preserves collision resistance for untrusted lookup strings.
2. Normalize once at registration, reject parser-reserved or duplicate tags through keyed membership,
   then publish the decorator and advance the existing parser-local generation exactly once.
3. Keep callback execution outside map mutation and preserve `&self` lookup. No global registry,
   duplicate vector, facade, or decorator iteration contract is introduced.
4. Preserve the compiled cache identity: parser identity plus decorator/emoji generations remain the
   complete key inputs for this slice. Existing artifacts remain immutable and are never rewritten.
5. Retain all current exact-tag behavior and registration errors. Add a static regression prohibiting
   linear decorator dispatch from returning.
6. Repeat the same 31-sample lanes after implementation. Expected average lookup work is `O(T)` rather
   than `O(T * D)`; registration becomes average `O(1)` membership/insertion. Report raw samples and
   registry memory tradeoffs without claiming product power or end-to-end parser acceptance.

Decorator panic/deadline/cancellation quota, project/session/plugin owner leases, unregister/revoke,
registration count admission, and complete diagnostics remain separate architecture work. They must
not be implied complete by a faster exact dispatch table.

## Implementation evidence

`DecoratorRegistry` now has one `HashMap<String, Box<dyn RichTextDecorator>>` owner. Registration uses
one `Entry` operation for duplicate admission plus insertion, parser-reserved tags retain the same
typed registration error, and builtin duplicate tags are an explicit internal invariant failure.
Dispatch uses borrowed `&str` lookup and invokes the callback after immutable resolution. No vector,
mirror index, global registry, or behavior-dependent iteration remains. Successful registration still
advances the parser-local decorator generation exactly once, so compiled cache identity is unchanged.

The existing custom/builtin/duplicate/reserved decorator behavior tests remain in place. A static
contract requires the unique exact-tag map and borrowed lookup and rejects `.iter().find` regression.
The combined Runtime Text static suite passes 38/38, targeted Rust 2024 formatting passes, the registry
is 143 lines, and source inspection reports one map owner and zero linear dispatch loops.

## Isolated post-change results

The same compiled benchmark, registry fixtures, 4,096 dispatches, and 31-sample lanes were run through
the indexed path after implementation. Registry construction remains outside the timing boundary.

| decorators | dispatches | first lookup-loop working-set delta | p50 | p95 | p99 | old/new p50 |
|---:|---:|---:|---:|---:|---:|---:|
| 16 | 4,096 | 16,384 bytes | 140 us | 158 us | 158 us | 3.69x |
| 256 | 4,096 | 12,288 bytes | 142 us | 230 us | 241 us | 51.98x |
| 4,096 | 4,096 | 12,288 bytes | 139 us | 151 us | 152 us | 836.79x |

Raw 16-decorator microseconds:

```text
149,132,127,130,134,138,132,138,143,144,140,134,121,133,138,158,135,140,140,141,136,141,142,140,141,139,143,139,143,158,140
```

Raw 256-decorator microseconds:

```text
145,162,143,139,141,129,132,139,141,143,136,149,134,142,135,142,123,128,124,123,146,143,135,147,241,143,142,143,137,230,158
```

Raw 4,096-decorator microseconds:

```text
152,139,140,136,148,133,143,127,130,134,134,140,141,147,144,131,135,140,142,141,138,134,151,138,139,146,136,137,141,145,133
```

Indexed p50 remains 140/142/139 us as decorator count grows 256x; the old path grew from 517 to
116,314 us. At 4,096 decorators, p50/p95 improve 836.79x/1,040.07x. This confirms removal of
decorator-count-dependent exact dispatch work in the isolated loop. The working-set delta above is
only the allocation-free lookup loop: both fixture registries were built before its baseline sample,
so it is not evidence about retained HashMap memory. Registry allocation/bytes, parser tokenization and
callback time, product RSS/power, managed Cargo, WGPU/PNG, and matched Unreal load remain pending.

A subsequent non-performance infrastructure slice added typed panic isolation, a 64 KiB default
per-call decorator metadata limit, and a 32 MiB default retained run-metadata limit. That work is
tracked separately in
[`../07/2026-08-30-rich-decorator-provider-admission.md`](../07/2026-08-30-rich-decorator-provider-admission.md)
and does not change the dispatch timings above.

## Callback-boundary follow-up

Because production dispatch now catches provider unwind, a second isolated benchmark measured the
complete old/new dispatch boundary rather than key lookup alone. Both paths use a boxed dynamic no-op
decorator that updates the checksum. The old lane performs vector lookup plus direct callback; the new
lane performs HashMap lookup, `catch_unwind`, and the same dynamic callback. Registry construction is
again outside the timing boundary; all lanes use 4,096 calls and 31 samples.

| decorators | old direct p50/p95/p99 | indexed catch p50/p95/p99 | old/new p50 |
|---:|---:|---:|---:|
| 16 | 541/595/600 us | 146/204/234 us | 3.71x |
| 256 | 7,869/11,296/11,979 us | 149/286/356 us | 52.81x |
| 4,096 | 112,965/152,013/158,922 us | 154/183/191 us | 733.54x |

Raw old/new 16-decorator microseconds:

```text
old=445,500,449,600,557,551,543,513,546,595,536,535,554,539,550,541,554,524,531,538,556,545,542,535,556,515,583,531,486,539,541
new=154,234,139,130,146,146,141,147,147,139,145,164,141,140,134,136,134,131,129,130,133,134,204,148,152,168,146,152,146,151,151
```

Raw old/new 256-decorator microseconds:

```text
old=7796,7955,7904,7309,7062,6986,6979,7438,7377,9408,8140,8296,8460,11979,7869,6980,8781,10918,8003,7887,8270,8929,11296,7394,7200,7269,6686,7014,7419,6953,8468
new=134,132,135,155,145,139,141,144,138,142,169,149,148,137,151,146,152,150,150,151,151,204,149,146,153,143,151,190,216,356,286
```

Raw old/new 4,096-decorator microseconds:

```text
old=98504,117118,128594,116946,129497,107421,106308,110700,109905,105642,126735,116267,104804,106437,102482,110647,111097,120510,111412,112965,122410,116109,139173,104179,111340,140994,136102,158922,152013,145020,99287
new=148,147,149,145,148,145,170,191,152,162,157,150,183,152,150,154,141,138,162,151,158,153,154,159,156,161,159,150,154,164,160
```

The indexed catch-boundary p50 remains 146/149/154 us across a 256x provider-count increase. This
confirms that panic isolation does not restore the removed linear provider-count term. It does not
measure real decorator callback work, metadata allocation/validation, panic execution, parser token
handling, registry retained bytes, product RSS, or power.
