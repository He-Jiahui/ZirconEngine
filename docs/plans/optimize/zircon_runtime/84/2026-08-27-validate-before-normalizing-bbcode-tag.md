---
title: Runtime84 Validate Before Normalizing BBCode Tag
category: zircon_runtime
report_id: Runtime84-validate-before-normalizing-bbcode-tag-2026-08-27
date: 2026-08-27
session_id: root-runtime84-validate-before-normalizing-bbcode-tag-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime84 Validate Before Normalizing BBCode Tag

## Scope

This slice removes temporary lowercase `String` allocations from rejected BBCode tag names and
decorator registrations. It preserves trimming, ASCII alphanumeric/underscore validation,
lowercase output for valid tags, and rejection of non-ASCII tag names. Valid tags still allocate
the one owned normalized name required by the token and decorator registries. The independently
modified HTML subset parser is outside this slice.

## Change

- Trim and validate the borrowed tag before creating an owned lowercase value.
- Use byte-level ASCII validation equivalent to the previous character predicates.
- Allocate the normalized `String` only after the tag passes validation.
- Cover valid mixed-case normalization plus invalid punctuation, whitespace, empty, and non-ASCII
  names with Rust tests and a source contract.

## Deterministic Performance Evidence

The standalone optimized Rust model rejects 65,536 tags whose final character is invalid and
alternates legacy/optimized order for 17 samples. The late invalid byte forces both paths to scan
the full trimmed tag. A counting allocator measures only normalization/rejection, and both
implementations return identical results for valid and invalid fixtures.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Rejected-tag allocations | 65,536 | 0 | 100% |
| Rejected-tag allocated bytes | 1,179,648 | 0 | 100% |
| Rejection batch P50 | 11.891 ms | 5.308 ms | 55.365% |
| Rejection batch P95 | 13.228 ms | 5.985 ms | 54.758% |

Evidence marker: `RUNTIME84_VALIDATE_BEFORE_NORMALIZING_BBCODE_TAG_MODEL_V1`.

## Validation

- `python -m unittest tools.tests.test_runtime84_validate_before_normalizing_bbcode_tag_performance_contract -v`: 3 passed.
- Exact-file Rust formatting and scoped diff checks passed.
- Managed Rust compilation and focused tests remain pending in the asynchronous coordinator batch.

## Remaining Parent-plan Work

Runtime84 still owns bounded parser stacks, URL/image policy, table/list layout, selection,
accessibility, cache generation, and product-scale security/performance validation gaps recorded in
the canonical review.
