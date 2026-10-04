---
title: Sound Plugin Assembly and Capability Current-Source Protected Plan Routing
date: 2026-08-24
status: routing_record_only
source_review: docs/plans/performance/01/2026-08-24-sound-plugin-assembly-capability-current-source-performance-review.md
---

# Sound Plugin Assembly and Capability Current-Source Protected Plan Routing

## Review ledger status

Sound root/package/runtime-plugin assembly **15/15 Rust files** completed E3 current-worktree static review at `2a1299f8bf8e5a3012860ff07a6fcf528e4721d8`; fingerprint `a24bd5618b5c86bcc59e9083b89f6ae18128ef64083ae1c33c1451def3282cfd`. All files pass standalone rustfmt and scoped diff check; no source changed. Protected ledgers remain unchanged because Cargo, current-source plugin selection/startup/provider load, ETW and power evidence are unavailable.

## Canonical owner notifications

| Finding | Canonical plan owner | Required update |
|---|---|---|
| Registered manager always uses default config instead of selected plugin options | Plugins11 + Runtime58 + Runtime48 | Inject resolved immutable project config generation into module manager construction. |
| Component schema/default options expose unsupported advanced features | Plugins11 + Runtime08b + Editor17 | Filter schema/options by applied provider profile and default unavailable paths off. |
| Seven advanced config fields have no production consumer | Runtime03 + Plugins11 | Report them unapplied/unsupported; reject or remove inert options until an owner exists. |
| Optional feature manifests reference absent current-source provider crates/artifacts | Runtime58 + Plugins11 | Require concrete artifact/ABI/platform receipt before Ready, selection or export. |
| `sound.enabled` is absent while partial feature flags are declared but unenforced | Runtime48 + Plugins11 | Define one applied enable/disable transaction across service, output and background work. |
| Poison recovery continues on potentially partially mutated state | Runtime48 + Runtime03 | Transition to Failed/Recovering and rebuild from immutable last-good; do not silently continue. |
| Immutable manifest metadata is rebuilt through overlapping helpers | Runtime58 + Plugins11 | After contract convergence, generate/share one declaration generation and measure startup allocation. |
| Package/module/optional dependency identities are not one resolved graph | Runtime58 + Plugins11 | Compile one project/target dependency-capability graph consumed by startup/schema/export. |

## Acceptance routing

Implementation order is capability truth -> config injection -> default hard cutover -> provider packaging -> lifecycle/failure -> immutable declaration -> dynamic qualification. Do not close this scope by changing defaults alone while manager construction still ignores project selection.

Dynamic acceptance records exact source/build/project/target/provider/device identity, startup/config-resolution P50/P95, allocation/RSS, thread/handle count, idle/load CPU, wakeups, device latency, reload/disable/unload time, export artifact receipt and power.

No Git milestone commit or quantified WeCom message is warranted by this static routing record.
