---
title: Editor48 View Invalidation Mask Unknown-bit Rejection
category: zircon_editor
report_id: Editor48-invalidation-mask-unknown-bits-2026-09-28
date: 2026-09-28
implementation_status: partial_implemented_pending_validation
validation_status: managed_validation_pending
---

# Editor48 view invalidation mask unknown-bit rejection

## Scope

E-MSG-P1-34 identifies that the serializable `EditorViewInvalidationMask` accepted every `u16`
value, including bits for which no refresh behavior is defined. An unknown bit could reach a
consumer and silently avoid the intended layout, reflection, or paint branch.

## Change

The mask retains its numeric `u16` serialization and its existing defined flags. Its custom
`Deserialize` implementation accepts only the union of the nine currently defined bits and
returns a deserialization error when any other bit is set. This validation also applies when the
mask is nested inside a serialized dirty set or message mark. Valid mask operations and the
in-memory representation are unchanged.

The focused regression round-trips the union of all defined flags, checks that it remains a
numeric value, and rejects both an unknown bit alone and that bit combined with defined flags.
The test was authored against the previous unchecked derived deserializer; Cargo execution is
pending at the grouped milestone boundary.

## Validation and remaining gate

- Pinned Rustfmt 1.94.1 read-only check and scoped `git diff --check` passed for both Rust files.
  A mapped-file Windows error prevented Rustfmt's write mode, so formatting was applied by
  guarded in-place edits and confirmed with the read-only check. Existing unrelated comments
  were preserved.
- Direct UTF-8, final-newline, no-BOM, no-replacement-character, and trailing-whitespace checks
  passed for the two Rust files and both records. No Cargo compilation or test pass is claimed.
- E-MSG-P1-34 remains partially open: a versioned cross-process mask codec and a documented
  forward-compatibility policy have not been added. The wider Editor48 performance and delivery
  gates also remain open.
