# Icon Resource Audit Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Produce a full human-readable and machine-readable audit of all Hub and Editor SVG icon/image resources, with duplicate/conflict analysis and a prioritized cleanup backlog.

**Architecture:** This is a documentation and inventory milestone. It scans existing SVG resources in place, writes a Markdown audit under `docs/zircon_editor/assets`, writes a JSON manifest beside it, and records backlog recommendations without moving, renaming, deleting, or wiring any asset into UI code.

**Tech Stack:** Markdown, JSON, SVG resource inspection, repository Glob/Grep tooling, PowerShell or shell commands for validation.

---

## File Structure

- Create `docs/zircon_editor/assets/icon-resource-audit.md` as the primary review document with YAML frontmatter, pack summaries, file-level inventory tables, duplicate/conflict notes, gap analysis, and P0/P1/P2 backlog.
- Create `docs/zircon_editor/assets/icon-resource-audit.json` as the machine-readable inventory for future asset-catalog, lint, or UI tooling.
- Modify `docs/zircon_editor/assets/editor-shell-svg-resources.md` only if the audit discovers stale counts or validation metadata for `zircon_editor_shell`.
- Modify `docs/zircon_editor/assets/engine-style-svg-resources.md` only if the audit discovers stale counts or validation metadata for `zircon_engine_style`.
- Modify `docs/zircon_hub/index.md` only if the audit discovers stale Hub SVG counts, resource paths, or validation metadata.
- Do not modify files under `zircon_hub/assets/**` or `zircon_editor/assets/**` in this plan.
- Do not modify Rust, Slint, or Cargo files in this plan.

## Milestone 1: Inventory Capture

### Implementation Slices

- [x] Capture all SVG paths under `zircon_hub/assets/**/*.svg`.
- [x] Capture all SVG paths under `zircon_editor/assets/icons/**/*.svg`.
- [x] Capture all SVG paths under `zircon_editor/assets/preview/**/*.svg`.
- [x] Group each path into one pack: `hub-brand`, `hub-icons-nav`, `hub-icons-actions`, `hub-icons-status`, `hub-icons-ui`, `hub-covers`, `editor-ionicons`, `editor-shell`, `editor-engine-style`, or `editor-preview`.
- [x] For each icon record, derive `name` from the file stem, `category` from the immediate parent directory, `owner` as `zircon-owned` or `third-party-legacy`, and `scope` as `hub`, `editor`, or `shared-future`.
- [x] Mark `wired_status` as `wired`, `asset-only`, or `unknown` based on existing docs and known code references. Use `wired` for Hub resource paths already documented as loaded through Slint or `view_model/media.rs`; use `asset-only` for unbound Editor shell, engine-style, Ionicons, and preview assets unless code search proves otherwise.
- [x] Record source notes: Hub resources and Zircon packs are `repository-generated`; Ionicons are `third-party-existing`; reference-informed engine-style icons are `zircon-generated-reference-informed`.

### Testing Stage: Inventory Capture Validation

- [x] Run Glob for `zircon_hub/assets/**/*.svg` and record the count in the audit.
- [x] Run Glob for `zircon_editor/assets/icons/**/*.svg` and record the count in the audit.
- [x] Run Glob for `zircon_editor/assets/preview/**/*.svg` and record the count in the audit.
- [x] Compare pack totals against the JSON `packs[].count` values and fix any mismatch before moving to Milestone 2.
- [x] Acceptance evidence: the Markdown summary totals and JSON pack totals match the fresh Glob output.

## Milestone 2: Audit Document And JSON Manifest

### Implementation Slices

- [x] Create `docs/zircon_editor/assets/icon-resource-audit.md` with this frontmatter shape:

```markdown
---
related_code:
  - zircon_hub/assets/brand/zircon-mark.svg
  - zircon_hub/assets/icons/nav/projects.svg
  - zircon_hub/assets/icons/actions/build-project.svg
  - zircon_hub/assets/icons/status/running.svg
  - zircon_hub/assets/icons/ui/search.svg
  - zircon_hub/assets/covers/project-elysium.svg
  - zircon_editor/assets/icons/ionicons/add-outline.svg
  - zircon_editor/assets/icons/zircon_editor_shell/toolbar/menu.svg
  - zircon_editor/assets/icons/zircon_engine_style/assets/mesh.svg
  - zircon_editor/assets/preview/editor-scifi-room.svg
implementation_files:
  - docs/zircon_editor/assets/icon-resource-audit.md
  - docs/zircon_editor/assets/icon-resource-audit.json
plan_sources:
  - user: 2026-05-18 continue evaluating and organizing icon resources
  - docs/superpowers/plans/2026-05-18-icon-resource-audit.md
tests:
  - Glob inventory counts for Hub and Editor SVG paths
  - Grep external-reference scan for href/image/http references
  - Grep non-ASCII scan for generated Zircon-owned SVG packs
doc_type: module-detail
---
```

- [x] In the Markdown document, add a `Scope` section that states the audit covers every SVG under Hub assets and Editor icons/preview, including existing Ionicons, but does not move or rewrite assets.
- [x] Add a `Pack Summary` table with columns `Pack`, `Root`, `Count`, `Owner`, `Primary Use`, `Current Status`, and `Notes`.
- [x] Add a `File-Level Inventory` section split by pack. Each table should include `Path`, `Name`, `Category`, `Owner`, `Status`, and `Audit Notes`.
- [x] Add a `Duplicate And Collision Review` section. At minimum evaluate repeated stems such as `play`, `grid`, `settings`, `material`, `package`, `folder`, `save`, `add`, `list`, and `help` across packs.
- [x] Add a `Gap Review` section for missing future icon families: asset import states, scene editing modes, source-control states, build pipeline status, plugin lifecycle states, profiler panels, debug overlays, and Hub cloud/team future states.
- [x] Add a `Backlog` section with P0/P1/P2 items. P0 must only include safe audit/doc or manifest consistency work. P1 may include naming and pack policy work. P2 may include future UI wiring or replacement decisions.
- [x] Create `docs/zircon_editor/assets/icon-resource-audit.json` with this top-level shape:

```json
{
  "generated_at": "2026-05-18",
  "scope": "hub-and-editor-svg-assets",
  "policy": {
    "moves_or_renames": false,
    "ui_code_wiring": false,
    "third_party_rewrites": false
  },
  "packs": [],
  "icons": [],
  "duplicates": [],
  "backlog": [],
  "validation": []
}
```

- [x] Populate `packs[]` with objects using fields `id`, `root`, `count`, `owner`, `scope`, `primary_use`, and `status`.
- [x] Populate `icons[]` with objects using fields `path`, `pack`, `name`, `category`, `owner`, `scope`, `wired_status`, `source_note`, and `audit_notes`.
- [x] Populate `duplicates[]` with objects using fields `name`, `paths`, `classification`, and `recommendation`.
- [x] Populate `backlog[]` with objects using fields `priority`, `title`, `paths`, `reason`, and `recommended_action`.
- [x] Populate `validation[]` with objects using fields `check`, `scope`, `result`, and `evidence`.

### Testing Stage: Document And Manifest Validation

- [x] Parse `docs/zircon_editor/assets/icon-resource-audit.json` with a JSON parser. Recommended command: `python -m json.tool docs/zircon_editor/assets/icon-resource-audit.json`.
- [x] Confirm the Markdown frontmatter exists and starts with `related_code` before `implementation_files`.
- [x] Confirm the Markdown and JSON totals match the fresh Glob output from Milestone 1.
- [x] If the JSON parse fails, fix the malformed JSON before any documentation claims are made.
- [x] Acceptance evidence: JSON parses successfully, and Markdown/JSON totals match the captured inventory counts.

## Milestone 3: Resource Integrity Scans

### Implementation Slices

- [x] Scan all audited SVG files for external references using the pattern `href=|url\(http|https://|<image`.
- [x] Scan Zircon-owned generated packs for non-ASCII content using the pattern `[^\x00-\x7F]`. Include Hub generated SVGs, `zircon_editor_shell`, `zircon_engine_style`, and `editor-scifi-room.svg`.
- [x] Do not require Ionicons to be ASCII-only unless the scan is being used as informational evidence. Treat Ionicons as existing third-party legacy resources.
- [x] Add scan results to `icon-resource-audit.md` under `Validation Evidence`.
- [x] Add scan results to `icon-resource-audit.json.validation[]`.
- [x] Update existing pack docs only if the fresh scans contradict their current metadata.

### Testing Stage: Integrity Validation

- [x] Run Grep for `href=|url\(http|https://|<image` over `zircon_hub/assets`, `zircon_editor/assets/icons`, and `zircon_editor/assets/preview`.
- [x] Run Grep for `[^\x00-\x7F]` over Zircon-owned generated SVG paths only.
- [x] Record exact match counts or `no matches` results in the audit document and closeout response.
- [x] If external references appear in generated Zircon-owned assets, stop and classify them as P0 backlog unless the user approves editing assets in a later plan.
- [x] Acceptance evidence: integrity scan results are recorded in both Markdown and JSON.

## Milestone 4: Closeout Review

### Implementation Slices

- [x] Inspect `git diff -- docs/zircon_editor/assets/icon-resource-audit.md docs/zircon_editor/assets/icon-resource-audit.json docs/zircon_editor/assets/editor-shell-svg-resources.md docs/zircon_editor/assets/engine-style-svg-resources.md docs/zircon_hub/index.md`.
- [x] Verify that no files under `zircon_hub/assets/**` or `zircon_editor/assets/**` were changed by this plan.
- [x] Verify that no Rust, Slint, Cargo, or build-tool files were changed by this plan.
- [x] Update the implementation checklist in this plan only after the corresponding evidence exists.
- [x] Prepare closeout summary with counts, validation commands, docs written, JSON parser result, and any backlog items that remain future work.

### Testing Stage: Closeout Validation

- [x] Run `git status --short` and separate this plan's files from pre-existing unrelated dirty worktree changes.
- [x] Run `python -m json.tool docs/zircon_editor/assets/icon-resource-audit.json` one final time.
- [x] Rerun the external-reference Grep scan after final edits.
- [x] Rerun the generated-pack non-ASCII Grep scan after final edits.
- [x] Acceptance evidence: closeout response reports fresh command results and does not claim Cargo or workspace tests passed unless those commands were actually run.

Closeout note: the worktree had many unrelated pre-existing modified and untracked Rust, Slint, asset, and docs files. This plan's implementation files are the new audit Markdown, the new audit JSON, and this plan checklist update; unrelated dirty files were inspected but not reverted or modified for this audit.

## Self-Review

- Spec coverage: covers file-level audit, all SVG scopes, existing Ionicons, machine-readable JSON, duplicates, naming conflicts, gap suggestions, backlog, and validation evidence.
- Placeholder scan: no TBD/TODO placeholder steps remain; all planned files and validation commands are named explicitly.
- Type consistency: JSON field names are consistent across milestone tasks: `packs`, `icons`, `duplicates`, `backlog`, and `validation`.
- Scope check: the plan intentionally avoids asset moves, renames, UI code wiring, third-party rewrites, and Cargo build/test claims.
