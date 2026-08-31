---
title: Neural Model Format CPU Current-Source Protected Plan Routing
date: 2026-08-24
status: routing_record_only
source_review: docs/plans/performance/01/2026-08-24-neural-model-format-cpu-current-source-algorithm-performance-review.md
---

# Neural Model Format CPU Current-Source Protected Plan Routing

## Review ledger status

Neural model/operator/CPU/GPU-support core production sources completed current-worktree static review: **14/14 Rust files**, fingerprint `64f13b0045687fa76589cbd9ebeb7a82ba38dfc345dc0cd976c35e064de6bc75`. Protected ledgers remain unchanged pending managed Rust and dynamic product evidence.

## Canonical owner notifications

| Finding | Canonical plan owner | Required update |
|---|---|---|
| Per-run validation, weight decode and full intermediate retention | Plugins02 | Introduce provider-qualified shared models and reusable instances/workspaces; retain scalar CPU only as oracle. |
| Converter-only executable truth and CPU/GPU semantic drift | Plugins02 | Compile one immutable `ValidatedNnGraph` consumed by every backend. |
| Hard-coded loader ceilings and whole weight copy | Plugins02 + Runtime64 | Replace format-private ceilings with accounted resource policy and shared/mapped generation storage. |
| No load/cache/reload/retirement lifecycle | Runtime64 + Plugins02 | Add async load state, identity, cancellation, last-good generation, leases and retirement receipts. |

## Acceptance routing

Implementation order is validated graph -> shared model/instance -> resource authority -> provider selection -> scale qualification. The current bounded-loader change must be rechecked in Plugins02, but static evidence does not warrant protected-ledger promotion, a Git milestone commit or a WeCom completion message.
