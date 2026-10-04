---
title: Editor629 Preallocated Palette Catalog Vectors
category: zircon_editor
report_id: Editor629-preallocated-palette-catalog-vectors-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor629 Preallocated Palette Catalog Vectors

Command-palette catalog construction now uses the descriptor iterator lower bound to reserve the
seed and enablement vectors before projecting descriptors. Registry snapshots provide an exact
size hint, while arbitrary iterators remain supported through the lower-bound contract.

Descriptor order, duplicate replacement semantics, BTreeMap lookup order, localization, and
enablement behavior remain unchanged. Only vector growth reallocations are removed.

The ignored Windows Release benchmark emits `EDITOR629_PREALLOCATED_PALETTE_CATALOG_BENCH_V1`
over 17 alternating sample pairs and 32,768 descriptors. The gate requires preallocated catalog
vector construction P95 to be at most 85% of the unreserved path P95.

No direct Cargo validation was run. The coordinator owns the combined Runtime629/Editor629 Windows
Release regression and performance batch. Receipt, measured P95, commit, push, and WeCom outcome
are recorded only after coordinator completion.
