# Zircon ZUI Bridge

Local Penpot plugin for reversible `.zui` v2 authoring. `.zui` remains the source of truth; Penpot shapes are an editable projection with runtime-only values retained in shared plugin metadata.

## Run

Use the Node version and pinned dependencies in the [workspace guide](../../README.md).
From the ZirconEngine root, build in a fresh approved artifact directory:

```powershell
& tools/penpot/tools/scripts/run-validation.ps1 -BuildOnly
```

The guide explains how to preview the resulting distribution at port 4213 and
add `http://127.0.0.1:4213/manifest.json` to Penpot.

The plugin imports a `.zui` file into one asset board. Select that board or any descendant layer before using **Export selected**.

## Contract

- Stable node IDs, components, and the bridge schema version live in `dev.zircon.zui` shared metadata; layer renames do not alter them.
- Geometry, literal paint, text, padding, gap, flex/grid mode, alignment, clipping, and semantic child order are editable projections.
- In a linear layout, the enabled main-axis gap in Penpot's layout inspector maps to the native container's `gap`. Its disabled cross-axis value does not create an ineffective `row_gap` or `column_gap` override. Grid gaps use `row_gap` and `column_gap`; FlowBox and WrapBox gaps use `vertical_gap` and `horizontal_gap`. Unsupported independent cross-axis edits block export.
- Auto-layout-derived positions and fill dimensions are not treated as authored edits. A free/auto transition materializes the child geometry that becomes semantically active, including an existing child-mount `slot.layout` override, while preserving unrelated slot fields.
- Supported edits trigger serialized native reflow. Layout audits and exports wait for that reflow to settle; unsupported edits fail before the renderer can overwrite them. Reflow preserves the original source, capture baselines, and export guard.
- Native size allocations from flow and Free/Overlay parents are recorded separately from authored dimensions. Capture retains manual dimensions through reflow while the parent and allocation kind remain the same; temporary helper sizes do not become source edits. Reparenting or switching between flow and Free/Overlay allocation starts a new dimension record.
- Imports, tokens, events, bindings, repeat contracts, slots, component definitions, style scopes, and unknown fields are retained in the source document metadata.
- Catalog imports keep the exact runtime source separately from the expanded design projection. No-edit export restores those original bytes, including comments and legacy fixture syntax. Mapped instance overrides are applied to the original nodes; edits to unmapped expanded internals or legacy fixtures fail explicitly.
- Applying a mapped visual delta preserves the original TOML table form, comments, and unrelated fields, including fields authored under dotted tables such as `[nodes.root.layout.container]`.
- Leaf component references become native Penpot library components and linked copies. Instance metadata retains the source node and prefab owner. Composite slot trees remain semantic boards; canonical default masters and editable composite components are still pending.
- Slot padding uses native child margins and resolved child dimensions to preserve flow and cross-axis insets. Hosts without independent-margin support reject asymmetric initial padding; supported side edits preserve the other source fields and tokens.
- Intrinsic content sizes are measured before allocation. Invisible font probes track text, rich runs, and resolved typography when a review board is reused; painter geometry and previous fill dimensions do not supply intrinsic text size.
- Scroll children retain their desired extent along the scrolling axis. Wrap children resolve row widths from their content minimums and axis constraints while preserving fixed content heights; resizing the parent recomputes the rows. Linear content mounts can use `slot.layout.linear_size.rule = "Auto"` to keep cards and property rows at their desired height.
- Native Wrap derives its parent alignment. The imported source alignment is preserved; unsupported parent alignment edits block capture. Supported child slot alignment remains available.
- Popup preview refreshes reuse existing option rows and labels. Their renderer-allocated relative geometry can follow a supported source layout change; direct edits to preview geometry or labels remain unmapped and block export. Changing option cardinality or kind requires import or review setup rather than live authoring.
- The public layout audit includes content measurements and stable source identities. Passing projection, host, or browser checks remains separate from native engine screenshot acceptance.
- Missing, duplicate, cyclic, or metadata-inconsistent semantic nodes block export.
- Mounted layout measurements retain the source file, source node, control, instance ancestry, and parent owner. Missing identities remain incomplete for strict engine comparison.
- Browser layout checks assign a unique `requestId` to each public `audit-preview-layout` message and consume only the matching `preview-audit` response. Fixed delays and previously rendered status attributes are not fresh layout evidence.
- Generated SVG icons use shape-relative path coordinates in the export guard, so ancestor layout translation preserves their source mapping. Unmapped path, paint, and relative geometry edits still block export. Re-import boards with older guards before exporting with the updated plugin.
- Export guard version 5 validates current invisible font probes and recorded popup geometry and unmapped popup-label typography without replacing its immutable baseline. Re-import assets created with older guard versions.
- Token expressions and runtime controls remain metadata-backed until an explicit mapping exists. Unsupported or mixed text/paint, unrepresentable gaps, and layout sizing changes outside the supported profile block export instead of silently losing meaning.

## Headless Bridge

The CLI uses the same parser, projection, and reconcile implementation as the plugin:

```powershell
pnpm --filter zircon-zui-plugin bridge project input.zui output.penpot.json
pnpm --filter zircon-zui-plugin bridge reconcile output.penpot.json rebuilt.zui
pnpm --filter zircon-zui-plugin bridge roundtrip input.zui rebuilt.zui
```

The JSON bridge file is a test and automation format, not a replacement for Penpot's native file format or Zircon's `.zui` format.

## Verification

Run the complete plugin gate from the ZirconEngine root:

```powershell
& tools/penpot/tools/scripts/run-validation.ps1
```

All dependency installations, tests and compiler outputs run in the staged
workspace under `D/E/F:\cargo-targets\zircon-local`. The command writes input
hashes and a validation receipt alongside the artifacts.

Unit tests cover reversible field mapping, native container gaps, layout mode transitions, and original-source application. The isolated host contract executes the actual plugin entry through Penpot's official `plugins-runtime` SES sandbox. The separate browser contract uses the official frontend with local RPC fixtures and covers source roundtrip, slider/segmented text edits, actual slot-margin coordinates, and native prefab instances with editable text. These checks do not establish engine rendering or full-catalog visual acceptance.

The full workbench export/apply contract edits spacing through Penpot's normal layer selection and layout inspector. It uses measured source ownership to choose the target; the plugin API stays in the runtime sandbox. Engine acceptance still requires applying the downloaded delta to its original `.zui`, syncing the product resource bundle, restarting the engine, and checking fresh geometry and screenshots against the same Penpot state.

## Native Editor DPI capture

Run the current Editor UI visual capture entry under `tools/analysis/visual` with `-DpiProfile 100` and `-DpiProfile 150` in separate governed output directories, using current managed Editor/runtime binary hashes and the visual source hash. Both profiles are required for workbench acceptance; the default profile is `150`.

| Profile | Case | Logical size | Physical pixels | Actual window DPI |
| --- | --- | --- | --- | --- |
| 100 | 1280x800 | 1280×800 | 1280×800 | 96 |
| 100 | 900x620 | 900×620 | 900×620 | 96 |
| 100 | 640x520 | 640×520 | 640×520 | 96 |
| 150 | 1280x800-dpi150 | 1280×800 | 1920×1200 | 144 |

The selected profile must match the real display hosting the Editor. Capture checks `GetDpiForWindow`, Winit scale, logical size, and physical client extent before and after positioning/interactions. DPI metadata cannot substitute for a real window at that scale. The `editor-ui-visual-matrix.ps1` module is included in the visual source fingerprint and capture manifest; changing its bytes invalidates prior evidence. `-SkipVisualOracle` leaves acceptance pending.
