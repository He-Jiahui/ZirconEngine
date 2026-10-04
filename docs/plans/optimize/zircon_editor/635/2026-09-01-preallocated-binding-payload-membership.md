---
title: Editor635 Preallocated Binding Payload Membership
category: zircon_editor
report_id: Editor635-preallocated-binding-payload-membership-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor635 Preallocated Binding Payload Membership

Binding schema projection now reserves its payload-key hash set from the known top-level payload
entry count plus the already-materialized schema suggestion count. The previous set grew from zero
while admitting the same ordered payload and default suggestions.

Explicit payload values still precede schema defaults, duplicate schema keys remain suppressed, and
diagnostic and item ordering are unchanged. The capacity is bounded by the exact number of key
admission attempts in the two existing loops.

The ignored Windows Release benchmark emits
`EDITOR635_PREALLOCATED_BINDING_PAYLOAD_MEMBERSHIP_BENCH_V1` over 17 alternating sample pairs and
65,536 unique payload keys. The gate requires preallocated P95 to be at most 85% of unreserved hash
membership P95.

No direct Cargo validation was run. The coordinator owns the combined Runtime/Editor regression and
performance validation. Measured P95, commit, push, and WeCom outcome are recorded only after
coordinator completion.
