# Editor UI UseMediaQuery Responsive Design

## Summary

Advance the editor MUI responsive UI lane by evaluating retained `UseMediaQuery` nodes during the runtime layout responsive pre-pass. The result should update a stable `matches` value from the current root viewport width and let existing templates/styles bind that value into `display`, `visibility`, or legacy `visible` attributes without creating an editor-only responsive path.

## Scope

- Target the existing runtime UI responsive pipeline under `zircon_runtime/src/ui/layout/pass/responsive_mui.rs`.
- Reuse Material UI default breakpoints already used by the Grid/Stack/Masonry responsive pre-pass: `xs=0`, `sm=600`, `md=900`, `lg=1200`, `xl=1536`.
- Keep `zircon_editor` as a consumer of shared runtime behavior through retained `.zui`/`.v2.ui.toml` assets.
- Do not change Hub Slint UI, GPU presenter ownership, command-stream batching, or unrelated MUI painter primitives in this slice.

## Architecture

`UseMediaQuery` remains a behavior utility component, not a drawn widget. Runtime layout computes its `matches` value before container, slot, and visibility resolution so downstream authored attributes can consume the result in the same layout frame.

The pre-pass should:

- Find retained nodes whose template metadata component is `UseMediaQuery`.
- Evaluate supported query forms against the current root viewport width.
- Write the computed boolean into the node's runtime metadata/attributes as `matches`.
- Preserve explicit authored `matches` only as a fallback when no query is present or the query is unsupported.
- Mark render/hit/input/layout dirty domains only when `matches` actually changes and a dependent visibility or layout result changes.

Initial query support stays intentionally narrow and deterministic:

- `(min-width: Npx)`
- `(max-width: Npx)`
- MUI shorthand props such as `min_width`, `max_width`, `breakpoint`, or `up`/`down` if already represented as retained attributes.
- Unsupported query strings fall back to `defaultMatches` / `default_matches`, then existing `matches`.

## Data Flow

1. Template compilation keeps `UseMediaQuery` descriptor props and classes as today.
2. Surface build projects the node into `UiTreeNode.template_metadata.attributes`.
3. `apply_mui_responsive_layout(...)` builds one viewport context from root size.
4. The pre-pass evaluates `UseMediaQuery` nodes and updates `matches`.
5. Existing responsive visibility/container/slot logic runs after that and consumes current attributes.
6. Editor retained UI and Material Lab observe ordinary runtime node state and styling, not a special editor branch.

## Error Handling

Unsupported or malformed query values must not panic or poison layout. They should choose a stable fallback in this order:

1. `defaultMatches`
2. `default_matches`
3. existing `matches`
4. `false`

Non-finite root widths are already normalized to `0.0` by the viewport context.

## Testing

Add focused runtime tests beside the existing MUI responsive layout tests:

- `UseMediaQuery` flips `matches` from false to true when root width crosses `(min-width: 900px)`.
- `UseMediaQuery` flips true to false for `(max-width: 899px)`.
- `display`/`visibility`/`visible` bindings that depend on `matches` update in the same compute-layout frame.
- Unsupported queries use `defaultMatches` without changing layout unexpectedly.

Validation target:

- `rustfmt --edition 2021 --check` on touched runtime UI responsive files and tests.
- `cargo test -p zircon_runtime --lib mui_responsive_layout --locked --jobs 1 --message-format short --color never`, or focused exact filters if shared workspace load blocks broad filtered execution.
- Update `docs/zircon_runtime/ui/layout/pass.md` with the new UseMediaQuery behavior and validation notes.

## Acceptance

The slice is complete when editor/runtime MUI responsive assets can express viewport-dependent UI using `UseMediaQuery` without editor-private code, and the runtime layout pass keeps one source of truth for Grid/Stack/Masonry, visibility, and query-driven responsive state.
