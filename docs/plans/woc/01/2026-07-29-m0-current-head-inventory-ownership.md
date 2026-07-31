# M0 Current-Head Inventory Ownership

Owner: [WOC ZrVM one-to-one replication plan](../01-woc-zrvm-one-to-one-replication.md)

| Milestone | Scope | Status | Date | Evidence / residual risk |
|---|---|---|---|---|
| M0 | Standalone WOC project identity, fixed `5ef9f7cb21cd8875b6d2c49701015dfcd78de35a` inventory, and reconstruction ownership for every catalog row | `accepted-static` | 2026-07-29 | `reference_inventory.mjs --check`, `woc_contract_codegen` inventory suite, and the complete native workspace are green. This accepts only source identity and classification; real ZrVM transactions, current-head behavior rebase, asset import, retained UI, and product-host acceptance remain open. |
