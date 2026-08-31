Plan: docs/plans/optimize/zircon_runtime/84-runtime-rich-text-markup-parser-token-style-span-inline-object-link-image-table-list-layout-selection-accessibility-security-product-integration-current-source-review.md
Milestone: M8
Status: completed
Files: ["zircon_runtime/src/text/rich/parser.rs", "zircon_runtime/src/text/rich/parser/performance_tests.rs", "tools/tests/test_runtime84_grapheme_alignment_performance_contract.py"]

# Runtime84 Grapheme Alignment Metadata Clone Collapse

## Scope delivered

This batch optimizes the parser's final run-to-grapheme alignment without changing public rich-text
contracts or parser syntax.

- Canonical ASCII runs now bypass per-byte grapheme iteration after proving exact contiguous text
  coverage, non-empty in-range spans, and the absence of adjacent mergeable metadata.
- Unicode and non-canonical inputs still follow the complete grapheme path, but compare borrowed
  style, inline-object, and link metadata before cloning. Metadata is cloned only when a new output
  run is emitted.
- The normalized output vector starts with the input run count as its capacity. Gap, overlap,
  combining-mark, adjacent-equal-run, and empty-input cases are compared against the legacy
  algorithm in the Rust regression module.

The broader Runtime84 plan remains open: versioned parser contracts, budgets, diagnostics,
service/provider ownership, incremental documents, security, accessibility, product migration,
and the full fault/scale qualification matrix are not claimed by this slice.

## Fresh testing evidence

TDD first produced three failing source-performance contracts against the per-grapheme clone path.
After implementation, all three pass, Python bytecode compilation passes, and Rust 1.94.1
`rustfmt --check` passes for production and benchmark modules.

Five process-level repetitions of a conservative standalone Rust benchmark produced these
median-of-run nearest-rank values. Its ASCII legacy loop iterates bytes directly, omitting the real
Unicode segmentation overhead and therefore understating the old path cost.

| workload | legacy | optimized | reduction |
| --- | ---: | ---: | ---: |
| 250k canonical ASCII bytes, P50 | 23.2002 ms | 0.0236 ms | 99.898% |
| 250k canonical ASCII bytes, P95 | 27.0321 ms | 0.0357 ms | 99.868% |
| ASCII metadata clones | 250,000 | 128 | 99.949% |
| 50k combining-mark graphemes, P50 | 4.6029 ms | 0.8230 ms | 82.120% |
| 50k combining-mark graphemes, P95 | 6.2599 ms | 1.2083 ms | 80.698% |
| Unicode metadata clones | 50,000 | 1 | 99.998% |

The managed Windows validation batch will rerun the existing rich-text behavior suite, explicit
legacy equivalence tests, formatting, and the actual `unicode_segmentation` release benchmark.
No local Cargo command or Cargo dry-run was launched.

## Review

The change stays in the private parser module and its test-only child. The ASCII proof rejects any
input that the legacy normalizer would repair or merge, while the fallback retains the monotonic
run cursor and exact grapheme ownership rule. Independent review remains an integration gate after
managed validation returns.
