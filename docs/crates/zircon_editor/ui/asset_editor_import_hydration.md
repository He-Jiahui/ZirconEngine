---
related_code:
  - zircon_editor/src/ui/host/asset_editor_sessions/imports/generation.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/imports/traversal.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/hydration.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/refresh/imports.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/refresh/pipeline/job.rs
plan_sources:
  - docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
tests:
  - zircon_editor/src/ui/host/asset_editor_sessions/imports/tests.rs
  - tools/tests/test_editor07_ui_asset_import_physical_cache_contract.py
doc_type: module-detail
---

# UI Asset Import Hydration

`UiAssetImportTraversal` owns one import hydration generation. Initial editor hydration and deterministic refresh delegate to one lossy collector before walking their widget and style roots; a watcher worker shares one traversal across every affected document in its batch. Each consumer commits the resulting document maps, diagnostics, and dependency edges after the walk. A traversal is never retained across source revisions.

The traversal separates physical and logical identity. The canonical resolved source path owns the file read target, parser-mode selection, and the key for `Arc`-backed cached read, parse, and v2 projection results, including failures. An alias or symlink path therefore cannot cache content or parsing semantics selected by a different identity. The original import string keys the resolved document maps, so references such as `res://ui/button.zui#Primary` and `res://ui/button.zui#Compact` retain distinct logical rows while sharing one physical parse. Nested imports expand once per canonical physical path, which terminates cycles and prevents diamond graphs from repeating disk reads or parsing.

Expected-kind validation remains a logical-edge operation. A cached physical document is checked against every reference's requested widget or style kind before that reference is materialized. Caching therefore does not suppress fragment aliases or expected-kind diagnostics.

This boundary removes repeated physical import work inside one hydration generation. Watcher-driven I/O/parse runs in the background refresh job; explicit hydration remains a deterministic caller-owned operation. Source-typing debounce and property-level presentation deltas remain separate requirements before the parent performance failure can close.
