---
related_code:
  - zircon_runtime_interface/src/ui/template/asset/document.rs
  - zircon_runtime/src/ui/template/asset/component_contract/validation.rs
  - zircon_runtime/src/ui/template/asset/compiler/component_instance_expander.rs
  - zircon_runtime/src/ui/template/asset/compiler/prototype_instancer.rs
  - zircon_runtime/src/ui/template/asset/prototype_store.rs
  - zircon_runtime/src/ui/v2/component_reference.rs
  - zircon_runtime/src/ui/v2/cache.rs
  - zircon_runtime/src/ui/v2/component_instancer.rs
  - zircon_runtime/src/ui/v2/file_cache.rs
  - zircon_editor/src/ui/template/catalog.rs
  - zircon_editor/src/ui/template_runtime/builtin/component_descriptors.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/value_media/icon.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tooltips/commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_alerts/inline.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_alerts/toast/entry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons/content/metrics.rs
  - zircon_editor/assets/ui/editor/components/workbench/modules
  - zircon_editor/assets/ui/editor/components/catalog.toml
implementation_files:
  - zircon_editor/src/ui/template/catalog.rs
  - zircon_editor/src/ui/template_runtime/builtin/component_descriptors.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/value_media/icon.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tooltips/commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_alerts/inline.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_alerts/toast/entry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons/content/metrics.rs
  - zircon_editor/assets/ui/editor/components/workbench/modules
  - zircon_editor/assets/ui/editor/components/catalog.toml
plan_sources:
  - docs/plans/zircon_editor/editor_layout/12-widget-slot-componentization.md
  - docs/plans/zircon_editor/editor_layout/13-taffy-css-constraint-language.md
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
tests:
  - zircon_editor/src/tests/ui/template/catalog_registry.rs
  - zircon_editor/src/ui/template_runtime/builtin/component_descriptors.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/tests/visual_metadata.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tooltips_tests/adaptive.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_alerts_tests/adaptive.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons_tests/paint.rs
  - zircon_editor/src/tests/ui/boundary/zui_asset_governance/workbench_modules.rs
doc_type: module-detail
---

# Component Composition Contract

Editor UI composition has three tiers. Primitives provide one visual or interactive capability;
composites arrange primitives through named slots; region panels compose reusable controls into a
workbench region. A page must fill existing composite or region slots before adding a new
primitive.

`UiNamedSlotSchema` is the shared asset-level slot authority. It preserves the existing
`required` and `multiple` contract and adds two backward-compatible fields:

| Field | Meaning | Legacy default |
| --- | --- | --- |
| `kind` | Intended `UiSlotKind` for the slot's child-layout family | absent; parent container inference remains authoritative |
| `accepts` | Direct child component identities accepted by the slot | empty set means `Any` |

Every asset compiler path validates explicit `accepts` values before expansion. A direct child is
identified by its local component name, imported component suffix, or native widget type. This
keeps validation independent of the editor crate and prevents a second template compiler policy.
An empty accept set deliberately preserves all existing assets.

An imported `component_ref` has exactly one non-empty `asset#Component` fragment. The ordinary
and prototype expanders validate that syntax before resolving a slot fill. V2 adapts the same
parser at its error boundary: whole-asset widget imports remain valid dependencies, while a named
import or explicit component reference with an extra fragment fails as `InvalidDocument` before
prototype lookup. An ambiguous extra fragment therefore cannot pass accept-set validation under a
different identity than expansion.

`EditorComponentCatalog` mirrors the author-facing composition vocabulary with
`EditorComponentTier`, `EditorSlotContract`, and `EditorPropContract`. Every prop records its
declared value type and an `EditorPropDefault`: `Literal` preserves a local TOML text, boolean,
integer, float, or string-list value; `Token` records a token-backed default; and `None` leaves
the value to the runtime's component parameter schema. The legacy serialized `default_token`
field remains readable and migrates to `Token` during catalog deserialization. The catalog does
not select a layout solver: slot-to-family mapping remains the runtime-interface
`UiSlotKind::layout_engine_family` contract consumed by the layout pipeline.

## Builtin Catalog Coverage

`catalog.toml` is the versioned, packaged typed-metadata source of truth for builtin descriptors.
It is intentionally not a UI asset descriptor: it declares component identity and composition
contracts, while every visual document it references remains a `.zui` asset.
Reusable entries under `res://ui/editor/components/` resolve to an explicit
`[components.<component_id>]` declaration. Host view entries remain semantic catalog identities
backed by their retained-host projection and binding namespace, rather than pretending every view
is a reusable wrapper component.
`builtin_component_descriptors` parses it before host-template registration, rejects unsupported
versions and duplicate identities, and returns the same entries for runtime registration. Every
asset under `components/workbench/primitives` appears as an `EditorComponentTier::Primitive`; its
document ID is `res://ui/editor/components/workbench/primitives/<path>` and its binding namespace
remains the component identity. Regression tests compare this loaded inventory against the
physical primitive asset tree and require a one-to-one 45-entry match.

Host, composite, and region-panel entries also live in the asset so named slots and parameter
defaults are visible to authors without introducing a second Rust-side list. Concrete visual token
defaults stay with component assets; the catalog records only composition contracts and the token
reference needed by the component interface.

## Implementation Status

Completed in the runtime and editor catalog:

- `UiNamedSlotSchema` carries backward-compatible `kind` and `accepts` metadata, and ordinary,
  prototype, and V2 expansion reject an explicit accept-set mismatch.
- `parse_component_reference` is the interface-level authority for `asset#Component` syntax.
  Component-contract preflight, both asset expansion paths, V2 prototype-store construction, V2
  component expansion, and V2 file-source discovery use it through their local error boundary.
  An ambiguous reference with extra fragments consistently fails as `InvalidDocument` before
  import lookup; V2 slot `accepts` compares the parsed component identity rather than an
  `asset#Component` locator; a V2 whole-asset widget import remains a checked dependency.
- The editor catalog records component tier, named-slot contracts, typed defaults, and preserves
  legacy `default_token` input without retaining conflicting token metadata after migration.
- `catalog.toml` is parsed as the versioned builtin component manifest before components are
  registered. Its parser rejects unsupported versions, empty payloads, and duplicate component
  identities, slot or prop names, non-`.zui` or non-`res://ui/editor/` document references,
  unknown schema fields, or malformed token defaults; runtime registration consumes this typed
  metadata rather than a parallel hard-coded descriptor list. The builtin descriptor leaf also
  verifies every referenced packaged file exists.
- The former `catalog.v2.ui.toml` descriptor-style path has been hard-cut to `catalog.toml`.
  Catalog APIs and host error propagation use `Manifest` terminology only, so the retired suffix
  cannot re-enter through a compatibility alias. The current manifest covers 69 components and
  every document reference ends in `.zui`.
- The catalog now exposes authored content and interaction contracts for the core primitive
  families: alert, label, caption, chip, icon, divider, list row, property row, section title,
  status item, toast, button, icon button, field, dropdown, number field, checkbox, radio,
  command palette, confirmation dialog, context menu, dialog, drag overlay, dropdown popup,
  notification center, popup menu, progress bar, range slider, search input, segmented control,
  skeleton, slider, tab, toggle, and tooltip. It records their authored content, interaction,
  selection, and visible-state inputs and defaults, leaving visual token and layout ownership in
  the referenced primitive `.zui` documents.
- `show_icon` (or its serialized `showIcon` spelling) is a shared visible-content contract. A
  literal `false` clears the existing projected `icon_name`; Tooltip, inline Alert, and Toast
  painters then omit their optional glyph and start text at the component's normal left inset.
  An omitted or `true` value retains the authored icon and its original text budget.
- `WorkbenchButtonKind::Primary` is a text hierarchy contract as well as a surface contract. Its
  label uses the Runtime Text `strong` run style, including during intrinsic-width measurement;
  secondary, tertiary, and danger buttons remain regular unless their authored `font_weight` is
  explicitly strong. This preserves semantic emphasis without changing button geometry or
  introducing per-template font-family overrides.
- All Workbench module buttons inherit their surface and border from the shared semantic selector.
  The 145 current module buttons may declare their `button_variant` and `button_color`, but none
  may bypass the selector with raw `background_color` or `border_color` values. The 52 module
  canvases similarly inherit their recessed surface, soft border, 1px border width, and compact
  radius from `.workbench-module-canvas`. This keeps the Unreal-derived normal, hover, pressed,
  disabled, and focused state matrix and the module content hierarchy consistent across pages.
- `WorkbenchComponentDrawer` is an integration sample, not a parallel style system. Its button,
  icon-button, input, selection, list, table, feedback, and text samples provide only content and
  state data to their referenced primitives. Its nine sample containers use the shared
  `.workbench-component-sample-card` chrome, which owns the panel surface, 1px border, and 4px
  radius. This prevents the component atlas from bypassing Runtime Text, atomic state, or
  responsive layout contracts with local visual and pixel-offset props.
- Static status, 2026-08-02: the manifest's 69 document references all resolve under
  `res://ui/editor/` to packaged `.zui` files; its 110 explicit parameter defaults include five
  token-backed values that all retain `$` syntax. `rustfmt --check`, scoped diff checks, and
  retired catalog-symbol scans pass. Cargo remains deferred to managed milestone verification;
  no screenshot or `target/` artifact was generated.
- Builtin descriptors classify host/composite/region entries and catalog all 45 physical
  workbench primitive assets. The regression test scans the asset tree and requires an exact
  one-to-one catalog match.
- `ActivityDrawerWindow` declares all seven activity/content slots as multi-child container
  slots accepting only `Container`. Both production consumers mount only container panels; the
  asset test freezes the exact seven-slot schema, compiles both consumers through the V2 prototype
  store, and proves an injected `IconButton` fill is rejected.
- `WorkbenchPanelHeader` declares a required single-child linear `title` slot accepting the shared
  caption or section-title atoms, plus a multi-child linear `actions` slot accepting standard
  header action atoms. The packed catalog carries the same two contracts, and governance tests
  freeze both the asset declaration and catalog metadata.
- `WorkbenchPropertyEditorRow` declares a required single-child linear `value` slot for standard
  property editors (field, dropdown, toggle, slider, range slider, checkbox, or number field).
  Blend Space details continues to populate it with shared fields and dropdowns; the same asset
  and catalog contract is available to Inspector and extension panels.
- `WorkbenchTabStrip` keeps a multi-child linear `default` slot, now accepting only shared
  `WorkbenchTab` atoms. Unnamed child mounts resolve to this default slot, so tab bars remain
  declarative while rejecting arbitrary controls in the tab lane.
- `WorkbenchDiagnosticRow` declares required, single-child linear `severity` and `message` slots;
  both only accept `WorkbenchStatusItem`. The shared validation-log consumer mounts its two
  Runtime Text status columns through those slots at the shared 28px row-height token, so
  arbitrary visual controls cannot enter a diagnostic row or introduce a local density.
