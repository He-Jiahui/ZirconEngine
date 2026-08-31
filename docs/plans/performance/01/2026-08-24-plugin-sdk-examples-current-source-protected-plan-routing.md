---
title: Plugin SDK Examples Current-Source Protected Plan Routing
date: 2026-08-24
status: routing_record_only
source_review: docs/plans/performance/01/2026-08-24-plugin-sdk-examples-current-source-product-performance-review.md
---

# Plugin SDK Examples Current-Source Protected Plan Routing

## Review ledger status

Plugin SDK Examples **7/7 Rust files** completed E3 current-source static review at `8a5bd5580000debd99bdd96e437cc7bc017468a7`; fingerprint `d9d0f8a2bb47a47564cee64757fd6062350d9686806c9606c705208a98f70e12`. Six files pass standalone rustfmt; `extensions.rs` has one import-order-only mismatch. Diff check passes and no source changed. Protected `review.md` and `pending.md` remain unchanged because Cargo, product graph, real carrier behavior, WPR/ETW and power evidence are unavailable.

## Canonical owner notifications

| Finding | Canonical plan owner | Required update |
|---|---|---|
| Physical Editor-manifest scan promotes a Standard sample into product metadata | Plugins20 M0/M1 + Editor06 + Plugins06 | Add role/visibility/default/explicit-load/Shipping policy and an explicit ProductPluginCatalog; repository scan only inventories candidates. |
| Source provider catalog omits the package while native carrier exports zero contributions/commands/bridge | Plugins20 P0-02/M2 + Plugins06 + Plugins01 | Treat it as explicitly imported Sample, then close one executable workflow and source/library/native behavior parity before availability. |
| Asset/content roots and three required resources are absent | Plugins20 P1-11/12/G09 + Plugins08 | Resolve and hash all required roots/URIs before atomic contribution publication; do not add placeholder files. |
| Source Model output and API 0.1 disagree with checked-in Data output and API 0.2 | Plugins20 M1 + Plugins01 manifest owner | Generate all projections from one typed definition and reject field drift by source digest. |
| Four commands, importer and authoring surfaces have no executable handlers/documents | Plugins20 P1-17..20 + Plugins08 | Reuse canonical model import provider and implement one real window/import/toolkit/settings workflow with typed failure and lifecycle receipts. |
| Native cdylib depends on full Editor source crate but carries no behavior | Plugins20 P1-14/16 + Plugins01 | Extract a narrow ABI projection crate and measure build/link/artifact/load costs only after executable parity exists. |
| Existing five tests prove isolated metadata only | Plugins20 G01..G32 + Editor06 | Test default exclusion, explicit selection, resource closure, real behavior, carrier parity, rollback and unload generation fencing. |

## Acceptance routing

Implementation order is carrier isolation -> canonical package compilation -> executable golden sample -> carrier parity -> dynamic qualification. The default MVP Editor must incur zero row/provider/resource/artifact/load work for this sample. Explicit selection may incur work only after readiness is proven.

Dynamic acceptance requires current-source default and developer-selected Editors, source/library/native builds, real model import and window workloads, unload/reload/failure cases, and BuildSet-bound build time, artifact size, load/materialization, CPU, RSS/allocation, I/O, wakeups and energy. RenderDoc is used only for the real selected window/imported model frame, not for package admission or CPU claims.

No Git milestone commit or quantified WeCom message is warranted by this static routing record.
