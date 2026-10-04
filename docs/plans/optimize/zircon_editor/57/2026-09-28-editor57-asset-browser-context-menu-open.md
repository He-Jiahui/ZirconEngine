---
title: Editor57 Asset Browser Context Menu Open
category: zircon_editor
date: 2026-09-28
implementation_status: partial_implemented
validation_status: scoped_static_checks_passed_managed_validation_pending
performance_status: product_gate_pending
---

# Editor57 Asset Browser context menu Open

## Scope

This is a narrow ED57-P0-03 activation slice. It adds an explicit Open action to the retained
Asset Browser row context menu and routes it through the existing `AssetCommand::OpenAsset` path.

## Implemented behavior

- The row context menu presents localized Open and existing Delete actions. Open uses the existing
  `menu.assets.open.label` translation and is the primary action.
- The host encodes the row locator into the retained menu target path. The callback bridge decodes
  that captured locator and returns a typed `AssetCommand::OpenAsset` binding. It does not infer a
  locator from the current selection.
- The callback-bridge regression supplies a menu target path containing
  `res://materials/runtime.zmat` and asserts the Open binding retains that exact locator. The host
  encoding and bridge decoding paths are statically reviewed; a real row right-click through menu
  construction is not covered by this regression. The binding does not read current selection, so
  later selection B cannot replace its context target A. The existing `OpenAsset` execution remains
  authoritative for stale or unavailable locators and toolkit resolution.
- Delete continues to resolve the UUID portion of the target path, including when the path also
  carries the encoded Open locator. The existing Delete binding regression now uses that production
  target-path shape.

## Remaining ED57-P0-03 gates

This slice does not implement Enter activation, common activation intent, or an opened/reused/
unavailable/failed receipt. It does not atomically fence a context target against catalog generation
changes between menu creation and activation, nor confirm native toolkit presentation. The exact
captured locator prevents current-selection substitution; stale and unavailable outcomes still depend
on the existing `OpenAsset` dispatch boundary. ED57-P0-03 remains open.

Pinned rustfmt and scoped static checks passed. Cargo tests, managed validation, and performance
measurements were not run in this task and remain pending.
