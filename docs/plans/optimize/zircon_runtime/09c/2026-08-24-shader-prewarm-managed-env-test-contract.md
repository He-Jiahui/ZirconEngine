# Shader Prewarm Managed Environment Test Contract

- Date: 2026-08-24
- Plan: `docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md`
- Scope: shader prewarm Python orchestration tests only
- Status: local validation passed; coordinator acceptance pending

## Change

Four orchestration tests now mock `managed_cargo_environment`, assert the expected
`targets_root/shader_prewarm` and cache-root arguments, and verify that the returned
environment is forwarded to `subprocess.run`.

This keeps the tests aligned with the Windows managed-build-root contract without
weakening production path validation or creating Cargo output during Python tests.

## Measured Result

- Shader prewarm contract batch: `4 ERROR -> 0 ERROR`.
- Passing contract tests: `119 -> 123`; one executable-dependent acceptance test
  remains an explicit environment skip.
- Real managed Cargo environment setup in these mocked orchestration cases:
  `4 -> 0` calls (`100%` elimination of test-side build-directory setup).
- Production shader prewarm performance: unchanged; no product performance gain is
  claimed by this test-only repair.

## Validation

- `python -m unittest discover -s tools/tests -p 'test_zircon_build_shader_prewarm*.py'`
  - `Ran 124 tests in 10.064s`
  - `OK (skipped=1)`
- `python -m py_compile tools/tests/test_zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm_acceptance_handoff.py`
- `git diff --check -- tools/tests/test_zircon_build_shader_prewarm.py tools/tests/test_zircon_build_shader_prewarm_acceptance_handoff.py`

No Cargo command was started for this change.
