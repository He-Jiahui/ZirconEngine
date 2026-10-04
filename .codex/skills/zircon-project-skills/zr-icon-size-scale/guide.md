# Zircon Icon Size Scale

Use this skill for icons in Hub, `.zui` assets, retained UI painters, and native UI rendering. It does not govern illustrations, photographs, preview thumbnails, or world-space assets.

## Rule

Author an icon's role with a named tier, never a one-off numeric size. Use only `s`, `m`, `l`, or `xl`; nearby visual sizes must converge to one of those tiers. The renderer may map a tier to logical pixels internally, but authored UI must not guess or tune icon dimensions one pixel at a time.

Choose the tier from the icon's role:

- `s`: inline metadata, compact rows, and dense affordances.
- `m`: normal compact controls, menus, and form-leading icons.
- `l`: primary toolbar/action icons and normal icon buttons.
- `xl`: focal empty/status icons or deliberately prominent visual feedback.

Use the exact scale and migration rules in [the icon-scale reference](references/icon-scale.md). Pair this with `zr-flow-layout-semantics`: icon size is a semantic visual token, not a substitute for moving a control into place.
