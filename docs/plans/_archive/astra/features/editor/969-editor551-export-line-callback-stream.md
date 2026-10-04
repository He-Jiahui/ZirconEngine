---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/02/2026-08-30-editor551-export-line-callback-stream.md
implementation_files:
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/execution/output_capture.rs
tests:
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/execution/output_capture.rs
---

# Editor969 Editor551 export line callback stream

Incremental output decoding now delivers each line through a callback, leaving
the collecting adapter test-only and removing the production temporary line
`Vec` per chunk. Split boundaries, CRLF trimming, maximum length, ordering,
tail retention, and durable writes remain covered. Marker
`EDITOR551_CALLBACK_LINE_STREAM_BENCH_V1` models 65,536 chunks and zero
production temporary collections. Managed Editor Release validation remains
pending.
