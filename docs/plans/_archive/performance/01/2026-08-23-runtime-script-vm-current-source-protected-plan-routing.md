---
title: Runtime Script VM Current Source Protected Plan Routing
date: 2026-08-23
status: routing_only
related_report:
  - docs/plans/performance/01/2026-08-23-runtime-script-vm-current-source-and-direct-input-query-revalidation.md
protected_targets:
  - docs/plans/performance/review.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
---

# Runtime Script VM Current Source Protected Plan Routing

This record only supplies merge input for protected ledgers and other plans. No protected file was modified.

## `review.md` suggestion

Do not add the module yet. Static current-source coverage is complete and the direct input-query work reduction is confirmed, but Runtime21's four P0s, process-global backend serialization, execution/GC/product gates, ignored Vampire tests and all current BuildSet dynamic evidence remain open.

## `pending.md` suggestion

`zircon_runtime/src/script`: 102/102 current-source static composite review complete; awaiting reproducible Zr toolchain/BuildSet, bounded `.zro` verifier, per-world execution/context/GC ownership, typed `ScriptWorldTransaction`, enforced execution/memory budgets, real-provider Vampire tests, and managed Cargo/WPR product evidence. Direct `button_pressed` removes snapshot cloning, but raw string parsing, service resolution and input locking remain until Runtime99r/App06 cut over to compiled actions.

## Canonical routing

| Issue | Target plan |
|---|---|
| parser/type/SemIR/codegen, toolchain receipt, `.zro` verifier, artifact generation, execution budget | Runtime21 |
| generic package/backend lifecycle, per-slot lease, concurrency, isolation and product-scale evidence | Runtime07 |
| Zr provider/carrier/catalog, typed host/reflection transaction, VM context/GC/reload and plugin product qualification | Plugins16 |
| compiled action, frame input view, device/action ownership and raw-key retirement | Runtime99r / Runtime117 |
| source workspace, LSP, build, debugger, profiler and Script Class/Component product | Editor31 |
| Vampire action migration, gameplay/HUD/menu behavior and current product receipt | App06 |
| cross-language ABI, diagnostic, budget/fuse and certification | Interface04/05/07 |

Do not add another VM manager, reload coordinator, host registry, input snapshot cache or string-call dispatcher. Preserve the dense prepared `ScriptCallTable`, borrowed host values, cooperative GC scheduling and direct concrete input query, then converge them under one qualified artifact/world generation.

## Promotion gates

1. A clean source/build receipt reproduces the toolchain and artifact digest; runtime never compiles developer source.
2. `.zro` fails closed on malformed header, section, width/endian, length, checksum, opcode, CFG, stack/type/layout or budget before materialization/execution.
3. `1/100/10k` package/instance and multi-world work no longer serializes on a process-global lock; context/root/safepoint behavior has sanitizer/model evidence.
4. Host calls and ticks enforce fuel, deadline, cancellation, call/effect/byte/allocation budgets; GC is live-byte driven and attributable per slot/world/generation.
5. Vampire uses typed action state and real provider tests; the ten transferred ignores are removed only after equivalent current tests run.
6. Managed Windows WPR records frame/host-call/GC/lock/thread/I/O/RSS/power metrics on the same current-source executable. RenderDoc is required only for visible script-driven frame correctness and cannot promote CPU/GC performance alone.
