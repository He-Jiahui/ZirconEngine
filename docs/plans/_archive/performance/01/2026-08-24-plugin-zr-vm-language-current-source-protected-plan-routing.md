---
title: Plugin Zr VM Language Protected Plan Routing
date: 2026-08-24
status: routing_record_only
source_review: docs/plans/performance/01/2026-08-24-plugin-zr-vm-language-current-source-performance-review.md
---

# Plugin Zr VM Language Protected Plan Routing

## Review ledger status

`zircon_plugins/zr_vm_language` completed an E3 current-source static review over **37/37 Rust files** plus focused catalog/App/Runtime scheduler and hot-reload reads. Protected `docs/plans/performance/review.md` and `docs/plans/performance/pending.md` remain unchanged because the real backend and product have no current dynamic acceptance receipt.

## Canonical owner notifications

| Finding | Canonical plan owner | Required update |
|---|---|---|
| NativeDynamic carrier is metadata-only, real backend is separately feature-gated and the plugin is disabled/Partial | Plugins16, Plugins01/06 and App target-composition plans | Produce `LanguageActivationPlan`; make carrier capability truthful and fail missing provider/toolchain before startup. |
| External `E:/Git/zr_vm` receipt changed to `4af30a6c...` with 32 dirty entries | Runtime21 and Plugins16 | Replace stale revision/count evidence, freeze a clean reproducible source/build receipt and keep `source_recheck_required` until then. |
| Runtime load/hot reload compiles source while holding lifecycle and process VM control paths | Runtime21, Runtime07 and Editor31 | Move compile/verify to Editor/cook BuildSet publication; Runtime stages immutable artifacts and atomically switches generation leases. |
| One process mutex serializes all packages/worlds/exports/GC/drop and underwrites unsafe `Send/Sync` | Runtime07 and Plugins16 | Certify native context/root/safepoint rules, then implement per-domain owner or isolated bounded worker; add contention/isolation receipts. |
| O(1) dense tokens still end in JSON field ABI and per-byte VM object/index operations | Plugins16 and Interface01/05/07 | Generate typed value/layout/buffer ABI and batch `ScriptWorldTransaction`; retain JSON only for debug/compat. |
| Global catalog revision invalidates every package token and can trigger full table rebuild on later resolve | Runtime07 reflection owner and Plugins16 | Use stable schema IDs, affected-owner immutable segments and generation leases; prohibit runtime full-table rebuild discovery. |
| Seven conservative systems and sequential callbacks prevent access-aware batching | Runtime07 task/schedule owner and Plugins16 | Compile Script Component access plans, batch by world/package/export and deterministically commit commands/events. |
| GC budget is post-observed/non-preemptive and ordinary calls have no execution quota | Runtime07 and Interface05 | Enforce fuel/deadline/cancel/host-call/allocation limits at VM safepoints; add heap/live/native/fragmentation telemetry and quarantine. |
| Ignored token benchmark omits compile, lock, ABI, World, GC and product work | this performance review and Plugins16 M8 | Build current-source 1/100/10k-instance and 1/4/16-world WPR/ETW workload matrix with correctness parity. |

## Acceptance routing

The package may move from static-reviewed to accepted only after M0-M7 close clean toolchain/product truth, offline verified artifacts, typed bulk ABI, certified execution ownership, access-aware transaction scheduling, enforceable GC/reload budgets, Editor workflow and current-source dynamic evidence. Required receipts include zero Runtime source compiles, zero frame reflection JSON, zero per-byte VM value allocations for bulk bytes, no process-global serialization across independent domains, bounded stale/cancel/fault behavior and BuildSet-bound P50/P95/P99 plus power data.

No Git milestone commit or quantified WeCom message is warranted by this static routing record.
