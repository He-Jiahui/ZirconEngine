# Zircon I18n And Content Contracts

Use this skill for user-facing Hub, editor, runtime, and `.zui` UI content. It applies to visible strings, descriptions, action labels, placeholders, empty states, collection rows, and editable values. It does not require translating diagnostic identifiers, route IDs, test names, source comments, or fixture-only test data.

## Content rule

- New or changed production UI must use the existing localization contract (`UiLocalizedTextRef.key`, optional table, and locale catalog) for every user-facing string: visible copy, placeholders, validation text, tooltips, titles, `aria`/accessibility names, and alternate text. If a higher-level adapter calls the field `text_key`, translate that alias at the adapter boundary. Do not add a direct English/Chinese string merely because a source fixture happened to contain it.
- Treat user-editable, host-provided, configuration-provided, or extensible content as a prop/binding/state value. Component defaults may define behavior, but they must not invent visible product copy when the caller has not supplied it.
- Literal samples belong under a test fixture or test builder with an explicit test purpose. They are not production fallback data and must not become the default collection rendered to users.
- Keep independently interactive labels separate. Keep ordinary prose, including inline emphasis, as one localized rich-text source so translators retain word order and punctuation.

## Collection rule

For every new or changed collection, decide and test whether it expands its parent, stretches into an available slot, is bounded and scrolls, pages, or virtualizes. Test empty, one, normal, and over-capacity data; include a narrow container and a long localized label/value. For large or unbounded data, use the retained `ScrollableBox` virtualization contract instead of materializing every row.

Read [the implementation reference](references/content-and-collection-contracts.md) before changing a UI asset, component descriptor, or native semantic painter. Pair this skill with `zr-flow-layout-semantics` for layout and `zr-icon-size-scale` for visual assets.
