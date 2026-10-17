---
title: UI Source Replay and Canonical Savepoint Completion List
category: astra-editor
report_id: Editor-ui-source-replay-savepoint-2026-10-14
date: 2026-10-14
implementation_status: source_reviewed
validation_status: native_and_performance_pending
---

# UI source replay and canonical savepoint completion list

This fixes existing basic authoring/persistence regressions that affect F3/F4 save and close. The internal follow-up tag F2 is not the product Scene-runtime milestone.

## Implemented and statically checked

- [x] Prepare fallible local source/document/projection/catalog replay before changing visible local state or committing the Undo/Redo cursor; preserve invalid-source diagnostics and last-good preview.
- [x] Reconnect canonical-save source history across both adjacent neutral runs and the first source-changing boundary in each direction, preserving authored neighbor states and the complete history.
- [x] Map UTF-8 cursor endpoints once, refresh source range caches and keep source cleanliness independent from navigation-only replay changes.
- [x] Clear only the clean source-buffer dirty owner by revision CAS; preserve other owners, the actual durable receipt and primary publication/durability errors.
- [x] Freeze the nine-file source packet and pass scoped formatting, whitespace and patch-guard checks. Root matched current source hashes to the packet.
- [x] Complete independent semantic review of the frozen source packet; no static blocker found, with native/product/performance acceptance still pending.

## Acceptance still required

- [ ] Execute the combined ordinary Editor arm with its previous floor 32, 60 newly selected disjoint cases (11 authored, 49 existing), and three existing native palette-input cases: floor 95 plus all individual named-case requirements.
- [ ] Execute the unchanged Editor150 and Editor811 producers alongside every previously required producer: ignored Release arm floor 12. Preserve Editor150's 21 paired samples, 2,048 builds, 256 effects and optimized P95 at most 70% of legacy.
- [ ] Preserve all original Runtime/Editor ratios, counts, workload sizes, absolute budgets, resource caps, complete source/environment seals and process-termination evidence.
- [ ] Complete native tests, performance, product/UI and applicable MVP acceptance. Static evidence does not satisfy these gates.

At this source-record freeze, Batch12 is planned and has not yet been handed off. Subsequent file-handoff and receipt evidence is recorded separately in the task manifest. Root and source agents have run no Cargo/native/product/performance tests or Python unittest for this packet. The next independent Runtime repair starts after the actual grouped handoff, without compilation waits or live coordinator monitoring.

The patch, exact hashes, test/module wiring, cost limits and asynchronous lifecycle are recorded in the [optimization record](../../../optimize/zircon_editor/63/2026-10-14-ui-source-replay-and-savepoint-record.md) and its linked frozen evidence. Source agents created no native products or fixtures; actual released space is 0 bytes. Minimal source/evidence/plan records are retained until native and performance acceptance is reconciled.
