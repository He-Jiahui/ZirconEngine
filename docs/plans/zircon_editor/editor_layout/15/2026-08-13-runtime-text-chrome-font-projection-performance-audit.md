---
status: implementation_complete_static_reviewed_managed_validation_pending
owner_plan: docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
runtime_authority: docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
---

# Runtime Text Chrome Font Projection Performance Audit

## Scope

This audit covers the retained editor host conversion from `ChromeCommandStream` text commands to
runtime `UiSurfaceDrawList` text commands, plus the Runtime shared-database bootstrap needed for
headless retained measurement to resolve the same packaged fallback face before GPU UI startup. It
does not change shaping, layout, rasterization, glyph atlas ownership, the Runtime Text ABI, or the
editor typography preference model.

The current-source review follows the Text09 rule that an algorithm change requires a confirmed
owner and deterministic work evidence first. Unreal Slate remains the primary reference: font
identity is resolved by the font cache and reused by the element batching path rather than resolved
again for every text draw element. Zircon's existing `HostTextFont` cache and Runtime Text database
remain the authoritative owners; this slice only removes repeated hot-path access to them and
pre-registers Runtime Text's checked-in default face under its permanent bootstrap owner.

## Confirmed Bottleneck

Before this repair, every chrome text command independently called `ui_text_font_family(...)` and
`ui_text_font_weight(...)`:

- both calls cloned the complete `HostTextPreferences`, including all three family strings;
- the family call acquired the global `HostTextFont` cache mutex;
- the family and weight could be read from separate appearance snapshots;
- the required owned `UiSurfaceCommandKind::Text.font_family` string was then allocated as well.

For `N` text commands, conversion therefore performed `N` global font-cache acquisitions and
`2N` full preference snapshot clones in addition to the unavoidable `N` owned command-family
strings. This is deterministic O(N) global synchronization on a per-element rendering path. It is
the same class of scaling error prohibited by the open Text09 font-handle failure, even though it
occurs in the editor host rather than the runtime glyph registry.

## Forward Design

One draw-list conversion owns one lazy font projection context:

1. A stream containing no text never resolves a font.
2. The first text command captures one `HostTextPreferences` value.
3. That snapshot resolves the three bounded faces (`Ui`, `UiStrong`, `Mono`) once through the
   existing `HostTextFont` cache.
4. Every borrowed or owned command conversion thereafter selects a copied face projection without
   locking or cloning preferences.
5. Each output command still owns its runtime family string because that is the existing runtime
   surface ABI; changing it to an interned/Arc identity is out of scope without profiler evidence.
6. Runtime discovers normal system faces first, then registers the complete checked-in
   `default.font.toml` manifest under its permanent bootstrap owner. Face 0 receives the private
   retained-fallback alias, while face 1 remains available to the manifest's CJK route. The later
   `res://fonts/default.font.toml` asset attaches to those same source identities instead of
   registering a second TTC copy. The alias gives retained fallback the exact packaged bytes without
   shadowing an explicitly selected system `Fira Mono`; it is registered in both the logical matcher
   and glyphon's backend database so native shaping observes the same identity.

The resulting synchronization complexity is O(1) per draw list: zero font-set captures for a
non-text stream and exactly one for any non-empty text stream, independent of `N`. A cache miss
resolves the bounded three-face set under one slow-path cache operation. Appearance changes remain
coherent because family and weight come from the same captured preference snapshot.

## Verification Contract

- A 1,000-command borrowed conversion and the equivalent owned conversion each capture the host
  font set exactly once; their combined regression count is exactly two.
- A non-text conversion captures it zero times.
- Both conversions project the same resolved UI/strong/mono family and normalized weight.
- Existing Runtime Text measurement and retained software paint continue to resolve the same
  `HostTextFont` entries.
- A private fallback alias resolves to the same logical face through glyphon's backend query and is
  removed with its final asset owner; explicit code text honors the resolved family rather than
  replacing it with the generic monospace family.
- Focused Cargo and runtime profiling remain coordinator-owned; until their receipt, this record is
  `managed_validation_pending` and makes no wall-time, WGPU, power, or screenshot acceptance claim.

## Static Review Outcome

The second source review verified the private fallback identity end to end:

- the retained CPU measure/paint fallback and Runtime native-text projection use the same named
  identity, `Zircon Runtime Fallback Mono`;
- the Runtime bootstrap registers that identity as a second glyphon `fontdb::FaceInfo` which shares
  the packaged TTC source and face index with the primary face, rather than relying on a host-font
  fallback;
- bootstrap registers both checked-in manifest faces under the permanent owner; attaching and then
  removing the GPU UI owner reuses those exact source keys without changing render inputs, retaining
  both bootstrap faces for headless text;
- logical matching and glyphon shaping both map the alias to the same `FontFaceId`, while an
  explicitly requested `Fira Mono` can still select the separately discovered system face;
- retiring the final owner removes both the primary and alias backend entries, so no stale private
  alias can keep the packaged bytes alive; and
- explicit code-family requests now keep their resolved family in native shaping instead of being
  replaced by the generic monospace selector.

The changed Rust files are formatted and have a clean scoped diff check. Managed Cargo, profiling,
and current-source screenshot receipts remain required before this slice can be accepted.
