# Runtime 10 Dynamic API filter current result

Date: 2026-07-10

Status: in progress

## Executable baseline

The available default-feature runtime test binary selected 103 `dynamic_api` tests: 85 passed, 8 failed, 10 were ignored because they require `backend-zr-vm` and its binding library, and 7335 were filtered out. The run completed in 54.25 seconds.

All 85 executed behavior tests passed. The eight failures were stale archive/current-owner guards for the event split, FFI panic boundary, headless profiles, mirror counts, shared-data split, UI single-source contract, UI v2 verdict, and Dynamic API Vampire test-owner naming.

## Current-source evidence

- standalone Runtime10 Dynamic API session guards: 11/11;
- exact Dynamic API Vampire naming guard: 1/1;
- current aggregate structure audit: Dynamic API sections carry no risk; only the separately owned Runtime01 Physics/Jolt drift remains globally visible;
- scoped rustfmt: passed.

Status anchor: `runtime_10_dynamic_api_old_binary_85_passed_8_failed_10_ignored_all_8_current_guards_passed_fresh_filter_pending`.

A newly compiled default-feature binary must rerun the complete filter before Runtime10 is promoted.
