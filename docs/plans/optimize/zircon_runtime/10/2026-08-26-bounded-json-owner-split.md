# Runtime 10 bounded JSON owner split

## Scope

- Target: `zircon_runtime/src/dynamic_api/bounded_json.rs`.
- Baseline: clean 724-line tracked owner containing 607 production lines and 117 inline test lines.
- Priority sources: Runtime 10, the engine structure convention/review findings, the Runtime43 dynamic-session review, the runtime-interface 09 host review, and the 2026-08-23 dynamic API performance review.
- This slice changes ownership only. It does not remove a safety pass, change payload limits, claim a latency/power improvement, or close the bounded JSON optimization.

## Algorithm review before optimization

Inbound `decode` currently performs:

1. a 4 KiB-chunk lexical nesting scan;
2. an allocation-free serde syntax-graph preflight capped by `max_encoded_bytes + 1`;
3. typed deserialization;
4. a business-item count supplied by the payload owner.

The successful inbound path therefore performs two to three full wire traversals before/while constructing the business value. Existing performance and interface reviews classify this as P1, but they contain no current WPR/Tracy/allocator measurement because managed runtime validation is blocked. They explicitly require legal, malformed, deeply nested, and wide payloads at 1 byte, 256 KiB, and 1 MiB before replacing the algorithm.

The primary local Unreal references were `Runtime/Json/Public/Serialization/JsonSerializer.h`, `JsonReader.h`, and `JsonWriter.h`. Unreal separates reader state, serializer traversal, and writer state rather than putting every policy in one facade. Zircon adopts that ownership separation but retains its stricter ABI byte/item/depth/deadline policy and does not infer performance parity from Unreal's C++ implementation.

## Implemented layout

| Owner | Responsibility | Current lines |
|---|---|---:|
| `bounded_json.rs` | Dynamic API facade and unchanged decode/encode/validate stage order | 104 |
| `bounded_json/error.rs` | Bounded payload error contract and limit classification | 59 |
| `bounded_json/deadline.rs` | Cooperative processing deadline and 4 KiB deadline reader | 69 |
| `bounded_json/preflight.rs` | Syntax-graph visitor and JSON value item traversal | 185 |
| `bounded_json/writer.rs` | Counting/materializing writers and lexical nesting tracker | 218 |
| `bounded_json/tests.rs` | Existing behavior tests plus owner-boundary contract | 163 |

`BoundedJsonError`, `BoundedJsonWriter`, and `json_value_item_count` retain their original `dynamic_api` visibility through the root re-export. All helper owners remain private.

## Preserved invariants

- `checked_slice` still enforces the byte ceiling before dereferencing foreign input.
- Empty input, encoded bytes, business items, nesting depth, processing time, and JSON errors retain the same variants and text.
- Decode still scans nesting, runs syntax preflight, performs typed decode, and then evaluates the exact business-item closure in the same order.
- Encode and validate still use one serde serialization with materializing/counting writers respectively.
- The 4 KiB scan/read chunk, deadline checkpoints, nesting lexical state, byte-overflow handling, and no-empty rules are unchanged.
- All five original behavior tests were moved unchanged; one structure contract was added.

## Current evidence and status

- Scoped `rustfmt --edition 2021 --check` passed for the facade and five child files.
- Static migration comparison retained all 48 old function definitions and all 10 old type definitions.
- Production string literals are `20/20` with zero delta.
- Static counts are unchanged for the 4 KiB nesting scan, syntax preflight, typed decode, two serde encode sites, 11 deadline checks, three nesting inspections, item-limit errors, and nesting-depth errors.
- Root size changed from 724 to 104 lines; all production owners are at or below 218 lines.
- Production files contain no new `allow`, `unwrap`, `expect`, `panic`, `todo`, or `unimplemented` escape path.
- Managed Cargo and profiler validation were not requested while bypassing the shared validation blocker. Status is `implemented_static_passed_managed_validation_deferred_algorithm_unchanged`.

## Required optimization evidence

Before DYN-P1-059/RHOST-P1-073 implementation, capture per stage: scanned bytes, JSON values, keys/string bytes, allocations/allocated bytes, deadline checks, lock hold/wait, CPU p50/p95/p99/max, and rejection reason for legal/malformed/deep/wide 1 B, 256 KiB, and 1 MiB payloads. Then compare a single budget-aware typed visitor or arena decoder against the retained defense path, including hostile inputs. The work must also move decode outside the session owner lock; deleting preflight alone is not an accepted optimization.

No CPU, memory, latency, energy, or power improvement is claimed in this record.
