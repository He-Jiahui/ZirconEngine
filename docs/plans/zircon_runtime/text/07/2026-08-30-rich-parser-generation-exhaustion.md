# Rich parser identity and generation exhaustion review

Date: 2026-08-30

Status: `current_source_review_complete / unreal_owner_review_complete /
non_reusing_identity_and_generation_implemented_static / managed_validation_pending`

## Scope

This slice covers the identity tuple used by compiled-rich cache keys:
`parser_identity`, `decorator_generation`, and `emoji_generation`. It does not introduce a
process-global provider service, unregister/revoke leases, targeted cache invalidation, or product
performance claims. Those remain RRT-P1-010, RRT-P1-013, and RRT-P1-016 work.

## Current-source finding

`RichTextParser` currently allocates parser identities through `AtomicU64::fetch_add(1).max(1)` and
advances decorator/emoji generations through `wrapping_add(1)`, mapping zero back to one. Both paths
can reuse an identity already present in the process-global compiled cache after numeric exhaustion.
The registration paths mutate their registry first and advance generation second, so a checked
implementation added only after mutation would leave new provider state published under an old cache
identity when the counter is exhausted.

The failure mode violates the cache contract even though reaching `u64` exhaustion is operationally
remote: identity code must be correct at its representable boundary and must not encode collision as
normal success. This is a correctness/ownership repair rather than an optimization; no latency or
power claim is authorized and no performance profile is required for the branch replacement.

## Unreal reference boundary

Local Unreal `URichTextBlock::RebuildWidget` creates decorator instances owned by the widget and passes
strong decorator references into one `FRichTextLayoutMarshaller`. `SetDecorators` replaces the
marshaller's decorator array through the widget-owned path. The marshaller retains the parser,
writer, and decorators used by its layout; it does not identify unrelated widget/provider sets by a
wrapping process-global integer.

Zircon currently uses a shared compiled cache, so it needs an explicit qualified generation until the
larger RuntimeRichTextService/lease cutover exists. The aligned minimum is not to imitate Unreal's
object pointers in a global cache; it is to ensure the temporary numeric identity never aliases an
older owner.

## Required contract

1. Parser identity allocation uses a monotonic atomic compare/update path and transitions to an
   explicit exhausted state without returning a reused value.
2. A parser constructed after identity exhaustion remains a valid Rust value but every compile fails
   with typed `ParserIdentityExhausted`; it never enters cache lookup or construction.
3. Decorator and emoji next-generation values are computed with `checked_add` before registry
   mutation. Exhaustion returns each public registration API's typed `GenerationExhausted` error and
   leaves the registry and current generation unchanged.
4. Generation `u64::MAX` may be published once when advancing from `u64::MAX - 1`; the next mutation
   fails. Generation zero and wrap-to-one are forbidden.
5. Parser identity exhaustion maps to generic layout failure, not the rich representation budget
   diagnostic. It is an owner lifecycle failure.
6. Unit tests use local counters/parser state to exercise the boundary without modifying the
   process-global allocator. Static contracts reject `fetch_add` and `wrapping_add` regression and
   verify admission ordering.

## Validation gates

- failing static contract before implementation;
- focused Rust boundary tests written in the owner module;
- reproducible Runtime Text static suite;
- Rust 2024 formatting and scoped diff-check;
- managed Cargo remains required before milestone acceptance.

This record must not be used to close provider unregister/revoke, cache retirement, process-global
service ownership, WGPU/PNG, RSS, or package-power gates.

## Implementation evidence

`RichTextParser` now stores `Option<NonZeroU64>` identity state. The process allocator uses
`AtomicU64::fetch_update` with `checked_add`; once the counter reaches its terminal sentinel it leaves
the atomic unchanged and every later request returns no identity. An exhausted parser returns typed
`ParserIdentityExhausted` before source admission or cache access.

Decorator and emoji registration now call `next_decorator_generation()`/
`next_emoji_generation()` before their registry mutation. Both use `checked_add`; the public
registration error types return `GenerationExhausted` and the current registry/generation remain
unchanged. `u64::MAX - 1` advances once to `u64::MAX`, while the following mutation fails. UI maps
parser identity exhaustion to `LayoutFailed`, separate from representation budgets.

The owner-local Rust regression exercises a local atomic at the terminal boundary, both generation
transitions, no decorator/emoji publication after exhaustion, and compile failure for an exhausted
parser. It is written but not run because managed Cargo remains occupied outside this slice. The
current reproducible Runtime Text static suite passes 35/35; source guards report `fetch_add=0`,
`fetch_update=1`, `wrapping_add=0`, and `checked_add=2`. Rust 2024 formatting passes. Managed Cargo,
external API compilation, cache retirement/leases, WGPU/PNG, RSS, and power remain pending.
