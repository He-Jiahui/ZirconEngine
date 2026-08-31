# RRT-P1-020 typed dependency closure foundation

Status: `RRT-P1-020_typed_image_dependency_foundation_static_complete / icon_font_widget_decorator_lease_and_managed_validation_pending`

The old `CompiledRichText::resource_ids()` contained only image textures, while its only production
consumer forwarded every identifier to the UI texture streamer. Extending that raw slice with an icon,
font, widget, or decorator identity would create type confusion.

The compiled artifact now retains a sorted and deduplicated `Arc<[RichTextDependency]>`. The first
admitted variant is `ImageTexture(ResourceId)`, which texture discovery explicitly matches. Cache
residency accounts the enum slice and the ambiguous `resource_ids()` API is removed. Construction remains
`O(R + D log D)` with `O(D)` temporary memory and borrowed artifact reads.

The complete Runtime Text static suite passes 59/59 in the final 0.363 s rerun. Focused Rust behavior tests are written,
but managed Cargo has not run. Icon/font asset leases, qualified widget identity, decorator last-use lease,
resource readiness/outcome, WGPU/PNG, profile, RSS, and power evidence remain open. See
[`../../../zircon_runtime/text/07/2026-08-30-rich-typed-dependency-closure-foundation.md`](../../../zircon_runtime/text/07/2026-08-30-rich-typed-dependency-closure-foundation.md).
