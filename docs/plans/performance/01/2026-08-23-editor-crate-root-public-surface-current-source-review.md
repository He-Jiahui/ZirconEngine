---
title: Editor Crate Root Public Surface Current Source Review
date: 2026-08-23
scope:
  - zircon_editor/src/lib.rs
status: static_complete_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/51-editor-project-startup-open-create-authority-hub-handshake-session-guard-focus-recent-recovery-product-integration-review.md
references:
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/RenderCore.Build.cs
---

# Editor Crate Root Public Surface Current Source Review

## 1. Coverage

The current `zircon_editor/src/lib.rs` surface is **1/1 Rust file**, **74 physical / 71 non-empty lines**, **4,539 bytes**, and no test markers. Its workspace-relative `path + NUL + raw bytes + NUL` SHA-256 is `7636affd68996ec7c3860859cc3794e9fa257ed13d29f372c7241b4a5fbde17e`. The file is clean in the shared worktree and was read with all direct non-`dev` consumers.

## 2. Result

The file contains no frame-time algorithm, allocation loop, lock or event polling. It declares `core`, `scene` and `ui`, then explicitly re-exports command/keymap, editing intent, runtime gateway, startup, editor plugin, host/export-wizard and retained-host run APIs. App entry and plugins import named symbols rather than a root glob, so no runtime copy or dispatch cost is attributable to the re-export layer itself.

The engineering issue is public-surface coupling. Export Wizard alone contributes a large symbol set to the crate root, while command, plugin, gateway, host lifecycle and scene UI are separate owners. This can increase documentation/semver surface and downstream incremental rebuild fan-out. It is not evidence of startup or frame regression, and moving exports without compiler/consumer measurements would create churn rather than performance improvement.

Unreal's module build rules distinguish public dependencies from private and target-specific dependencies. The applicable constraint is to expose only cross-module contracts and keep implementation/tool-specific ownership private; Zircon should not copy Build.cs or C++ header structure.

## 3. Plan and acceptance

1. Preserve the root as a declaration/facade only; no implementation logic, caches, globals or product fallbacks may enter it.
2. Inventory external consumers by owner and classify stable cross-plugin contracts versus App-only/editor-internal helpers. Export Wizard internals move behind its owner module only if consumer migration is a hard cut with no compatibility re-export.
3. Measure clean/incremental `zircon_editor`, `zircon_app` and representative editor-plugin builds before and after any surface split: duration, rebuilt crate count, compiler peak RSS, metadata/rlib size and changed public item count.
4. Run editor startup WPR only against a current-source product to separate module load/startup work from compile-time coupling. This root file itself is not a RenderDoc target.

Static root-file `rustfmt --check` passed with child traversal disabled. No production/test code was changed. Cargo/build/startup measurements remain pending, so this root remains dynamically pending and should receive one concise protected-ledger entry only after owner adoption.
