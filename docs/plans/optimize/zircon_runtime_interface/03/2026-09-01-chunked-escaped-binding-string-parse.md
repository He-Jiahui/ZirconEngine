---
record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/binding/model/parser.rs
related_tests:
  - zircon_runtime_interface/src/ui/binding/model/parser/string_performance_tests.rs
  - tools/tests/test_runtime_interface03_binding_string_parse_performance_contract.py
  - zircon_runtime_interface/src/ui/binding/model/parser/string_performance_tests.rs::runtime_interface03_batch49_50_chunked_escaped_binding_string_release_benchmark
---

# Chunked escaped binding string parse

## Scope

Escaped binding strings previously pushed every ordinary scalar one at a time. The parser now
appends each borrowed UTF-8 chunk between escape delimiters with `push_str` and decodes only the
escape itself per character. The accepted escape set, invalid-escape position, unterminated-string
result, Unicode preservation, and public binding syntax are unchanged.

## Verification

- TDD RED: the focused contract found no borrowed-chunk append path or escaped-string release
  benchmark.
- Focused Batch49-50 binding-string performance contract after implementation: `3/3` passed.
- Batched static regression after Batch49-50: `120/120` passed (`108` RuntimeInterface03
  performance contracts, `9` input-routing receipt contracts, and `3` asset-palette performance
  contracts).
- Rust behavior coverage uses the same former growing oracle across every supported escape and the
  malformed-input cases.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

Ownership receipt: initial exact-path lease request `d7e00102675348b1b75ecb47c90bfbe2`; refreshed
batch lease request `1630eb8489394345a24cff4a7e96fc60`; baseline attribution request
`b2bedff5ad0c4a2f9bac276139c2f4cc` (`attributed`).

## Performance contract

The ignored release benchmark parses a long binding string with three separated escape sequences
50,000 times over 11 alternating samples. It compares the former per-character growing parser with
borrowed chunk appends and requires at least 20% P95 improvement. Terminal nanosecond values must
come from the managed Windows receipt before integration, push, or WeCom reporting.

Combined Batch49-50 snapshot `2745` was created by request
`94d1a8dd5174464bafe6fd9e5e206db0`. Batched release request
`runtime-interface03-batch49-50-20260901-r1` was rejected before ticket creation by
`validation_ticket_external_worktree_dirty` for external worktree `E:\\Git\\zr_vm`. No Cargo run,
commit, push, or terminal performance value exists; the external worktree remains untouched.
