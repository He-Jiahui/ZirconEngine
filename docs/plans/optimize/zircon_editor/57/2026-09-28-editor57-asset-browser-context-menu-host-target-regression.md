---
title: Editor57 Asset Browser Context Menu Host Target Regression
category: zircon_editor
date: 2026-09-28
implementation_status: test_added_pending_validation
validation_status: scoped_static_checks_passed_managed_validation_pending
performance_status: product_gate_pending
---

# Editor57 Asset Browser context menu host target regression

## Scope

This record adds a real retained-host regression for the explicit context-menu Open slice. It
exercises the Browser right-button callback, retained context-menu target path, typed Open binding,
and asset binding dispatch.

## Regression behavior

The fixture contains asset A (`res://grid.albedo.png`) and asset B (`res://materials/runtime_demo.mat`).
The test establishes B as the selected asset, sends a right-button event through the actual
`PaneSurfaceHost` callback at A's Browser row, and checks that the retained context target path and
typed Open dispatch both carry A's locator. It also verifies the selection remains B. This proves
that the host route does not substitute current selection for the right-clicked row.

The regression stops at the typed `AssetHostEvent::OpenAsset` dispatch boundary. It does not invoke
asset toolkit execution or prove toolkit presentation.

## Validation and remaining gates

Pinned rustfmt and scoped static checks passed. Cargo tests and managed validation were not run here.
The test path already contained earlier ED57 changes; those changes remain part of the workspace and
were preserved. ED57-P0-03 still requires Enter activation, a common activation intent and outcome
receipt, catalog-generation fencing at execution, toolkit presentation evidence, and performance
gates.
