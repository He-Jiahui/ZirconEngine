---
title: Editor48 Topic Deserialization Admission
category: zircon_editor
report_id: Editor48-topic-deserialization-admission-2026-09-28
date: 2026-09-28
implementation_status: partial_implemented_pending_validation
validation_status: managed_validation_pending
---

# Editor48 topic deserialization admission

## Scope

`EditorTopic::parse` requires at least two nonempty dotted segments containing lowercase ASCII
letters, digits, `_`, or `-`. The public serde `Deserialize` derived for its private `String`
field accepted strings that `parse` rejects. `EditorMessageDelivery` also derives its nested topic
from that type, so a complete delivery JSON value could carry an invalid topic through the DTO
boundary. This is a narrow admission gap within E-MSG-P1-26; no production external JSON decode
caller has been established by this slice.

## Change

`EditorTopic` retains its string wire shape and `Serialize` implementation. Its explicit
`Deserialize` implementation reads a string and passes it through `EditorTopic::parse`, returning
a serde error for an invalid topic. The focused tests exercise direct topic JSON and complete
`EditorMessageDelivery` JSON, including valid round trips, empty and malformed segments,
uppercase and non-ASCII input, and a non-string topic field.

The preexisting TODO next to `EditorTopic` identified this precise bypass. Its original line is
preserved byte for byte, with an adjacent comment marking this narrower gap resolved.
The module guide at `docs/crates/zircon_editor/core/editor_message.md` now states the
string wire and parse-on-deserialize contract for direct and nested topics.

## Validation and remaining gates

- The regression was written and wired before the implementation. With the old derived
  deserializer, its invalid-string assertions are expected to fail; that is a source-level RED
  deduction, not an executed test result.
- Scoped Rustfmt, whitespace, UTF-8, reference, frozen-path, and exact-preimage inverse checks
  are recorded in the Editor1036 handoff. Managed Cargo compilation and test execution remain
  pending with the grouped milestone; no dynamic pass is claimed here.
- E-MSG-P1-26 still needs length and budget admission, a schema registry, namespace ownership,
  and plugin capability checks across the other identifiers and message fields. A versioned
  external wire protocol and real external decode integration remain separate product gates.
  No performance improvement or Editor48 acceptance is claimed.
