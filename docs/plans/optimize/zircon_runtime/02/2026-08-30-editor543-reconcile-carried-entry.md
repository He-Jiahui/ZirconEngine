---
title: Editor Reconcile Carried Entry 543
category: zircon_editor
report_id: Editor543-reconcile-carried-entry-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor Reconcile Carried Entry 543

The UI asset reconcile loop previously used `BTreeMap::contains_key`, cloned the selected instance
ID, then searched the same tree again with `get`. Selection now returns the map's `(key, value)`
references through `get_key_value`, `range`, or `first_key_value` and carries that entry through the
visit. Cursor reset, allowance accounting, import ordering, and next-session traversal are unchanged.

The ignored Release evidence `EDITOR543_RECONCILE_CARRIED_ENTRY_BENCH_V1` models 65,536 visits to
an existing cursor. Tree lookups fall from 131,072 to 65,536, a 50% reduction, while the hot path
also avoids cloning the selected ID. A standalone Rust 1.94.1 `opt-level=3` check used a 1,024-entry
string-keyed `BTreeMap` and 1,000,000 existing-cursor visits per sample; the 11-sample median changed
from 535.49 ms to 223.06 ms, a 58.34% improvement on this machine. This is local lookup evidence,
not watcher end-to-end latency.

## Static evidence

- TDD RED: the structural gate found `contains_key` plus the second `sessions.get` lookup.
- TDD GREEN: selection carries `sessions.get_key_value(instance_id)` and neither legacy lookup
  remains.
- Existing watcher tests exercise paged reconcile ordering and allowance-bounded cursor progress.
- `rustfmt 1.94.1 --edition 2021` and scoped `git diff --check` pass.
- `zircon_editor/src/ui/host/asset_editor_sessions/refresh/reconcile.rs` SHA-256:
  `c873ccf40ad47c7886ba7aedc9b6858508f7496a11f8d22bfb2d01cbea97a33d`.
- `zircon_editor/src/ui/host/asset_editor_sessions/watcher/tests.rs` SHA-256:
  `bd58fe1418fa9f437c31c5b22c8b8e1567a72f3c3c32de43e624d7689e0b9ab6`.

## Acceptance gates

1. Managed Windows native Release compilation and focused Editor tests pass.
2. Existing, missing, exhausted, and allowance-limited cursors preserve reconcile behavior.
3. Ignored evidence emits the Editor543 marker and reports the 2-to-1 tree-lookup gate.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.
