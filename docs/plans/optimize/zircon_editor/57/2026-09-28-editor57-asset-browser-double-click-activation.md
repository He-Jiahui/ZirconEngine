---
title: Editor57 Asset Browser Double Click Activation
category: zircon_editor
date: 2026-09-28
implementation_status: partial_implemented
validation_status: scoped_static_checks_passed_managed_validation_pending
performance_status: product_gate_pending
---

# Editor57 Asset Browser double click activation

## Scope

This is a narrow ED57-P0-03 slice. Asset Browser content clicks already select a row, and
`EditorAssetEvent::OpenAsset` already resolves the locator through the active asset type registry,
checks the enabled toolkit and open command, and opens the toolkit view. The retained Browser
pointer callback did not connect those two paths.

## Implemented behavior

- A second primary click on the same visible asset within 500 ms dispatches the existing
  `OpenAsset` event using the locator from the current pointer projection. Both clicks must match
  UUID, locator, resource revision, catalog revision, view mode, and visible item identity. The 500 ms window matches the existing hierarchy double click gesture window.
- The candidate is scoped to Browser surface, UUID, catalog revision, view mode, and visible
  item-generation identity. Folder and query are not copied into the pointer projection; their
  cache-key changes rebuild item-generation identity, which indirectly invalidates the candidate. A
  blank click, a different asset, a changed list generation, or a dispatch error clears or expires
  the pending first click. Activity and Browser maintain independent click state.
- Before dispatch, the host compares the current Browser catalog revision, view mode, visible item
  generation identity, UUID, locator, and resource revision with the pointer projection. A stale
  target reports status and asks for another double click. The existing `OpenAsset` execution remains responsible for exact type, enabled toolkit,
  and command checks; no success receipt is added here.
- Stale and unavailable-target status messages use `asset.activation.target_changed` and
  `asset.activation.target_unavailable`; both keys have English and Chinese translations and carry
  no interpolated identifiers.

## Regression coverage

The click classifier regression covers same-target double click, different UUID, locator and resource
revision changes, changed catalog revision/view, rebuilt item-generation identity, expired window, blank-target reset,
independent surfaces, and rebuilt visible-item generation. A retained-host route regression drives
the actual Browser callback twice and checks the matching `OpenAsset` journal record has no dispatch error and reports changed=true. A second regression changes the query after the first click without refreshing the retained surface, then verifies the stale projection does not dispatch `OpenAsset`. Existing runtime
asset-open integration tests continue to cover toolkit route creation and unavailable toolkit
behavior.

## Remaining ED57-P0-03 gates

This slice does not implement Enter activation, an explicit Open command, context-menu activation,
reference activation, or a common `AssetActivationIntent` / opened-reused-unavailable-failed receipt.
The existing OpenAsset event is locator-based and does not bind the execution to a catalog generation
and resource revision atomically. It also does not prove a native UI session accepted and presented
the toolkit. These gates remain open; this record does not close ED57-P0-03.

No Cargo validation or performance measurement was run in this slice. Managed editor validation,
interactive Browser coverage, and the full 100k/1M product gates remain pending.
