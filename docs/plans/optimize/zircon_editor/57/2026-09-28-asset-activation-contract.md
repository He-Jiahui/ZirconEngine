---
title: Editor57 asset activation intent and receipt contract
category: zircon_editor
date: 2026-09-28
implementation_status: contract_defined_execution_pending
validation_status: scoped_static_checks_passed_managed_validation_pending
performance_status: product_gate_pending
related_code:
  - zircon_editor/src/core/asset/mod.rs
  - zircon_editor/src/core/asset/activation.rs
  - zircon_editor/src/core/asset/activation/tests.rs
---

# Editor57 asset activation intent and receipt contract

## Defined contract

This bounded ED57-P0-03 slice defines an editor-owned, serializable activation target.
`AssetActivationIntent` carries a typed `AssetUuid` and `AssetUri`, the captured catalog revision,
an optional resource revision matching the current Browser snapshot, and one of double click,
Enter, Open command, context menu, or reference sources. The two identities stay together so a
later selection or locator reuse cannot silently become a different requested target when an
executor eventually checks the current catalog and resource authority.

`AssetActivationReceipt` retains its intent and distinguishes `Opened`, `Reused`, `Unavailable`,
and `Failed`. Opened and reused results carry a view instance ID; unavailable and failed results
carry a reason or error message. The receipt is a host-issued terminal contract, not a claim that
the current `OpenAsset` path produces it.

The added JSON regressions round-trip all five sources, exact UUID and locator, large catalog and
resource revisions including an absent resource revision, and all four terminal results. A decode
regression rejects an unsupported locator scheme through the existing typed `AssetUri` parser.

## Execution and acceptance still open

The current Browser double click and context menu paths still dispatch locator-only
`EditorAssetEvent::OpenAsset`; they neither construct this intent nor return this receipt.
Execution must still compare UUID, locator, catalog revision, and resource revision against current
authority, resolve the exact enabled asset type and toolkit, distinguish a newly opened view from
reuse, and report actual native presentation. Enter and the remaining activation sources need
their product routes. `EditorEventResult::failure` currently sets `value` to `None`, and its
dispatch failure path cannot place a `Failed` activation receipt in the journal without a later
execution-result change.

Only exact-file rustfmt and scoped static checks are claimed for this slice. Managed Editor Cargo
validation, live Browser interaction, native presentation, and the original 100k/1M performance
gates remain pending; no latency or allocation improvement is claimed.
