---
record_kind: milestone
status: implemented_pending_validation
created_at: 2026-09-12
plan: docs/plans/astra/optimize/01-review-and-repair.md
milestone: Neural P0-03 signed ONNX dimension and UTF-8 identity admission
session: astra-neural-graph-admission-20260912
---

# Neural P0-03 signed ONNX dimension and UTF-8 identity admission

The ONNX reader previously cast protobuf `int64` dimension varints directly to
`u32`. Negative values (and positive values beyond the V1 `u32` range) could
therefore become wrapped shapes before the converter's later checks. The
reader now reinterprets the wire value as signed `i64`, applies a checked
`u32::try_from`, and returns the typed `OnnxReadError::InvalidDimension` before
publishing the parsed tensor or value-info shape.

It also previously used `String::from_utf8_lossy` for ONNX names and string
attributes, silently normalizing malformed source bytes into a different asset
identity. `read_string` now accepts only UTF-8 and returns the typed
`OnnxReadError::InvalidUtf8String` before a malformed field can enter the
parsed graph.

Focused reader regressions cover negative packed and unpacked TensorProto
dimensions, oversized dimensions, negative `ValueInfo` shape dimensions, and
malformed UTF-8 TensorProto-name and attribute-string fields. The change is
deliberately limited to these identity and dimension admission boundaries;
parser byte/item/depth/string budgets, symbolic dimensions, and external-data
handling remain open under the Neural review's P0-03/M2 work.

## Evidence boundary

- TDD RED source probe confirmed the malformed UTF-8 regression existed while
  the production reader still performed lossy identity normalization.
- `rustfmt --edition 2021 --check`, scoped `git diff --check`, and a source
  contract confirming all dimension paths use the checked helper and no
  production lossy string conversion remains pass.
- No Cargo, native, GPU, or product command ran. Managed Windows validation is
  still blocked by the dirty external `E:\\Git\\zr_vm` worktree.

Post-edit SHA-256 fingerprint:

```text
zircon_plugins/neural/editor/src/onnx/reader.rs 26029F0CF5DB809972F57E9AABF168AEE4826E1111220D763DEF5C1523A24A60
```

This record does not claim full parser hardening or Windows acceptance. No
commit was created.
