# Runtime 15 structure-convention old-binary baseline

Date: 2026-07-10

Status: `runtime_15_structure_convention_old_binary_750_passed_553_failed_historical_drift_classified_fresh_filter_pending`

The available default-feature library-test binary predates the current child-owner and status-output tree. Its `structure_convention` filter selected 1303 tests and reported 750 passed / 553 failed / 0 ignored / 6135 filtered in 158.33 seconds.

Failure-domain classification:

| Domain | Failures |
|---|---:|
| `test_file_budget` | 468 |
| `production_file_budget` | 57 |
| `module_convention_gate` | 8 |
| `runtime_dead_code` | 4 |
| `provider_boilerplate` | 2 |
| `diagnostics_surface` | 2 |
| `graphics_dead_code` | 2 |
| ten remaining single-owner guards | 10 |
| total | 553 |

The dominant failures reference the pre-split test-owner/status tree and historical file budgets. This result is therefore retained as a drift baseline, not promoted as current-source acceptance or used to reopen already verified child-owner slices. A fresh current-source default-feature aggregate remains required.

## Current target-client profile follow-up

A later current-source target-client lib-test binary selected the same 1303 `structure_convention` tests and reported 776 passed / 527 failed / 0 ignored / 6146 filtered in 116.81 seconds.

| Domain | Failures |
|---|---:|
| `test_file_budget` | 448 |
| `production_file_budget` | 54 |
| `module_convention_gate` | 8 |
| `runtime_dead_code` | 4 |
| `diagnostics_surface` | 2 |
| `graphics_dead_code` | 2 |
| nine remaining single-owner guards | 9 |
| total | 527 |

This narrows the historical failure set by 26 but remains red and belongs to a different feature profile. Status: `runtime_15_structure_convention_target_client_776_passed_527_failed_current_profile_pending`.
