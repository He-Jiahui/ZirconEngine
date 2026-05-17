# Hub Project Workbench Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the Hub Projects dashboard operate on an explicit selected project so cards, rows, and Quick Actions have a visible, predictable target.

**Architecture:** Keep project selection as transient runtime UI state in `HubRuntime` and `HubSnapshot`, not persisted config. Project projections carry a `selected` flag into Slint so cards and table rows can render selection feedback, while Rust resolves Quick Actions against the selected project with a latest-recent fallback.

**Tech Stack:** Rust `zircon_hub`, Slint 1.16.1 UI, repository-owned SVG/icon components, scoped Cargo validation.

---

## Files And Responsibilities

- Modify `zircon_hub/ui/shared.slint` to add `selected: bool` to project card and row data.
- Modify `zircon_hub/src/state/hub_snapshot.rs` to include selected project path and keep filtering tests updated.
- Modify `zircon_hub/src/app/view_model.rs` to mark selected project cards/rows and add unit tests for selected projection.
- Modify `zircon_hub/src/app/runtime.rs` to store selected project path, expose selection callbacks, make Quick Actions target the selected project, and correct View All Projects behavior.
- Modify `zircon_hub/ui/app.slint` and `zircon_hub/ui/projects.slint` to wire selected project callbacks, selected visuals, and View All Projects semantics.
- Modify `zircon_hub/ui/components.slint` if the shared `DataTable`/row component needs selection styling.
- Modify `docs/zircon_hub/index.md` to describe the selected-project workflow and validation evidence.

## Milestone 1: Selected Project State And Projection

- Goal: `HubSnapshot` carries a selected project path and view-model rows expose `selected`.
- In-scope behaviors: project card selection flag, compact list selection flag, recent table selection flag, selected path surviving search/filter only as transient state.
- Dependencies: existing `RecentProject`, `HubSnapshot::filtered_recent_projects`, project card/table projections.
- Implementation slices:
  - [x] Add `selected_project_path: Option<PathBuf>` to `HubSnapshot`.
  - [x] Update existing `HubSnapshot` test constructors to provide `selected_project_path`.
  - [x] Add `selected: bool` to `ProjectCardData` and `RecentProjectRowData`.
  - [x] Update `project_card` and `recent_project_row` helpers to compare paths against the selected path.
  - [x] Add focused view-model tests proving selected card and row projection.
- Testing stage:
  - Run scoped Hub tests in the final milestone testing stage, not after this slice.
- Lightweight checks:
  - Use compiler feedback only if Slint/Rust type errors block implementation.
- Exit evidence:
  - Covered by final `cargo check -p zircon_hub --locked --offline --jobs 1` and `cargo test -p zircon_hub --locked --offline --jobs 1`.

## Milestone 2: Selection Behavior And Quick Action Targeting

- Goal: UI selection callbacks update runtime state and Quick Actions target the selected project when available.
- In-scope behaviors: selecting a card/row, opening a selected/recent project, packaging selected project, installing selected project, latest-recent fallback when nothing is selected, status detail naming the target.
- Dependencies: Milestone 1 selected path projection.
- Implementation slices:
  - [x] Add `selected_project_path: Option<PathBuf>` field to `HubRuntime`.
  - [x] Add `select_project_path`, `selected_or_latest_recent_project`, and `select_latest_recent_project_if_needed` style helpers scoped inside `runtime.rs`.
  - [x] Make `package_recent_project_to_output` use selected project first, then latest recent.
  - [x] Make `open-editor` Quick Action open the selected project when one exists; keep launching editor without project only when no project is selected.
  - [x] Wire `select-project(string)` from Slint to Rust and apply snapshots after selection.
  - [x] Add runtime or state tests where helpers can be tested without launching child processes.
- Testing stage:
  - Run scoped Hub tests in the final milestone testing stage.
- Lightweight checks:
  - Use scoped `cargo check` only if type integration blocks UI wiring.
- Exit evidence:
  - Covered by final check/test/build.

## Milestone 3: Projects UI Interaction And Documentation

- Goal: Projects page visibly marks the selected project and View All Projects performs dashboard reset/expand behavior.
- In-scope behaviors: selected card border, selected data-table row styling, card/row click selects then opens only through explicit action affordances already present, View All Projects clearing search/filter and expanding card flow.
- Dependencies: Milestone 1 data projection and Milestone 2 callback.
- Implementation slices:
  - [x] Add `select-project(string)` and `view-all-projects()` callbacks through `ProjectsPage` and `HubWindow`.
  - [x] Update `ProjectCard`, `ProjectFlow`, `DataTable`, and row components to pass and display selected state.
  - [x] Change `View All Projects` action from opening the import panel to clearing search/filter and expanding project cards.
  - [x] Update `docs/zircon_hub/index.md` with the selected-project workflow.
- Testing stage:
  - Run `cargo check -p zircon_hub --locked --offline --jobs 1`.
  - Run `cargo test -p zircon_hub --locked --offline --jobs 1`.
  - Run `cargo build -p zircon_hub --locked --offline --jobs 1`.
  - Capture the running Hub Projects page with `.codex/skills/zircon-project-skills/capture-hub-window-screenshot/scripts/capture-hub-window.ps1` and inspect alignment/selection visuals.
  - Debug and correct failures, then rerun affected commands.
- Lightweight checks:
  - None expected before the final testing stage unless Slint type-checking blocks progress.
- Exit evidence:
  - Passing scoped Cargo commands plus a fresh screenshot artifact under `target/hub-visual-check/`.

## Self-Review

- Spec coverage: selected project state, Quick Action target binding, View All Projects semantic correction, visual selected feedback, docs, and validation are all represented.
- Placeholder scan: no TBD/TODO placeholders remain.
- Type consistency: `selected_project_path` is the Rust state field; Slint data structs use `selected`; UI callback names use kebab-case Slint style.

## Execution Evidence

- `cargo fmt -p zircon_hub --check`: passed.
- `cargo check -p zircon_hub --locked --offline --jobs 1`: passed.
- `cargo test -p zircon_hub --locked --offline --jobs 1`: passed, 58 tests.
- `cargo build -p zircon_hub --bin zircon_hub --locked --offline --jobs 1`: passed.
- Real Hub screenshot captured at `target/hub-visual-check/hub-project-workbench.png` using an isolated sample-project config.
