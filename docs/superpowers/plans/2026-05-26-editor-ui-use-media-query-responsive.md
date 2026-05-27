# Editor UI UseMediaQuery Responsive Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Evaluate MUI `UseMediaQuery` in the shared runtime responsive layout pre-pass so editor UI assets can drive viewport-dependent layout and visibility without editor-private code.

**Architecture:** Keep the implementation inside `zircon_runtime/src/ui/layout/pass/responsive_mui.rs`, which already owns viewport-time MUI breakpoint resolution for Grid, Stack, Masonry, and visibility. `UseMediaQuery` remains a behavior utility component; the pre-pass writes a computed `matches` boolean into template metadata attributes before existing visibility/container/slot resolution runs. Editor assets and tests consume this through ordinary retained runtime metadata.

**Tech Stack:** Rust, `zircon_runtime`, `zircon_runtime_interface` UI tree DTOs, TOML `Value`, existing MUI responsive layout tests and docs.

---

## Current Baseline

- `apply_mui_responsive_layout(...)` already runs before full and incremental measurement via `layout_tree.rs` and `incremental.rs`.
- `responsive_mui.rs` already normalizes the root width and resolves MUI breakpoint-shaped scalar/table/array values.
- `style_apply/mui_layout_classes.rs` emits static `MuiUseMediaQuery-match` when authored `matches` or `defaultMatches` is true, but runtime layout does not yet compute `matches` from `query`.
- `UiTemplateNodeMetadata.attributes` is mutable at runtime and is already used by property mutation paths for dynamic retained properties.

## Files

- Modify: `zircon_runtime/src/ui/layout/pass/responsive_mui.rs`
  - Add a first pre-pass that evaluates `UseMediaQuery` nodes and updates `metadata.attributes["matches"]`.
  - Add small helpers for supported query parsing and fallback matching.
- Modify: `zircon_runtime/src/ui/tests/mui_responsive_layout.rs`
  - Add v2 surface coverage for `UseMediaQuery` `min-width`, `max-width`, and fallback behavior.
  - Assert that query-driven visibility changes happen in the same `compute_layout(...)` frame.
- Modify: `zircon_runtime/src/ui/tests/template_grid_flow.rs`
  - Add legacy template coverage for the same metadata path so both v1 template and v2 asset builders are protected.
- Modify: `docs/zircon_runtime/ui/layout/pass.md`
  - Document `UseMediaQuery` runtime evaluation and record validation evidence.
- Modify: `.codex/sessions/20260526-1815-editor-ui-continuation.md`
  - Update current step, touched modules, and validation notes.

## Milestone 1: Runtime UseMediaQuery Evaluation

- Goal: `UseMediaQuery` nodes compute `matches` from the current root width before responsive visibility/container/slot logic runs.
- In-scope behaviors:
  - `(min-width: Npx)` returns true when `viewport.width >= N`.
  - `(max-width: Npx)` returns true when `viewport.width <= N`.
  - Numeric fallback props `min_width` / `max_width` support the same comparisons when no supported query string exists.
  - Unsupported or malformed queries fall back to `defaultMatches`, then `default_matches`, then existing `matches`, then `false`.
  - Recomputing the same width does not dirty unchanged nodes.
- Dependencies:
  - Existing `MuiResponsiveViewport` width normalization and `UiTemplateNodeMetadata.attributes` mutation.
  - Existing `apply_responsive_visibility(...)` running after the query pass.

### Implementation Slices

- [ ] Add `apply_use_media_query_matches(tree, viewport)?` as the first call inside `apply_mui_responsive_layout(...)`.
- [ ] Implement `use_media_query_match_for_node(...)` that:
  - returns `Ok(None)` for non-`UseMediaQuery` nodes,
  - reads `query`,
  - parses `(min-width: Npx)` and `(max-width: Npx)`,
  - falls back through `defaultMatches`, `default_matches`, `matches`, and `false`.
- [ ] Update the node only when `metadata.attributes["matches"]` differs from the computed TOML boolean.
- [ ] On match changes, mark the node dirty for render/hit/input and layout. Use layout dirtiness because downstream responsive visibility may collapse or reveal dependent nodes during the same frame.
- [ ] Add tests in `mui_responsive_layout.rs` for:
  - `UseMediaQuery` with `(min-width: 900px)` false at width `720` and true at width `960`.
  - `UseMediaQuery` with `(max-width: 899px)` true at width `720` and false at width `960`.
  - a node whose `display` uses breakpoint metadata still updates in the same frame after query evaluation.
  - unsupported query uses `defaultMatches = true` and stays stable.
- [ ] Add equivalent legacy-template assertions in `template_grid_flow.rs` by extending `GRID_FLOW_TEMPLATE_TOML` with `UseMediaQuery` nodes and asserting computed metadata after `compute_layout(...)`.
- [ ] Update `docs/zircon_runtime/ui/layout/pass.md` to describe query evaluation order, supported query syntax, fallback behavior, and dirty-domain semantics.
- [ ] Update `.codex/sessions/20260526-1815-editor-ui-continuation.md` with new touched modules and expected validation.

### Testing Stage: UseMediaQuery Responsive Gate

Run these after implementation slices are complete:

```powershell
rustfmt --edition 2021 --check zircon_runtime/src/ui/layout/pass/responsive_mui.rs zircon_runtime/src/ui/tests/mui_responsive_layout.rs zircon_runtime/src/ui/tests/template_grid_flow.rs
```

Expected: exit code `0`.

```powershell
cargo test -p zircon_runtime --lib mui_responsive_layout --locked --jobs 1 --message-format short --color never
```

Expected: the focused `mui_responsive_layout` tests pass. Existing unrelated warning noise is acceptable; Rust errors or test failures are not.

```powershell
cargo test -p zircon_runtime --lib template_mui_responsive_layout_recomputes_from_viewport_breakpoints --locked --jobs 1 --message-format short --color never
```

Expected: the legacy template responsive test passes.

```powershell
cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never
```

Expected: exit code `0`. Existing warning noise is acceptable.

```powershell
git diff --check -- zircon_runtime/src/ui/layout/pass/responsive_mui.rs zircon_runtime/src/ui/tests/mui_responsive_layout.rs zircon_runtime/src/ui/tests/template_grid_flow.rs docs/zircon_runtime/ui/layout/pass.md docs/superpowers/specs/2026-05-26-editor-ui-use-media-query-responsive-design.md docs/superpowers/plans/2026-05-26-editor-ui-use-media-query-responsive.md .codex/sessions/20260526-1815-editor-ui-continuation.md
```

Expected: no task-owned whitespace errors. Repository LF/CRLF notices may appear and should be reported separately if present.

### Debug / Correction Loop

- If a test fails because metadata attributes do not update, inspect whether the node is built through v1 `UiTemplateTreeBuilder` or v2 `build_tree_from_arena(...)`; both should project props into `UiTemplateNodeMetadata.attributes`.
- If query parsing fails, add a focused helper case for the exact string format instead of broad CSS media-query parsing.
- If layout dirtiness causes unrelated snapshot drift, reduce the dirty domains only after verifying visibility-dependent tests still update in the same frame.
- If Cargo is blocked by unrelated shared workspace load, record the blocker and keep static/rustfmt/diff evidence, but do not claim Cargo tests passed.

### Exit Evidence

Milestone 1 can be promoted only when:

- Runtime code and tests are formatted.
- Focused runtime MUI responsive tests pass or a concrete unrelated blocker is recorded.
- Runtime lib check passes or a concrete unrelated blocker is recorded.
- `docs/zircon_runtime/ui/layout/pass.md` includes the new behavior and validation notes.
- The session note reflects current status and remaining risk.
