---
title: Plugin Opus Importer Current Source Protected Plan Routing
date: 2026-08-23
status: routing_only
source_report:
  - docs/plans/performance/01/2026-08-23-plugin-opus-importer-current-source-product-gap-review.md
---

# Plugin Opus Importer Current Source Protected Plan Routing

| Existing owner | Required adoption |
|---|---|
| Plugins07 importer architecture | Treat Opus as unavailable until a real backend lease, bounded operation and derived artifact writer exist. Own source/dependency/subasset/cache/sandbox/determinism gates. |
| Plugins11 sound product | Own codec/playback artifact, streaming decoder/ring, audio deadline/underrun and seek/loop behavior. Preserve import/playback separation. |
| First-party runtime catalog owner | Add a typed missing-provider diagnostic for enabled `OpusImporter`; after implementation, generated provider slot and capability readiness must bind to the validated provider generation. |
| Plugin SDK/native ABI owner | Define decoder/import operation negotiation, stateful lease, unload/reload and terminal receipt contracts. Registration manifest alone must not imply decode capability. |

Protected `docs/plans/performance/review.md` and `docs/plans/performance/pending.md` remain untouched. The protected-ledger owner may add one concise `opus_importer` pending entry after adopting this report; RenderDoc is not an audio acceptance requirement.
