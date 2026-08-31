# Rich link tooltip metadata owner review (2026-08-30)

## Status

`RRT-P1-030_typed_target_and_tooltip_metadata_static_complete /
RRT-P1-040_qualified_link_child_and_managed_validation_pending`

This is a non-acceptance infrastructure slice. Managed Cargo/host tests, hover presentation,
screen-reader action publication, WGPU/PNG, timing/allocation/RSS/power, commit, and WeCom remain
open.

## Problem

After the typed target hard cut, `LinkRef` still retained only the destination. HTML `title` was
reported as unsupported, BBCode had no link metadata form, and `UiTextLinkHit` discarded any future
secondary metadata. Adding tooltip text directly to the surface tooltip state would have been the
wrong boundary: that state owns overlay IDs and timers, not arbitrary rich-run strings.

## Implementation

- `LinkRef` now owns `tooltip: Option<Arc<str>>`; active-tag/run/hit clones share the allocation.
- HTML `<a href="..." title="...">` admits `title` through the existing whitelist.
- BBCode keeps `[url=...]` and adds `[url href="..." title="..."]` for named metadata.
- Parser validation still constructs one `UiRichLinkTarget`; tooltip text does not weaken destination
  admission.
- Decorator request metadata and compiled residency accounting include tooltip bytes.
- `UiTextLinkHit` carries the same `Arc<str>` with the target and source range. It does not create a
  tooltip overlay or host navigation request by itself.
- Serde omits absent tooltip while retaining the existing scalar `href` wire field.

## Evidence

TDD first added
`test_rich_link_tooltip_is_shared_from_parser_to_hit_projection`; it failed on the missing model
field. After implementation:

- Runtime Text static contracts: 58/58 in the final 0.236 s rerun.
- Scoped `rustfmt --edition 2024 --check`: pass.
- Scoped `git diff --check`: pass, with line-ending warnings only.
- Compiled residency calculation is isolated in the 76-line `compiled/memory.rs` leaf; after the
  typed-dependency follow-up, the orchestration root is 730 lines.
- Rust parser tests cover HTML and BBCode tooltip preservation.
- The cache-eviction link-hit regression checks that the prepared compiled artifact still projects
  `Open help` after cache eviction.

The Rust tests are written but have not run because the managed Cargo acquisition gate remains
blocked. No latency or allocation improvement is claimed.

## Remaining boundary

Tooltip content is now available at the qualified hit boundary. Hover overlay creation, tooltip
identity, visited/disabled state, action kind, navigation policy, trust/principal, and stable
accessibility child/action identity remain RRT-P1-040/RRT-P1-042 work. They must consume this metadata
instead of reparsing markup or attaching raw text to an unqualified surface node.
