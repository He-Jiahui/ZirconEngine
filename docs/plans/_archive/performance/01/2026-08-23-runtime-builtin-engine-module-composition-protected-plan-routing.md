---
title: Runtime Builtin and Engine Module Composition Protected Plan Routing
date: 2026-08-23
status: routing_only
related_report:
  - docs/plans/performance/01/2026-08-23-runtime-builtin-engine-module-composition-currentness-adoption.md
protected_targets:
  - docs/plans/performance/review.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
---

# Runtime Builtin and Engine Module Composition Protected Plan Routing

This file is routing input only. No protected ledger or numbered plan was modified.

## `review.md` suggestion

Do not promote either folder. All current Rust files have static coverage, but the lazy factory panic P0, multi-authority composition, effective-manifest/provider filtering, repeated descriptor materialization, transactional publication and dynamic startup evidence remain open.

## `pending.md` suggestion

`zircon_runtime/src/builtin` + `zircon_runtime/src/engine_module`: static current-source composite review complete; awaiting terminal factory handling, stable identity/schema, one immutable `RuntimeCompositionPlan`, exact manifest/provider filtering, transactional extension/module publication, Runtime-App-Core single-generation hard cut, and managed startup WPR evidence. Current UI Document Importer manifest changes are preserved but do not close composition truth.

## Canonical routing

| Issue | Target plan |
|---|---|
| Profile/target/manifest/provider/extension selection and composition compiler | Runtime42 |
| module proposal, compiled service contract, factory/context, descriptor single evaluation and snapshot continuity | Runtime46 |
| Core activation, call admission, rollback, reverse teardown and service state | Runtime01 |
| product role, App host inputs, EngineEntry receipt and shutdown | App01 |
| first-party provider/catalog closure and required selection P0s | Plugins06 |
| native provider trust/ABI/admission | Plugins01 |
| stable identity, BuildSet and dynamic ABI generation | Runtime24/43 + Interface plans |
| UI Document Importer provider/package semantics | Runtime85/87 + UI asset plans |

Do not create another catalog, PluginGroup, descriptor cache, extension-family vector facade or App-side graph builder. The only acceptable convergence is one validated plan generation, one compiled descriptor/binding per module, one transactional publication and one Core lifecycle owner.

## Promotion gates

1. Factory panic/error/cancel always resets state, wakes waiters and produces a typed terminal result.
2. Every effective manifest selection has one resolution row; disabled/unselected providers contribute no module or extension work.
3. App startup performs one composition build and one author proposal evaluation per module; query, bootstrap and Core share the same generation/hash.
4. Rejected or partial activation publishes zero new contribution; rollback and reverse teardown census are complete.
5. `1/100/1,000/10,000` scale receipts include builds, visits, descriptor calls, clone bytes, CPU/RSS, locks and p95; complexity follows the frozen graph plus selected contributions.
6. Current-source WPR/ETW proves startup/main-thread/wait/I/O/RSS/power budgets; RenderDoc only validates first visible frame generation and cannot promote CPU performance by itself.
