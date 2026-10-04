# Translating the references into ZirconEngine

ReactBits supplies the interaction vocabulary. ZirconEngine supplies the actual ownership, tokens, geometry, accessibility, and platform behavior. Use both layers explicitly.

For section-level composition, start at the [ReactBits Blocks catalogue](https://pro.reactbits.dev/docs/blocks); for product behavior, start at the [Application UI catalogue](https://pro.reactbits.dev/docs/app-ui). A Blocks variant can establish hierarchy and rhythm, but it never replaces the local ZUI/Hub ownership or an Application UI reference for interactive states.

## Local authority by surface

| Surface | Inspect first | Typical ReactBits families |
| --- | --- | --- |
| Hub frame and navigation | `zircon_hub/web/src/components/shell/HubWindow.tsx`, `TopBar.tsx`, `NavigationDrawer.tsx`, `zircon_hub/web/src/theme/tokens.ts` | App Shell, App Sidebar, Navbar, Mobile, Command Menu |
| Hub pages and shared regions | `zircon_hub/web/src/pages/**`, `zircon_hub/web/src/components/{data,inputs,overlays,feedback}/**` | Cards, Data Table, Dashboard, List, Filtering, File Manager, Empty State, Settings Form, Forms, App Dialog, Notifications |
| Hub visual reference path | `docs/ui-and-layout/hub.png`, `docs/ui-and-layout/hub-ai-reference-manifest.json`, `docs/ui-and-layout/hub-web-reference/**`, `docs/zircon_hub/ui/responsive-component-system.md` | Use the matching shell/data/state families from the map |
| Editor workbench shell | `zircon_editor/assets/ui/editor/host/workbench_shell.zui`, `zircon_editor/assets/ui/editor/windows/workbench_window.zui`, `zircon_editor/assets/ui/theme/editor_workbench_strict.zui`, and the retained-host projection under `zircon_editor/src/ui/retained_host/**` | App Shell 4, App Sidebar, Navbar, App Dialog, Command Menu, Mobile |
| Editor design language and geometry | `docs/ui-and-layout/design-language-contract.md`, `docs/ui-and-layout/componentized-workbench-shell.md`, `docs/ui-and-layout/editor-workbench-design-export.md`, `docs/ui-and-layout/workbench.png` | Shell, Cards, Data Table, List, Monitoring, Agent Activity |
| Shared runtime UI | `zircon_runtime/src/ui/**`, `zircon_runtime_interface/src/ui/**`, `docs/ui-and-layout/shared-ui-core-foundation.md` | Use the closest family, then preserve the shared retained-tree/layout/input contracts |

Candidate owners are routing hints, not permission to bypass a module boundary. Read the current source and its contract tests before selecting an edit point.

## Layout translation rules

1. **Frame first.** Establish the outer app shell, navigation ownership, header/toolbars, and content viewport before adding cards or controls. A three-column reference should become explicit left/context/main slots with a defined collapse path, not three unrelated absolute-positioned panels.
2. **Data shape drives primitive.** Use a table for comparable columns and row actions, a list/tree for ordered or hierarchical activity, cards for bounded summaries, and a dashboard only when metrics/charts are the task. Do not turn a table into a card grid solely to avoid responsive work.
3. **Progressive disclosure is structural.** Put arguments, citations, tool output, diffs, metadata, and run substeps behind clearly discoverable disclosure rows or detail panels. Keep the summary/action row usable when details are collapsed.
4. **State is part of the layout.** Model the applicable idle, empty, loading, progress, success, warning, error, retry, and canceled states before styling. Async or agent work must reserve a stable place for progress and cancellation; destructive work must expose impact, scope, and the confirmation decision in the same flow.
5. **Responsive behavior changes priority, not meaning.** Start with the existing Hub `1568x1003` reference canvas or the editor's documented workbench tiers, then define what collapses, stacks, scrolls, or moves into a drawer/sheet/icon rail. Keep the primary action and current context available at every supported width. Never solve overflow by clipping labels or placing controls at guessed screen coordinates.
6. **Scroll ownership is singular.** Assign scrolling to the content region that owns the long list/log/form. Sticky headers, toolbars, result counts, and action bars may remain fixed within that region, but avoid nested scroll containers that steal pointer or keyboard focus.
7. **Shared visual language wins.** Consume existing Hub tokens or editor design tokens for surfaces, typography, radius, density, borders, focus, and semantic status. ReactBits' visual treatment is a reference for hierarchy and rhythm, not a second palette.
8. **Interaction semantics are visible.** Preserve stable control identity, keyboard navigation, focus indication/restoration, escape and click-away behavior for overlays, accessible names, live-region/status updates, and permission/risk copy. A visually similar layout that hides an approval or retry path is not an acceptable translation.
9. **Reference images are not runtime UI.** `hub.png`, AI drafts, design-board sheets, and `workbench.png` can guide or compare layout, but must not be rendered as the product shell or used to mask missing component structure.

10. **Project semantic content, not screenshots.** Agent/chat layouts should carry message arrays, composer text, streaming/error state, and resolved text style through the retained projection. Native painters may derive geometry from those fields, but must emit real text/render commands with a clip and a reserved action area. A Penpot mirror is useful for visual comparison; it is not a substitute for the native command contract.

## Mapping record

Include a short record in the active plan, review, or handoff. Fill only the state and validation fields that apply, but always name a primary ReactBits link.

```text
Surface / route:
Primary ReactBits reference:
Supporting references (state or interaction):
Local owner and authority files:
Regions and slot ownership:
Responsive transitions (wide / regular / compact / mobile):
Applicable states (empty / loading / progress / error / approval / etc.):
Keyboard, focus, and permission behavior:
Validation commands and evidence:
```

For a compound surface, prefer a small composition such as `App Shell + Data Table + Filtering + Empty State` or `AI Chat 1 + Agent Activity 2 + Tool Calls 2 + Agent Approval`. Do not claim that a reference was followed if the mapping record is absent or only names a generic aesthetic.

For the current agent workspace slice, the semantic handoff is `AgentChat.messages -> collection_items` and `ChatComposer.composer_text -> value_text`; keep this mapping stable when adding markdown, citations, attachments, or richer message rows.

## Current reference fixtures

Use these project-owned fixtures as executable examples when a new surface is close to an existing family:

- `zircon_runtime/tests/fixtures/ui/reactbits_agent_workspace.zui` — App Shell 4 + AI Chat 1 + Agent Activity 2 + Tool Calls 2 + Agent Approval; responsive Stack shell with a wide three-column layout, regular compact-context summary, narrow main conversation flow, streaming conversation, and approval workflow.
- `zircon_runtime/tests/fixtures/ui/reactbits_agent_native_components.zui` — AI Chat 1/2/5/6/7/9 + Prompt Input 3 + Agent Activity 1; minimal semantic `AgentChat`/`ChatComposer` painter contract with a source-owned multi-turn thread, continuous assistant prose, right-aligned user bubbles, streaming state, and composer action.
- `zircon_runtime/tests/fixtures/ui/reactbits_workbench_interaction_surfaces.zui` — App Shell 4 + Command Menu + App Dialog + Notifications + Mobile; command, dialog, confirmation, notification, drag, and toast states in a responsive composition.
- `zircon_runtime/tests/fixtures/ui/reactbits_agent_workflow_components.zui` — Agent Plan + Tool Calls + Agent Approval + AI Usage; source-owned workflow status, bounded progress, destructive approval, and usage detail cards.
- `zircon_runtime/tests/fixtures/ui/reactbits_data_surface_components.zui` — Data Table + Filtering + File Manager + List + Empty State + Dashboard + Cards + Monitoring; source-owned filters, folder tree, table rows, metrics, and collection recovery guidance.
- `zircon_runtime/tests/fixtures/ui/reactbits_settings_form_components.zui` — Settings Form + Forms + App Dialog + Notifications; label-left fields, a windowed API-key collection, unsaved actions, feedback, and a destructive confirmation path.
- `zircon_runtime/tests/fixtures/ui/reactbits_auth_onboarding_components.zui` — Authentication + Onboarding; centered provider sign-in, one rich-text terms flow, verification digits, a horizontal stepper, and a windowed workspace picker.
- `zircon_runtime/tests/fixtures/ui/reactbits_kanban_scheduling_components.zui` — Kanban 1/5 + Scheduling 1/7; WIP/capacity columns, status cards, a date strip/month flow, a bounded agenda, and a booking summary with semantic end-aligned time values.

These are translation fixtures, not replacements for product-owned layouts. Preserve their source-owned semantic props and state routes when using them as a regression reference; do not copy their coordinates into an unrelated surface.

## Validation routing

Choose the smallest gate that exercises the changed surface:

- **ZUI source, retained projection, or native painter changed:** from `dev/penpot/plugins/apps/zircon-zui-plugin`, regenerate/check the source catalog and run the focused Penpot visual harness:

  ```powershell
  pnpm exec tsx tools/zui-layout-catalog.ts --check
  pnpm exec tsx tools/zui-layout-visual-validation.ts --source <repo-relative-zui-path> --workers 1
  ```

  Require every requested viewport to finish with `status: passed`, zero semantic overflow, and zero invalid geometry. Inspect the generated `penpot.png` and at least one narrow evidence image; leave `result.md` pending until native engine capture and the human review boundary are actually satisfied.

  When the Runtime test target is compilable, run the native WGPU gate through the physical-output-checked local wrapper. Select the fixture required by the changed surface, then expand to the declared catalog acceptance gate:

  ```powershell
  $env:ZUI_LAYOUT_SOURCE = 'zircon_runtime/tests/fixtures/ui/reactbits_data_surface_components.zui'
  $env:ZUI_LAYOUT_CASE = 'default-1280x800-dpi1'
  & .\tools\dev\local-cargo.ps1 test -p zircon_runtime --features ui --test zui_native_visual_acceptance export_all_zui_native_visual_acceptance -- --ignored
  Remove-Item Env:ZUI_LAYOUT_SOURCE -ErrorAction SilentlyContinue
  Remove-Item Env:ZUI_LAYOUT_CASE -ErrorAction SilentlyContinue
  ```

  Record native screenshot/geometry/text evidence separately from the Penpot result. A compiler, device, or asset-root failure is a pending acceptance gate—not a visual pass—and must be handed to the owning Runtime boundary before claiming engine parity.

- **Hub web/reference source changed:** run the relevant export (the exporter supports a targeted page option where documented), then run:

  ```powershell
  node docs/ui-and-layout/hub-web-reference/validate-responsive.mjs
  node docs/ui-and-layout/hub-web-reference/validate-interactions.mjs
  node docs/ui-and-layout/hub-web-reference/validate-visuals.mjs
  ```

  Use `docs/ui-and-layout/hub-design-board/export-design-board.mjs` and `validate-design-board.mjs` only when the design-board source changed. AI drafts and design-board PNGs are not runtime acceptance evidence.

- **Running Hub visual changed:** use `.codex/skills/zircon-project-skills/capture-hub-window-screenshot/SKILL.md`. Capture the real `Zircon Hub` window and require the state-specific text; do not accept a helper window, stale page, or black WebView surface.

- **Editor workbench layout changed:** run the existing preview verification when applicable:

  ```powershell
  npm --prefix tools/editor-workbench-preview run design:verify
  npm --prefix tools/editor-workbench-preview run design:verify:reference-negative
  ```

  Add the focused editor contract gate named by the touched module; do not substitute a screenshot-only check for a retained-host/layout contract.

- **Contract or runtime behavior changed:** run the focused Hub/editor/runtime contract tests required by the owning module, with the repository's managed validation policy. A ReactBits link does not replace a local contract test.

- **Skill/documentation-only change:** run the skill validator, local-link/Markdown checks, and the project skill synchronizer. Do not run a Rust build merely to validate prose.

If an external page is unavailable or license-gated, use its category page as the fallback, preserve the URL in the mapping, and mark the unverified detail rather than fabricating a layout description.
