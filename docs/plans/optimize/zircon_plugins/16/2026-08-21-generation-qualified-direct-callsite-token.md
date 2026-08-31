# Plugins16 Generation-Qualified Direct Callsite Token Optimization Record

- Date: 2026-08-21
- Owner: `optimize-plugins16-callsite-token-r1-01a00797-20260821`
- Source plan: `docs/plans/optimize/zircon_plugins/16-first-party-zr-vm-language-source-runtime-dist-catalog-reflection-callsite-host-interface-gc-hot-reload-product-integration-review.md`, NZR-P1-022
- Status: implementation and focused static validation complete; managed release timing queued

## Problem

`ScriptCallTable` allocated every callsite token through one process-global
atomic and duplicated every compiled callsite into a token-keyed `HashMap`.
Token dispatch therefore paid a hash lookup even though callsites were already
compiled in deterministic dense order. The opaque global sequence also carried
no table identity, so cross-table rejection depended only on map membership.

## Change

- One process-global atomic now allocates a nonzero generation once per compiled
  table rather than once per reflected field.
- A token encodes its table generation in the high 32 bits and its one-based
  dense ordinal in the low 32 bits.
- The duplicate token `HashMap` is replaced by one immutable dense callsite
  vector. Token reads and writes validate the generation and ordinal, then use
  direct indexing.
- Existing name resolution remains available for package loading. Existing
  catalog snapshot guards and reflected read/write behavior are unchanged.
- Regression tests cover generation separation, one-based dense ordinals,
  cross-generation rejection, zero ordinals, out-of-range ordinals, and the
  previously required no-token-reuse behavior.

## Deterministic Performance Evidence

The managed release gate compiles 4,096 reflected fields and performs 32 full
token-resolution rounds per sample. It compares the former token `HashMap`
lookup with the generation-qualified dense lookup using the same compiled sites
and token order.

| Measure | Legacy | Optimized | Gate |
|---|---:|---:|---:|
| Compile-time global atomic operations | 4,096 | 1 | exact |
| Retained token hash entries | 4,096 | 0 | eliminated |
| Token dispatches per sample | 131,072 hash lookups | 131,072 direct indexes | exact |
| Timing distribution | 21 samples | 21 samples | alternating first-run order |
| Nearest-rank P95 | pending | pending | optimized <= 50% of legacy |

Exact Windows P50/P95 values remain pending the combined coordinator batch and
must be written here before integration acceptance.

## Current Execution Evidence

- Integration Session: `root-runtime-interface03-activate-link-failure-20260831`.
- Exact-path ownership transfer applied by request
  `7788690352e049958d59223c90847d58`, from preview
  `6a34f771bb334e428bef3d596538264e` and fingerprint
  `4962ef7107a638dc7fce4400bad7ba8b228b04dcf1df1b9984003964621c9244`.
- Current source SHA-256:
  `script_call_table.rs=6485FCDF76C1CF3BC5586AB713AC7D38F736D823C0E6497E3DBDCF4E1391948C`,
  `tests.rs=3D18095D32175F35DB57C39359185A3D3F211ED91AE9231E1E04CD5444897AD3`.
- Deterministic work-model manifest SHA-256:
  `3826DE9DB61E243A8BD83B74CBE4413B809D847369034D04DC34CA6A08C7C3B7`.
  The model records `4,096 -> 1` compile atomics, `4,096 -> 0` retained
  token-hash entries, and `131,072 -> 0` token-hash lookups while preserving
  `131,072` direct-index dispatches.
- Focused source/model contract passed locally `7/7`; the source-bound managed
  static ticket is `2acef3a01bd94382b3d948612c6c0971`.
- Exact ignored Windows release benchmark ticket
  `74b6e322d0394b3f9cc81afadff564ef` is queued. Its 21 alternating sample
  pairs are the only accepted source for the pending P50/P95 values.

The pinned Plugins16 child validator is
`zircon-validation-plugins16-callsite-token.ps1` at SHA-256
`DF42A70B1AB6FFA87D694941C133AB00ECB406786537C5BF12104393557FF5A7`.
It is aggregated with the existing six plugin batches by
`zircon-validation-plugin-super-batch-seven.ps1` at SHA-256
`604F14D060369A782343475D69C142B89F443851995D6DB3E941CCAEBDCF30F2`.

## Acceptance

- Two tables compiled from the same registry use different high-bit
  generations while each table starts dense ordinals at one.
- A token from another table, a zero ordinal, or an out-of-range ordinal fails
  with `InvalidToken` before dispatch.
- Existing callsite resolution, catalog revision, read, and write regressions
  run in the same managed Cargo group as the new token tests.
- `generation_qualified_direct_token_release_benchmark` emits 21 alternating
  raw sample pairs, recomputable nearest-rank P50/P95 values, and exact atomic,
  hash-entry, hash-lookup, and direct-index counts.
- Exact-file Rustfmt and scoped diff checks are green. Cargo regressions and
  release timing are queued in the managed Windows lane; no synchronous wait
  or duplicate Cargo invocation is used.

## Remaining Scope

The token now identifies its owning compiled table generation, but it does not
yet encode package principal, world generation, schema identity, or field-level
authority. Explicit rebind receipts and exhaustion diagnostics beyond the
existing typed capacity error also remain. NZR-P1-022 is therefore only partly
closed; the broader package/world ownership contract must land with the planned
typed reflection and `ScriptWorldTransaction` work.
