---
status: implemented_pending_validation
plan: docs/plans/astra/optimize/01-review-and-repair.md
milestone: NNET-P1-025 bounded HTTP response collection
session: astra-net-http-response-limit-20260921
---

# Bounded HTTP response collection

## Finding

Both HTTP client transports buffered an entire response before publishing a
`NetHttpResponseDescriptor`. The plain HTTP Hyper path called `collect()` on an
unlimited incoming body, while the HTTPS Reqwest path called `bytes()`. A peer
could therefore grow process memory independently of the request timeout.

## Implemented Slice

The HTTP feature now owns one 16 MiB response-body ceiling. Hyper wraps the
incoming body in `http_body_util::Limited`. Reqwest rejects an oversized
declared length and otherwise consumes chunks while checking the accumulated
length before reserving or appending. Exact-limit responses remain accepted;
limit-plus-one responses fail before a public response descriptor is created.

This closes only the response-limit part of `NNET-P1-025`. Client pooling,
stream-to-consumer delivery, cancellation, idempotent retry policy, backoff,
and `Retry-After` handling remain open under the network owner plan.

## Validation Boundary

The TDD static contract records the bounded shape for both transports and the
Rust route regressions cover exact-limit and limit-plus-one local responses.
Formatting, static tests, and scoped diff checks are required for this slice.
Managed Cargo and product network validation remain pending; no acceptance is
claimed without their terminal receipts.
