# Runtime 01 current-source tech-stack baseline

Date: 2026-07-11

Status: `runtime_01_tech_stack_current_source_baseline_11_passed_cargo_pending`

## Scope

This baseline revalidates the Runtime 01 dependency-governance source assertions before entering the milestone Cargo testing stage. It does not change production code, manifests, dependency versions, or owner decisions.

## Evidence

- `cargo metadata --no-deps --locked --format-version 1`: exit 0; the workspace lockfile matches current manifests.
- `audit_runtime_structure.py --json`: exit 0; Runtime 01 reports manifest files 5/5, tech-stack guards 12/12, current behavior anchors 6 with no missing anchors, no missing Cargo-gate anchors, and `risks = []`.
- Direct current-source `rustc --test zircon_runtime/src/tests/extensions/tech_stack_dependency_guard.rs`: 11 passed / 0 failed / 0 ignored / 0 filtered, 65.04 seconds.
- Direct current-source `rustc --test zircon_runtime/src/tests/runtime_absorption/tech_stack.rs`: 2 passed / 0 failed / 0 ignored / 0 filtered, confirming the six-anchor documentation mirror and folder-backed route owner.

The direct guard run covers prerelease version pins, external ZrVM pairing, interface/editor dependency boundaries, ZIP archive ownership, plugin-owned Jolt, editor-only dependency backlog, fontdue migration ownership, and the shared text-shaper boundary.

## Remaining gate

The plan remains `in_progress`. Package-level `tech_stack`, `extensions`, `text_shaper`, physics plugin, and `export_build_plan` Cargo filters still require execution against a stable current-source window. Active text/render/plugin Cargo lanes prevented starting another heavyweight compile during this baseline; no package or workspace pass is claimed.
