---
name: zr-ui-layout-reference
description: Design, change, or review ZirconEngine Hub, editor, or runtime UI layouts using approved ReactBits references and local UI contracts.
---

# Zircon UI Layout Reference

Use this skill for any ZirconEngine application-facing layout work: Hub pages, editor workbench panes, runtime UI shells, AI/agent surfaces, data views, forms, overlays, auth/onboarding, workflows, and their responsive states.

Apply [flow semantics](../zr-flow-layout-semantics/guide.md) when authoring or reviewing layout. Select [icon sizing](../zr-icon-size-scale/guide.md) for icon changes and [content contracts](../zr-i18n-content-contracts/guide.md) for visible copy, editable values, or collections. Reuse these references when already read. The ReactBits links remain the functional-design reference.

The two top-level catalogue views are the design index: [ReactBits Blocks](https://pro.reactbits.dev/docs/blocks) is the source for reusable marketing/product section composition, while [ReactBits Application UI](https://pro.reactbits.dev/docs/app-ui) is the source for application shells, controls, data surfaces, and workflow states. The detailed links in the reference map are the approved choices for a concrete surface.

## Non-negotiable reference rule

Every new or changed layout must be mapped to at least one approved ReactBits reference in [the reference map](references/reactbits-reference-map.md). Choose a primary reference for the page/frame and add supporting references for distinctive states or interactions (for example, loading, approval, tool output, empty, or error). Record the mapping in the active plan, review note, or final handoff so another agent can reproduce the design decision.

"Reference" means borrow the information architecture, region relationships, interaction model, and responsive behavior. Do not copy ReactBits Pro source code, private assets, screenshots, or license-gated registry files into ZirconEngine. The linked pages are external design references, not a permission to install or redistribute them.

The repository's contracts remain the implementation authority. If a ReactBits pattern conflicts with a Zircon token, accessibility rule, runtime/editor ownership boundary, or platform constraint, keep the local contract and explain the adaptation in the mapping. Do not invent an unreferenced layout merely because a page is difficult to translate; compose the closest listed families and mark any uncertainty.

## Workflow

1. **Locate the owner.** Identify the actual page, shell, component, asset, and state owner before drawing regions. For Hub, inspect `zircon_hub/web/src/**` and the HTML/CSS reference under `docs/ui-and-layout/hub-web-reference/**`. For editor/runtime UI, inspect the relevant `zircon_editor/**`, `zircon_runtime*/**`, `.zui` assets, and `docs/ui-and-layout/**` contracts.
2. **Classify the surface.** Select the smallest matching family in the reference map (shell/navigation, AI/agent, data, forms/overlays, auth, or workflow). For landing/marketing-style composition, first classify the section against the Blocks catalogue, then use the closest Application UI family for interactive behavior. Use one primary link plus only the supporting links needed to explain the behavior.
3. **Inspect embedded references before inferring layout.** When the selected reference renders in an iframe or embedded preview, inspect the frame's rendered DOM, HTML structure, CSS layout rules, and relevant interaction script/state before translating it. Determine which elements are one reading flow, independent controls, labels, values, or status before using a screenshot. A screenshot verifies the result; it is not permission to reverse-engineer a coordinate layout from pixels.
4. **Describe the layout before implementation.** State the regions and their owners, semantic text alignment, inline rich-text boundaries, min/max sizing, scroll and overflow policy, responsive transitions, state matrix, focus/keyboard behavior, and any approval or destructive-action gate. Prefer shared primitives and tokenized constraints over page-local geometry.
5. **Implement the translation.** Reuse existing Hub or retained-editor primitives and the shared state/data projection. Preserve semantic labels, stable control identity, focus restoration, live progress, retry/cancel behavior, and status severity. Do not use screenshot-as-runtime markup or fixed coordinate hit targets.
6. **Validate the reference path.** Run the smallest affected local web/reference validators and visual checks described in [the Zircon translation guide](references/zircon-layout-translation.md). A real Hub window capture is runtime evidence; AI drafts and design-board sheets are structure-review aids only.
7. **Hand off with evidence.** Report the selected ReactBits links, the local authority files, responsive/state coverage, and the validation commands/results. If the task is documentation-only, use structural/link checks rather than an unrelated Rust build.

## Fast routing

- App frame, sidebar, tabs, command/search, or mobile navigation: `app-shell`, `app-shell-4`, `app-sidebar`, `navbar`, `command-menu`, `mobile`.
- Chat, composer, agent activity, tool output, permissions, plans, or usage: the `ai-chat`, `prompt-input-3`, `agent-activity`, `tool-calls`, `agent-approval`, `agent-plan`, and `ai-usage` references.
- Projects, catalogs, metrics, tables, files, monitoring, filters, or empty states: `card`, `data-table`, `dashboard`, `analytics`, `list`, `filtering`, `file-manager`, `monitoring`, and `empty-state`.
- Settings, record forms, dialogs, notifications, auth/onboarding, paywalls, boards, or scheduling: `settings-form`, `forms`, `app-dialog`, `notifications`, `authentication`, `onboarding`, `paywall`, `kanban`, and `scheduling`.

Read the reference map only for the family being changed, then read the translation guide when local owners or validation gates are involved.
