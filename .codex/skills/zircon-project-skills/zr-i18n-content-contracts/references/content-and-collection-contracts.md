# Localized Content And Collection Contracts

## Existing localization path

ZUI supports a localized text reference such as:

```toml
props = { text = { text_key = "editor.asset_browser.title", table = "editor" } }
```

The compiled value is the repository's `UiLocalizedTextRef` (`key` is its
runtime field); the TOML adapter accepts `text_key`. If a higher-level adapter
exposes another alias, convert it at that adapter boundary instead of
inventing a second runtime contract.

`UiLocalizedTextRef`, `collect_document_localization_report`, and the locale catalog are the authority for key discovery and validation. A fallback is a migration aid, not a place to hide a new production string. Ensure the selected locale catalog owns the key before accepting the layout.

Component inputs that can change at runtime must remain props, bindings, or state values. That includes list items, filter labels, user-entered text, approval scope, action labels, status descriptions, empty-state recovery content, tooltips, and accessible names. The same localization rule applies to `accessibility_label`, `aria-label`, `title`, and alternate text; they are not an escape hatch for literals. A native painter may render provided content but must not synthesize an English visible or accessible fallback.

## Fixture boundary

`tests/**`, including `tests/fixtures/**`, may contain literal examples because they are test inputs. Mark fixture-only sample collections clearly and assert the behavior they demonstrate. Do not copy those literals into production defaults or generic component descriptors.

## Collection decision record

Before implementation, record one answer for each collection:

| Question | Expected decision |
| --- | --- |
| Can it grow without a practical bound? | Virtualize or page it. |
| Is a fixed viewport required? | Make that region the one scroll owner and expose scroll affordance. |
| Is the collection naturally small and finite? | Allow content-driven expansion, with a tested maximum. |
| Does content need to size the parent or fill an available slot? | Choose intrinsic expansion versus cross-axis `stretch`, then test both the smallest and largest supported extent. |
| Does a row contain a long label or localized value? | Define wrapping/elision and preserve the row's semantic alignment. |

For retained fixed-row virtual lists, use `ScrollableBox` with `container.virtualization = { item_extent = ..., overscan = ... }`. The current invalidation diagnostic warns for a scrollable container with 250 or more direct children without virtualization; treat that as a lower bound, not permission to materialize an otherwise unbounded collection.

## Minimum regression coverage

1. Localization key is collected and resolves in its intended catalog.
2. Caller-provided/editable text replaces the default without a source change.
3. Empty, one-item, normal, and over-capacity collections preserve the documented policy.
4. A narrow viewport and long localized text do not overlap unrelated controls.
5. Parent intrinsic expansion and available-slot `stretch` follow the documented choice at minimum and maximum content extents.
6. A scrollable or virtualized collection renders only its intended window and preserves the overall content extent.
