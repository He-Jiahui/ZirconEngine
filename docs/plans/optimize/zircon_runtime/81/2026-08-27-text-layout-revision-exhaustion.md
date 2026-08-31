# Runtime Text surface layout revision exhaustion hard cut

Date: 2026-08-27

Status: `surface_text_revision_wrap_removed / exhausted_identity_fail_closed /
uncacheable_layout_fallback_preserved / retained_key_call_sites_converged /
static_checks_complete / managed_validation_pending`

## Finding

`UiLayoutCache::advance_text_layout_revision` used `wrapping_add(1)`. The surface later formed a
`TextDocumentKey` from `(node_id, text_layout_revision)`. After exhaustion, changed text could
therefore republish revision zero and alias an earlier retained parsed document/hard-line cache entry.
Exact source comparison currently rejects a stale parsed document, but identity correctness must not
depend on every future cache retaining that secondary comparison.

This was the product-side counterpart of the internal document revision defect fixed earlier.

## Implementation

- Revision advance now uses checked arithmetic. `u64::MAX` is an exhausted, unpublishable sentinel;
  it never returns from `retained_text_layout_revision()` and never wraps on later invalidations.
- Both surface extraction key-construction sites use the accessor. There is no direct raw revision to
  `TextDocumentKey` conversion left.
- Pending owner layout metadata carries `Option<TextDocumentKey>` explicitly. An exhausted owner
  still resolves text layout, preserves editable state and may use the unretained viewport window;
  only cross-frame retained document identity/cache reuse is disabled.
- Shape prewarm and ordinary exact-source layout/cache paths are unchanged. No text is dropped when
  the retained identity is unavailable.

Using `MAX` as a sentinel intentionally leaves the final numeric value unpublished. That conservative
choice avoids a new serialized exhaustion flag while guaranteeing that changed source is never
published under an earlier `(owner, revision)` pair.

## Evidence and open work

- Interface regressions lock `MAX - 1 -> MAX -> MAX` and `retained revision == None`.
- Render-owner regression locks a `None` key request and successful layout resolution.
- Source scans find only the two accessor-qualified surface key construction sites and no text-layout
  revision `wrapping_add`.
- Rust 2024 formatting passes for the 89-line interface owner, 248-line prewarm owner and 519-line
  prewarm test owner; scoped diff checks pass.

Managed Cargo, serde round-trip at the sentinel, surface extraction integration, long-lived node-pool
fault injection, WGPU and PNG remain pending. This correction does not wire the internal
`TextDocument` into the product surface and does not close Runtime82 document authority.
