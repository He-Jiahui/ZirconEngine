# Editor268 Move Inspector Projection Identity

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime323-editor268-performance-batch-20260829aw-v1`

## Scope

Inspector template projection previously cloned five owned identity strings into the visual-field
model and retained the originals for final pane publication. The projection now moves those strings
into the visual fields, consumes the fields after node generation, and transfers the same buffers
into the pane result. Field rendering and final pane values are unchanged.

## Static Evidence

- Identity String clones per template projection: `5 -> 0`.
- Identity heap allocations per template projection: `5 -> 0`.
- Name, parent, transform text, and delete-enabled values retain their existing ownership boundary.

## Performance Gate

The ignored Windows release benchmark emits
`EDITOR268_MOVE_INSPECTOR_PROJECTION_IDENTITY_BENCH_V1`. It transfers five 96-byte identity fields
for 4,096 rows across 31 interleaved sample pairs and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
