---
title: Editor57 Popup Enter Consumption
category: zircon_editor
date: 2026-09-28
implementation_status: regression_added_pending_validation
validation_status: scoped_static_checks_passed_managed_validation_pending
performance_status: product_gate_pending
---

# Editor57 active popup Enter consumption

## Scope

An active popup owns Enter even when its current row is absent. In that state the popup keyboard
route returns an idle redraw result, but the host keyboard dispatcher must still stop the event from
falling through to the unhandled/native keyboard callback. This does not request a redraw merely to
represent consumption.

## Regression behavior

The retained host regression opens the page overflow popup with no hovered or selected hidden row and
dispatches a real Enter KeyEvent through dispatch_native_key_for_test, which calls
dispatch_keyboard_event. It checks that the unhandled keyboard callback is not invoked. A paired
no-popup case verifies ordinary Enter still reaches that callback.

## Validation and remaining gates

Cargo and managed validation remain pending. The grouped coordinator source
attribution completed under request `9006f4b3eef74e2a837772ecf0341d68`.
This slice only closes
keyboard fallback leakage for Enter when an active popup has no current row. ED57-P0-03 still requires
Enter to activate valid targets through the common activation intent and opened/reused/unavailable/
failed receipt, catalog-generation fencing at execution, toolkit presentation evidence, and
performance gates.
