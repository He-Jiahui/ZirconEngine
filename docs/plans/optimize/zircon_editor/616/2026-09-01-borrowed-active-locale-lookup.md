---
title: Editor616 Borrowed Active Locale Lookup
category: zircon_editor
report_id: Editor616-borrowed-active-locale-lookup-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor616 Borrowed Active Locale Lookup

The single-key translation path now keeps the active-locale read guard and performs the bundle
lookup through its borrowed locale identity. It no longer clones the locale's `Arc<str>` on every
translation. Fallback ordering, returned translation ownership, and poison recovery remain
unchanged.

Focused source coverage rejects the clone-per-lookup path. The ignored Windows Release benchmark
emits `EDITOR616_BORROWED_ACTIVE_LOCALE_LOOKUP_BENCH_V1` over 17 alternating sample pairs and
262,144 lookups per sample. The gate requires borrowed lookup P95 to be at most 80% of the retired
clone-per-lookup path and reports the exact allocation counts.

No direct Cargo validation was run. The coordinator owns batched compilation, focused regressions,
ignored Release performance evidence, integration, push, and one-shot WeCom publication.
