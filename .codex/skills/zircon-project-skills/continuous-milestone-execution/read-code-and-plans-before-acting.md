# Read Code And Plans Before Acting

- Before proposing or generating code, read the existing implementation that the milestone touches.
- Read the related tests, specs, pending plans, TODO notes, and skill guidance that define the current intended direction.
- For `zirconEngine`, this commonly includes the touched `zircon_*` crates, `.github/workflows/ci.yml`, `.codex/plans/`, and relevant files under `docs/` when they exist.
- Reconstruct the current state before deciding the next action: what is already implemented, what is missing, what the plan says should happen next, and what constraints the existing design already imposes.
- Do not invent a new implementation direction while ignoring an existing project plan unless the evidence shows the existing plan is invalid; if so, explain the conflict and re-plan explicitly instead of drifting silently.
- Treat partial reading as insufficient context when the task spans parser, instruction generation, runtime, CLI, or shared tests.
