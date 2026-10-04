---
title: Neural ONNX Editor Dist Current-Source Protected Plan Routing
date: 2026-08-24
status: routing_record_only
source_review: docs/plans/performance/01/2026-08-24-neural-onnx-editor-dist-current-source-performance-review.md
---

# Neural ONNX Editor Dist Current-Source Protected Plan Routing

## Review ledger status

Neural Editor production completed static review at fingerprint `948d24490003c5730c3c2077ddbb1081c68b93fa17ecb7f346794cee29192f03`; assembly/feature/Dist completed at `101fb2c137f612da6b4907c323caeda23577ad5676362cc0fcca489c2de5facb`. Protected ledgers remain unchanged pending executable validation.

## Canonical owner notifications

| Finding | Canonical plan owner | Required update |
|---|---|---|
| Unbudgeted ONNX parser, lossy identity and payload duplication | Plugins02 | Use maintained parsing or strict aggregate budgets and typed source semantics. |
| Synchronous whole-file import and direct overwrite | Editor04 + Editor09 + Plugins02 | Run a cancellable background import transaction with staged, validated, atomic publication. |
| Whole previous artifact retained for undo | Editor04 + Plugins02 | Store content-addressed generation/transaction receipts rather than model bytes. |
| Import success disconnected from runtime/provider readiness | Plugins02 + Editor04 | Qualify cook/install by target, provider, build and artifact generation. |
| Metadata-only Dist shape | Plugins02 | Keep capability explicit until static/native lifecycle and behavior parity execute. |

## Acceptance routing

Implementation order is bounded source admission -> background job -> atomic artifact transaction -> generation receipt -> target/provider cook -> product qualification. Static review does not warrant protected-ledger promotion, a Git milestone commit or a WeCom completion message.
