---
title: Runtime624 Preallocated Animation Graph Traversal
category: zircon_runtime
report_id: Runtime624-preallocated-animation-graph-traversal-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime624 Preallocated Animation Graph Traversal

Animation graph evaluation now reserves cycle-detection membership from the frozen graph node
count before recursive traversal. The previous temporary `HashSet` grew from zero even though the
maximum simultaneous path membership cannot exceed the graph's node count.

Output-node selection, depth-first input ordering, cycle suppression, blend weights, mask
inheritance, and the existing first-seen mask-target projection remain unchanged. Focused source
coverage rejects a zero-capacity traversal set.

The ignored Windows Release benchmark emits
`RUNTIME624_PREALLOCATED_ANIMATION_GRAPH_TRAVERSAL_BENCH_V1` over 17 alternating sample pairs,
32,768 node IDs, and 32 insert passes. The gate requires preallocated membership P95 to be at most
85% of the unreserved baseline.

No direct Cargo validation was run. The coordinator owns the combined Runtime624/Editor624 Windows
Release regression and performance batch. Receipt, measured P95, commit, push, and WeCom outcome
are recorded only after coordinator completion.
