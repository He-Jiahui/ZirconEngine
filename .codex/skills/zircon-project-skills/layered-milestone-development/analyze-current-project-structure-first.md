# Analyze Current Project Structure First

- Start every plan by surveying the current repository shape and the concrete goal of the task.
- At minimum account for `Cargo.toml`, the converged root packages (`zircon_app`, `zircon_runtime`, `zircon_editor`), the runtime-internal spine under `zircon_runtime/src/core/`, any affected feature modules under `zircon_runtime/src/`, `.github/workflows/ci.yml`, `.codex/plans/`, and `docs/` when present.
- Read the relevant specification, plan, and test files before deciding milestone boundaries. For scripting/runtime behavior, treat the current crate APIs, inline tests, and CI workflow as completeness inputs rather than decoration.
- Record the current baseline: what already works, what is known broken, what is unimplemented, and which layers depend on each lower layer.
- Derive milestone order from shared dependencies. Lower shared behavior must stabilize before higher orchestration, integration, or CLI-level work.
- If the task only touches one slice of the project, still state which lower layers can invalidate that slice.
