# Reporting Rules

- State whether you ran workspace-wide validation or a narrowed package scope first.
- State the exact Cargo commands you ran and whether `--locked` was included.
- If a failure is platform-specific, state whether it reproduced against the CI command shape from `.github/workflows/ci.yml`.
- If the change touches shared APIs or workspace wiring, state whether you expanded back to workspace-level validation after any crate-local loop.
- State whether you removed obsolete or compatibility paths from the touched area, or state explicitly if none existed.
- Do not claim acceptance while touched code still contains compatibility-only branches for old behavior.
