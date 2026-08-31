# Rich typed dependency closure foundation

Date: 2026-08-30  
Scope: `RRT-P1-020` image-resource foundation only  
Status: `RRT-P1-020_typed_image_dependency_foundation_static_complete / icon_font_widget_decorator_lease_and_managed_validation_pending`

## Current-source finding

`CompiledRichText::resource_ids()` collected only `InlineObjectRef::Image.texture`, while its name
claimed to expose every resource owned by the artifact. The only production consumer was
`graphics/scene/resources/ui_texture.rs`, which treated every returned identifier as a 2D texture.
Adding an icon, font, widget, or decorator identity to that untyped slice would silently route it into
texture loading.

Current identities also prevent a complete closure from being fabricated safely:

- `Image` already owns an admitted `ResourceId` and is loadable by the texture streamer.
- `Icon` owns a family string and glyph, not a font/icon asset lease.
- `Widget` owns a bare `u64`, not a surface/generation-qualified child lease.
- decorator identity is parser/provider generation, not a loadable resource handle.
- Surface font discovery owns UI-tree font asset paths, while rich run family overrides are still
  family names rather than font asset identities.

## Reference and decision

Local Unreal `FSlateImageRun` retains the same `FSlateBrush` for measure and paint and explicitly
releases a dynamic brush. `FSlateIcon` resolves style-set/style names to a brush, and the rich image
decorator resolves that brush before constructing the run. A widget decorator constructs a real
`FSlateWidgetRun` child. The relevant local sources are:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/SlateImageRun.h`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Text/SlateImageRun.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Textures/SlateIcon.h`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Text/TextDecorators.cpp`

Zircon therefore uses a typed compiled dependency boundary. The first admitted variant is
`RichTextDependency::ImageTexture(ResourceId)`. The GPU texture collector explicitly selects that
variant. Family strings, bare widget ids, and parser generation numbers cannot enter the closure as if
they were loadable resources.

## Implementation

- `text/rich/compiled/dependency.rs` is the single collection owner. It walks canonical runs once,
  sorts typed dependencies, and deduplicates them before artifact publication.
- `CompiledRichText` retains `Arc<[RichTextDependency]>` and exposes `dependencies()`; the ambiguous
  `resource_ids()` API and field are removed.
- compiled residency accounts the typed slice by `size_of::<RichTextDependency>()`.
- UI texture discovery matches `ImageTexture` before forwarding the `ResourceId` to texture loading.
- focused Rust tests cover sorted image deduplication and the UI rich adapter checks the typed variant.

For `R` admitted rich runs and `D` emitted dependencies, construction remains `O(R + D log D)` time and
`O(D)` temporary memory, the same asymptotic algorithm as the previous image-only sort/dedup. Artifact
and render reads are borrowed slices; no second cache or per-frame parse was added. This is an ownership
correction, not a measured performance optimization.

## Evidence and open gates

- failing-first static contract reproduced the missing typed owner;
- complete Runtime Text static contract suite: 59/59 in the final 0.363 s rerun;
- focused Rust files pass `rustfmt --edition 2024 --check`;
- scoped `git diff --check` passes;
- `compiled.rs` is 730 lines, `compiled/dependency.rs` 63, and `compiled/memory.rs` 76.

Managed Cargo/rustc did not run because the managed acquisition path remains unavailable. No WGPU,
framebuffer, PNG, allocation, latency, RSS, power, or Unreal parity claim is made. The following remain
open:

- typed icon asset and optional font-backed icon face/collection lease;
- qualified widget child identity and lifetime;
- provider/decorator last-use lease publication;
- rich family override to font asset dependency mapping;
- resource readiness/outcome propagation to layout, paint, and accessibility;
- managed Cargo, host, WGPU/PNG, profile, RSS, and power validation.

No screenshot was produced because this slice changes dependency metadata rather than pixels; a source or
strategy screenshot would violate the Text07 evidence policy.
