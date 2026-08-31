---
title: Editor text value and media generation performance review
date: 2026-08-22
module: zircon_editor retained host text_layout/value_media projection and shared preview owner
priority: MVP-P0 editor retained UI content
status: source_reviewed_m1_applied_static_pass_dynamic_pending
reference_engine: Unreal Engine Slate text layout cache and invalidated image attributes
---

# Goal

Compile text/layout style and typed value/media capabilities once per component generation. Stable
nodes must not repeatedly probe the generic TOML map for dozens of unrelated fields, recursively
materialize collection values only to show a count, or lock the preview-image cache when no media or
icon was requested.

## Reviewed source

- Rust files: 24/24
- lines: 1,333
- bytes: 43,380
- joined raw source-bytes SHA256:
  `166cf7b2fce0f7d51b1113c3ecf0ad8c96fd6fa61c048b6413456d8502c48e44`
- owning commit at review: `a922089697e41e07fa29e3e42a5e4c9afc1ae31b`

| File | Lines | Bytes | SHA256 |
| --- | ---: | ---: | --- |
| `layouts/views/preview_images.rs` | 482 | 15,068 | `3c560940d4d368978dd07774d04f1caf870508112e98235506eca20dbc7fef9e` |
| `layouts/views/view_projection/visual_assets.rs` | 53 | 1,856 | `178d76f15922acbebdde116efe52db3b4bb67395389aa73dcb2833cc982ab301` |
| `pane_component_projection/attribute_values.rs` | 42 | 1,210 | `db29debfeda5dc98db5e67938f674552ff1eb590f693b2fd57f72ee7b3ca625e` |
| `pane_component_projection/badge.rs` | 33 | 1,152 | `090c41207a8975fcf29e6d46da841e836094d0c02ec55f5bd663b5ca7ce31d5a` |
| `pane_component_projection/preview_images.rs` | 1 | 62 | `cd74acd88eeca05ecb1910abb0bfaa52bab5c7274e10a833ef8ddfaa456892c6` |
| `pane_component_projection/progress_value.rs` | 63 | 1,869 | `484c47f9fe01238e1d1e527e209520ee1095023aeafa0bf35e7a38418f33e1da` |
| `pane_component_projection/string_lists.rs` | 6 | 275 | `67f19a422bc491df0ddc7bf198650d2cb5e18252711bfd951006226deaf8c95a` |
| `pane_component_projection/text_layout/attributes.rs` | 14 | 344 | `29791a0da95cc770fcd58bc105d85864d1fa034e200a7aefbb1221eed634a961` |
| `pane_component_projection/text_layout/label.rs` | 33 | 1,074 | `6894259cf3d4b970503f7709803389f05ac3106c4cc36b89dc9e511df5945b93` |
| `pane_component_projection/text_layout/mod.rs` | 60 | 2,317 | `9142c526a677b0d47e8e358f0402b92823b2201e4d605d6ebcf427b928e2b2bf` |
| `pane_component_projection/text_layout/model.rs` | 25 | 1,186 | `0fc01e6f16324284c927e42cfc49e201e836452717ee9b513a70d737c64001b1` |
| `pane_component_projection/text_layout/offsets.rs` | 59 | 2,459 | `267af76eb3c1c5cadd463d657fdfd7defd5c6cf9f9f85d67f0ebc46c83cc2236` |
| `pane_component_projection/text_layout/selected_segment.rs` | 32 | 1,169 | `7a83b1ebcfc3aa542b384164dfda6c8b3c101b3749a62ba0e38885fb4a6018d4` |
| `pane_component_projection/text_layout/text.rs` | 35 | 1,321 | `2cdca5f755beddd71071fb69fbf6ec96a373e7d99b76c4a12df013298995a7e3` |
| `pane_component_projection/text_layout/typography.rs` | 58 | 1,628 | `8d57129fb3d308ba8692831dae1a51fd711720d45363e4c875623241455f2a77` |
| `pane_component_projection/value_color.rs` | 39 | 981 | `968ef06062ffcb2f9a1cdad01df3049a1d3099e0a8f703deed89e1a012237855` |
| `pane_component_projection/value_media/icon.rs` | 29 | 703 | `183ce5667a32c136693d2aeca95c71e8324edaf619eda170cc198b9d2441b7dc` |
| `pane_component_projection/value_media/media.rs` | 22 | 612 | `33a910246c1bd574db0d90736a1ffdc1f64954c2a8c7953cce47ab1352fecbdd` |
| `pane_component_projection/value_media/mod.rs` | 61 | 2,181 | `050ecc913ee7ca21ceea946d328a2fcd746d91922c02cd5030b3e2ef3ad89ef2` |
| `pane_component_projection/value_media/model.rs` | 16 | 690 | `a80c799be95561cce249406706486b6427a6aa4442540653e96146410361acf7` |
| `pane_component_projection/value_media/number.rs` | 16 | 583 | `dfc0a2100c4fd84fd04ea561c6e6f5ce05f8c5445dfa6f964441485bbeffd034` |
| `pane_component_projection/value_media/text.rs` | 54 | 1,868 | `50adfb08561d5235dd8399239821f8325711912566d8a5dba1b11afdea3f12d8` |
| `pane_component_projection/value_media/vector.rs` | 10 | 312 | `6501ea4dbba03d10711b99aff22ac5ebdbded62b307adc6c9292dd1bc1b4635b` |
| `pane_data_conversion/pane_value_conversion.rs` | 90 | 2,460 | `7e2568466e3ec2e2418c3f22d87aca13d292dcdfdc5de10cc392da1c113878be` |

Supporting paths traced: generic host-node assembly, dialog/notification/badge value-text helpers,
retained text/value/image painters, `UiValue::from_toml/display_text`, preview cache callers, the
parallel layouts visual-asset projector and the existing visual-asset painter review.

## Correct foundations to retain

1. Preview cache entries include resource generation and source-bucket LRU eviction is bounded.
2. SVG parsing/raster and optional system-font initialization have attributable profiler scopes.
3. Scalar/vector/percent normalization is deterministic and slider/progress roles preserve their
   distinct value policy.
4. Badge/dialog/notification value-text helpers gate their role-specific work before parsing.

## Structural findings

### P0: every node projects one flat text-layout and value-media record

`host_template_node` unconditionally calls both projection bundles. With absent attributes,
text-layout attempts about 38 ordered-map reads across text, label, offsets, selected-segment and
typography fields and allocates a default left-alignment string. Value-media then attempts about 30
more reads across text, numeric/percent, color, media/icon and vector channels. A component rarely
owns all of these capabilities.

The target is a compiled content descriptor with optional `TextSpec`, `ValueSpec`, `MediaSpec` and
component-specific layout metrics. Dynamic value/text changes become small retained patches; style,
offsets and asset handles remain shared by generation.

### P0: collection summaries recursively materialize the complete value

The generic fallback reads `value`, `items` or `entries`, converts the complete TOML subtree to
`UiValue`, then calls `display_text`. Arrays and tables only display `"N items"` or `"N entries"`, so
the current algorithm is O(total descendants), clones every nested string/map key and allocates an
intermediate tree for an O(1) result.

M1 will read array/table lengths directly and preserve the existing scalar conversion path. Output is
byte-for-byte identical while nested collection visits and intermediate ownership fall to zero.

### P0: media-less nodes still serialize through the preview cache mutex

`load_preview_image_for_generation("", "", generation)` first locks the global cache. The first
ordinary node resolves an asset root, creates an empty candidate Vec and inserts an empty image under
the empty keys; every later ordinary node still locks, advances the LRU clock, performs source/icon
hash lookups and clones the empty image.

M1 will return `Image::default()` before the cache lock when both trimmed inputs are empty. Real image
and icon requests retain generation lookup, bounded eviction and decode/raster behavior.

### P1: preview ownership and invalidation remain too broad

Both layouts projection and retained-host pane conversion resolve and clone preview images into wide
node DTOs. The existing visual-assets report already owns painter resource/candidate/pixel-cache
problems. M2-M4 must converge these paths on one resource handle/generation and must not introduce a
second projection cache.

### P1: text layout is recomputed from raw aliases rather than dirtied by reason

Offsets, selected-segment metrics and typography are static for most nodes, but they are rebuilt with
content. Text/value changes can therefore dirty the whole node instead of an explicit text measure,
paint or accessibility channel. Default strings also become equality/cache payload.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Widgets/Images/SImage.h`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Widgets/Images/SImage.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Text/STextBlock.h`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Text/STextBlock.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Types/SlateAttributeDescriptor.h`

Slate `SImage` stores a brush as a retained attribute and registers the invalidation reason for image
changes; media-less widgets do not call an image resolver. `STextBlock` separates simple text from
full layout, retains `TextLayoutCache`, updates it only when text/style/desired-size flags require it,
and emits layout invalidation only when wrapping changes desired size.

The transferable invariants are typed optional content, resource handles, retained text layout and
reasoned invalidation. Zircon should not copy Slate pointer/macros or assume Unreal cache sizes and
timing budgets are valid Zircon targets.

## Target architecture

1. EditorUI06 compiles typed optional text/value/media/layout specs once per template/style/resource
   generation; aliases disappear after compilation.
2. EditorUI08 retains small content/value patches and one resource handle, not decoded `Image` plus
   media strings in every generic node.
3. Runtime UI09 defines text measure/paint/accessibility and image resource invalidation receipts.
4. Runtime Text owns reusable shaping/layout keyed by text/style/width/scale generations.
5. Editor10/Render13 own resource resolve/decode/raster/upload generations already identified by the
   visual-assets and asset-content plans; only visible/bounded-prefetch nodes request media.
6. Hard-cut the duplicate layouts and retained-host raw projection after all consumers migrate.

## Instrumentation and acceptance

Matrix: nodes `100/1k/10k`, text/value/media share `0/1/10/100%`, collection depth `1/8/64`,
collection items `0/1/100/10k`, media `0/1/100/10k`, stable/1% changes, scale `1/1.5/2`.

| Evidence | Acceptance |
| --- | --- |
| TOML lookups and projected owned bytes | stable generation: zero raw projection |
| collection descendant visits/allocations | summary is O(1); descendants visited = 0 |
| empty preview requests/locks/hash lookups | all zero |
| decode/raster/upload/resource handle copies | at most once per requested resource generation |
| text measure/layout rebuild and dirty reason | proportional to changed visible text/layout |
| CPU/allocation/RSS/input latency/context switches/power | same current-source executable before/after |
| RenderDoc | media upload/draw/pixel parity only; not CPU text/value acceptance |

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add lookup/owned-byte/collection-visit/preview-lock/text-layout counters and capture matrix. | attributable baseline |
| M1 | Bypass empty preview cache and summarize collection length without recursive conversion. | focused RED-to-GREEN contracts |
| M2 | Compile typed optional content specs and retain compact value/text patches. | stable raw projection = 0 |
| M3 | Converge preview image ownership on resource handles and reasoned text/image invalidation. | one generation owner |
| M4 | Hard-cut duplicate flat DTO/resolvers across layouts and retained host. | exact deletion contract |
| M5 | Run managed scale/content/WPR/power and RenderDoc media parity matrix. | quantified acceptance |

## M1 implementation result

The shared preview loader now returns the default image before profiling/cache synchronization when
both trimmed source and icon are empty. Real resource requests retain the original generation-aware
cache. Generic value text now reads array/table length directly; scalar values retain the existing
`UiValue` display policy.

Per media-less node:

| Empty preview work | Before cold / steady | After | Change |
| --- | ---: | ---: | ---: |
| global preview-cache Mutex acquisitions | 2 / 1 | 0 | -100% |
| steady source/icon hash lookups | 2 | 0 | -100% |
| empty cache insert and owned key strings | 1 insert / 2 strings | 0 | -100% |
| asset-root/candidate construction | 1 / 0 | 0 | cold -100% |

For a top-level array/table with `D` descendants and `K` nested map keys/strings:

| Collection summary work | Before | After |
| --- | ---: | ---: |
| descendants visited | D | 0 |
| intermediate UiValue nodes | 1 + D | 0 |
| nested key/string clones | up to K | 0 |
| final user-facing summary String | 1 | 1 |

M1 preserves `"N items"`/`"N entries"` byte output. It does not remove the roughly 68 absent-field
lookups in generic text/value/media projection, decoded-image ownership in wide DTOs or duplicate
layouts/retained resolvers; M2-M4 own those boundaries.

Post-M1 owner scope:

- Rust files: 24/24
- lines: 1,376
- bytes: 44,779
- joined raw source-bytes SHA256:
  `7a9ffbc8248649b68457e4697f09e851493bc7c49bb939076828ab2f28846c58`
- unchanged owner files: 22 retain the pre-M1 fingerprints above

| Changed file | Lines | Bytes | SHA256 |
| --- | ---: | ---: | --- |
| `layouts/views/preview_images.rs` | 497 | 15,477 | `cdc9c38894a299359d277a6825d4fd5c2c8f42bb9693cfc8f03d67c588dd369d` |
| `pane_component_projection/value_media/text.rs` | 82 | 2,858 | `f3e80f0878d42693e6799c192b0afddb9103b4267ee4d5335ea45dc86e505e29` |

Focused contract: `tools/tests/test_editor_text_value_media_projection_performance_contract.py`,
38 lines, 1,619 bytes, SHA256
`c765d27dc8f5f9d9bcaf2d9927e2806cadc6d478f1632cb3448aa58d88a6fe1e`.

## Validation state

- Full owner source review: passed, 24/24 Rust files.
- Host/painter/UiValue/parallel-layout/preview-cache consumers and Unreal sources above: read.
- M1 focused contract: RED 2/2 before the change, GREEN 2/2 after the change.
- Current owned performance-contract set: GREEN 49/49.
- `rustfmt --check` for changed Rust files and scoped `git diff --check`: passed.
- Rust regressions verify collection output parity and that empty preview requests do not populate the
  cache; they are present but not claimed passing until managed Cargo is executable.
- M0 and M2-M5 remain pending; no dynamic performance claim is made from static complexity evidence.
- Managed Cargo remains unavailable because Session
  `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552` is terminal `archived` and rejects Cargo
  launch with `cargo_session_not_executable`.
- WPR and RenderDoc remain pending a current-source launchable editor.

The module remains in `pending.md` until M0-M5 pass on one source/executable/content fingerprint.
