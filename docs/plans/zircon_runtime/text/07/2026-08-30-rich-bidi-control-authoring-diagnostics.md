# Rich bidi-control trust and authoring diagnostics

Date: 2026-08-30  
Scope: `RRT-P1-041` source-ranged evidence and parser trust gate  
Status: `RRT-P1-041_trust_gate_and_balanced_isolation_static_complete / managed_copy_a11y_render_and_profile_pending`

## Current-source review

The shaping layer already resolves UAX#9 levels from the logical source, retains per-glyph bidi levels,
and projects visual order without replacing logical ranges. Copy and accessibility offsets are explicitly
logical. The missing owner was therefore not another bidi algorithm in shaping.

The rich parser admitted raw Unicode bidi controls in every format. BBCode also exposed `lrm/rlm/alm`,
legacy embedding/override tags, and isolate tags as equivalent literal substitutions. HTML numeric entities
could decode to the same controls. Only HTML syntax recovery produced authoring diagnostics, so consumers had
no bounded typed evidence that visually reordered content contained invisible direction controls.

## Reference and boundary decision

Local Unreal `FSlateTextShaper::ShapeBidirectionalText` delegates directional analysis to `TextBiDi`, while
`FTextLayout` retains ascending logical run ranges and performs visual block ordering afterward. Zircon keeps
that responsibility split: the shaping/layout owner consumes admitted Unicode; the rich parser owns source
trust and authoring evidence.

The following shortcuts are rejected:

- inserting FSI/PDI during shaping, because it would create characters with no logical source identity;
- stripping or replacing controls inside layout, because copy, hit testing, accessibility, and paint would
  no longer share one source projection;
- treating all controls as overrides, because directional marks, embeddings, overrides, and isolates require
  different authoring and future policy decisions;
- claiming a trust gate from warnings alone.

## Implemented parser policy

- `RichTextAuthoringDiagnosticCode` now distinguishes bidirectional mark, embedding/pop, override, and
  isolate controls with stable codes 013 through 016 and catalog keys.
- Non-overlapping source slices emitted by Plain, Markdown-inline, HTML-subset, and BBCode pass through one
  `push_source_bidi_control_diagnostics` owner before visible text append.
- HTML entity decoding reports the exact entity byte range to the same owner without allocating a source-map
  vector or making a second entity scan.
- BBCode literal control tags use the recognized token range, so synthesized Unicode still points to authored
  source.
- All diagnostics consume the existing `max_authoring_diagnostics` quota and truncation receipt. Recovery is
  `PreservedAsText`; the compiled visible/logical string is unchanged.
- `RichTextContentTrust` is a typed per-compile input. The existing `compile` entry point is fail-closed
  `Untrusted`; only the explicit `compile_with_content_trust` entry point can select `TrustedAuthoring`.
- Untrusted content accepts directional marks and balanced isolates, but rejects legacy embeddings, pops, and
  overrides with `BidiControlNotAllowed` and the exact authored range. Trusted authoring accepts legacy controls
  only while their explicit stack is balanced.
- `UnbalancedBidiControl` rejects unmatched terminators and unterminated openers. A dedicated default depth of
  125 follows the UAX#9 explicit-level bound and reports `BidiControlDepthExceeded` before stack growth.
- Trust is part of `RichTextArtifactKey` and retained by `CompiledRichText`, so an artifact compiled under trusted
  policy cannot satisfy an untrusted lookup. UI maps policy/balance failures to `LayoutFailed`; depth remains a
  typed parser budget failure.

The added scan is linear over emitted source bytes. Source slices are non-overlapping, so the added work is
`O(B)` per parse and does not multiply by run count. HTML entity observation is fused into the existing decode
loop. No new cache, retained source map, unbounded label, or per-frame work is introduced.

## Remaining validation gate

The policy and cache identity are now implemented, but acceptance still requires managed Rust execution of the
raw/entity/tag matrix and malformed balance corpus. Copy, hit testing, accessibility, and paint must then prove
that accepted controls retain one logical source projection while visual ordering remains a shaping/layout
projection. This nonvisual parser slice does not replace the final product WGPU/PNG and performance qualification.

## Evidence boundary

- failing-first static contracts reproduced both the missing unified owner and the absent typed trust/cache gate;
- focused Rust regressions cover all four formats, exact raw/entity/tag ranges, default legacy-control rejection,
  balanced isolates for both trust levels, trusted balanced overrides, unmatched controls, depth admission, cache
  identity, stable code uniqueness, logical text preservation, and shared diagnostic-budget truncation; they are
  written but not run;
- the complete infrastructure static suite passes 38/38 in the final 0.090 s rerun;
- focused `rustfmt --edition 2024` and scoped `git diff --check` pass;
- parser root/bidi leaf/root tests/bidi tests are 800/195/986/215 lines.

Managed Cargo/rustc, malformed-balance execution, WGPU/PNG, screen-reader/copy validation,
timing/allocation/RSS/power, commit, and WeCom remain open. No performance or visual improvement is claimed.
No screenshot is produced because this slice changes nonvisual parser admission and diagnostic metadata; a
source/strategy image would violate Text07 evidence policy.
