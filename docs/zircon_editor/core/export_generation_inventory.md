---
related_code:
  - zircon_editor/src/core/export/inventory.rs
  - zircon_editor/src/core/export/stages/executor.rs
  - zircon_editor/src/core/export/stages/compile_host.rs
  - zircon_editor/src/ui/host/native_dynamic_export_preparation/staging.rs
plan_sources:
  - docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
tests:
  - zircon_editor/src/core/export/inventory.rs
  - tools/tests/test_editor15_export_generation_inventory_contract.py
doc_type: module-detail
---

# Export Generation Inventory

`ExportGenerationInventory` is the only owner of export filesystem content identity. Stage code requests artifacts by path; it does not recursively read files, probe tools, or maintain a second digest cache.

## File And Directory Identity

Existing paths are canonicalized before lookup. A file is read and BLAKE3-hashed at most once in one inventory generation. Directories form deterministic Merkle digests from ordered child names and child digests, so a reported root and its separately reported children reuse the same file nodes.

The persistent cache lives at `.zircon/cache/export/file-inventory-v1.json`. Windows reuse requires size, `FILE_BASIC_INFO` write/change markers, volume serial and `FILE_ID_INFO`; Unix uses size, mtime/ctime, device and inode. If strong identity cannot be obtained, the entry is not cacheable and content is read again. Directory refresh removes deleted descendants from the persistent cache. Rebuilt subtree invalidation removes the subtree, cached ancestors and persistent records before outputs are re-read.

## Tool And Parameter Identity

Python, Cargo, rustc and target-required Node identities are probed once per inventory generation. Unchanged tool identities do not dirty or rewrite the persistent cache. The executor also computes the immutable preset/command parameter digest once and reuses it for both CompileHost and PlatformBundle preparation.

## Persistence And Consumers

Inventory and native-staging manifests are encoded through the shared atomic persistence helper. Files are written and synced to a unique staging path; Windows replacement uses `MoveFileExW` with replace-existing and write-through semantics.

CompileHost writes complete stdout/stderr artifacts plus a byte-count/BLAKE3 manifest while retaining only 64 KiB tails in memory. Native dynamic package staging consumes the inventory and applies changed/deleted/renamed deltas under `.zircon/cache/export/native-dynamic`; unchanged warm staging copies zero files and bytes. Wizard output, pane projection and report parsing have separate bounded/cache contracts but must not create another filesystem digest authority.

## Invariants

- Metadata is only a strong-identity fast path; untrusted identity never suppresses content verification.
- No stage-local recursive digest helper or destructive per-run native staging tree is allowed.
- Full logs remain artifacts; live UI events carry one typed delta and terminal state retains bounded tails.
- A performance claim must report content bytes read, content hash count and p95 from the explicit ignored warm-cache test.
