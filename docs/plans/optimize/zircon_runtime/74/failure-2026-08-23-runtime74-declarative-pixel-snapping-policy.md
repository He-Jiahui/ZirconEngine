---
handoff_kind: failure
status: open
failure_scope: cross_plan
created_at: 2026-08-23
summary_slug: runtime74-declarative-pixel-snapping-policy
origin_plan: docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md
fixing_plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/12
fixing_child_dir: docs/plans/optimize/zircon_runtime/74
plan_link_mode: child_record_only
related_code:
  - zircon_runtime_interface/src/ui/layout/geometry.rs
  - zircon_runtime_interface/src/ui/layout/metrics.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime/src/ui/template
  - zircon_runtime_interface/src/tests/ui_geometry_metrics.rs
  - zircon_runtime/src/ui/tests/v2_asset/pixel_snapping.rs
  - zircon_runtime/tests/ui_pixel_snapping_policy.rs
---

# Runtime74 declarative pixel-snapping policy: design failure handoff

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md`
- 来源执行切片：M6 device-pixel geometry, analytic AA, and current-source visual acceptance
- 修复责任计划：`docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md`
- 交接原因：Runtime74 owns the compiled `.zui` template and render-command contract through
  which a declarative per-node snapping policy must reach `UiGeometry`.

## 失败现象与复现证据

- `UiPixelSnapping` defaults to `Enabled` in
  `zircon_runtime_interface/src/ui/layout/geometry.rs`, and `UiGeometry::from_frame` preserves that
  default for every element.
- `UiLayoutMetrics::default()` also enables snapping. `UiGeometry::from_frame_with_metrics` applies
  floor/ceil device-pixel snapping to the whole render frame, while `render_clip_frame` in
  `ui/surface/render/command.rs` applies the same list-level policy to clips.
- Current `.zui`, template schema, compiled-node, and style sources contain no declarative
  `pixel_snapping` property. A template author therefore cannot preserve subpixel motion for a
  transformed, scrolling, or animated node while retaining snapping for static one-pixel chrome.
- Managed native WGPU job `d2835f22545141cca42e0ce2d087afc0` proves the analytic rounded-box
  shader preserves a 0.25-physical-pixel translation when those coordinates reach the backend.
  The missing layer is the declarative command path before WGPU, not shader precision.
- `zircon_runtime_interface/src/ui/surface/render/command.rs` currently contains foreign
  Runtime74 transient-element changes. UI12 did not edit or take over that shared file.

## 最低共享层根因

Pixel snapping is modeled only as a batch-level `UiLayoutMetrics` switch. It is not part of a
compiled node, resolved style, or render command, so a single paint conversion applies the same
floor/ceil choice to unrelated static and moving elements. The renderer already supports
fractional geometry, but the `.zui` authoring and compilation boundary cannot express when to use
it.

## 架构修复验收

- Add one typed declarative snapping policy that survives `.zui` parsing, component expansion,
  compiled package serialization, resolved command generation, and paint-element construction.
- Keep static chrome, one-pixel separators, and text origins eligible for device-pixel snapping.
- Allow transformed, scrolling, and animated content to disable render-frame and clip-frame
  snapping without changing its logical hit-test geometry.
- Include the snapping policy in every cache or producer generation that can reuse compiled paint
  elements.
- Add focused 125% and 150% DPI regressions proving a disabled node preserves a 0.25 physical-pixel
  translation while an enabled static divider remains device aligned.
- Re-run the current-source Editor build and UI12 WGPU screenshot matrix after the lower-layer
  contract is integrated.

## 禁止临时方案

- Do not globally disable snapping or change its default merely to make one animation pass.
- Do not quantize layout, hit-test, scroll, or animation state to the device grid.
- Do not increase whole-window supersampling or SVG raster scale to hide a command-geometry policy
  defect.
- Do not add an Editor-only side table that bypasses the compiled `.zui` command authority.

## 修复结果与回传

Open validation state: the current source candidate now defines
`UiPixelSnappingPolicy::{Inherit, Disabled, SnapToPixel}` and carries optional per-node
`pixel_snapping` authoring through `.zui` parsing, component expansion, the compiled arena,
template metadata, resolved command style, and paint-element construction. Omitted component-mount
values preserve the component prototype; an explicit `inherit` resumes parent policy inheritance.
The resolved policy changes render-frame and clip-frame device snapping only, leaving arranged and
hit-test geometry unchanged. Because the resolved style participates in command serialization, a
policy change also changes the command cache generation.

Focused source regressions cover component inheritance/override and compiled-package round trips,
plus 125% and 150% DPI geometry where `disabled` preserves a quarter physical-pixel translation and
`snap_to_pixel` aligns both paint bounds and clips. Product `.zui` declarations now opt Workbench,
standalone Welcome, Activity Drawer, Host Shell, Editor Main Frame, Floating Window Source,
Component Showcase, and Material Demo/Lab roots out of inherited snapping; Activity Drawer covers
Asset Window and UI Layout Editor. The rounded Dropdown, drag overlay, single/range sliders,
progress bar, and skeleton also opt out, while the status bar and divider remain explicit
snap-to-pixel chrome. The shared RuntimeInterface fallback remains enabled for compatibility;
product behavior is selected through the existing per-node policy instead of changing that public
default. Runtime divider, dropdown, progress, skeleton, and slider command producers no longer
round logical owner geometry before policy resolution; the native Editor slider path follows the
same boundary. Fractional control geometry therefore reaches analytic paint, while explicit static
chrome remains device aligned. Python `.zui` contract tests and static formatting/diff guards pass.

This handoff remains open until a current-source Rust validation run compiles the focused tests and
the Editor product is rebuilt for native WGPU multi-DPI framebuffer screenshots. Existing managed
Cargo/coordinator results predate this source candidate and are not acceptance evidence.

The missing source-level regressions are now explicit in
`zircon_runtime/src/ui/tests/v2_asset/pixel_snapping.rs`. They cover `.zui` parse -> compiled arena
-> TOML compiled-package round trip, omitted component-mount preservation plus explicit `inherit`
override, and parent-policy resolution into render commands while fractional arranged and hit-test
frames remain unchanged. The module is registered from `zircon_runtime/src/ui/tests/v2_asset.rs`;
rustfmt, seven focused source guards, and the related Python contract matrix (27/27) pass. No Cargo
or rustc was started for this addition, so the handoff status remains `open`.

On 2026-08-26, `tools/build/build-editor.ps1 -Ephemeral` submitted a fresh current-worktree product build
for `E:\ZirconBuilds\editor-ui12-pixel-policy-20260826-b9e2f4`. The coordinator accepted
`cargo.acquire` request `392d7578d453402c841fd43864921cb9` but returned
`command_post_timeout` after reconciliation found no terminal result. The validator exited before
Cargo/rustc started, no matching process remained, and the bundle directory was not published.
This is coordinator-admission evidence only; it is not a current-source compile fingerprint.

A later one-shot managed Editor build from main HEAD
`166720dcb59c57fb4b33c34b859dc1a3f572b222` plus the current shared overlay targeted
`E:\ZirconBuilds\editor-ui12-current-166720d-20260826-r2`. The coordinator again rejected the
request before Cargo/rustc with `unmanaged_artifacts_detected`: its current report includes the
stale `D:\ZirconBuilds\tooling15-wave98-runtime-20260826-195306` cleanup reservation and external
wave99/resource-management artifact paths. UI12 did not delete those foreign paths or retry. No
Cargo/rustc process or `.git/index.lock` remained, and the requested bundle directory was absent;
therefore this attempt also produces no current-source compile fingerprint or visual evidence.

On 2026-09-01, UI12 corrected the remaining product-root policy mismatch without changing the
shared default. The focused source contract first failed on the Workbench root's inherited
`snap_to_pixel`, then passed 6/6 after the Workbench, Welcome, Activity Drawer, and Dropdown paths
published `disabled`. A follow-up audit first failed all six newly covered independent surface
roots on their absent declarations, then passed after the root-only correction; the complete Editor
`.zui` matrix passed 355/355 and all 253 Editor assets parsed. A managed Windows
`zircon_runtime_interface --locked` build was rejected before Cargo by the foreign cleanup
reservation `E:\cargo-targets\zircon-engine\cache\cargo-metadata-home`. After artifact audit
returned `unmanaged = []`, a managed `build-editor.ps1` attempt ended with exit 1 when the
coordinator runtime descriptor became unavailable; it returned no attributable Job/stdout/stderr,
published no bundle, and left zero Cargo/rustc processes. The handoff remains open for focused Rust
execution plus current-source Editor WGPU multi-DPI screenshots.

## 2026-09-08 lower-layer executable regression

Stable fixing Session: `failure-roll-01a07160-runtime74`, baseline 601. The original
record was adopted unchanged from its archived owner through transfer fingerprint
`8729b948202d4e8173af6d62c97244dedda52cc07bf8e0fbadf70bb99f147bad`;
snapshot 3179 preserves that record and the HEAD-clean geometry test preimage
`60d9ad8c6133648b1262eac2d3f0c76b7256262560f3aaadbf22b6ce0c76dbf6`.

Managed full-interface job `dd18d653fe3c494c91fbe57f5959580a` reached the original
fractional-DPI regression and failed before its snapped branch: logical 10.6 at
125% DPI is physical 13.25, but the fixture asserted 12.25. The lower production
path already preserves logical layout/hit geometry and resolves the per-command
policy for both paint bounds and clip bounds. Only the 125% test coordinate changes
to 9.8; the 150% case, quarter-pixel expectation, tolerance, and snapping checks
remain intact.

Source snapshot 3180 freezes ui_geometry_metrics.rs at
`a4f7b7ee9c47eb2338fe720fef81d4da3ae7245174ee820a4e05d388fc2badb5`.
Immutable input `runtime74-pixel-policy-3180-20260908` derives from the previously
sealed Interface3170 input with only this exact owned object replaced. Its manifest
is `d5e3166653914d98b498c1ecc2bf52882509874d3cb19a96a180ef0ad453951b`.
Windows managed job `2262233236a640fcb6675021fc0f4b93` ran all five
`tests::ui_geometry_metrics::` tests and passed 5/5, exit 0. This includes 125%/150%
DPI policy overrides, unchanged logical geometry, clip conversion, policy cache
generation, and image raster-size parity. The command retained `--locked`,
`--no-default-features`, `--lib`, and static linking in the coordinator E pool.
Logs and the source-bound receipt are in that input's
`results/runtime74-pixel-policy-3180.{json,log}`.

Open state: `lower-geometry-tests-passed_upward-and-formal-validation-pending`.
Compiled-package/component/runtime extraction, current Editor WGPU multi-DPI
screenshots, independent review, formal fixing-Session binding, and closeout remain
required. The operational job is not a formal failure closeout ticket. No fixed
return, commit, or WeCom notification is claimed.

## 2026-09-08 upward compile results

Two distinct Windows locked/static gates used the same immutable input above:

- Job `4e66fb554c0d4acdb6ed0234a963d54f`, Runtime `--lib` with `ui` and filter
  `pixel_snapping`: 139 compile errors, no tests executed. The source copy lacks
  six Editor icon/ZUI fixtures and has unresolved shared UI/Text/Graphics test
  consumers. Log: `results/runtime74-pixel-policy-upward-3180.log`.
- Job `c2cd21c65c794ed8a790177036c7d56e`, Runtime integration target
  `ui_pixel_snapping_policy` with `ui` and filter `pixel_snapping`: four production
  compile errors, no tests executed. The frozen input still calls a test-only
  rich parser, treats fallible parsed text as a value in inline widgets, and passes
  `UiTextRange` to the compact tuple-range helper. Log:
  `results/runtime74-pixel-policy-integration-3180.log`.

The existing lower-owner handoffs are
[Text03 parser/export contract](../../../zircon_runtime/text/03/failure-2026-09-06-editor-text-export-contract.md)
and [Text03 paragraph range contract](../../../zircon_runtime/text/03/failure-2026-09-07-paragraph-list-prefix-range-contract.md).
Current source inspection finds the parser test helper gated with `cfg(test)`,
fallible inline parsing handled with `.ok()?`, and paragraph prefixes validated
through `checked_source_range_to_u32`. Those later source changes are not in this
input and have not been adopted into a new Runtime74 validation snapshot. The
paragraph file retains an active Text03 owner, so this Session does not overwrite
or transfer it. Upward acceptance remains suspended pending exact owner evidence.
