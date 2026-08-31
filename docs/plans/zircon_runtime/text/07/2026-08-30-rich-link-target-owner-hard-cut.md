# Runtime Text rich link target owner hard cut (2026-08-30)

## Current-source finding

HTML and BBCode link admission already parsed an engine `ResourceLocator` and restricted it to
`res/lib/package/builtin`, but `LinkRef` immediately converted the admitted value back to `String`.
Link hit testing cloned that string into `UiDispatchEffect`; the effect application boundary then
maintained a second path/scheme algorithm and occasionally reparsed the locator before another clone
entered `UiDispatchHostRequestKind`.

This was an authority split, not a missing local fast path. The parser and input transaction could
disagree as the two allowlists or normalization rules evolved. The allocation-free application check
also optimized the second implementation instead of removing it.

## Reference boundary

Local Unreal `FSlateHyperlinkRun` retains `FRunInfo`, shared text/range, style, view model, and navigation
delegate under one run owner. `SRichTextHyperlink` invokes that retained navigation metadata; it does not
reparse hyperlink text at pointer release. Relevant source:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/SlateHyperlinkRun.h`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Text/SlateHyperlinkRun.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Input/SRichTextHyperlink.h`

Zircon follows the retained-metadata ownership principle, while preserving its stricter engine-resource
policy and serializable host request boundary. Unreal's arbitrary metadata map is not copied as Zircon's
security model.

## Implemented contract

1. RuntimeInterface owns `UiRichLinkTarget`, whose private `Arc<ResourceLocator>` can only be created by
   checked `parse`/`from_locator` constructors or checked deserialization.
2. The constructor accepts the existing scheme-less authoring shorthand as `res://`, canonicalizes path
   components and labels, and admits only `res`, `lib`, `package`, and `builtin`. Empty, escaping,
   network, memory, and empty-label targets fail before a rich artifact or effect exists.
3. `LinkRef` now retains `UiRichLinkTarget`. HTML/BBCode parsing no longer converts the admitted target
   to a string.
4. Link hit testing, `RequestLinkActivation`, and `ActivateLink` carry the same typed target. Their clones
   share the private locator allocation through `Arc`.
5. Effect application validates only the real `UiNodeId` input owner. The duplicate `split_once`/`Path`/
   `ResourceLocator::parse` link validator and its obsolete invalid-target runtime error are removed.
6. Serde keeps the existing `href: "res://..."` wire field and scalar string representation. The Rust
   field name changes to `target/link_target` without adding an object wrapper to host JSON.
7. Link metadata admission and compiled residency count the retained canonical locator components.

## Algorithm and performance boundary

Target construction performs one bounded locator parse/normalization over `B` input bytes: `O(B)` time
and one retained locator allocation. Run splitting, hit projection, effect publication, transaction
cloning, and host-request projection clone an `Arc` in `O(1)` and do not rescan the path. Pointer effect
application no longer parses or walks link text.

This is a structural correctness hard cut. No timing, allocation, RSS, power, or cross-engine performance
gain is claimed because managed profiling did not run. Any further representation or dispatch optimization
still requires the E-drive matched profile required by the optimization plan.

## Evidence and remaining gates

- The failing-first static contract initially failed because `rich_link_target.rs` did not exist.
- `UiRichLinkTarget` Rust tests cover canonical engine schemes, shorthand/label normalization, escape,
  memory/network rejection, checked serde, and the retained scalar wire shape.
- Existing HTML/BBCode, artifact retention, table projection, horizontal/vertical hit, input transaction,
  and host-request tests now assert the typed target.
- Rustfmt and scoped diff-check pass. The complete Runtime Text static suite passes 57/57 in 0.215 s.

Current status is `RRT-P1-030_typed_link_target_foundation_static_complete /
RRT-P1-040_qualified_link_child_and_managed_validation_pending`.

Typed link action kind, tooltip, interactive state, navigation policy, trust/principal, qualified semantic
child identity/action routing, managed Cargo, host integration, AccessKit/screen-reader, real WGPU/PNG,
timing/allocation/RSS/power, milestone commit, and WeCom remain open. This non-visual slice creates no
strategy screenshot.
