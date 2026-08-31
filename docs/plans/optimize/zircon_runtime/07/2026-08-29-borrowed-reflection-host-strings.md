---
title: Runtime07 Borrowed Reflection Host Strings
category: zircon_runtime
report_id: Runtime07-borrowed-reflection-host-strings-2026-08-29
date: 2026-08-29
session_id: root-runtime07-incremental-capability-set-builder-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime07 Borrowed Reflection Host Strings

## Scope

This slice removes transient `String` allocations from the ZrVM reflection host's resolve and
write calls. Script arguments are already exposed as scoped borrowed views, so the host can run
reflection lookup and JSON mutation while the argument frame remains alive.

## Change

- Replace the owned `expect_string` extractor with a generic `borrow_string` visitor helper.
- Pass borrowed type paths, member names, and JSON payloads directly to reflection operations.
- Preserve argument validation text and the `ZrVM reflection host call failed:` error prefix.
- Keep integer conversion and read-result ownership unchanged; only input copies are removed.
- Add a Python source contract for the borrowed helper and all three call sites.

## Deterministic Performance Evidence

The standalone optimized Rust model runs 16,384 resolve-like calls per sample across 31 samples,
using the same type-path and member-name lengths as the production boundary. Both runs produced
checksum `31`.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls per 16,384 calls | 32,768 | 0 | 100.000% |
| Requested allocation bytes | 720,896 | 0 | 100.000% |
| Run 1 call P50 | 2.1752 ms | 0.0104 ms | 99.522% |
| Run 1 call P95 | 7.0305 ms | 0.0114 ms | 99.838% |
| Run 2 call P50 | 2.6002 ms | 0.0105 ms | 99.596% |
| Run 2 call P95 | 8.6112 ms | 0.0120 ms | 99.861% |

Evidence marker: `RUNTIME07_BORROWED_REFLECTION_HOST_STRINGS_MODEL_V1`.

## Validation

- The pre-change Python contract failed its three new implementation checks; the post-change run
  passed all 4 tests.
- `python -m py_compile` passed for the source contract.
- `rustfmt --edition 2021 --check` passed for the production source and model after formatting.
- `git diff --check` passed for the scoped production and contract paths.
- The standalone model retained equivalent input-length checksums across two runs, with positive
  P50/P95 reductions in both runs.
- Managed ZrVM real-backend compilation and tests are pending in the next asynchronous Runtime07
  batch; this slice is not a commit or WeCom milestone until that ticket completes successfully.

## File Fingerprints

- `zircon_plugins/zr_vm_language/runtime/src/real_backend/reflection_host.rs`
  SHA-256 `E3F7D940E2905B1ECEFA139C12685FF2C72E215C5E2E68BE5A647B796938E71C`
- `tools/tests/test_runtime07_borrowed_reflection_host_strings_performance_contract.py`
  SHA-256 `E4447B40593C03400031944BCCCF1582AA6C7445E54E2F2ACAF841DD0CFF0533`
- `.codex/state/session-coordinator/runtime07-borrowed-reflection-host-strings-model.rs`
  SHA-256 `515934A2C55B290BEF2380A544C46DA77FCDAA7824BE94B6B8C0AAA1C8770B38`

## Remaining Parent-plan Work

Runtime07 still owns deterministic resolver and catalog generations, package trust constraints,
transactional lifecycle, isolation, execution budgets, and product-scale editor/app/export/cook
acceptance in the canonical review.
