# Runtime51 Resource-Management Projection No-Op Fast Path

- Date: 2026-08-24
- Session: `optimize-runtime51-query-merge-heap-r1-20260824`
- Related plan: Runtime51 M6 projection persistence direction
- Status: candidate implemented; managed batch validation and product measurements pending

## Scope

`ResourceManagementProjection::apply_delta` previously cloned a shard map and materialized an
`Arc<ResourceManagementRow>` before detecting that a changed `ResourceRecord` had the same
management projection as the current row. Updates to non-projected fields such as `source_hash`,
importer metadata, artifact locator, dependency ids, and config hash can now compare against the
current row first. A matching projection leaves the existing generation untouched.

`ResourceLocator::matches_display` compares the canonical scheme, path, and optional label to the
row's stored locator text without formatting a temporary `String`. The comparison covers every
field materialized by `ResourceManagementRow::from_record`: id, kind, primary locator, revision,
state, and diagnostic count.

## Deterministic Work Reduction

For a metadata-only record update whose management row is already present, the fast path avoids
one `ResourceManagementRow` allocation, its `Arc` allocation, locator display formatting, and the
copy-on-write shard `HashMap` clone. It also preserves the existing management-generation `Arc`
instead of publishing an equivalent generation.

This is a source-level allocation and publication reduction, not a p95, allocation-rate, RSS,
VRAM, fragmentation, or end-to-end product-performance claim. The lookup still reads the current
immutable row, and real workload measurements remain required.

## Validation

The focused regression checks assert that a `source_hash`-only update preserves generation identity
and that the existing row is compared before management-row construction. Scoped `rustfmt --check`
passed. Scoped `git diff --check` reported only the repository's CRLF normalization warnings.

Cargo compilation and behavior tests are intentionally deferred to the managed multi-package batch:

```text
cargo test -p zircon_runtime_interface -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never
```

This candidate does not close Runtime51 M6/M8 or the product performance target until that batch
and current-source representative p95/RSS/VRAM measurements have passed.
