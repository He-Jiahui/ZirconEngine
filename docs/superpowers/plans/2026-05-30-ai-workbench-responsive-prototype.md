# AI Workbench Responsive Prototype Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a responsive, self-contained HTML/CSS/JS workbench prototype that normalizes the AI-generated editor reference images into one consistent, interactive design system.

**Architecture:** Create a standalone prototype under `docs/ui-and-layout/ai-workbench-style/prototype/`. The HTML owns semantic shell regions, CSS owns all layout and visual language with reusable tokens, and JavaScript owns page switching, drawer switching, compact-mode interactions, and lightweight overlays.

**Tech Stack:** Plain HTML, CSS, and JavaScript. No third-party libraries and no full-screenshot UI reuse.

---

## Files

- Create `docs/ui-and-layout/ai-workbench-style/prototype/index.html`: semantic shell, toolbar, navigation, drawer slots, workspace regions, and migration-focused annotations.
- Create `docs/ui-and-layout/ai-workbench-style/prototype/base.css`: design tokens and global element rules.
- Create `docs/ui-and-layout/ai-workbench-style/prototype/shell.css`: workbench shell, top bar, primary tabs, rail, grid slots, and status bar.
- Create `docs/ui-and-layout/ai-workbench-style/prototype/panels.css`: drawers, tables, fields, bottom output, and overlays.
- Create `docs/ui-and-layout/ai-workbench-style/prototype/stages.css`: CSS-generated scene, graph, UI, montage, asset, diagnostic, and project preview surfaces.
- Create `docs/ui-and-layout/ai-workbench-style/prototype/responsive.css`: medium and compact viewport behavior.
- Create `docs/ui-and-layout/ai-workbench-style/prototype/page-data.js`: declarative page, drawer, bottom-pane, and tool data.
- Create `docs/ui-and-layout/ai-workbench-style/prototype/stage-renderers.js`: center-stage renderers for scene, material, UI, montage, assets, diagnostics, and project views.
- Create `docs/ui-and-layout/ai-workbench-style/prototype/view-utils.js`: shared HTML escaping utilities.
- Create `docs/ui-and-layout/ai-workbench-style/prototype/app.js`: tab/drawer switching, tool selection, overlay handling, layout presets, dock-preview state, and responsive state classes.
- Create `docs/ui-and-layout/ai-workbench-style/prototype/README.md`: explains source PNG references, normalized style rules, interaction model, and Rust UI migration mapping.

## Milestone 1: Prototype Shell And Style System

- [ ] Create the prototype directory and files.
- [ ] Define a single workbench shell with top command bar, primary page tabs, left rail, left drawer stack, central workspace, right drawer stack, bottom output drawer, and status bar.
- [ ] Implement CSS tokens for dark surfaces, 1px separators, teal state, compact controls, low-radius panels, table rows, forms, graph nodes, timelines, and viewport mock content.
- [ ] Add breakpoints so wide screens show the full docked layout, medium screens narrow drawers, and mobile/tablet screens collapse drawers behind toggles without losing navigation.

## Milestone 2: Interactive Page Model

- [ ] Add page data for Scene Editor, Material Editor, UI Editor, Montage Editor, Asset Browser, Diagnostics, and Project Overview.
- [ ] Implement main tab navigation without page reload.
- [ ] Implement drawer tab switching and selected tool state.
- [ ] Add lightweight command/menu overlays using HTML/CSS rather than screenshots.
- [ ] Ensure each page has a distinct central workspace while preserving the same shell structure and visual style.
- [ ] Split JavaScript into declarative page data, stage renderers, shared view utilities, and app orchestration before extending interaction state.
- [ ] Add layout presets and explicit side/bottom size tokens for Taffy migration.
- [ ] Add selected-region highlighting, dock-preview targets, and a layout inspector tray that describes the active node contract.

## Milestone 3: Documentation And Verification

- [ ] Document how AI PNG references were normalized instead of copied exactly.
- [ ] Document the shell-region mapping to Rust window/pane concepts.
- [ ] Run syntax checks for HTML/CSS/JS using local tooling.
- [ ] Start a local static server and inspect the prototype at desktop and mobile viewport sizes.
- [ ] Confirm no third-party library imports and no embedded full-reference screenshots are used by the prototype UI.
