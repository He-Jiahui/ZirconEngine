---
title: Runtime01 Shared Registry Name Storage
category: zircon_runtime
report_id: Runtime01-shared-registry-name-storage-2026-08-26
date: 2026-08-26
session_id: root-runtime01-shared-registry-name-storage-20260826
implementation_status: implementation_complete
validation_status: managed_validation_queued
---

# Runtime01 shared registry name storage

## Scope

- Parent scope: Runtime01 registry build and resolution bookkeeping, specifically repeated ownership of validated service registry names across descriptors, dependencies, graph nodes, and activation plans.
- Baseline: `8e56165c4c789416c328898d3d8937d934b52efa`, epoch `443`; `registry_name.rs` source blob `7d19185511e98d306a885d48446b1acda4e40820`.
- Owners: `RegistryName` value storage and focused behavior contract, the standalone clone workload, and this record.
- This slice preserves name validation, module/kind/service offsets, equality, hashing, `Borrow<str>`, display, and serde behavior. It does not change registry graph algorithms, lifecycle state, resolution admission, or the remaining Runtime01 acceptance gates.

## Change

- `RegistryName` now stores its validated immutable value as `Arc<str>` instead of `String`.
- `new` retains the owned `String` through every validation error path and moves it into `Arc<str>` only after validation succeeds.
- `from_parts` builds the canonical value in one `String` and moves the completed value into shared storage without cloning it.
- Derived `Clone` now increments a reference count instead of allocating and copying the full registry name. A direct Rust contract proves clones share the same allocation while preserving full, module, kind, and service views.

The registry graph intentionally clones names into multiple immutable ownership locations. Shared string storage removes those repeated string allocations without changing the public name API or coupling graph-node lifetimes.

## TDD and local evidence

- RED: `python -m unittest tools.tests.test_runtime01_shared_registry_name_performance_contract -v` initially failed 4/4 because the shared field, both no-clone promotion paths, and the Rust pointer-sharing contract were absent.
- GREEN: the same focused source contract passes 4/4 after implementation.
- `rustfmt 1.94.1 --edition 2021 --check --config skip_children=true` and scoped `git diff --check` pass.
- The standalone model is compiled with `rustc 1.94.1 -O`; it does not use Cargo or the shared build lane.

The deterministic Rust model measures 31 alternating legacy/shared sample pairs. Each sample constructs 65,536 distinct registry names and makes eight owned clones per name. Both representations produce checksum `8279578959801848613`. Four sequential local runs passed the acceptance thresholds:

| Run | P50 reduction | P95 reduction | Legacy allocations | Shared allocations | Allocation reduction |
|---:|---:|---:|---:|---:|---:|
| 1 | 68.068% | 67.931% | 589,824 | 131,072 | 77.778% |
| 2 | 66.255% | 67.284% | 589,824 | 131,072 | 77.778% |
| 3 | 67.384% | 68.727% | 589,824 | 131,072 | 77.778% |
| 4 | 64.845% | 72.249% | 589,824 | 131,072 | 77.778% |

These timings isolate construction plus repeated immutable-name cloning. They do not claim complete registry build, resolve, or lifecycle latency improvement.

## Current-source revalidation

- The current-source release benchmark preconstructs 65,536 distinct legacy and shared names, then measures eight owned clones per name across 21 alternating sample pairs. It emits nearest-rank P50/P95 values and requires both shared percentiles to be no greater than 50% of legacy.
- The source-bound deterministic model covers 524,288 owned clones. Legacy `String` clones require 524,288 immutable payload allocations and copy 21,318,048 payload bytes; shared `Arc<str>` clones require zero payload allocations and copy zero payload bytes, a 100% payload-allocation and payload-copy reduction. Vector storage and reference-count operations remain outside those payload counts.
- Static current-source evidence passes 9/9 contracts, Python compilation, PowerShell AST parsing, Rust 1.94.1 formatting, and scoped diff checks. The source manifest is `D8F55509EA0FE9A67ED55BCF61CAD9367854C958427FB12AD9AF382A203F061A`; `registry_name.rs` SHA-256 is `43ACD21E5E310E2F605C309BFB7B64A5B38DBAE2ECE08BE1E0D038202E1147DB`.
- The standalone model is structural work evidence, not a product timing claim. Managed release output remains authoritative for P50/P95 acceptance and for the WeCom performance summary.

## Async validation

Managed ticket `f2a72aab4f6a40f7b986bd5c09792403` is queued against snapshot `2445` and source manifest `d73f10b0eb68816ad52d61331a6bd03767daaca32453ba7676cfb7c072935ac7`. Its single batch command owns all static, correctness, and release-performance execution for this current-source slice; terminal P50/P95 evidence remains pending and is not inferred from queue admission.

The coordinator validation batch runs all nine focused Python contracts, the `registry_name_clones_share_value_storage` Rust test, and the ignored `registry_name_clone_release_benchmark_evidence` release benchmark from one script. Static Rust formatting and scoped diff checks have already passed locally without Cargo. The two Rust commands intentionally share one managed ticket rather than creating one request per assertion.

Acceptance requires 9/9 source contracts and 2/2 Rust tests to pass, clone-work checksum parity, `524,288` legacy versus zero shared payload allocations, zero shared payload-copy bytes, and managed P50/P95 reductions of at least 50%. Cargo validation remains required even while foreign workspace inputs or unmanaged build artifacts can prevent the coordinator from starting the command. Integration and automatic WeCom publication remain coordinator-owned after managed validation succeeds. The WeCom message must include managed P50/P95 and the exact payload-allocation/copy reductions and label the managed timings as eight-clone evidence for 65,536 preconstructed registry names; the earlier construction-plus-clone runs remain historical supporting evidence only.
