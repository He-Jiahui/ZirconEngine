---
title: Editor Scene Selection Current Source Protected Plan Routing
date: 2026-08-23
status: routing_only
source_report:
  - docs/plans/performance/01/2026-08-23-editor-scene-selection-current-source-and-duplicate-toggle-m0.md
---

# Editor Scene Selection Current Source Protected Plan Routing

| Existing owner | Required adoption |
|---|---|
| Editor74 Selection Authority | Adopt the 6-file current fingerprint and duplicate-toggle M0. Keep implementation status open: M0 only canonicalizes duplicate targets and does not create the authority/request/receipt/delta product. |
| Editor59/60/70/73 viewport, hierarchy, eligibility and region owners | Produce unique qualified candidates when possible, but rely on authority canonicalization. Route region changes as one atomic mutation and preserve policy generation. |
| Editor61 document/world owner | Supply document/world/session identity and lifecycle transitions used by the authority; replacement/deletion must produce explicit prune/remap receipts. |
| Editor63 transaction owner | Consume immutable before/after selection snapshots or commit tokens; do not maintain a second mutable authority. |

Protected `docs/plans/performance/review.md` and `docs/plans/performance/pending.md` remain untouched. The protected-ledger owner may add one concise `scene/selection` entry after adoption; the adjacent active-migration viewport/mode folders remain pending for a new currentness freeze.
