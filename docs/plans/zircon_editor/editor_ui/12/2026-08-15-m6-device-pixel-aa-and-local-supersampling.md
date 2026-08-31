# UI12 M6 Device-Pixel AA And Local Supersampling

## Problem

The retained editor already creates its native GPU and softbuffer surfaces from
`Window::surface_size()`, so the final backing extent is the physical client
extent. The visible quality defect is downstream of that boundary:

- software rounded rectangles classify one pixel-center as fully inside or out;
- native-resize snapshots use physical-pixel-center bilinear sampling with
  linear-light color interpolation;
- SVG assets must be keyed by their physical display target and locally
  supersampled before linear-light resolve;
- the WGPU path uses a one-sample solid pipeline; rounded fills and borders are
  evaluated by an analytic signed-distance shader with screen-space derivatives.

Increasing only the `.zui` radius cannot repair those sampling defects.

## Unreal Reference Contract

The local Unreal source establishes the target architecture:

- `WindowsPlatformApplicationMisc.cpp` resolves per-monitor effective DPI.
- `SWindow.cpp` composes application scale and window DPI into the local to
  screen/layout transform while keeping window bounds in desktop pixels.
- `SlateRHIRenderer.cpp` creates the viewport/backbuffer from the physical
  window viewport extent.
- `SlateRHIRenderingPolicy.cpp` explicitly sets the Slate graphics target
  sample count to one. Slate does not depend on whole-window MSAA or scene TSR;
  its edge quality comes from output-resolution rasterization and analytic or
  asset-local filtering.
- `SlateShaderCommon.ush` evaluates rounded-box signed distance and derives the
  transition width with `fwidth`, then applies `smoothstep` coverage.
- `ElementBatcher.cpp` keeps rounded boxes as one analytic quad, passes local
  size plus four independent corner radii to the pixel shader, and leaves
  pixel snapping as an explicit per-widget policy instead of quantizing every
  moving or transformed control.
- The rounded-box shader derives a minimum screen-space outline width from
  `fwidth(pos)`, so a subpixel outline fades continuously instead of
  disappearing or becoming a binary one-pixel staircase.
- `SlateCore/Private/Rendering/SlateVectorGraphicsCache.cpp` keys vector raster
  resources by local size and draw scale, then rasterizes the cached pixel size.
  DPI therefore participates in the cached source resolution without requiring
  a supersampled whole-window backbuffer.
- `SlateElementPixelShader.usf` uses screen-space derivatives for SDF text and
  antialiased lines.
- `SlateElementPixelShader.usf` includes `GammaCorrectionCommon.ush`, keeps a
  source-linear permutation, and applies `GammaCorrect` at the display boundary;
  `SlateRHIRenderingPolicy.cpp` supplies the selected display gamma. Zircon must
  likewise keep coverage blending in linear light instead of treating sRGB UI
  token bytes as unmarked linear values.

The Windows product path is also process-DPI-aware before its first window is
created. The locked `winit` 0.31.0-beta.2 Win32 backend defaults event-loop DPI
awareness to enabled and requests `PER_MONITOR_AWARE_V2`, with V1 and legacy
fallbacks. Zircon does not call `with_dpi_aware(false)`, forwards
`ScaleFactorChanged`, and rebuilds the physical shell projection from the
window's current scale factor. Windows bitmap virtualization is therefore not
an accepted explanation for low-resolution output without contrary runtime
evidence.

Unreal's scene TSR/screen percentage is not the Slate UI raster contract. Slate
is composed at output resolution; density-independent layout, output-pixel
rasterization, analytic coverage, and filtered assets keep the UI sharp.

## MagicaVoxel Reference Contract

The local `dev/MagicaVoxel-0.99.7.2-win64/ui_reimplementation_guide.md`,
`config/ui/top.ui`, and `config/ui/editor.sty` provide a second, concrete
reference for editor interaction design. The useful parts are architectural,
not a request to copy its palette or private ABI:

- A persistent view tree owns stable identity, parent/child structure, bounds,
  visibility, and interaction state. The parser builds that tree from a small
  text DSL through a registered widget factory instead of hard-coding every
  screen in the parser.
- `HBox`/`VBox` use explicit measure and arrange phases. The measured size is
  derived from visible children and margins; arrange only writes final frames.
  This is the same boundary required by Zircon's retained `UiTree` and keeps
  layout from mutating editor business state.
- Interaction and paint are separate delegates. A control turns input into a
  command (`onClick = 'cmd undo'` in `top.ui`), while the painter resolves the
  current state and draws the result. Zircon keeps the equivalent separation in
  `.zui` event routes, template binding, retained dispatch, and primitive
  painters.
- Styles resolve state-specific overrides with `-h` (hover), `-s`
  (selected/pressed), and `-d` (disabled) suffixes, falling back to the base
  property. Zircon expresses the same contract through semantic state tokens
  such as `hover_background_color`, `pressed_background_color`,
  `selected_background_color`, and `disabled_background_color`; new product
  assets must use those tokens rather than per-node interaction colors.
- The reference top bar keeps a compact 32px band, 28px icon actions, explicit
  command widths, and a `hint` on every unfamiliar action. The editor style
  sheet uses a restrained gray hierarchy, a 6px ordinary rounded tier, larger
  panel radii only where the surface warrants it, and state contrast rather than
  decorative gradients. Zircon's 6/8/10/12 radius tokens and tooltip routes are
  the corresponding modernized implementation.
- 2D geometry is generated into a batch after state resolution. Zircon follows
  the same ownership rule with analytic rounded quads, local SVG supersampling,
  retained damage, and one publication-time hit-test authority; it does not
  supersample the entire editor surface or parse/rasterize on each pointer move.

A native reference capture was also taken from the bundled executable rather
than inferred only from its configuration. The 900x620 physical client image is
`.codex/state/ui12-reference/magica-voxel-900x620.png` (SHA-256
`A33D6679540B8F1B6CC6A286956260DE6A6F233C9DD62A596B6394132358D242`).
Its ordinary dropdowns and icon groups remain in the compact 24-30 pixel tier;
the enlarged 18x18 top-left dropdown crop at
`.codex/state/ui12-reference/magica-voxel-dropdown-corner-12x.png` preserves 33
distinct ARGB values. The pixel run contains intermediate edge and border
tones around an approximately six-pixel compact radius. This is direct evidence
for output-resolution coverage and stable one-pixel boundaries, not for a
whole-window supersampled or globally pill-shaped UI.

This mapping is now reflected in the Workbench assets: stable `control_id`
identities, explicit control-frame popup anchors, semantic state props, shared
measure/arrange containers, and source-bound event routes. The remaining product
proof is runtime visual capture at the three physical extents below; static
contract tests are supporting evidence, not a substitute for those screenshots.

## Zircon M6 Contract

1. The native presentation surface remains at least the physical client extent.
2. Layout and pointer input remain in the existing logical/device transform;
   quality scaling must not change control size or hit geometry.
3. Software rounded fills and borders use subpixel coverage and alpha blending,
   not binary center tests.
4. Software image scaling uses premultiplied-alpha-safe bilinear sampling.
5. SVG icons whose fitted physical content edge is at most 32 pixels use a
   bounded 4x local raster; larger vectors use 2x, and either path falls back
   to 1x before exceeding the 4096-pixel edge limit. The selected-axis resolve
   decodes sRGB samples, accumulates
   linear-premultiplied color and coverage, then returns straight sRGBA8 for
   the RHI admission boundary. This is selective supersampling, not a permanent
   4x whole-window memory and fill-rate tax.
6. WGPU rounded fills and borders use the original shape rectangle plus the
   clipped visible rectangle in a pixel-domain analytic SDF fragment path. Edge
   width comes from screen-space derivatives. Raising tessellation segment
   counts is not an accepted substitute.
7. The software presenter and deterministic CPU snapshot resolve only the
   rounded edge band with an 8x8 local coverage grid. Fully inside and fully
   outside pixels keep the signed-distance fast path, so the fallback gains
   1/64 coverage precision without supersampling the complete editor surface.
8. The global quality floor is 1.0 physical pixel per output pixel. Optional
   quality factors may only increase local render resolution and must stay
   bounded by the vector-raster edge limit.
9. RHI image payloads remain straight-alpha sRGBA8 at the producer boundary.
   WGPU decodes each admitted generation to linear light, premultiplies once,
   then stores the result in an sRGB texture encoding. Hardware sRGB sampling
   decodes before linear filtering, and the image fragment remains linear and
   premultiplied. Renderer-owned external textures retain their native texture
   transfer function and must explicitly declare `Opaque` or `Premultiplied`;
   byte-view overrides and post-filter premultiplication are forbidden because
   they respectively force gamma-space blending or square edge coverage.
10. Pixel snapping is an explicit primitive/widget choice. It may align static
   one-pixel dividers to the device grid, but it must not globally quantize
   rounded geometry, transforms, scrolling, or animation.
11. Vector raster identities include the final physical destination extent and
    tint; the bounded supersampled source extent is derived from that identity.
    Compiled WGPU batch plans use producer generation plus projection size: the
    producer must advance generation whenever commands or payloads change,
    including destination geometry, effective DPI, radius, or border width.
    Unversioned draw lists are deliberately ineligible for compiled-plan reuse.

## Acceptance

- Rounded fill and border pixel tests contain fractional edge coverage and keep
  opaque interiors/background exteriors.
- The software rounded-fill regression distinguishes 8x8 edge coverage from
  the former 4x4 quantization, and the native WGPU readback changes when the
  same rounded quad moves by 0.25 physical pixels.
- A 2x2 to 3x3 software image scale produces a blended center sample.
- SVG raster tests prove bounded adaptive 4x/2x sources while the destination
  frame stays unchanged, including 1x fallback at the raster edge limit.
- WGPU tests prove the analytic shader consumes rounded-box distance,
  `fwidth`, and coverage for both fill and border without clip-radius drift.
  The outer and inner border distances each derive their own pixel footprint
  with `fwidth`; a scaled thin border therefore does not reuse the outer edge's
  transition width for its inner edge.
- A real offscreen WGPU image-pipeline readback proves that filtering from a
  transparent texel to half/fully opaque blue yields premultiplied midpoint
  pixels `[0, 0, 64, 64]` and `[0, 0, 128, 128]`, with no transparent red
  bleed or second alpha multiplication.
- A real sRGB-target readback proves 50% white coverage encodes near sRGB 188,
  not gamma-space 128. Surface selection prefers BGRA/RGBA sRGB, solid token
  colors are decoded before premultiplication, glyphon uses `Accurate` mode on
  sRGB targets, and the explicit non-sRGB fallback uses glyphon's `Web` mode.
- Current-source editor screenshots at 640x520, 900x620, and 1672x941 must show no
  staircase corners, resize-snapshot interpolation blocks, clipped command labels, or
  placeholder icon boxes.
- The Workbench top-toolbar actions use a dedicated large-radius tier (10 logical
  pixels) across normal, hover, pressed, selected and focus-visible states.
  Ordinary fields and controls remain on the 8px control tier, while popup and
  panel surfaces retain 12px. The static toolbar-radius contract is green at
  3/3. The companion device-pixel contract is green at 18/18 and locks the native
  physical surface extent, one-time logical-to-physical projection, analytic
  WGPU coverage, bounded adaptive SVG raster, and softbuffer Fontdue text 8x local
  raster scale.
- Visual review uses the captured client-area PNG at 1:1 physical pixels as the
  authority. Enlarged crops may expose transition structure, but nearest-neighbor
  zoom pixels are not themselves evidence that the product rendered at a lower
  resolution. Native resize snapshots use a bilinear physical-pixel-center
  filter; they are a transient continuity aid and never replace a full-size
  repaint after the resize transaction ends.
- Radius review preserves the declared hierarchy instead of judging every shape
  against one number: compact rows and minor surfaces resolve the 6 logical-pixel
  tier, ordinary fields and buttons resolve 8, large controls resolve 10, and
  popup/panel shells resolve 12. At fractional DPI each tier is multiplied by the
  reported scale factor before rasterization; it must not be rounded back to the
  logical value or replaced globally by a pill radius.
- A passing rounded corner has a continuous, monotonic edge transition around all
  four quadrants with no flat polygon segment, missing corner pixel, or asymmetric
  one-pixel step. A one-logical-pixel border remains continuous around the same
  analytic contour and does not alternate between zero and two physical pixels.
- SVG and text inspection checks edge filtering separately from geometry: icons
  retain recognizable silhouettes without boxed placeholders or nearest-neighbor
  blocks, while glyph stems remain stable and legible without color fringes,
  clipped ascenders/descenders, or integer-DPI-only spacing.
- The MagicaVoxel reference is used for its compact hierarchy, restrained state
  contrast, and smoothly covered rounded controls, not as a literal palette or
  radius trace. Zircon must retain the editor token hierarchy and prove equal or
  better physical-pixel edge continuity in the three current-source captures.
- The product profiling geometry for the captured run reports
  `presenter_backend = "gpu"`; a silent softbuffer fallback is not WGPU visual
  evidence even when the desktop image looks nonblank.
- Windows client-area captures are the displayed-frame authority. The built-in
  first-presented-frame PNG remains useful as a deterministic CPU projection
  cross-check, but `HostWindowHandle::take_snapshot()` repaints the presentation
  in software and therefore cannot prove the WGPU framebuffer path by itself.
- Each product capture records `GetDpiForWindow(hwnd)` beside the client extent
  and the profile's effective scale factor. The reported physical client size
  must match the WGPU surface extent rather than a DPI-virtualized logical size.
- The three acceptance extents run in separate Editor processes. A
  profile-only initial client-size override is applied before native-window
  creation, and each process must complete its first GPU present with
  `window_client_size` equal to its requested 640x520, 900x620, or 1672x941
  physical extent before the desktop pixels are captured.
- A declarative snapping regression proves static one-pixel chrome remains
  aligned while a node with snapping disabled preserves a 0.25 physical-pixel
  transform at 125% and 150% DPI.
- `.zui` radius tiers are tuned only after the raster fixes are visible.

## Current Evidence

Status on 2026-08-26 is `implementation_in_progress_visual_acceptance_pending`.

- The softbuffer native-resize continuity path now samples the frozen frame at
  physical target pixel centers with bilinear, linear-light RGB interpolation;
  same-size copies remain direct. This removes the previous nearest-neighbor
  block artifacts during an in-progress resize without changing the final
  full-repaint authority.

- Managed `zr_rhi_wgpu` focused tests pass for the analytic vertex/shader ABI,
  fractional rounded geometry, clip preservation, retained-cache offscreen
  rendering, and disjoint rounded-quad batching. The batching regression was
  updated from the removed polygon-fan contract to assert two six-vertex
  analytic quads, no ordinary-instance fallback, and retained radius/border
  parameters.
- Native WGPU readback now covers both premultiplied fractional edge alpha and
  a quarter-physical-pixel translated rounded quad. Managed jobs
  `4b24fdee59da4ce5af3ff96d73f690a5` and
  `d2835f22545141cca42e0ce2d087afc0` both completed with exit code 0; the
  latter proves the aligned and 0.25-physical-pixel-shifted framebuffers differ
  while both retain fractional edge alpha.
- A native Windows lower-layer visual proof now exercises the real
  `WgpuUiSurfacePresenter` in three independent processes at exact 640x520,
  900x620, and 1672x941 physical client extents. Every run reported
  `UI12_VISUAL_BACKEND=wgpu-ui-surface` and a matching
  `UI12_VISUAL_READY size=<width>x<height> draws=19 commands=84
  backend=wgpu-ui-surface`; the captured client images are
  `.codex/state/ui12-wgpu-visual-proof-640x520.png`,
  `.codex/state/ui12-wgpu-visual-proof-900x620.png`, and
  `.codex/state/ui12-wgpu-visual-proof-1672x941.png`. All three report 96 DPI
  in the current validation environment and pass exact-size, sampled-color,
  luminance-range, and known WGPU-content pixel checks. Because 1672 physical
  pixels exceed the active desktop width, that image is assembled from
  non-overlapping client tiles captured after moving the same renderer-owned
  HWND; the complete surface contains no black off-screen tail or tile seam.
- The capture driver selects the window by the fixed Zircon proof title and
  process handle, hides the console host, and raises the render HWND before
  each client-area sample. This fail-closed policy was added after visual review
  rejected an earlier terminal-window capture even though its weak size and
  nonblank checks passed. A nearest-neighbor 8x inspection crop of the popup
  corner is recorded at
  `.codex/state/ui12-wgpu-visual-proof-popup-corner-8x.png`. Its 18x18 source
  region contains 20 distinct ARGB values and 40 non-flat transition samples
  among 324 pixels after known endpoint colors are excluded. At 1:1 the rounded
  buttons, panels, and popup edges are continuous, one-pixel borders remain
  stable, and text is legible; the enlarged crop exposes analytic pixel
  coverage rather than a hard binary or six-segment polygon edge.
- That executable is a validation-only frozen-snapshot harness which submits
  representative product UI primitives directly. It proves the lower WGPU
  raster, target-size, transfer-function and rounded-coverage path, but it does
  not load the complete current Editor `.zui` tree and is not accepted as any
  of the three product screenshots required below.
- The pre-color-space WGPU `ui_surface::` baseline was green at 116/116 through
  the Windows managed validator. Its 14/14 native-submission subset
  includes the two rounded-coverage regressions, premultiplied transparent-edge
  sampling, and the retained submission/presentation ordering contracts in one
  test binary. The broader matrix also covers batching, geometry, shader ABI,
  render passes, retained/image/shared caches, text, resize, damage, and
  presentation behavior. The new sRGB/linear-light hard cut has not inherited
  that acceptance and requires a fresh full matrix plus product screenshots.
- Static current-source implementation now prefers sRGB surface formats, routes
  solid and image fragments through target-specific linear/fallback entry
  points, stores CPU UI assets as sRGB-encoded linear-premultiplied texels,
  samples external products through their native texture views, selects
  glyphon `Accurate`/`Web` from the target transfer function, and counts sRGB
  retained copies. Tests pin the conversion (`50% blue -> encoded 188`), format
  selection, text mode, retained byte count, external-view hard cut, and a real
  sRGB 50% coverage readback. Rustfmt, diff checks, and source guards are green;
  dynamic WGPU execution remains pending. The product viewport export uses the
  scene renderer's `FINAL_COLOR_FORMAT = Rgba8UnormSrgb`, so its native sample
  view supplies the intended single hardware decode instead of the deleted
  byte-view override.
- The product WGPU text presenter does not consume the generic integer-sized
  glyph bucket inspected during the DPI audit. It passes already-scaled
  physical `f32` font size and line height directly into glyphon/Swash. A new
  `text_metrics_preserve_fractional_physical_sizes` boundary regression pins
  13.333333px and 16.666666px by their exact `f32` bits so future cache work
  cannot silently integerize product text. Rustfmt and source guards are green;
  its dynamic run waits for the shared managed Cargo lane.
- Visual-asset target tests now pin fractional physical extents: a 17x13 logical
  frame resolves to 22x17 at 125% and 26x20 at 150%, while oversized targets
  clamp only after rounding up to the 4096 physical-pixel edge limit. The
  raster cache key already separates the final width, height, and tint. An
  end-to-end vector regression now parses a circle SVG, rasterizes through the
  small-icon 4x source, resolves to the unchanged 9x9 physical target, and
  requires transparent, opaque, and fractional-alpha output pixels. The local
  box resolve now accumulates alpha-weighted linear color rather than averaging
  sRGB bytes; its opaque red/green/blue/white fixture resolves to encoded 188
  gray instead of the gamma-darkened 128 while transparent hidden RGB remains
  excluded. Non-square SVG content is now rasterized at its aspect-preserving
  physical content extent and centered into a transparent texture at the complete
  requested physical target extent. The uploaded texture and final WGPU frame
  therefore have identical dimensions, so the sampler cannot stretch a fitted
  100x50 source back across a 100x100 destination. The physical-size payload is
  derived from DPI-projected render bounds, and the final WGPU min/mag filters
  remain linear.
- The retained-host metric projection previously fed generic controls from the
  6px `small_radius` token even though the declared control tier is 8px. It now
  reads `control_radius`, keeps 6px available for genuinely small controls, and
  has a regression that gives the two tokens different values so the tiers
  cannot be silently collapsed again. The metric-scale regression also pins
  125% and 150% DPI without integer rounding: the 8px control radius becomes
  10px and 12px, while a 1px border remains 1.25px and 1.5px before analytic
  coverage.
- The legacy/base Editor theme remains maintained only for the component showcase
  and product-binding fixture. Its generic `Button.primary` and
  `Button.secondary` recipes previously collapsed ordinary buttons into the 6px
  `small` tier; they now use the 8px `control` tier while `.inset` and
  `.chrome-selected` remain compact. Asset Browser, Assets Activity, Project
  Overview, Welcome, Hierarchy, Inspector, and Console now import the strict
  Workbench theme. Hierarchy additionally consumes `WorkbenchSearchInput`; the
  shared projection maps its native `SearchField` root to the same
  `InputField`/`input-field` semantics and component-owned text path as the other
  text inputs. All seven Rust-loaded roots pass no supplemental legacy sources,
  so the file cache recursively resolves each root document's widget/style graph
  as the single dependency authority. The focused dynamic-view family/layout
  contract passes 8/8. The six live host stencils for the activity rail, dock
  header, menu bar, menu popup, page tabs, and status bar also import only the
  strict theme while retaining their existing node/control identities and
  absolute stencil geometry. Their semantic classes now select the shared
  Workbench rail/tab/popup/status recipes; numeric font sizes and positive radii
  below 6px are absent. Interactive rail, menu, popup-row, page-tab, dock-tab,
  and close stencils use a transparent idle surface plus the 6px compact radius;
  dynamic active rail/tabs switch to the inset surface, while hover and press
  retain the shared state colors. The retained painter now treats the authored
  `transparent` variant as zero alpha at idle instead of falling back to a panel
  fill, with a lower regression preserving hover/press feedback. The host
  projection continues to pass no supplemental style sources. If a chrome
  stencil is unavailable, its activity-rail, menu, page-tab, document/side-dock
  tab, close, and overflow controls now preserve the same transparent idle
  surface and shared 6px radius instead of reverting to an empty surface and
  zero radius; the activity rail also returns usable fallback nodes rather than
  an empty model. A lower regression covers the active/idle fallback styles. The top-level
  Workbench window authors initial boolean state only for its ten menu/overlay
  nodes, all false, leaving subsequent live state to the Rust bridge. The
  host-chrome contract passes 5/5 and the complete non-Cargo Editor `.zui`
  matrix passes 158/158. A tracked repository scan lists 317 `.zui` paths; all
  316 currently present files parse, with only the pre-existing removed
  `zircon_editor/assets/ui/editor/animation_editor.zui` absent. Product WGPU
  validation remains pending.
- A tracked current-source audit of 124 workbench `.zui` files finds 19 small,
  11 control, 2 large, and 4 panel radius-token references. The 22 zero-radius
  declarations are confined to continuous panel/root bands and viewport scene
  artwork. Live controls therefore retain the 6/8/10/12 hierarchy; M6 does not
  replace ordinary buttons with pills or globally inflate every shape.
- The strict workbench stylesheet no longer paints a full accent border for
  ordinary selected, open, or pressed buttons, rails, tabs, rows, segmented
  controls, or dropdowns. Persistent tabs now use the neutral selection fill;
  checkbox/radio/toggle marks retain their local accent identity, and the
  focus-visible outline still cascades after all primary-state recipes. A
  focused contract test pins that separation. The stylesheet parses as TOML,
  rustfmt and per-selector source guards pass, and product visual confirmation
  remains part of the pending current-source captures.
- The Editor bundle previously published only `zircon_runtime/assets`. Because
  product asset-root selection is stable once the adjacent `assets` directory
  exists, the resolver could not fall back per file to the 629 Editor assets;
  themes, `.zui` documents, and SVG icons would therefore be absent from the
  actual bundle. `build-editor.ps1` now merges the disjoint runtime and Editor
  asset trees into one product root, and its fixture requires one file from
  each source tree before publication succeeds. The complete build-script
  Pester suite is green at 17/17, including asset merging, explicit target
  forwarding, staged cleanup, root-bound publication, and junction/reparse
  rejection.
- The software fallback now uses boundary-only 8x8 local coverage. Its focused
  editor test is current-source ready. The foreign Editor01 E0507 has been
  repaired in the shared worktree, but the focused dynamic test remains behind
  the current full Editor compile and product-capture gates.
- Windows DPI awareness is not disabled by Zircon: the locked winit Win32
  backend requests per-monitor V2 before creating the event target, and the
  Editor consumes physical surface size plus `ScaleFactorChanged`. The capture
  gate still records `GetDpiForWindow` so this static fact is checked against
  the running product.
- Static review found a separate pre-WGPU contract gap: `UiPixelSnapping` and
  `UiLayoutMetrics` default to enabled, but compiled `.zui` nodes expose no
  per-node opt-out. The open Runtime74 handoff
  `failure-2026-08-23-runtime74-declarative-pixel-snapping-policy.md` requires
  a typed declarative policy; global snapping must not quantize moving or
  transformed content.
- The exact `zircon_runtime_interface` design-token test passes with the
  6/8/10/12 pixel small/control/large/panel radius hierarchy.
- Managed current-source `zircon_runtime` build job
  `86fb00309a674a86830939caf8fbed6f` passes. This supersedes the earlier
  editor attempt whose dependency graph still reported 27 shared runtime
  errors.
- The imported Editor01 failure
  `failure-2026-08-23-editor01-viewport-toolbar-cache-signature-move.md` is
  repaired in current source. The same-size remap branch now replaces hit
  control IDs in `cached.signature` in place and builds from
  `&cached.signature`, so it neither consumes nor clones the complete hot-path
  signature.
- UI12 coordination now uses successor session
  `editor-ui12-zui-aa-visual-acceptance-r5-21242973-20260823`; the coordinator
  captured base `4cc19615076c76c45f1fcdd587563fe5274ad8fd` while `main` was
  advancing. Managed Editor job `b2faf5f3a58e4708b7980c1b54f35f75`
  reached `zircon_editor` and returned exactly three errors. The prior asset
  visibility and native URI-borrow errors were absent. Cargo fingerprint
  evidence identified only two stale exports for deleted stateless menu
  wrappers and one missing explicit import for
  `HostAssetSurfaceInteractionState`. Current source now exports only the
  state-aware menu authorities and imports the canonical interaction state for
  the batched asset-surface writeback. Exact rustfmt, diff checks, and source
  guards pass.
- Managed current-source Editor check job
  `1cc68d7bb3704c2e9543f8f62920a435` superseded that three-error fingerprint.
  It reached `zircon_editor` and returned 14 errors while concurrent typed-state
  and browser-virtualization migrations were still landing. Current source had
  already resolved 13; the remaining `SceneModeId` diagnostic now formats both
  IDs through `as_str()` instead of requiring a new public `Display` contract.
- The first real `target-editor-host` product build job
  `97ce87826c5b4c2ba78a7edbde5d2c95` reached `zircon_editor` and reduced the
  current-source blocker set to two errors: an unconstrained reflected-update
  `collect()` and a moved inspector-dispatch error on the checkpoint-restore
  path. The collection now names `Result<Vec<_>, InspectorEditError>`, and
  rollback uses an explicit match so a successful restore returns the original
  typed error while a failed restore owns it inside
  `InspectorBindingRollback`. Exact rustfmt, diff checks, source guards, and the
  existing typed rollback regression agree with the repair.
- Managed product retry `4faab81839d749a0a744638d61593002` reached the current
  `zircon_editor` production target and returned exactly three errors. The
  reflected-update function assigned its input map to `updates` but returned it
  where `Vec<ReflectedInspectorUpdate>` was required; current source now keeps
  the borrowed input as `dynamic_fields` and assigns the world-query result to
  `updates`. The remaining two `E0308` diagnostics are owned by Runtime74's
  active typed binding-dispatch migration in `editor_event_dispatch.rs`: two
  branches have not yet applied the already-declared
  `From<EditorOperationDispatchError>` conversion. They are routed through
  `docs/plans/optimize/zircon_runtime/74/failure-2026-08-23-ui12-editor-event-binding-operation-error-conversion.md`.
  The build failed before publication, so the requested bundle directory does
  not exist and no compile or visual success is claimed.
- Managed product Job `77e9ab0281934ab4bc662d33b1c34a1b` exposed a lower
  validator defect before compiling workspace source. The low-space path first
  created managed `TEMP`/`TMP`/`CARGO_HOME`/`SCCACHE_DIR` inside the Cargo target
  and then ran `cargo clean --target-dir`, deleting the temporary directory used
  by `link.exe`; concurrent build scripts consequently failed with `LNK1104`.
  `validate-matrix.ps1` now recreates all three managed directories immediately
  after a successful clean and before the next Cargo stage. A focused Pester
  regression physically removes and restores them and passes 1/1; PowerShell
  parsing and `git diff --check` pass. The pre-existing validator suite reached
  86 passes, while 20 CLI dry-run cases were rejected by the concurrent
  Particles09 CPU-lane reservation rather than by the directory repair.
- The corrected managed product Job `42ac6d93102c4b399ad9286c3d8018ed`
  rebuilt dependencies and reached current `zircon_editor`, reducing the shared
  blocker set to seven Runtime74 typed-error migration diagnostics across
  `ui/host/mod.rs`, `editor_event_dispatch.rs`, `menu_action.rs`, and
  `asset_access.rs`. The updated Runtime74 handoff contains the exact lines and
  required public/error conversions. Job exit 101 was released with no live
  compiler process; atomic publication left
  `E:\ZirconBuilds\ui12-current-b9277856c5f2-srgb-r1-20260823` absent, so no
  partial product is accepted for screenshots.
- Managed product Job `faecda1f1a2444bc87f69138156cf38d` superseded that
  fingerprint after the shared runtime fixes landed. It reached the production
  `zircon_editor` target and returned six current-source errors: two reflected
  scene-write completion matches still expected `()` after the helper began
  returning `bool`; three typed binding-dispatch branches returned their inner
  error type without applying the existing `From` boundary; and `OpenView`
  returned `EditorError` directly from the typed menu-action match. Current
  source now discards only the reflected helper's completion flag with
  `Completed(_)` and uses `Ok(...?)` at each typed boundary. Rustfmt and diff
  checks are green for all three files, and the complete non-Cargo `.zui`
  contract matrix passes 36/36. A fresh managed product compile is still
  required; this static repair does not claim a published bundle or visual
  acceptance.
- The next isolated managed build targeted
  `E:\ZirconBuilds\ui12-current-471bb732e368-srgb-r2-20260823`. It compiled
  `zircon_runtime`, reached `zircon_editor`, and did not reproduce the prior six
  typed-dispatch/reflected-write errors. It then stopped on 11 newer shared
  Editor diagnostics: four visibility errors around the viewport-chrome damage
  re-export and seven primary/cascading diagnostics around project-sync access
  to poison-recovering asset-state helpers. Current source now scopes the two
  damage helpers to `host_contract` and the read/write state helpers to the
  editor-asset-manager `manager` module. Exact rustfmt and diff checks pass.
  The failed build published no bundle, so this is blocker repair rather than a
  compile or visual pass.
- Live Workbench navigation no longer authors duplicate business state in its
  `.zui` defaults. Module tabs, transform tool, activity rail,
  scene/inspector panel tabs, and the initial scene row are
  projected once by the retained bridge and subsequently updated through the
  same mutation authority. Run-mode and layout labels/checkmarks now follow the
  same rule: `.zui` retains only the neutral item catalog while Rust projects
  the initial Play In Editor and Default Layout indicators. Shared Save, Browse,
  Compile, Diff, and Simulate routes are momentary `invoke` commands rather than
  a synthetic mutually exclusive selection group; Compile keeps its primary
  identity through the authored filled/accent variant. The same momentary rule
  now covers core and extension panel actions such as Apply, Compile, Validate,
  Import, Simulate, Bake, and Preview: 50 command nodes no longer author live
  `selected`/`checked` state, and their Rust dispatch paths preserve feedback
  without mutating an exclusive selection group. The generated bottom-panel
  Open action follows the same contract. The four toolbar menus now also author
  zero hidden coordinates; their trigger-frame anchor is calculated and
  published before visibility changes on every open. A focused 5/5 structural
  regression rejects authored `selected`/`checked`/`value`/pressed state,
  authored menu indicators, non-momentary module-command dispatch, and nonzero
  toolbar-menu positions while the Rust interaction tests remain the dynamic
  acceptance owner. The same focused guard now runs 6/6 and covers the shared
  animation transport composite: Play/Loop no longer author persistent state
  in `.zui`; the retained bridge projects the initial Play, Pause, Record, and
  Loop values before the first frame and remains the owner of later commands.
- The visual radius hierarchy now agrees at both primitive defaults and the
  strict Workbench stylesheet: ordinary buttons/fields/dropdowns use the 8px
  control tier, floating popup/context/dropdown menus use the 12px panel tier,
  compact rows and tabs retain 6px, and structural chrome retains square
  boundaries. The complete `test_editor_zui_*contract.py` discovery passes
53/53 after these additions, and all 253 current Editor UI `.zui` assets parse
  as TOML in the 2026-08-26 live-worktree scan.
  These assets landed after the failed product snapshot and still require the
  requested GPU product screenshots.
- Keyboard focus is now a shared border-only recipe for buttons, icon buttons,
  rail buttons, tabs, segmented controls, fields, dropdowns, number fields,
  checkboxes, radios, toggles, sliders, tree rows, list rows, and table rows.
  It uses the focus ring token without replacing hover, pressed, or selected
  fills, so keyboard navigation remains visible while the visual state
  hierarchy stays stable.
- The product-reachability radius guard recursively follows imports from
  `workbench_window.zui`, rejects the legacy `editor_unreal_dark.zui` theme,
  pins the exact 6/8/10/12 token hierarchy, and rejects any nonzero radius below
  6px outside viewport scene decoration. Its focused owner file passes 18/18.
  This keeps low-radius orphan/reference templates and viewport artwork from being
  mistaken for the material control style used by the running Workbench.
 - The two-row toolbar's declared 66px height now has one exact source formula:
 34px command row + 4px shared gap + 28px dense module row. Eleven module tabs
   consume the same 28px dense-height token as the row, while the 30px compact
   command controls retain a 2px vertical breathing room inside the command row.
   The module tab strip now clips and scrolls horizontally while keeping the 34px
   More Modules action outside that clip, so the overflow trigger remains reachable
   at the 420/640px responsive tiers instead of being hidden at the end of the tab row.
   The Workbench skeleton now consumes the same toolbar, document-tab, and
   status-bar height tokens as the live window rather than repeating 66/32/24.
   A focused 12/12 layout contract pins the total budget, every child height, the
  shared entry chrome generation, and the viewport-first priority order:
  viewport > activity rail > left drawer > right drawer, with the main band
  above the bottom component drawer. The existing Rust vertical-alignment test
   also targets the live `WorkbenchToolbarAssets` control instead of the removed
   `WorkbenchToolbarNew` identity.
 - The Inspector's two panel tabs now keep `130px`/`120px` as wide-layout
   preferred widths but stretch down to an `88px` minimum. This fits the
   compact `196px` right drawer at the regular breakpoint without changing
   the existing tab routes or hit-control identities; the focused layout
   contract now includes this overflow guard.
  - Scene Tree instances now inherit `$editor.density.row_height` for all ten
    virtual rows. The previous one-off `30px` Props row is removed, so authored
    row geometry, virtual-row spacing, and pointer hit bands share one density
    authority.
  - Inspector content is now a `ScrollableBox` with `Auto` scrollbar policy and
    `Receive` input policy. Its existing sections and Change/Submit routes remain
    unchanged, but the fixed content extent no longer clips the lower controls at
    the minimum regular-height window.
  - Inspector Transform rows now use a content-driven `WrapBox`: each axis label
    and field stays together in a bounded `76px` group. Regular-width panels can
    wrap to two columns while the wide panel keeps three columns, eliminating
    the previous `222px` horizontal minimum that clipped values in a compact
    right drawer.
  - Viewport toolbar display controls now follow the same tier policy as the
    drawer budget: `Lit` remains available at every width, `Perspective` joins
    at Narrow, angle joins at Regular, and speed joins at Wide. The toolbar
    therefore requests 64/180/258/336px by tier instead of clipping a fixed
    336px row into the 420/640px viewport.
  - The four Workbench toolbar menus keep their trigger as a control-frame
    anchor, explicit bottom-start/bottom-end placement, collapsed initial state,
    and overlay z-index. The source contract now rejects a popup that falls back
    to a stale absolute coordinate or opens during initial surface publication.
  - The Workbench component drawer no longer puts seven fixed-width sample cards
    in one clipped horizontal row. Its body is a vertically scrollable surface;
    the top gallery uses a two-column MasonryBox and the table/feedback samples
    stack vertically. The largest card pair plus the shared gap fits inside the
    640px regular minimum window, so narrow regular windows expose complete cards
    through scrolling instead of losing controls at the row edge.
    These standalone cards now use the 12px panel radius tier; ordinary buttons
    and fields remain on the 8px control tier, preserving a visible hierarchy
    between contained controls and framed surfaces.
  - The scene tree keeps its search/filter row fixed while the virtualized tree
    rows live in a vertical `ScrollableBox` with an automatic scrollbar and
    pointer input policy. Growing hierarchies therefore scroll through the same
    retained row sequence instead of clipping at the drawer boundary.
  - Activity-rail buttons now use the compact 30px token inside the 34px rail;
    the primitive and legacy shell instances share the same hit target and
    small padding, eliminating the previous 48px child overflow into the dock.
 - The current Workbench import graph references 48 path-backed SVG assets.
  Every reference resolves to the Editor/runtime asset roots merged by the
  product build, and every resolved SVG declares a `viewBox`; no missing file
  or fixed-canvas SVG was found in this product-reachable set. Dynamic product
  screenshots must still prove that these sources rasterize instead of reaching
  a runtime placeholder.
- The same product-reachability scan rejects raw hexadecimal colors in page and
  component documents. Literal colors remain owned only by Editor tokens and
  the allowlisted viewport scene artwork; live buttons, fields, menus, panels,
  and module assets must consume semantic tokens/recipes instead of introducing
  local interaction colors. The 1,419-line mixed Workbench stylesheet is now a
  978-line strict shell theme plus a 457-line spatial extension. The strict
  theme owns 114 generic selectors and zero raw colors. The spatial theme
  imports strict first, then owns 56 non-overlapping transform/axis/viewport
  selectors and exactly 40 raw tokens named under `workbench_viewport_*` or
  `workbench_axis_*`. Strict now imports only `editor_tokens.zui`; the live
  Workbench graph no longer pulls in the independent 3,056-line Material/MUI
  showcase theme whose 642 selectors match no product-reachable Workbench node.
  A focused guard pins that ownership and import order; the complete
`test_editor_zui_*contract.py` discovery remains green at 53/53, all 253
  Editor UI `.zui` documents parse, and a new raw interaction color in the
  strict theme fails instead of silently expanding the scene-art exception.
  The 123-document product-reachable import graph now accepts only
  `editor_tokens`, `editor_workbench_strict`, and
  `editor_workbench_spatial`; composite-theme consumers no longer reimport the
  token stylesheet. The full 253-document structural audit reports 253 unique
  asset IDs, zero missing imports, and zero import cycles.
- The live componentized top toolbar still declares 31 event bindings, but its
  five registered Editor commands no longer use route aliases: Open Project,
  Save Project, Enter Play Mode, Exit Play Mode, and Open Asset Browser now
  author canonical `UiActionRef.action` identities. The explicit Workbench
  binders for Open, Save, and Play also emit `EditorCommand` payloads, while
  Stop resolves directly from its asset action. The remaining 26 toolbar routes
  are module/tool/menu navigation identities and are not mechanically renamed.
  The generic host fallback retains exactly two frozen project route aliases;
  its asset and shared template-binding files already contain another owner's
  changes and remain outside this batch. The Asset Browser declaration is
  canonical, but its shared typed `AssetSurface` runtime binding stays with the
  active Editor57 owner. Static action/bridge authority tests bring the complete
ZUI contract discovery to the 53/53 result above; the Rust governance and
  retained-dispatch tests still require the next managed Editor validation.
- The persistent Workbench status-bar icons now obey the same trigger contract:
  Snap reuses the existing `Tool/ToggleSnap` binding and Target reuses
  `ViewportToolbar/FrameSelection`, while the World-space globe remains a
  non-interactive indicator because no corresponding command route exists.
  This prevents a painted icon from advertising a click target without a
  dispatch path.
- The Blend Space validation log now has one authoritative interaction path:
  All/Errors/Warnings/Infos and Clear publish canonical extension actions from
  the `.zui` component, the Bridge keeps filter selection and diagnostic-row
  visibility synchronized, and the existing output row reports the resulting
  state. The focused static contract covers route uniqueness and the complete
  ZUI suite is green at 55/55; managed Editor compilation is still pending.
- The product-reachable component-drawer menu remains intentionally open as a
  component interaction sample, but it no longer authors the fixed
  `popup_anchor_x = 16` / `popup_anchor_y = 20` position. Its popup metadata now
  names `WorkbenchMenuTitle` as the control-frame owner, declares
  `bottom-start` placement, and uses the shared small-gap token. The absolute
  popup-anchor governance allowance for this live Workbench asset was deleted;
  the focused contract and the full 123-document reachable-graph guard prove
  that no live asset retains a nonzero numeric absolute popup anchor, so
  resize/DPI layout, popup projection, and hit testing consume the same trigger
  geometry while existing menu selection and close behavior remain intact. The
  same drawer now resolves its 103 nonzero container-gap, slot-padding, and
  `layout_gap` atoms through the shared `small`/`regular`/`medium` density roles;
  their 4/6/8 logical-pixel values are unchanged, but density policy no longer
  has local duplicate owners. The live toolbar's final private 2px module-tab
  gap likewise resolves through `editor.density.gap.xsmall`.
- The live notification center no longer combines a `(-24, 76)` authored frame
  with a separate zero-origin popup anchor. Before opening, its bridge now
  publishes one anchor from the current logical root width and the resolved
  top-toolbar bottom edge; resize republishes the same authority before the
  host projection refreshes. The 8px horizontal safe area produces anchor
  widths of 344 at 360 logical pixels and 1184 at 1200, while the popup's 4px
  placement gap resolves through `editor.density.gap.small`. The authored
  position and anchor values remain zero, so DPI projection scales the resolved
  geometry once instead of preserving a competing fixed desktop coordinate.
- The mounted Workbench layout now keeps the physical/logical boundary explicit:
  drawer budgeting consumes the logical size produced by the caller's
  normalized `scale_factor`, while responsive-tier visibility receives the
  original physical size plus that scale. Previously both passes silently used
  logical coordinates with a `1.0` scale, so a 2x physical surface could select
  the wrong drawer breakpoint. The existing scaled drawer regression and a
  static source contract now pin the conversion boundary.
The focused Python contract and complete 53/53 ZUI suite are green; the new
  Rust anchor tests are formatted and statically reviewed but remain pending
  the next managed Editor compile.
- The notification center's native-host painter previously flattened the
  component's 12px panel and 6px row tiers into one 8px control radius, while
  the Runtime painter already consumed the intended 12/6 hierarchy. The host
  projection now recognizes `panel_radius` as its generic corner-radius source,
  the native panel painter prefers that projected value, and its fallback
  metrics preserve the DPI-scalable `control + small gap` panel tier and
  `control - 2 * border` row tier. The focused Rust tests pin both the alias and
  12/6 native hierarchy; their execution remains pending the managed Editor
  compile, while the asset-level radius contract is included in the green
53/53 static suite.
- The command palette had the same split authority in its native-host path:
  `.zui` and the Runtime painter specified 12px panel and 6px compact-control
  tiers, while the native metrics independently derived 10/8/7 for the panel,
  search field, and result rows. Native projection now derives 12/6/6 from the
  same DPI-scalable host roles (`control + small gap` and
  `control - 2 * border`). The asset-level guard pins the panel/search token
  distinction and the focused Rust metric test pins the native projection;
  Rust execution remains pending the managed Editor compile.
- Confirmation dialogs now follow the same authority: their native-host shell
  previously ignored the projected 12px `.zui` radius and derived 10px from
  border width. It now prefers the node's projected radius and uses the
  DPI-scalable panel tier as fallback; action controls retain the ordinary 8px
  control tier. Focused Rust tests pin both fallback and node override.
- Popup menus previously painted a 12px `.zui` shell as 6px in Runtime and 8px
  in the native host because both paths reused row/control radii for the outer
  surface. Both outer-surface paths now use the 12px panel tier, while selected
  and hovered rows remain 6px. Both paths also prefer the projected
  `corner_radius`, so a `.zui` override remains authoritative instead of merely
  matching the default by coincidence. This preserves the intended hierarchy
  instead of globally inflating every control.
- Tooltip ownership is also converged. The shared painter-family contract
  already classified Tooltip as a panel-radius surface, but the live asset used
  6px and the native host used 8px. The live `.zui`, Runtime metadata path, and
  native fallback now resolve to 12px; native painting also honors a projected
  node override. Toasts retain their existing 12px asset/Runtime radius, and
  the native toast path now matches it. Their primitive now identifies as the
  catalog-backed `Snackbar` instead of `Alert`, so Runtime reaches its Toast
  state reducer, timer/layout policy, and painter just as the native host does
  through `WorkbenchToastRoot`. Alerts had the same 6px asset, 8px
  native, and 12px painter-family split; their live asset and both painter
  paths now use the panel tier and the host honors the projected node value.
  The focused radius contract is green at 18/18 and the full ZUI suite at
53/53. Added Rust metric/override tests are formatted and statically reviewed
  but still require the next managed Editor compile.
- The complete Editor `.zui` inventory contained 141 unique literal icon names.
  Before the 2026-08-26 audit, 37 names had no packaged vector candidate and
  could fall through to the native host's quad-built manual glyphs (or a generic
  placeholder) whenever development-only MUI modules were unavailable. Every
  literal name now resolves to an existing packaged SVG through the semantic
  alias table or its explicit asset path. The fail-closed Python contract scans
  every Editor `.zui`, including popup menu item metadata, and verifies both the
  mapping and the target file; the focused SVG/vector contract is green at 7/7.
  Dynamic icon parameters remain runtime data and are intentionally outside the
  literal-source inventory.
 - Tooltip arrows no longer round their size to an integer and build a diamond
   from one-pixel-high quads. The native host now submits two cached instances of
   a packaged diamond SVG for border and fill. They use the same physical target,
   bounded adaptive local supersampling, linear-light resolve, tint cache, clipping and
   retained image path as other vector UI assets; command count is constant
   instead of scaling with arrow height. A static guard rejects restoration of
   `HostPaintCommand::quad` in the tooltip-arrow owner.
 - Product-reachable fixed glyphs no longer silently fall back to logical-pixel
   segment grids when a packaged vector is unavailable. Inspector section-title
   Cube/Transform/Mesh, buttons/IconButtons, tree-row actions/disclosure/object
   icons, table-row actions, tooltip info, chip/dropdown chevrons, field steppers,
   list-row adornments, checkbox ticks, search glyphs, and Inspector row marks
   now submit their packaged SVG through the same physical-target raster path. The former
   grid modules were deleted; the only remaining quads in these paths are
   analytic control surfaces such as rounded slots and dividers. The focused
   packaged-vector contract is green at 17/17, and the new `field-stepper.svg`
   uses a stroked two-arrow viewBox rather than four rectangular glyph segments.
 - Alert and Toast status marks no longer use the retired 18px segment grid.
  Info, Success, Warning and Error now select monochrome line-art SVGs that are
  tinted from the resolved semantic color and rasterized at the physical target;
  the close action reuses the packaged vector close mark. The staircase Warning
  rows, dotted close diagonals, palette cutout dependency and their private
  segment modules were deleted rather than retained as a second painter.
 - Popup menu/option adornments and the three persistent Workbench status-bar
  icons now follow the same vector authority. Check, submenu, add, folder, save,
  trash, snap, world and target each emit one tinted cached image command instead
  of 2-7 small quads. The obsolete popup assets/segments/symbols hierarchy and
  status icon segment implementations were removed. Ordinary panel surfaces,
  dividers, tracks and analytic circles remain quad primitives by design; the
   audit does not misclassify those axis-aligned shapes as vector icons. The
   focused packaged-vector contract was green at 10/10 before the fixed-glyph
   cutover and is now green at 17/17.
  Chrome rendered the five new vector shapes at 18/36/72px into
  `.codex/state/ui12-vector-glyph-contact-sheet.png`; direct inspection found no
  clipping or broken strokes. Pixel checks on the five 18px cells found
  102-180 changed pixels per glyph and fractional edge colors on every shape,
  providing asset-level evidence without treating this contact sheet as an
  Editor product screenshot.
- A fresh managed Windows product build was submitted on 2026-08-25 with
  `\.\tools\build-editor.ps1 -Ephemeral`. The coordinator admitted target
  `F:\cargo-targets\zircon-engine\ephemeral\check\ba774352932c4e7daea94f508ab8064d`;
  the build reached current workspace crates and reported exactly one error,
  E0004 in the new Editor51-owned
  `project/engine_compatibility/directional_range.rs` caret-range match. No
  UI12 source appeared in the diagnostic set, but the normal `zircon_app`
  product path did not compile, so no bundle, Editor screenshot, or profile is
  claimed. The lowest shared cause is routed in
  [`failure-2026-08-25-engine-compatibility-caret-range-exhaustiveness.md`](../../../optimize/zircon_editor/51/failure-2026-08-25-engine-compatibility-caret-range-exhaustiveness.md).
- A broader managed `zr_rhi_wgpu --lib` audit compiled successfully and passed
  220/221 tests. The sole non-UI failure is a Runtime90 readback fixture that
  drops its only returned WGPU Queue before creating a command encoder, so
  wgpu-core panics before the intended capacity-overflow assertion. It is
  routed as `failure-2026-08-23-runtime90-readback-layout-test-queue-lifetime.md`;
  the UI12-owned native submission matrix remains independently green at
  14/14.
- The software rounded-coverage, bilinear-image, and SVG-resolve tests remain
  pending until the current Editor test binary compiles. The product capture
  driver is now fail-closed and process-isolated
  per extent: it requests the physical client size before window creation,
  hides the console-subsystem host and waits for a titled render window,
  enables first-present profile geometry export, rejects any run whose reported
  `presenter_backend` is not `gpu`, requires the exported
  `window_client_size` and Win32 client extent to equal the requested size,
  records the selected window title, `GetDpiForWindow`, and its scale factor,
  and redirects stdout/stderr beside the evidence. Client extents wider than
  the active virtual screen are captured as non-overlapping tiles from the same
  topmost renderer-owned HWND instead of accepting an off-screen black tail.
  Those current-source product screenshots have not been captured and no
  visual acceptance is claimed.
- A 2026-08-25 frozen-snapshot product build reached `zircon_runtime` and then
  stopped with 139 diagnostics spread across concurrent animation, ECS, text,
  render-graph, resource-streaming, platform and related migrations. The
  snapshot required one validation-only lifetime annotation in its untracked
  interface build helper; no corresponding main-worktree edit was made. This
  fingerprint explains why no Editor bundle was published, but it is not
  labelled a current-source fingerprint because the shared main worktree kept
  advancing during the build. The complete 640x520, 900x620 and 1672x941
  product captures therefore remain the acceptance blocker even though the
  independent lower-layer WGPU visual proof is now green.
- A fresh managed ephemeral product attempt started from main HEAD
  `8ee9411db24b7b4bdaf3fe028194642a7557c0b6` plus the current shared worktree
  overlay on 2026-08-25. It compiled `zircon_runtime_interface`, `zr_rhi`, and
  `zr_rhi_wgpu`, then stopped while compiling `zircon_runtime` with 134 errors.
  The current live-worktree fingerprint spans animation compiler exports, ECS
  schedule/removal/tick-policy convergence, text shaping outcomes, shared UI
  layout/state-reducer visibility and ownership, render-graph declarations,
  scene-renderer resource materialization, and platform registry ownership. It
  supersedes the older 27- and 7-error validation-copy reports, but is not an
  immutable snapshot because the shared overlay remains concurrently owned.
  The managed process exited, no Cargo/rustc/link process remained, and atomic
  publication left
  `E:\ZirconBuilds\ui12-current-8ee9411-20260825-r1` absent. Consequently no
  product screenshot or performance result is claimed from this attempt.
- Workbench command-bar density now keeps action meaning when the Ultra tier
  removes button text. `Save`, `Browse`, and `Compile` carry explicit tooltip
  metadata in the `.zui`; the shared Workbench tooltip projection resolves a
  non-empty explicit tooltip before its legacy icon-button `label` fallback,
  so ordinary `WorkbenchButton` commands are covered without pretending they
  are icon-button primitives. This follows Unreal's command-info separation of
  label and description and MagicaVoxel's compact action-plus-hint pattern.
  Runtime accessibility already consumes the same `tooltip` key after visible
  text, preserving a useful name when responsive density clears `text`. The
  focused static contract is green at 2/2; a 420px product pointer regression
  pins the 34px Compile command and `Compile Current Module` hover result but
  remains pending the next managed Editor test compile.
- Responsive command reachability now preserves one trigger authority as well
  as one visible meaning. When Ultra density collapses the direct Asset
  Browser, Open Project, and Save Project controls, the main menu resolves the
  same binding IDs used by the authored toolbar. Open and Save therefore retain
  the canonical `file.project.open` and `file.project.save` command-registry
  payloads instead of falling back to the older `workbench.project.*` menu
  routes; Play uses `runtime.play_mode.enter` at every layout tier. This follows
  Unreal's command-list model and MagicaVoxel's compact menu fallback: layout
  may move an action, but it must not fork its execution identity. Two stale
  Rust regressions were updated to the current hard-cutover contract, and the
  static authority guard now fails if either responsive test restores the old
  route payload. Rust execution remains pending the next managed Editor compile.
- The command bar no longer paints every secondary action as a persistent
  bordered tile. Nineteen menu, file, module, transform, run, layout, and theme
  actions now carry `workbench-toolbar-quiet-action`: their normal surface and
  border are transparent, hover/press supply a transient state layer, and
  checked/selected/open states retain a visible accent-backed surface. Compile
  deliberately omits the quiet class and remains the single continuously
  emphasized primary action. This follows Unreal's
  `NoBackgroundViewportToolbar` and raised-button conversion in
  `StarshipStyle.cpp`, whose normal brush is transparent, and MagicaVoxel's
  `bn-ic` recipe, which declares hover/selected treatment without a permanent
  icon-button fill. The change keeps the existing 10px toolbar radius in every
  state rather than returning to sharp boxes. The focused radius/hierarchy
  contract is green at 4/4; current-source product screenshots remain pending.
- The eleven module tabs in the command bar now use the same restrained visual
  grammar without changing the base tab component. Their default background
  and border are transparent; hover, press, checked, and selected states reveal
  a local 10px surface, with accent text/border reserved for persistent
  selection. Ordinary panel/document `WorkbenchTab` instances retain the 6px
  compact tier. This mirrors MagicaVoxel's `option`/`tab-panel` recipes, where
  text changes carry the default/hover distinction and selection supplies the
  stronger signal, while preserving Unreal's separation between toolbar and
  ordinary tab styling. The focused radius/hierarchy contract is now green at
  5/5; visual product acceptance remains pending the current-source Editor.
- Popup menus now preserve the intended shell/row hierarchy through the native
  painter as well as `.zui`: the panel remains on the 12px radius tier, compact
  rows remain on the 6px tier, idle rows are transparent, and hover/press or
  persistent selection alone supplies a fill. A focus-only row keeps that
  transparent interior and emits an independent rounded accent outline. The
  row command previously treated an absent background as permission to discard
  the whole surface, silently dropping this keyboard-focus outline; it now
  skips only when both fill and outline are absent. Separator geometry remains
  inset by the menu content padding rather than touching the panel corners. A
  focused static contract is green at 1/1, and unit plus offscreen pixel
  regressions are source-ready for the next managed Editor compile.
- Popup templates no longer author an interaction that has not happened. The
  dropdown and context-menu primitives clear focused/hovered row collections,
  hovered IDs, and pending/open submenu IDs; both primitives and the live
  Workbench context-menu instance start with `focused_index = -1`. Selection
  and disabled-option data remain semantic inputs. Runtime directional
  navigation now treats an out-of-range focus as no focus before any clamp:
  Next/First enters at the first enabled row and Previous/Last at the final
  enabled row. This agrees with the Editor native popup keyboard target, which
  already uses the same edge-entry rule for open product menus. Focused static
  ownership guards are green at 2/2; reducer behavior tests are source-ready
  for the next managed Runtime compile.
- Native synthesized popup rows now consume the authored
  `layout_padding_left/right/top/bottom` and `layout_spacing` metadata instead
  of dividing the complete rounded panel into adjacent rows. Workbench menus
  therefore retain an 8px horizontal inset, a 4px vertical inset, and 4px gaps:
  the 12px panel shell remains visible around 6px row fills. Runtime rendering
  reads these values from component metadata; the Editor host projects and
  physical-scale converts them once, then paint, pointer hit testing, and
  keyboard targeting share the same O(1) row-frame calculation. Padding and
  gaps remain blocked popup interior rather than resolving to a neighboring
  row. When placement bounds compress a popup, arranged rows derive their
  height from that final frame; the 24px minimum remains a desired-size input
  and cannot push painted or interactive rows outside the panel. The focused
  popup visual contract is green at 3/3; geometry and
  offscreen pixel regressions are source-ready, while current-source product
  capture remains pending.
- Workbench menus now separate display copy from command identity. Product
  `.zui` rows declare a stable `action=menu.item.*` flag, and both the Runtime
  painter/state resolver and Editor event projection prefer that value while
  retaining the first segment as the visible label. Toolbar menus, the generic
  popup primitive, its component-drawer instance, dynamic asset and responsive
  module-overflow rebuilds, and scene/module/generic context-menu providers all
  preserve explicit IDs. Localization or wording changes therefore cannot
  silently change click dispatch, checked state, hover state, or keyboard
  activation. Label-derived IDs remain only as a compatibility fallback for
  generic un-migrated menu assets and focused legacy fixtures. This follows
  Unreal's separation of command identity from presentation while retaining
  MagicaVoxel's compact label-oriented menu surface. The focused identity
  contract is green at 4/4; Runtime and Editor behavior tests are source-ready
  for the next managed compile.
- Editor popup adornments now forward authored `icon=` semantic names directly
  to the shared SVG asset resolver instead of filtering them through a closed
  six-icon enum. Checked rows and submenu disclosure retain explicit state
  adornments, while product icons such as edit, copy, pin, grid, play, search,
  target, and reset share the same physical-size SVG raster/cache path as other
  native-host icons. Copy has a packaged 24x24 vector asset; copy, pin, and
  rotate-counterclockwise have stable aliases. This removes the prior silent
  disappearance of valid `.zui` menu icons without adding manual low-resolution
  glyph geometry. The focused icon contract is green at 2/2 and the complete
  Editor `.zui` contract matrix is green at 71/71; current-source product
  capture remains pending.
- Native template popup rows now publish one shared label/shortcut column
  geometry per row. The trailing boundary first reserves the optional SVG
  adornment slot; the shortcut then uses Runtime glyph measurement and aligns
  to that boundary; the label receives the remaining width after the shared
  20px menu-column gap. This replaces the independent 58%/38% split that could
  overlap a long label, shortcut, and right-side icon. Narrow rows collapse the
  label column before allowing overlap, and option rows consume the same
  geometry without creating an empty shortcut column. The focused column
  authority contract is green at 2/2; Rust geometry regressions are formatted
  and source-ready for the next managed Editor compile.
- Menu shortcut text now shares the effective keymap authority used by keyboard
  dispatch. The manager's override-aware `EditorKeymapService` snapshot is
  carried by `WorkbenchViewModel`; command-registry menu items replace default
  descriptor chords with that snapshot before extension items are appended.
  The componentized toolbar main menu projects Open, Save, and Command Palette
  from the same snapshot rather than hard-coding default strings. Its retained
  cache key includes only those three displayed chords plus the asset-menu
  generation, so stable frames remain O(1), while a rebind or explicit unbind
  republishes once and immediately changes or removes the visible hint. The
  focused effective-shortcut contract is green at 2/2 and the complete Editor
  `.zui` contract matrix is green at 75/75; behavioral Rust regressions are
  source-ready for the next managed Editor compile.
- Product popup leaf commands no longer advertise nonexistent child menus.
  Command Palette uses the search SVG, disabled Network Preview uses the route
  SVG, and More Tools uses the overflow SVG; none sets `submenu` merely to
  obtain a chevron. The dynamic main-menu projection and component defaults
  preserve the same semantic flags. The low-level chevron state remains
  available for a future menu-item model that publishes actual children, but a
  direct command cannot enter that visual or interaction branch. The focused
  popup icon/leaf contract is green at 3/3 and the complete Editor `.zui`
  contract matrix is green at 76/76.
- The componentized Workbench main menu no longer forces every projected row
  into its authored 190px fallback width. Asset-generation or effective-keymap
  identity changes build the raw rows once, measure the longest label and
  shortcut with the Runtime text authority, reserve the optional trailing SVG
  adornment column, and cache the resulting intrinsic logical width. Stable
  resize frames only clamp that cached value to the current logical shell and
  update fixed width/height constraints in O(1); they do not enumerate asset
  rows or remeasure glyphs. The authored `.zui` width remains the minimum, and
  physical text/adornment measurements are normalized by the presentation
  scale before entering logical layout. The focused measured-width contract is
  green at 2/2, the complete Editor `.zui` contract matrix is green at 78/78,
  and all 316 present tracked `.zui` files parse successfully. The one missing
  tracked file remains the externally deleted `animation_editor.zui`; it was
  not restored. Product visual capture remains pending.
- Runtime Workbench context menus now use the same structured-row measurement
  authority as the componentized main menu. Each open request measures its
  actual labels and shortcuts, ignores separators, reserves semantic trailing
  adornments, and publishes content-derived fixed width/height constraints
  before the refreshed overlay frame is built. The 220px minimum remains an
  authored `layout_min_width` property in `.zui`, so repeated context opens can
  grow or shrink without treating the prior dynamic width as a new minimum;
  width and height clamp to the current logical shell, with physical text and
  icon metrics normalized by presentation scale. The focused measured-popup
  contract is green at 3/3 and the complete Editor `.zui` contract matrix is
  green at 79/79. Product visual capture remains pending.

- Component Lab focus and disabled samples no longer impersonate retained
  interaction state with `.workbench-focused` / `.workbench-disabled` classes.
  The focus sample authors the standard `focus_visible` fixture state, disabled
  field/list samples retain `disabled=true`, and the strict theme uses
  `:focus-visible` / `:disabled` selectors exclusively. This preserves the
  component-state preview while keeping keyboard focus visibility and disabled
  styling on the same runtime selector authority used by product controls. The
  current Editor `.zui` Python matrix is green at 96/96, focused source guards
  and rustfmt pass, and the Rust theme regression is source-ready; managed Rust
  execution and current-source product capture remain pending.
- Compact icon actions now share one hover-hint resolver. Explicit `tooltip`
  metadata wins, while both `workbench-icon-button` and
  `workbench-rail-button` may reuse a non-default authored label. The six
  activity-rail actions therefore expose their existing semantic labels without
  duplicating command or display identity, and the non-icon-class Delete sample
  declares its tooltip explicitly. The current product-reachable static audit
  reports zero unresolved icon-only Click hints; the Editor `.zui` matrix is
  green at 119/119, the companion static suffix/presentation matrix at 22/22,
  and all 253 current Editor UI declarations parse.
- Passive Workbench scroll containers now use the narrow interaction contract:
  `input_hoverable = true` plus layout `input_policy = "Receive"`. The HUD
  extension strip, Gameplay Effect canvas, and module-tab strip no longer set
  broad `input_interactive`, which would also infer clickable and focusable
  state. All six writable Workbench ScrollableBox declarations now receive
  pointer/scroll routing without becoming synthetic Click targets or Tab stops;
  the focused layout contracts are green at 23/23.
- Non-portal Workbench overlays now shrink within the Ultra surface instead of
  relying on desktop-sized fixed minima. The root Overlay supplies tokenized
  safe margins to Preferences and toast; Preferences uses stretch constraints
  with zero minima, while standard Runtime popups retain their existing
  anchor/flip/clamp authority. Retained category/content scroll offsets are
  projected into `TemplatePaneNodeData` and atomically updated by one host
  callback. One `SettingsWindowLayout` owns both row projections, pointer hits,
  wheel clamping, list clips, and token-sized scrollbars, so plugin-contributed
  categories remain reachable when the sidebar overflows. Its category sidebar contracts to roughly 120px
  at Ultra, while the shared layout selects 52/62/72-percent value-column starts
  by available width so resettable controls retain a usable budget. Partially
  scrolled rows clip to the settings viewport; enum/color editors remain
  window-clipped overlays. The current
  Editor `.zui` matrix is green at 121/121, all 253 declarations parse, and the
  touched Rust source passes rustfmt. Cargo execution and WGPU visual capture
  remain pending.
- A 2026-08-27 compatibility capture used the existing 2026-08-10 profiling
  executable with current staged assets to inspect authored layout through the
  real Windows/WGPU first-present path. It exposed the module Save label as
  `Sa...`; Save and Browse are now 34px icon actions with explicit accessible
  labels/tooltips, Compile remains the labeled primary command, and the module
  group budget contracts from 388px to 300px. The second capture confirms that
  the truncation is gone. The welcome sidebar subtitle is now the scannable
  `Recent projects`; equal stretch margins center its max-1000px recent/main
  content block on wide windows, with a third capture confirming symmetric
  margins and no column clipping. Additional 900x620 and 640x520 physical-client
  captures keep both welcome columns inside the window. The current Rust toolbar
  projection also keeps Save/Browse at 34px icon-only in every layout tier while
  Compile remains labeled, so a rebuild cannot restore the clipped text buttons.
  The Welcome asset now removes its page-local showcase/material state recipes,
  imports only the strict Workbench theme, and uses the shared `WorkbenchButton`
  and `WorkbenchField` primitives for all six actions and both fields without
  changing control IDs or routes. The component nodes now own the fixed 80px and
  118px action widths directly instead of relying on parent-slot overrides, so
  component expansion cannot stretch both actions to equal width. A 1440x900
  compatibility capture confirms that the shared shell, primary/secondary
  action, and field recipes still render with the two-column layout intact. The
  old executable leaves the disabled `Open` label unpainted even though the 80px
  button frame remains; current source retains `text = "Open"`, provides an
  explicit disabled foreground token in `WorkbenchButton`, and only patches the
  disabled flag and route in Rust, so this is recorded as a legacy component-
  expansion difference rather than a current asset regression. The focused
  Welcome and complete Editor `.zui` matrices are green at 7/7 and 130/130, all
  253 Editor declarations parse,
  and the touched Rust source passes rustfmt. These images are layout and shared
  style-wiring evidence only: the old binary does
  not contain the current rounded-SDF shader or Rust interaction projection and
  cannot satisfy current-source AA, popup, or trigger acceptance. Its empty
  toolbar band after resize and raw `page:*` host-tab labels are recorded as
  legacy-host differences: current breakpoint contracts keep command groups
  reachable, and the current title projection consumes `MainPageSnapshot.title`;
  a welcome view-model regression separately asserts the internal page ID and
  visible `Welcome` title.
- Exact 96-DPI compatibility recaptures then exposed a separate short-height
  defect that the earlier boundary-only review missed: the startup chooser row
  was absent at 640x520 and 900x620, and its four original long labels truncated
  after the row became reachable. Eight non-interactive welcome regions now keep
  their preferred/max height but allow a zero minimum, so the vertical allocator
  collapses hero/status/preview decoration before fields and actions. The visible
  labels are `Default`, `Showcase`, `Assets`, and `UI Layout`; full names remain in
  tooltips and all control IDs/routes are unchanged. The final
  [640x520](../../../../../artifacts/ui12-reference/zircon-legacy-current-assets-640x520-height-priority.png)
  and [900x620](../../../../../artifacts/ui12-reference/zircon-legacy-current-assets-900x620-height-priority.png)
  client captures show all four labels without ellipsis, overlap, or action loss.
  The focused Welcome matrix is 7/7, the complete Editor `.zui` Python matrix is
  130/130, and all 318 currently present repository `.zui` files parse. These two
  images remain legacy-executable asset-layout evidence only, not current-source
  WGPU AA or interaction acceptance.
- The zero-minimum regions also exposed an authority split below the asset: frame
  capture represents a fully collapsed node as absent, while the retained painter
  previously interpreted absence as permission to synthesize legacy fallback
  geometry. `resolve_welcome_frame(...)` now accepts the explicit asset-layout
  authority state. Missing current-layout frames resolve to an invisible frame;
  fallback remains available only when no asset layout exists, and each of the
  five main-column semantic painters rejects an invisible frame before emitting
  commands. The focused static regression completed RED then 7/7 GREEN, the Rust
  regression is source-ready, and all touched Rust files pass rustfmt. No Cargo or
  current-source visual success is claimed for this correction.
- The product Welcome asset already instantiates `WorkbenchField` and
  `WorkbenchButton`, leaves `corner_radius` unowned at page scope, and inherits
  `$editor.control.radius.control` from both shared component roots. A stale Rust
  bootstrap regression still expected native `TextField`/`Button` nodes and a
  copied `4.0` radius. It now asserts the shared component identities, rejects a
  page-local radius, and follows the token reference instead of copying the current
  `8.0` value. The focused Welcome matrix is 8/8, the radius matrices are 25/25,
  the complete Editor `.zui` Python matrix is 131/131, and all 318 currently
  present repository `.zui` files parse. Rustfmt and scoped diff checks pass; no
  production geometry, events, or painter code changed, and no Cargo or
  current-source visual success is claimed for this correction.
- A task-first density pass removes the redundant steady-state `Open or Create`
  hero and `Ready` presentation budget from Welcome. The four associated spacer
  nodes now have exact zero height; the 52px project preview follows the location
  field before validation and actions. A subsequent wide-screen review moves the
  four startup targets directly after the project actions with a shrinkable
  `$editor.density.gap.large`; the flexible fill now follows the startup row and
  absorbs only the space below the complete task cluster. Control IDs, routes,
  focus order, and business-command semantics are unchanged. An isolated legacy product bundle
  consumed the current assets after five unsupported Blend Space validation
  callbacks were removed from the temporary copy only. Its
  [1672x941](../../../../../artifacts/ui12-reference/zircon-legacy-current-assets-welcome-task-first-1672x941.png)
  and exact-client
  [640x520](../../../../../artifacts/ui12-reference/zircon-legacy-current-assets-welcome-task-first-exact-640x520.png)
  captures show the task order and retain both project actions plus all four
  startup targets without overlap or clipping in the earlier bottom-anchored
  arrangement. Their SHA-256 values are
  `45ED544CA732F850239A423621295EAA858E417E2A457B01EE7FB9DD67AEC683` and
  `53E9AB0AF8F63B82FA84B5C4C3EF4B07110F45A06841235C0449B58451DA3D43`.
  The focused Welcome matrix is green at 9/9, the complete Editor `.zui` matrix
  is 132/132, the device-pixel/AA guard is 10/10, and all 318 repository `.zui`
  files parse. The old executable's missing disabled `Open` label remains a known
  legacy component-expansion difference. These captures predate the task-cluster
  correction, prove only legacy asset-layout compatibility, and do not satisfy
  current-source WGPU AA, popup, layout, or input acceptance. The current static
  closure after the task-cluster correction is Welcome 9/9, the complete Editor
  `.zui` Python matrix 178/178, device-pixel/AA 11/11, Tooltip 5/5, and all 318
  currently present repository `.zui` documents parsed. The permanent scan-only
  UI Asset Editor workspace retains SHA-256
  `5DEFDAD04BE2837AF3CD012C63D6801D0136F8B1CAC36B492E6799C921381F09`.
- A current-worktree build snapshot generated its own consistent offline lock
  without modifying the shared `Cargo.lock`, but the managed coordinator
  rejected the snapshot repository key. A subsequent main-root managed build
  stopped before Cargo/rustc on `unmanaged_artifacts_detected`, including the
  stale `D:\ZirconBuilds\tooling15-wave105-runtime-20260826-231249`
  reservation. No foreign reservation or artifact was released, and unmanaged
  Cargo was not used.
- The 2026-08-26 managed product-build attempt reached Cargo with target
  `F:\cargo-targets\zircon-engine\ephemeral\check\eabc667e94744fe5b7d1e052356b17e0`
  but stopped before rustc because `--locked` detected that the shared manifests
  and current `Cargo.lock` still require convergence. This is neither Editor
  compile evidence nor a UI source diagnostic. The shared lock/manifests were
  not overwritten, and no product executable, WGPU screenshot, or profile was
  produced.
- The 2026-08-27 current-source resolution audit closes the static extent chain.
  Winit `Window::surface_size()` publishes an integer physical client extent to
  `HostContractState`; the GPU presenter passes that same extent through the
  chrome command stream and `UiSurfaceDrawList`; WGPU configures the swapchain
  at the same width and height. Logical layout is converted once through the
  DPI projection with floating-point frame, font, radius, and border metrics.
  There is no steady-state low-resolution editor texture that is then stretched
  to the window. The native-resize transaction may temporarily crop/copy its
  exact-texel frozen projection, but it marks the retained baseline invalid and
  the next ordinary present rebuilds at the new physical extent. Per-monitor
  `ScaleFactorChanged` updates the retained scale before entering that same
  resize transaction. A translated surface resize contributes only its physical
  extent to the presenter, and the next redraw commits a full interactive frame
  update after applying the presenter resize, so a monitor transition cannot end
  on the frozen projection or a stale logical scale.
- Both product backends now have an explicit static quality guard. WGPU rounded
  fills and borders use physical-pixel rounded-box SDF coverage with independent
  outer/inner `fwidth`, premultiplied alpha, and surface-format-aware sRGB/linear
  conversion. The softbuffer fallback renders directly at `surface_size()` and
  evaluates only boundary pixels with an 8x8 local signed-distance coverage
  grid. SVGs rasterize into a bounded adaptive 4x/2x local source, resolve alpha-aware
  premultiplied colors in linear light, and publish a texture whose dimensions
  equal the requested physical target. Its raster cache key includes the
  upward-quantized physical width, height, and tint, so a high-DPI request
  cannot reuse a smaller cached icon. Retained text consumes the already
  device-projected font size: Swash rasterizes and hints at that native pixel
  size, and glyphon submits it at scale 1.0 against the physical viewport rather
  than enlarging a lower-resolution text surface. Only the softbuffer Fontdue
  fallback performs bounded 8x local coverage sampling. The separate Runtime
  scene glyph atlas keeps a 2px zero gutter and uses fixed-mip linear sampling;
  glyphon 0.11 retains its intentional fixed-mip nearest sampler for the
  integer-positioned, native-size Editor glyph coverage atlas. This distinction
  must remain explicit until the product WGPU path owns a padded configurable
  sampler and proves it visually. Avatar image rounding multiplies source alpha by an 8x8
   local rounded-box coverage only in the boundary band and invalidates the
   unmasked atlas fast path after pixel mutation. Cached MUI-X chart bitmaps use
   bounded 4x4 local samples for discs, polylines, arcs, pie contours, and slice
   boundaries; their resolve averages premultiplied colors in linear light before
   returning straight sRGBA8. The final-size CircularProgress raster now caches
   analytic annulus coverage with its topology and scales only straight-alpha
   output at the inner and outer silhouette. It also caches the angular arc-length
   derivative and resolves start/end caps with alpha-weighted linear-light
   track/fill mixing instead of a binary color boundary. A 24px source-ready pixel
   regression requires transparent, opaque, and fractional alpha plus an opaque
   mixed-color endpoint; an independent 37.5% calculation finds 108 fractional
   silhouette pixels and two opaque endpoint-mix pixels. The missing-icon fallback
   no longer emits integer-pixel diagonals: it resolves 4x4 local samples at or
   below 32px and 2x2 samples above that threshold without allocating an enlarged
   bitmap; conservative sparse-stroke rejection avoids sub-sampling transparent
   interior pixels. Sample-grid points and timeline keys no longer approximate
   diamonds with `2r+1` binary one-pixel quads. Both consume one cached 4x4
   locally-resolved diamond image command per layer; a radius-three raster contains
   24 fractional pixels while reducing its outer shape from seven commands to one.
   The image frame is positioned by half of its odd raster edge, rather than its
   integer radius, so the raster sample centre stays on the authored point instead
   of shifting the complete glyph down and right by half a device pixel.
   Product `template_*_glyphs` renderers may not
   submit manual paint primitives except for the analytic status-signal circle;
   representative Alert, checkbox, tree-disclosure, and Tooltip glyphs are
   guarded on the SVG asset path. `test_editor_ui_device_pixel_aa_contract` is
   green at 30/30 for these cross-layer invariants. The new Rust pixel regressions
   remain source-ready until a managed current-source Editor compile runs. This is source evidence, not
   a substitute for the pending current-source WGPU product captures.
- The 2026-08-27 palette correction maps the authored and first-frame Runtime
  surface ladder to Unreal Slate's neutral `Background/Panel/Header/Dropdown`
  roles (`#151515/#242424/#2F2F2F/#383838`) and reserves blue for
  selection/accent/focus. The retained Primary button state owner now pairs
  accent/focus fills with the shell-background inverse foreground
  (7.82:1/8.15:1) and
  pairs its dark selected pressed fill with primary text (8.86:1), matching the
  existing Primary icon-command projection. A focused 3/3 contract locks the
  neutral role ladder, Runtime parity, contrast thresholds, state-source
  projection, and Welcome's consumption of the central palette. Legacy-exe
  captures exposed the former dark blue bias but cannot validate these new
  values; current-source WGPU captures must still verify luminance separation,
  state contrast, rounded-edge coverage, and text clarity together.
- The post-contrast static pass is green at 178/178 editor `.zui` Python tests
  and 318/318 TOML parses for files present in the worktree. The index still
  names one externally deleted `animation_editor.zui`; it was not restored or
  counted as a parse failure. Rustfmt and scoped diff checks pass, and the
  permanent scan-only UI Asset Editor workspace retains SHA-256
  `5DEFDAD04BE2837AF3CD012C63D6801D0136F8B1CAC36B492E6799C921381F09`.
  No Cargo, WGPU capture, or product profile was run for this static pass.
- A single 2026-08-27 managed Editor build retry was rejected before Cargo and
  rustc with `unmanaged_artifacts_detected`; the reported path is
  `D:\ZirconBuilds\tooling15-wave131-runtime-20260827-042112`. No current-source
  compiler fingerprint, executable, screenshot, or profile was produced. The
  external artifact was not deleted and the coordinator is not being polled.
- The next single managed retry after the Core/extension live-state cutover was
  also rejected before Cargo/rustc. Its current reservation is
  `D:\ZirconBuilds\tooling15-wave139-runtime-20260827-062048`, and the gate also
  reported `F:\cargo-targets\zircon-engine\ephemeral` as unmanaged. This attempt
  produced no rustc fingerprint or executable; neither external path was
  deleted or released, and no coordinator polling was started.
- The 2026-08-27 dynamic-product style audit closed two retained projection
  bypasses in Asset Browser. The list-mode summary type badge no longer authors
  a `3.0` radius and now consumes `EditorControlTokens::workbench_dense().small_radius`,
  matching the thumbnail and activity-list badge family. Table-name compaction
  no longer measures text with a copied `10.0` size; it uses
  `EditorTypographyTokens::WORKBENCH_CAPTION_SIZE`, the same authority authored
  by `WorkbenchTableRow`. Source guards reject any new nonzero numeric radius
  below 6 px or numeric `FONT_SIZE` constant in the product view projection
  layer. The focused product-view matrix is green at 10/10, the complete Editor
  `.zui` Python matrix is green at 160/160, and the current DPI/SVG/device-pixel/
  physical-sampling matrix is green at 31/31. Direct rustfmt, Python compilation,
  scoped diff checks, and the permanent scan-only UI Asset Editor hash are green.
  These remain static/source checks; current-source Editor compilation, WGPU
  captures, pixel inspection, and the 1000/1000/200 product profile are still
  required for M6 acceptance.

## Performance Guardrails

- Do not supersample the complete editor surface by default.
- Keep rounded coverage work inside rounded primitive bounds.
- Cache final SVG pixels by the quantized physical destination extent and tint;
  derive the bounded supersampled source as a transient raster target.
- Preserve retained damage rendering and command generation caches.
- Record physical presented pixels separately from local supersampled source
  pixels so profiling does not misreport the swapchain extent.
- Follow
  `docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md`:
  popup geometry and hit testing keep one publication authority, event-time
  input remains read-only/local, and steady-state SVG interaction performs no
  parse, raster, or upload work.
- Product profiling must complete 1000 source-bound clicks, 1000 pointer moves,
  and 200 resize steps, reporting CPU core/system utilization, RSS/private
  memory, input-to-damage, and damage-to-submit distributions. A 999/1000 run
  is incomplete evidence rather than a pass.
- The current-source profiling/capture Pester matrix is green at 114/114. It
  validates fail-closed counters, GPU timestamps, presenter, cache churn,
  surface publication, resize, process CPU/memory, latency, scale fixtures,
  and source/binary fingerprint gates. The source-manifest regression now locks
  the exact 16-file capture-tool closure rather than a stale count of 11; every
  listed tool exists and is hashed into product evidence. Two earlier stale harness tests were repaired
  without relaxing thresholds: resize counter assertions now follow the
  extracted semantic-evidence helper, and the process-budget fixture uses the
  same `PSCustomObject` shape as deserialized product evidence.
- The current `.zui` suffix/wording, documentation authority, fixture boundary,
  and physical UI-asset cache Python matrix is green at 45/45. Directory-backed
  test owners are scanned recursively so module splits cannot silently disable
  the guards. These static suites validate the harness and source governance;
  they do not replace the pending 1000/1000/200 product run.
- The 2026-08-26 current-worktree popup authority follow-up removes the last
  product-reachable numeric popup anchors. Notification Center and tooltips use
  control identity, the Workbench context-menu primitive and live instance use
  one transient pointer anchor, and Command Palette plus Dialog/ConfirmDialog
  resolve from the arranged surface root. The unused catalog-only
  `WorkbenchDropdownPopup` import was removed from the product window without
  deleting its standalone catalog sample. The current Editor `.zui` Python
  matrix is green at 96/96, all 248 current Editor `.zui` files parse, and the
  product-reachable numeric-anchor owner set is empty. This is static evidence:
  no current-source Editor binary, WGPU product screenshots, or 1000/1000/200
  profile was produced in this follow-up, so M6 acceptance remains pending.
- The 2026-08-27 tooltip-trigger follow-up moves live Workbench icon hints onto
  the Runtime `UiInputManager` delayed-open path. Native pointer metadata keeps
  its original timestamp domain; a dedicated input-timer deadline participates
  in the existing `WaitUntil` policy without borrowing the asset-maintenance
  slot or starting a local thread. The `.zui` popup opens only after Runtime
  retained state becomes visible, while pointer activity, keyboard/IME, focus
  loss, and host menu/page-overflow overlays cancel or occlude the candidate.
  The retained painter consumes projected fade progress. The initial audit found
  no frame-driven transition owner, so it correctly rejected a static `.zui`
  fade; the follow-up now supplies that owner in Runtime `UiInputTimerState` and
  lets Editor project its progress without keeping a second clock.
  The focused source contract is green at 5/5, but Cargo and current-source WGPU
  interaction evidence remain pending.
- The tooltip timing reference was tightened against local Unreal Slate source:
  the shared Runtime/Editor default summon delay is now 150 ms, with the default
  boundary covered at 149.999 ms hidden and 150 ms visible. Authored
  `tooltip_delay_ms` values still override the default, including intentional
  `0` and `500` ms values. Unreal's separate 100 ms intro now maps to a
  manager-owned timeline sampled through the existing input-timer wake slot at
  intervals no longer than 16 ms. The Workbench `.zui` declares `fade` with
  linear easing, while Editor only projects Runtime progress and status.
- Workbench icon-tooltip extent is content measured at the Runtime show boundary
  instead of leaving every hint inside the authored 280 px frame. A visible
  tooltip remeasures the same candidate after shell-size or device-scale reflow,
  so its width cannot remain stale after crossing a monitor or shrinking the host.
  The product `.zui` owns a 96 px compact floor and Unreal-aligned 1000 logical
  px wrap policy; the bridge measures the active title with Runtime font metrics,
  adds shared padding/clip guard, clamps to the logical shell, and marks layout
  dirty only when the exact fixed width changes. The Host painter's previous
  `row_height * 10` bubble ceiling is now `row_height * 31.25`, preserving the
  1000 px default across device-scale changes. Source regressions require `Move`
  to remain 96 px, `Compile Current Module` to expand without exceeding a 420 px
  shell, and the same visible long title to remeasure to the 96 px floor after
  the shell shrinks to 120 px. Only an exact width change marks layout dirty.
  Cargo and a current-source WGPU tooltip capture remain pending.
- The Tooltip intro source regressions lock normalized progress at 0.0 on Show,
  0.5 after 50 ms, and 1.0/`entered` after 100 ms. New candidates, pointer or
  keyboard activity, focus loss, overlay occlusion, and explicit dismissal clear
  the same Runtime timeline. Consecutive current-source WGPU frames are still
  required before this motion is visually accepted.
- One managed current-source Editor build was submitted from HEAD `301685c` plus
  the current overlay. Coordinator request `a6f74582554e476890dc3cd66dbcfbfd`
  accepted `cargo.acquire` but returned `command_post_timeout` without a terminal
  result or rustc diagnostics. The recovery URL was not polled and no duplicate
  Cargo request was submitted, so this is not compile or WGPU evidence.
- A later single `build-editor.ps1 -Ephemeral` attempt on 2026-08-27 was rejected
  before Cargo/rustc by `unmanaged_artifacts_detected`. The only current cleanup
  reservation path was another session's
  `D:\ZirconBuilds\tooling15-wave158-runtime-20260827-121700`; UI12 did not
  delete, release, or poll it and did not submit a duplicate build. This attempt
  produced no compiler fingerprint, executable, screenshot, or profile.
- The 2026-08-27 product typography follow-up applies the local Unreal Slate
  `STextBlock::ComputeDesiredSize` font-measure authority and MagicaVoxel's
  measure/arrange separation to retained Editor text. The initial audit found
  53 fixed label slots below their authored 96-DPI point-size line height
  (Asset Browser 40, Assets Activity 13); the seven Rust-loaded product roots
  now report zero. Compact Details and Utility preview layouts derive their
  body/caption/overlay line heights from `EditorTypographyTokens`; a row that
  cannot fit a complete line is hidden rather than rendered with a reduced font.
  The retained template fallback also preserves the authored/style font size for
  every positive-height slot instead of applying
  `requested.min(available_height)`, while empty or invalid slots still emit no
  text command. The focused dynamic-product contract is green at 14/14 and the
  complete Editor `.zui` Python matrix is green at 164/164. The protected UI
  Asset Editor remains scan-only with SHA-256
  `5DEFDAD04BE2837AF3CD012C63D6801D0136F8B1CAC36B492E6799C921381F09`.
  One new managed Editor build attempt was rejected before Cargo/rustc by
  `unmanaged_artifacts_detected`; cleanup reservations belonged to Tooling15
  wave159/wave160 and the reported unmanaged path set also included wave161.
  UI12 did not delete, release, poll, or retry those external resources, so a
  current-source executable, WGPU screenshot, and 1000/1000/200 profile remain
  required before acceptance.
- The 2026-08-27 strict product-surface and interaction follow-up removes the
  last inert legacy surface classes from Asset Browser, Assets Activity, and
  Project Overview; every explicit product class now resolves through the
  strict Workbench theme. Compact Asset Browser content headers derive complete
  body/caption line heights from `EditorTypographyTokens` and hide a line when
  its slot cannot contain it. The mesh-import path field now emits the existing
  `workbench.asset.mesh_import.path.set` action, and that draft value is carried
  through `AssetWorkspaceSnapshot`, projection-cache identity, and field-value
  reprojection so a presentation rebuild cannot erase the edit. The focused
  dynamic-product and complete Editor `.zui` contract matrix is green at
  165/165; scoped Rustfmt, TOML parsing, Python compilation, and diff checks are
  green. The protected UI Asset Editor remains scan-only with SHA-256
  `5DEFDAD04BE2837AF3CD012C63D6801D0136F8B1CAC36B492E6799C921381F09`.
  These are source-level results only; current-source Editor compilation, WGPU
  product captures, and the 1000/1000/200 profile remain acceptance requirements.
- The one managed Editor retry after this follow-up was rejected before
  Cargo/rustc by `unmanaged_artifacts_detected`. The two current cleanup
  reservations are external Tooling15 paths
  `D:\ZirconBuilds\tooling15-wave162-runtime-20260827-134546` and
  `D:\ZirconBuilds\tooling15-wave163-runtime-20260827-135832`. UI12 did not
  delete, release, poll, or retry those resources, and the attempt produced no
  current-source compiler fingerprint, executable, screenshot, or profile.
- The component-lab icon-button samples now have complete trigger authority.
  Add, Open, Save, Delete, Show, Hide, Lock, and More each publish a unique
  `ComponentLab/*` click identity from `.zui`; the Workbench binding table maps
  every identity to its corresponding `component_lab.icon_button.*` action,
  and the shared preview-action registry routes all eight actions through the
  existing `ComponentLabPreview` feedback record instead of generic menu
  normalization.
  The status-bar World icon remains explicitly non-interactive because it is a
  state indicator, not a command. The focused reachability contract is green at
  4/4 and the complete Editor `.zui` matrix is green at 166/166. All 248 current
  Editor `.zui` files parse, and the protected UI Asset Editor SHA-256 remains
  `5DEFDAD04BE2837AF3CD012C63D6801D0136F8B1CAC36B492E6799C921381F09`.
- A new `build-editor.ps1 -Ephemeral` attempt from HEAD `6494216` plus the
  current overlay was accepted at `cargo.acquire`, but coordinator request
  `172ecbe8c3ad463db54be82b204cc4f3` returned `command_post_timeout` before a
  terminal result. UI12 did not poll the recovery URL or submit a duplicate
  request. Cargo/rustc did not return a current-source fingerprint, so product
  WGPU screenshots and the 1000/1000/200 profile remain pending.
- A fresh native MagicaVoxel 0.99.7.2 reference capture was taken at 96 DPI on
  2026-08-27, independently of the blocked Zircon build. Its compact 50 x 30 px
  tool buttons use an approximately 8 px corner radius and retain one to two
  physical pixels of partial edge coverage; the 26-32 px icon controls use the
  same visual family at roughly 6 px. This empirically supports the current
  Zircon 6/8/10/12 px radius hierarchy. The differentiator to validate is
  physical-pixel edge coverage and icon sampling, not blind whole-frame
  upscaling or still-larger radii. The eventual WGPU captures must compare
  rounded corners, one-pixel borders, SVG diagonals, and text/icon stability at
  640 x 520, 900 x 620, and 1672 x 941; this MagicaVoxel capture is reference
  evidence only and is not a substitute for a current-source Zircon frame.
- A product-root reachability audit on 2026-08-27 traversed 126 `.zui`
  documents from the live Workbench window and seven Rust-loaded product views,
  while keeping the protected UI Asset Editor scan-only. It found two remaining
  parent-slot clipping defects: the Asset Browser and Assets Activity toolbar
  subtitle rows exposed only 10 logical pixels for a 12.8 logical-pixel caption
  line. Both rows now provide 13 logical pixels and Asset Browser explicitly
  authors the shared caption-size token. The new import-graph regression first
  failed on exactly those two rows and is now green with zero reachable
  violations. The complete Editor `.zui` matrix is green at 167/167, device-
  pixel AA contracts at 15/15, physical-sampling contracts at 2/2, and all 253
  current Editor UI `.zui` files parse. The protected SHA-256 remains
  `5DEFDAD04BE2837AF3CD012C63D6801D0136F8B1CAC36B492E6799C921381F09`.
  Current-source Editor compilation, three-size WGPU captures, and the
  1000/1000/200 interaction profile are still mandatory before visual acceptance.
- A follow-up current-source renderer audit traced the product submission from
  the retained Workbench projection through `UiSurfaceDrawList` and the WGPU
  surface. Layout is computed in logical units, then frames, clip rectangles,
  font metrics, border widths, and corner radii cross one explicit presentation-
  scale boundary before the draw list uses the native physical client extent.
  Rounded fills and borders use a pixel-domain signed-distance function with
  separate `fwidth` coverage for outer and inner edges; sRGB targets receive
  linear-light premultiplied output. Glyphon receives the physical projection
  resolution and physical font metrics with no post-raster scale. SVG icons are
  rasterized at the physical target with bounded adaptive local supersampling and a
  linear-light resolve; the icon atlas adds replicated one-pixel gutters and
  has no mip chain, so bilinear sampling cannot read an adjacent icon. This
  review found no evidence that the current product path renders a low-
  resolution UI surface and scales it up, and therefore does not justify whole-
  frame supersampling or MSAA as a speculative correction. Real product pixels
  remain the acceptance authority for fractional-DPI coverage and any path that
  may bypass this projection.
- One managed `build-editor.ps1 -Ephemeral` request from HEAD `ea35974` plus the
  current overlay was accepted at `cargo.acquire`, but request
  `9017ee80cca645f4b6120f1625f71230` returned `command_post_timeout` in
  `post_response` before rustc produced a fingerprint. UI12 did not poll the
  recovery URL or submit a duplicate request. No current-source executable,
  WGPU capture, pixel inspection, or 1000/1000/200 profile was produced, so M6
  acceptance remains pending.
- A follow-up managed build from the same HEAD plus the current shared worktree
  used a longer coordinator command window and entered real rustc compilation.
  Job `582144d2c24f4d0684533088bffe4f69` failed with exactly two current-source
  E0308 diagnostics in Runtime90-owned WGPU diagnostic readback: the RGBA8 and
  RGBA16F paths pass a `u32` row-byte count to the widened `u64`
  `DiagnosticTextureReadbackLayout::new` contract. No UI12-owned Rust or `.zui`
  file appears in the compiler blocker set. The Job was released and its
  ephemeral target deleted by the coordinator. The exact hashes and repair
  boundary were appended to Runtime90's existing diagnostics compile failure;
  UI12 did not edit the active untracked Runtime90 sources. No product bundle,
  three-size WGPU capture, pixel inspection, or 1000/1000/200 profile exists
  from this attempt, so M6 acceptance remains pending.
- The product interaction reverse audit now distinguishes inline `.zui` events
  from component-owned and host-owned input. Project Overview commands retain
  `dispatch_kind`/`action_id`; Hierarchy search and Welcome fields are projected
  through their Rust edit authorities. The former Inspector Tag and named Layer
  dropdowns were later proven to have neither events nor a matching persisted
  data model; the accepted replacement is recorded below. Asset Browser rows intentionally
  remain free of duplicated per-row events: the shared asset-content pointer
  bridge resolves the live row to an asset UUID, dispatches the `SelectItem`
  builtin binding, and the binding constructs `AssetCommand::SelectItem`.
  A new focused contract first failed when it incorrectly required the command
  type in the pointer layer, then passed after locking the actual two-stage
  boundary. The focused asset interaction plus product text-slot matrix is
  green at 4/4; the complete Editor `.zui` contract matrix is green at 170/170,
  and all 248 current Editor UI `.zui` files parse. The protected asset hash
  remains unchanged.
- Product screenshot acceptance now has a dedicated post-capture oracle at
  `tools/zircon_editor_ui_visual_oracle.py`. It consumes the native capture
  manifest and matching `ui_profile_geometry.json`, requires exactly one GPU
  process for each 640x520, 900x620, and 1672x941 physical extent, verifies
  `GetDpiForWindow` scale and framebuffer dimensions, then uses real profiled
  button/tab/field geometry to inspect all four rounded corners in linear RGB.
  It also consumes the always-visible `activity_rail_buttons` geometry, insets
  each control to exclude its outer chrome, and measures the SVG foreground edge
  coverage against the dominant local background in linear light. The gate also
  requires the profiled center/document/status layout frames to have positive
  area, keeps every layout/clickable frame inside the physical framebuffer with
  a half-pixel tolerance, and rejects different same-surface clickable controls
  whose sizes are within 5% and whose overlap reaches 98%. It permits collapsed
  side/bottom regions and ordinary parent/child containment. The gate requires
  observable fractional edge pixels for both rounded chrome and at least one
  activity-rail vector icon, and writes 8x nearest-neighbor diagnostic crops
  while preserving the 1:1 desktop PNG as visual authority. It does not infer
  full visual quality from those scalars: radius hierarchy, border continuity,
  text, SVG silhouette correctness, spacing, and visual hierarchy still require
  manual review of the original captures. Synthetic tests distinguish
  8x-resolved rounded/icon geometry from binary aliased geometry, including a
  mixed case where the rounded control remains smooth but the icon is binary;
  they also reject empty/overflowing primary layout frames, near-duplicate
  clickable layers, software presentation, missing extents, and duplicate
  processes. Capture-manifest schema v2 is also mandatory: the oracle rebuilds
  the aggregate source fingerprint from every listed on-disk source, verifies
  each source stays within the recorded repository, and independently rehashes
  the Editor and Runtime binaries against the managed-build receipt before any
  pixel analysis. Old schemas, missing provenance, source drift, binary drift,
  expected/actual receipt mismatch, post-capture PNG/profile-geometry drift,
  and product-bundle asset drift are rejected. Rounded coverage must include at
  least one control whose four corners each span multiple coverage levels and
  multiple physical rows and columns; isolated mixed-pixel noise in only two
  corners is not accepted as antialiasing. The product bundle must contain
  exactly the current Runtime and Editor source asset set; all 646 files are
  rehashed before analysis. The focused oracle matrix is green at 19/19.
- The 2026-08-28 current-source audit at HEAD `681588f` invalidated the prior
  Runtime90 blocker fingerprint. `production/device/diagnostics.rs` is now
  SHA-256 `883FE369...7CBC`; its RGBA8 and RGBA16F row-byte calculations start
  from `u64::from(width)`, and the native pick helper also accepts `u64` before
  calling `DiagnosticTextureReadbackLayout::new`. The former E0308 call boundary
  is therefore repaired in source. One managed `build-editor.ps1` attempt then
  failed before Cargo with `admission_checkpoint_stale`; it produced no rustc
  diagnostics and no bundle, and UI12 did not poll or retry the coordinator.
- Native product capture is no longer dependent on an untracked `.codex/state`
  helper. `tools/capture-editor-ui-visual.ps1` now owns the repeatable workflow:
  three visible Editor processes, exact physical client extents, GPU-only
  presenter evidence, native DPI recording, tiled desktop capture for displays
  smaller than the requested frame, stable `capture-manifest.json`, and default
  execution of `zircon_editor_ui_visual_oracle.py`. Its environment injection
  is compatible with Windows PowerShell 5.1 and restores every process variable
  immediately after launch. Capture now requires the exact Editor and Runtime
  SHA-256 values returned by the managed build, rejects a mismatch before it
  creates output or starts a process, and records expected/actual binary
  fingerprints. The shared `editor-ui-visual-source-binding.ps1` helper also
  hashes the Git revision, the profiling-critical source set, capture tools,
  and all Runtime/Editor assets. Each PNG and matching profile geometry JSON
  carries an independently verified SHA-256. The pre-build source hash must
  still match immediately before capture, and binaries older than the newest
  covered source are rejected. The focused PowerShell contract is green at
  8/8. A new current-source product
  bundle is still required before those three captures and the 1000/1000/200
  profile can run.
- A later 2026-08-28 managed product-build request used a fresh explicit output
  directory on `D:` after confirming no OS Cargo/rustc process and 93.85 GB of
  free space. Admission failed before Cargo with
  `unmanaged_artifacts_detected`, naming only the external Tooling15 directory
  `D:\ZirconBuilds\tooling15-local-benchmarks`; no UI12 source diagnostic or
  product bundle was produced. UI12 did not delete, release, or adopt that
  foreign artifact and did not submit another build request.
- After the external Tooling15 directory disappeared, a new managed product
  build froze the 534-file Editor UI source fingerprint and requested
  `D:\ZirconBuilds\ui12-current-editor-20260828-visual`. The coordinator accepted
  `cargo.acquire` request `2b23fbbf38e0433699c86965e61595e7` but did not return
  a terminal result during reconciliation; `build-editor.ps1` failed before
  Cargo with `command_post_timeout`. UI12 did not query the supplied recovery
  URL or submit another build. No rustc diagnostic, bundle, screenshot, or
  product profile was produced by this attempt.
- The later non-Cargo closeout at HEAD `384b578` records source fingerprint
  `1a6e1c4d...d58c2` across 940 files, including the 636 Editor and 10 Runtime
  product assets. The complete Editor `.zui` contract is green at 170/170, the
  focused DPI/device-pixel/analytic-AA/SVG/pixel-snapping matrix is green at
  43/43, the visual-oracle plus related interaction/text matrix is green at
  23/23, the native capture contract is green at 8/8, and all 248 current
  Editor UI `.zui` files parse. The protected UI Asset Editor hash remains
  `5DEFDAD0...381F09`. These are current-source static/tooling gates, not a
  substitute for the still-pending WGPU product captures and interaction run.
- The 2026-08-28 follow-up at HEAD `36d47fd` preserves fractional physical
  border widths through Editor paint recording and resolves software rounded
  edge coverage in linear light before encoding to sRGB. The fully opaque fast
  path remains intact, while the 8x8 boundary sampler still limits extra work
  to rounded primitive edge bands. Static contracts cover the 1.25 px recording
  path and the expected half-coverage white-over-black sRGB result.
- Welcome Recent drawing, pointer routing, scroll clamping, and profiling now
  consume the captured `WelcomeRecentListPanel` frame from the same projected
  `WelcomePaneLayoutData`. The former pane-size formula survives only as a
  missing-projection fallback; it is no longer an independent product hit-test
  authority. A Rust regression rejects a click in the stale formula viewport
  and accepts the same action inside the published list frame. Current non-Cargo
  evidence is green at 67/67 for the focused radius, Welcome, device-pixel AA,
  and visual-oracle set; the complete Editor `.zui` matrix is 171/171 and all
  248 Editor `.zui` files parse. The protected asset SHA-256 remains
  `5DEFDAD04BE2837AF3CD012C63D6801D0136F8B1CAC36B492E6799C921381F09`.
  The complete `ui-profile-capture` Pester contract is also green at 48/48:
  click, pointer-move, and resize evidence is bound to published product
  geometry; CPU core/system utilization and working-set/private-byte samples are
  mandatory; and incomplete operation counts such as 999/1000 fail closed.
- Current-source product validation is still pending. The shared uncommitted
  `zircon_app/Cargo.toml` overlay directly declares workspace `windows-sys` under
  `[target.'cfg(windows)'.dependencies]`, while the shared `Cargo.lock`
  `zircon_app` package entry does not list `windows-sys 0.61.2`. Consequently a
  managed `--locked` Editor build still fails before rustc asks for a lockfile
  update. The manifest addition and the lockfile's other dependency updates are
  foreign worktree changes, not HEAD `36d47fd`. UI12 does not revert the manifest,
  regenerate the foreign shared lock state, or submit duplicate builds while the
  mismatch is unchanged. No current-source product screenshot or 1000/1000/200
  profile is claimed by this static follow-up. The native three-size capture
  workflow itself is ready and its Pester contract is green at 8/8.
- The 2026-08-28 trigger-authority follow-up at HEAD `11cac2d08` removes the
  product-reachable dead Inspector Tag dropdown and the invented
  Default/Foreground/Background Layer options. The compact metadata row now
  exposes the real scene `RenderLayerMask.mask` as a stable-width numeric field.
  Snapshot publication reads `Scene::render_layer_mask`; bridge synchronization
  displays that same value; Change updates only the retained draft; and Submit
  accepts decimal, hexadecimal, or binary `u32` input before emitting a typed
  `UiBindingValue::Unsigned` Inspector field batch. The existing reflected scene
  command path remains the transaction, undo, and redo authority. A focused
  contract also rejects any enabled Inspector control without events, preventing
  another visually interactive but inert control from entering the product.
  Current non-Cargo evidence is green at 5/5 for this interaction slice,
  175/175 for the complete Editor `.zui` matrix, 74/74 for the combined radius,
  Welcome, Inspector, device-pixel AA, physical-sampling, and visual-oracle
  matrix, and 248/248 for Editor `.zui` parsing. Direct rustfmt checks pass for
  all 19 touched Rust files, scoped diff checks pass, and the protected UI Asset
  Editor SHA-256 remains
  `5DEFDAD04BE2837AF3CD012C63D6801D0136F8B1CAC36B492E6799C921381F09`.
  This source-ready slice does not replace the pending native WGPU screenshots
  or 1000/1000/200 product profile.
- The 2026-08-28 product-interaction audit at HEAD `11cac2d08` recursively
  resolves every product-reachable Workbench component to its native
  interactive primitive and rejects enabled consumers without events. It found
  and closed five real gaps: the Scene Tree search field now routes Change and
  Submit by binding identity, rather than a host control-name special case,
  into the existing hierarchy-filter authority, while the Component Lab
  number field and three slider samples route through a retained local-state
  bridge that updates numeric value, display text, and normalized position.
  The bridge trims only numeric parsing input and preserves authored whitespace
  for ordinary text and search edits. Welcome's two eventless field consumers
  are separately locked to their explicit `welcome_text` native-dispatch
  projection instead of being treated as undocumented exceptions.
  Their template bindings and preview action allowlist now share the same named
  action identities; the previously omitted Component Lab search preview
  actions are included as well. The focused contract is green at 4/4, the full
  Editor `.zui` matrix is green at 180/180, the combined AA, physical-sampling,
  visual-oracle, radius, Welcome, Inspector, and interaction matrix is green at
  95/95, and all 248 Editor `.zui` files parse.
  Direct rustfmt and scoped diff checks pass, and the protected UI Asset Editor
  SHA-256 remains
  `5DEFDAD04BE2837AF3CD012C63D6801D0136F8B1CAC36B492E6799C921381F09`.
  Product compilation remains deferred because the unchanged shared
  `zircon_app/Cargo.toml` Windows `windows-sys` dependency is still absent from
  the `zircon_app` dependency list in `Cargo.lock`; no screenshot or product
  profile is claimed by this non-Cargo slice.
- A later same-day read at HEAD `9201304352ed466ef7b5256b2c0ca02d133af4d2`
  confirmed that the shared lock owner had added `windows-sys 0.61.2` to the
  `zircon_app` package dependency list, so the prior manifest/lock mismatch no
  longer applied. UI12 waited for an unrelated `zircon_runtime --lib --offline`
  Cargo/rustc process to exit naturally, then submitted exactly one managed
  ephemeral product build to
  `D:\ZirconBuilds\ui12-current-editor-20260828-interaction-aa`. The
  coordinator accepted `cargo.acquire` request
  `224344288ed141b59e9b6b6c97bac055` but returned
  `command_post_timeout` after reconciliation without a terminal result. No
  Cargo/rustc process, published output, or product staging directory remained;
  UI12 did not query the recovery URL or retry. The current profiling/capture
  Pester set is green at 128/128, but no current-source EXE, WGPU screenshot, or
  product interaction profile was produced.
- The subsequent dynamic-product interaction audit found that
  `hierarchy.zui` still gave its search field the legacy
  `edit_action_id = "HierarchySearchQuery"`, forcing the retained host to route
  edits by `control_id`. The field now publishes the same explicit
  `Workbench/SceneSearchEdit|Commit` Change/Submit bindings as the componentized
  Scene Tree search. Projection therefore derives both edit and commit targets
  from authored events, while `HierarchySearchQuery` remains only the stable
  presentation control identity. The host control-name branch is removed and
  both surfaces reuse the existing hierarchy-filter authority. Project
  Overview's two eventless buttons were separately verified as intentional
  native-template projection: `OpenAssetsView` resolves
  `workbench.view.open.editor.assets`, while `OpenAssetBrowser` enters the
  canonical asset command route; no duplicate `.zui` events were added. The
  focused interaction guard is green at 5/5, the complete Editor `.zui` matrix
  is green at 181/181, direct rustfmt and scoped diff checks pass, and the
  protected UI Asset Editor SHA-256 remains
  `5DEFDAD04BE2837AF3CD012C63D6801D0136F8B1CAC36B492E6799C921381F09`.
  Current-source product compilation, WGPU screenshots, and the 1000/1000/200
  profile remain pending.
- A fresh managed product build at HEAD
  `a2d8d811c4a3a1fc1db6f5375c491e7e4502533f` entered Cargo normally and
  compiled through the UI/WGPU dependency graph in isolated target
  `F:\cargo-targets\zircon-engine\ephemeral\check\a8d8c56b523a4d08a9028d491b676e42`.
  It stopped with the single current-source E0004 in
  `zircon_runtime_host/src/foreign_output/item_count.rs:80`: the foreign-output
  budget match does not cover
  `WorldQueryResult::TransformSnapshot`. This exact mixed-tree lower-layer
  defect is already owned by the open RuntimeInterface01 handoff
  `docs/plans/optimize/zircon_runtime_interface/01/failure-2026-08-27-world-query-transform-snapshot-item-count.md`,
  which explicitly rejects an isolated upper-plan patch. No UI12 source was
  diagnosed. Cargo/rustc exited, no product output directory was published, and
  UI12 did not retry or modify the foreign world-query migration.
- Dynamic product-pane spacing now follows the shared density authority instead
  of copying today's numeric values. Hierarchy and Console shell gutters use
  `$editor.density.gap.medium`, Inspector uses
  `$editor.density.gap.large`, and their local section gaps use
  `$editor.density.gap.small`. Console's isolated 10-pixel side gutter was
  converged to the 8-pixel medium tier, leaving more room for repeated output
  while aligning both sides with its text inset. A focused token-map regression
  is green at 1/1 and the complete Editor `.zui` matrix is green at 182/182;
  scoped diff checks pass and the protected asset hash is unchanged.
- SVG local supersampling is now adaptive at the physical raster target rather
  than uniformly fixed at 2x. Content whose fitted maximum edge is at most 32
  physical pixels uses a 4x source raster, the normal vector path remains 2x,
  and either path falls back to 1x before it can exceed the 4096-pixel raster
  edge bound. The resolve is parameterized by the selected sample axis and
  continues to average premultiplied alpha in linear light before re-encoding
  sRGB. Only the final device-pixel RGBA is retained in the bounded LRU cache;
  the 4x source for a 32x32 icon is a short-lived 128x128 buffer and does not
  increase resident cache payload. Targets at or below 32 physical pixels also
  keep their exact width and height before cache lookup, so the resolved icon
  is not linearly rescaled a second time into its original frame. Larger vector
  previews retain 4/8/16-pixel resize buckets; across square targets 1..512
  this quality exception increases the bounded target set from 64 to 80 rather
  than making all resize extents unique. The focused AA, performance, and GPU
  residency source contracts are green at 39/39; direct rustfmt and scoped
  diff checks pass. This evidence
  establishes the bounded policy and resolve semantics only; current-source
  WGPU screenshots remain required to accept the visible edge quality.
- A 2026-08-28 local Unreal source recheck keeps the reference claim precise.
  `SlateVectorGraphicsCache.h` keys a vector by brush name and
  `(LocalSize * DrawScale).IntPoint()`, while
  `SlateVectorGraphicsCache.cpp` batches pending SVG raster work, publishes the
  resulting pixel size into dilated-border atlases, and sends oversized vectors
  to non-atlased textures. `SDPIScaler.h` defines DPI scale as physical pixels
  per Slate Unit. Zircon adopts that device-pixel identity, retained raster
  cache, atlas-padding, and logical/physical separation. Unreal does not expose
  a fixed 4x SVG constant in these owners, so Zircon's <=32px 4x resolve is an
  explicitly bounded project quality policy, not an attributed Unreal value.
- The dynamic asset-surface audit found that Assets Activity used
  `compact_left_drawer_max_width` (340 logical pixels) as a viewport breakpoint,
  although its authored tree/content minima require 188 + 320 pixels plus a
  gap. The responsive projector now follows the shared Workbench narrow
  breakpoint (640 logical pixels): 640/420 collapse the auxiliary tree and give
  the content panel the full root width, while 900 retains the regular two-column
  layout. Asset Browser already uses its independent column budget, retaining a
  152-pixel Sources panel at 640 and collapsing Details before content becomes
  unreadable. The new Rust product-projection regressions are source-ready; the
  static breakpoint regression completed RED/GREEN, the full Editor `.zui`
  matrix is green at 185/185, direct rustfmt and scoped diff checks pass, and the
  protected asset hash is unchanged. The Rust regressions remain pending the
  next current-source managed Editor compile.
- Square software-fallback borders now preserve the same fractional physical
  width as rounded borders and recorded WGPU commands. The former render-command
  branch applied `ceil()` before painting, so a 1.5-pixel border at fractional
  DPI became two fully opaque layers. It now publishes through the common border
  primitive; radius-zero fallback evaluates analytic outer-minus-inner rectangle
  coverage in linear light. The implementation scans complete rows only in the
  top/bottom border bands and scans only the left/right bands elsewhere, keeping
  large square panels at perimeter-times-border-width work instead of an
  area-wide 8x loop. Direct workbench borders used by the root skeleton, native
  docks, Welcome surface, and close/blocker dialogs now delegate to this same
  analytic radius-zero authority instead of composing four `floor`/`ceil`
  solid rectangles, so fractional placement cannot expand a nominal one-pixel
  edge into two opaque pixels. Focused static regressions completed RED/GREEN;
  the device-pixel/AA contract is green at 23/23 and the AA, SVG resize-bucket,
  and GPU-residency matrix is green at 45/45. The Rust pixel regression is
  source-ready and remains pending the current-source managed Editor compile.
- Radius-zero quad recording no longer converts fractional `f32` geometry to a
  `PixelRect` with `floor`/`ceil` before publishing WGPU commands. Square fills
  now retain the same original device-space frame as rounded fills, allowing the
  shared WGPU SDF/fwidth coverage path to resolve scrolling, scaling, and
  animation without whole-pixel jumps. Static one-pixel separators keep their
  explicit integer API and therefore remain snapped. A source-ready Rust
  regression pins fractional recorded geometry; execution remains pending the
  current-source managed Editor compile.
- Software radius-zero fills now resolve the same fractional rectangle geometry
  instead of filling the entire `floor`/`ceil` pixel envelope. Integer-aligned
  interiors keep the existing contiguous-row span path; only fractional top and
  bottom rows plus left and right edge pixels evaluate analytic rectangle
  coverage in linear light. A source-ready 0.5-pixel regression expects four
  25%-covered pixels near sRGB 137, while the static guard rejects area-wide
  supersampling. Rust execution remains pending the managed Editor compile.
- Software chrome-command replay no longer has a separate radius-zero
  `width.ceil()` four-edge loop. Square and rounded replay borders delegate to
  the same fractional analytic primitive, so a recorded 1.5-pixel width cannot
  become two opaque layers during replay. Runtime WGPU draw-list conversion
  continues to pass the original `f32` frame, width, and radius unchanged.
- CPU scaled-image sampling now matches the WGPU image path's color authority.
  Four bilinear samples are decoded from sRGB, interpolated as linear-light
  premultiplied values, composited with the decoded destination, and encoded
  back to sRGB. A 256-entry decode LUT and 4097-entry encode LUT keep transfer
  functions out of the per-pixel hot loop; opaque identity-size copies retain
  their existing direct-row fast path. Source-ready Rust expectations pin 50%
  white and transparent-blue coverage near sRGB 188 instead of the former
  gamma-space 128. The physical-sampling contract is green at 2/2; Rust
  execution remains pending the current-source managed Editor compile.
- Component Lab field edits no longer treat `WorkbenchInputSearch` as a hidden
  behavior switch. Change/Submit still enter exclusively through authored `.zui`
  bindings, while the bridge now selects the `query` state target from the
  control's declared property schema; control IDs remain stable identity only.
  The product interaction contract is green at 6/6, the complete Editor `.zui`
  matrix at 185/185, rustfmt passes, and the protected asset hash is unchanged.
  Rust execution remains pending the current-source managed Editor compile.
- Software alpha composition now has one retained-host color authority rather
  than separate gamma-space loops in shape spans, CPU images, text fallback,
  and viewport gizmo overlays. Source sRGB and destination sRGB are decoded,
  composed as linear-light source-over, and encoded back to sRGB through a
  shared 256-entry decode LUT and 4097-entry encode LUT. Opaque shape spans and
  identity-size images retain their direct-write and row-copy fast paths; no
  transfer `powf` remains in a per-pixel consumer. The common compositor also
  preserves destination alpha, so the transparent viewport overlay buffer no
  longer turns its first antialiased sample opaque. Source-ready Rust numeric
  regressions pin 50% white-over-black near sRGB 188, translucent image/text
  results, and transparent-overlay alpha. The device-pixel/AA contract is green
  at 25/25, physical sampling at 2/2, the combined AA/sampling/SVG-bucket/GPU-
  residency matrix at 49/49, and the complete Editor `.zui` matrix at 185/185;
  direct rustfmt and the protected hash pass. Rust execution and native WGPU
  visual acceptance remain pending the foreign world-query compile repair.
- The adaptive SVG 2x/4x resolve, MUI chart 4x local raster, and native-resize
  frozen-snapshot bilinear sampler now consume that shared transfer LUT too.
  Their former per-sample/per-pixel transfer `powf` implementations are gone;
  chart resolve publishes its linear-premultiplied aggregate and source alpha
  directly to the transparent source-over helper. Supersample factors, resize
  pixel-center mapping, SVG cache buckets, and chart coverage remain unchanged.
  The focused AA/physical-sampling/SVG-bucket/GPU-residency matrix remains green
  at 49/49 and direct rustfmt passes. This is a static hot-path result until the
  required 1000/1000/200 product profile records CPU and memory measurements.
- A 2026-08-29 Unreal source recheck makes the full-frame sampling policy
  explicit. `SlateRHIRenderingPolicy.cpp` configures ordinary Slate render
  targets with `NumSamples = 1`; `SlateShaderCommon.ush` resolves rounded boxes
  from analytic signed distance with `fwidth` and `smoothstep`. Zircon therefore
  keeps the native surface at the physical client extent and reserves 2x/4x
  temporary rasters for bounded vector assets instead of supersampling the whole
  editor backbuffer. The WGPU rounded path already followed that model, but the
  zero-radius instance path still emitted hard fractional edges and square
  borders were four flat quads with a forced one-pixel minimum. Fractional square
  fills now use the analytic SDF path; only fully pixel-aligned square fills keep
  the compact instance fast path. Square and rounded borders publish one analytic
  outline with the original `f32` width and frame, and the legacy `border_rects`
  path is removed. Source-ready Rust regressions cover the fractional fill fringe,
  a 0.625-pixel square border, and the aligned-fill fast path. A native offscreen
  regression also compares the summed alpha of 0.625-pixel and one-pixel square
  outlines so a future clamp cannot preserve vertex metadata while regressing the
  framebuffer. The static AA contract is green at 27/27 and physical sampling at
  2/2; rustfmt, scoped diff
  checks, and the protected asset hash pass. The focused managed `zr_rhi_wgpu`
  gate stopped before Cargo at coordinator `cargo.acquire` reconciliation with
  `command_post_timeout`, so no native readback or product visual acceptance is
  claimed.
- The native acceptance path is source-ready and remains fail-closed. The
  product capture script's PowerShell parser gate passes, and its focused Pester
  suite is green at 8/8 for exact 640x520, 900x620, and 1672x941 physical client
  extents, GPU-presenter evidence, current UI source binding, bundle asset-set
  parity, and Editor/Runtime receipt hashes. The sequence-bound interaction
  latency suite is green at 11/11, including the per-requested-extent resize
  gate; the capture contract records CPU core/system utilization plus working-
  set and private-byte samples for the 1000-click, 1000-pointer-move, and
  200-resize scenarios. The complete profile-output contract is green at 48/48
  and rejects incomplete counts, missing provenance, invalid process metrics,
  full invalidation, and visual/GPU cache churn. The complete current Editor
  `.zui` contract is green at 185/185. Existing
  `.codex/state/ui12-wgpu-visual-proof-*.png` images remain a
  lower-layer frozen-snapshot proof only and are not promoted to product
  acceptance. A current-source check still finds the foreign
  `WorldQueryResult::TransformSnapshot` arm absent from
  `zircon_runtime_host/src/foreign_output/item_count.rs`; no duplicate managed
  Editor build was submitted, and no current-source executable, product
  screenshot, or profile is claimed.
- A 2026-08-29 product-control audit separates authored control radius from
  viewport artwork geometry. Every positive radius below the 6-pixel compact
  control tier is confined to `workbench_viewport_panel.zui` scene details such
  as four-pixel handrails, eight-pixel beacons, axis marks, wall lights, and
  cargo insets. Raising those values to the control tier would distort the
  illustration rather than improve control smoothness. Interactive toolbar
  actions already consume the 10-pixel large tier in every state; fields and
  ordinary buttons remain at 8, while panel and popup shells remain at 12.
  `WorkbenchModuleDiff` and `WorkbenchModuleSimulate` also remain readable
  secondary text commands by an existing responsive-layout contract; there is
  no semantically accurate Diff icon in the current product asset family, so
  this audit rejects an ungrounded icon-only rewrite.
- The same audit traces the product resolution boundary end to end. The native
  window bootstrap supplies the physical client extent, retained layout divides
  by the effective DPI only for logical breakpoints, projected paint frames are
  multiplied once back into physical coordinates, and the WGPU surface,
  viewport, and draw-list target use that physical extent. SVG targets are
  derived from the final physical frame with `ceil`: <=32-pixel fitted content
  rasterizes at 4x, larger content at 2x, and the linear-premultiplied resolve
  returns the exact destination extent. This matches the local Unreal Slate
  policy: native-resolution presentation plus analytic or bounded local AA,
  rather than a permanently supersampled full editor surface.
- A second focused managed `zr_rhi_wgpu` attempt used the current source and the
  `wgpu_ui_` lib-test filter after confirming that no Cargo or rustc process was
  active. Request `f48b89748b114ed083519a8e264789a6` was accepted but failed at
  coordinator `cargo.acquire` reconciliation with `command_post_timeout` before
  Cargo started. The source-visible foreign `TransformSnapshot` match omission
  remains unchanged. No native test result, executable, screenshot, or profile
  is inferred from either control-plane failure.
- A trigger and state audit then compared the live product controls with the
  actual MagicaVoxel `top.ui` and Unreal `SButton`. Unreal keeps `DownAndUp` as
  the default pointer, touch, and keyboard contract and exposes down/up/precise
  activation only as explicit policies. MagicaVoxel marks only Undo and Redo as
  `repeated = 1`. Zircon's existing Press/Release/Click, three-button pointer,
  and click-count routes therefore remain the generic authority; pointer hold
  repeat is not introduced without duration, interval, capture/leave cancel,
  history-boundary, and accessibility contracts. The same audit found a real
  painter mismatch: Runtime already publishes `focus_visible_known` and the
  modality-derived `focus_visible`, while the Editor common selector still
  treated every semantic focus as a visible keyboard outline. The selector now
  honors live Runtime modality, retains authored focus only for static/legacy
  previews, and prominent command buttons consume that same authority instead
  of duplicating the fallback. Source-ready tests cover hidden pointer focus,
  visible keyboard focus, and authored static preview; direct rustfmt, scoped
  diff check, and five source guards pass. Cargo execution and visual acceptance
  remain pending.
- The accompanying current-source event inventory finds explicit `Press`
  activation only in the Component Showcase list-row and pointer-positioned
  context-menu demonstrations. Live Workbench commands retain release/click
  activation; no product command was broadened into pointer-down behavior. The
  focused focus-visible contract is green at 4/4, the complete Editor UI Python
  matrix at 71/71, and the complete Editor `.zui` matrix at 185/185. The
  inspected 1672x941 compatibility image predates the current `welcome.zui` by
  about five hours: current source and its density contract already place the
  startup chooser before `preview_bottom_fill`, so the old frame is historical
  evidence only and cannot fail or satisfy current-source visual acceptance.
- A specialized-painter follow-up removes the remaining visible-focus bypasses.
  Material TextField/state-layer, Asset thumbnail, Axis value field, Chip,
  Command Palette search, Inspector resource field, and Property scalar chrome
  now all consume `focus_visible_for_node`; semantic `focused` remains available
  for input ownership and surface paint eligibility. Pressed/open/selected,
  validation, and unknown-modality static previews keep their prior precedence.
  The focused contract is green at 5/5, the complete Editor UI Python matrix at
  72/72, and the complete Editor `.zui` matrix at 185/185; direct rustfmt,
  scoped diff check, and the protected-asset hash pass. Rust execution and
  current-source native visual acceptance remain pending.
- Asset Browser's product toolbar no longer spends two authored rows on twelve
  kind tabs or makes filter discovery depend on whichever chips happen to fit.
  It now shares the same sixteen stable filter identities and labels as Assets
  Activity through a view-level `asset_kind_filter` owner and exposes one
  readable native `WorkbenchDropdown`. Its Runtime `popup_open`, option
  geometry, dismissal, and keyboard navigation are the sole popup authority;
  no parallel Workbench context menu is opened. Selection still emits the typed
  `AssetCommand::SetKindFilter`; an unsupported current kind is appended as a
  selected, disabled option rather than displayed as All. At 640 logical pixels
  the responsive row reserves Search plus a 124-pixel filter trigger before
  lower-priority view/locate actions, while Import remains reachable. The
  Editor popup-row and keyboard-target paths preserve `asset` / `asset:browser`
  only for these options, while ordinary dropdowns retain `workbench_option`.
  A root Dropdown press with no action is reserved for Runtime popup toggling;
  `AssetSurface/SetKindFilter` option actions normalize to the asset Change
  contract. Focused trigger tests pass 24/24, complete Editor `.zui` 189/189,
  Editor UI 81/81, aggregate ZUI 223/223, and all 253 current `.zui` files parse.
  Direct rustfmt, scoped diff
  checks, and the protected asset hash pass. A single managed product build was
  accepted as coordinator request `ce363d0d866542c99b3810a7d0de1dce` but ended
  at `cargo.acquire` reconciliation with `command_post_timeout` before Cargo;
  no recovery polling or duplicate request followed, and no current-source EXE,
  screenshot, or profile is claimed.
- The page-template `host/asset_surface_controls.zui` filter had a separate
  executable value-domain defect: its ComboBox emitted lowercase
  `all/scene/mesh/material/ui`, while the sole event executor accepts exact
  canonical `All/Scene/Mesh/Material/Ui*` identifiers. It also exposed only five
  kinds. The host control keeps its existing ComboBox-owned Change event and
  `AssetSurface/SetKindFilter` binding, but now publishes the same sixteen
  canonical IDs and readable labels, defaults to `All Types`, and uses a
  124/168/220 logical-pixel width. The focused contract proves every authored
  option ID exists in `parse_asset_kind_filter()` and completed RED/GREEN at
  5/5. The complete Editor `.zui` suite is 187/187, Editor UI 81/81, aggregate
  ZUI 221/221, all 248 assets parse, and the protected hash passes. This is
  source evidence only; no current-source Rust execution or WGPU capture was
  produced after the coordinator timeout.
- The new Asset Browser filter trigger no longer falls through the legacy
  toolbar-chip identity by sharing its `AssetBrowserKind` prefix. Kind controls
  are classified as legacy chips only when their stable ID ends in `Chip` or
  `Button`; existing ViewMode and `workbench.asset.kind_filter.set` compatibility
  identities remain unchanged. `AssetBrowserKindFilterDropdown` therefore keeps
  its Dropdown painter and native Change/popup trigger. The focused contract
  includes a Rust identity regression; the latest complete matrix is recorded
  above. Direct rustfmt, scoped diff checks, and the protected asset hash pass.
  The Rust regression is source-ready; native product visual acceptance remains
  pending.
- A 2026-08-29 native MagicaVoxel reference capture now grounds the compact
  control comparison in displayed pixels: the bundled executable produced a
  900x620 physical client image and an 18x18 dropdown-corner inspection crop
  with 33 distinct ARGB values. The reference keeps ordinary controls in the
  24-30 pixel height tier and an approximately six-pixel compact radius; its
  clean edge is continuous output-pixel coverage, not a globally enlarged
  radius or whole-window supersample. In the product Asset Browser, the twelve
  old `toolbar_kind_*_chip` definitions and the unreachable secondary row are
  now physically absent rather than retained at zero size. Projection no longer
  constructs their text overrides or selected-state writes, and toolbar reflow
  no longer scans for seven missing secondary controls. The sole product filter
  remains `AssetBrowserKindFilterDropdown`; legacy chip-selection helpers stay
  isolated to their compatibility/performance tests. The focused interaction
  contract passes 7/7, aggregate ZUI 223/223, all 248 files in the current asset
  set parse, direct rustfmt and scoped diff checks pass, and the protected asset
  hash remains
  `5DEFDAD04BE2837AF3CD012C63D6801D0136F8B1CAC36B492E6799C921381F09`.
  Current-source Rust execution and the three product WGPU captures remain
  pending the shared Cargo lane.
- After the shared Cargo/rustc processes naturally cleared on 2026-08-29, one
  additional `tools/build-editor.ps1 -Ephemeral` invocation was allowed to run
  for the complete 904-second tool window. It emitted no stdout or stderr and
  the outer invocation timed out. Its managed `validate-matrix` child then
  exited naturally without ever starting Cargo or rustc; the bound
  `mvp-product-inputs-build-editor-cba2e1590b3b46fb8fc8c9089e1b27f5`
  staging directory was removed and contains neither `zircon_editor.exe` nor
  `zircon_runtime.dll`. No recovery URL was queried and no duplicate build was
  submitted. This is a control-plane/no-artifact result, not a Rust compile or
  product visual result.
- The newest managed current-source product build at HEAD
  `8aabbee3e99dc919f6da4611e3a44e8463a7fe7f` plus the shared overlay ran
  `tools/build-editor.ps1 -Ephemeral -SkipSmokeTest` and entered the real
  locked `zircon_app` Editor Cargo build. It stopped in `zircon_runtime` with
  28 shared-source diagnostics before compiling `zircon_editor`; representative
  failures are `text/font/shared.rs:66` E0015,
  `graphics/scene/gpu_scene/prev_transform.rs:34` E0502,
  `resource_streamer_ensure_material.rs:682` E0594, and
  `platform/host/service.rs:56` E0382. No current-source Editor executable,
  bundle, WGPU capture, or profile was produced, and cargo/rustc were absent
  after the managed command ended. The UI-owned static gates remain green at
  54/54 for the combined device-pixel AA and visual-oracle contracts, 85/85 for
  Editor UI contracts, and 223/223 for repository ZUI contracts; these results
  do not replace product visual acceptance.
- A focused painter audit found the MUI X shared `push_quad` helper was mapping
  every positive-width geometry border to `focus_ring`, making ordinary picker,
  chat, and demo surfaces look keyboard-focused. It now uses the neutral
  `palette.border`; focus-visible remains state-aware and is painted only by
  the dedicated selectors. The helper's source regression and scoped rustfmt/
  diff checks pass. This is a visual-role correction, not a replacement for
  the pending current-source Editor/WGPU capture.
- Vector resize-cache bucketing now preserves exact physical dimensions for
  non-square targets. Square toolbar icons still share bounded resize buckets,
  while rectangular SVGs can no longer be quantized to a square bitmap and
  stretched back into the requested frame. The cache remains bounded by its
  4096-entry/64-MiB LRU; scoped source, rustfmt, and diff checks pass. Native
  current-source WGPU product capture remains pending. The refreshed static
  matrices pass at 54/54 device-pixel AA plus visual oracle, 85/85 Editor UI,
  and 223/223 repository ZUI.
- The native screenshot oracle no longer accepts one isolated smooth control or
  icon as proof for an otherwise aliased surface. Among controls with at least
  two analyzable corners, at least half must have continuous fractional
  coverage on all four corners and a horizontal/vertical curve span consistent
  with the expected physical radius; at least half of analyzable activity-rail
  vector icons must also expose fractional edge coverage. The report publishes
  candidate counts, radius-qualified counts, and passing ratios. Synthetic
  binary-edge, sparse-noise, three-pixel-radius, and one-of-three token
  regressions pass at 23/23. The one-of-three rounded-control regression also
  enters through the capture manifest, rehashes the modified geometry profile,
  and proves the product analysis path invokes the population gate instead of
  leaving it as an isolated helper contract. The PowerShell native-capture
  chain passes 8/8 and invokes this oracle after binding three independent GPU processes to exact
  physical extents, DPI, source, binary, and bundle fingerprints; the rule
  remains pending against the three required current-source product captures.
- Product icon routing was re-audited after the raster corrections. Literal
  `.zui` icons resolve to packaged SVG assets, semantic popup icons resolve to
  the same vector candidate path, toolbar icon-only commands retain labels and
  tooltips, and legacy inferred Plus/Trash/Chevron glyphs now name packaged SVGs
  instead of drawing pixel-grid segments. The focused SVG, popup-icon, and
  toolbar-hint contracts pass at 26/26; the antialiased missing-icon raster is a
  last-resort fallback, not the normal product path.
- Text is now a first-class native screenshot acceptance surface rather than an
  inference from shape and icon quality. Profile geometry schema v4 publishes
  visible text-command frame, clip, RGBA, font size, line height, command index,
  and character count from the same `ChromeCommandStream` used to paint the
  optional reference frame; it never serializes the actual UI string. The
  visual oracle projects screenshot pixels between the dominant background and
  declared text endpoint in linear light, then requires fractional glyph-edge
  coverage across multiple bins, rows, and columns for at least half of the
  analyzable runs. A synthetic regression proves binary text is rejected even
  when rounded controls and vector icons remain antialiased. Current static and
  workflow gates pass at 57/57 combined AA plus visual oracle, 86/86 Editor UI,
  223/223 repository ZUI, and 8/8 native capture Pester; Python compile,
  rustfmt, scoped diff checks, and the protected UI Asset Editor SHA-256 also
  pass. The lower RuntimeInterface TransformSnapshot migration failure remains
  open at HEAD `f2df7ed2100a771881a3b7222b726789b0b40abd`, so no Cargo was started and
  no current-source product text pixels, three WGPU captures, or 1000/1000/200
  interaction profile are claimed. M6 acceptance remains pending.
- Rounded-control acceptance now consumes profile v4 `rounded_shapes` emitted from
  the same `ChromeCommandStream` that paints the reference frame. Each entry binds
  command index, physical frame, clip, corner radius, and border width; the oracle
  matches these entries to profiled controls and rejects a control below the 4px
  physical-radius floor instead of inferring an 8px radius from the window scale.
  Focused AA/oracle and Editor UI contracts are 59/59 and 87/87 respectively;
  native current-source WGPU captures remain pending.
