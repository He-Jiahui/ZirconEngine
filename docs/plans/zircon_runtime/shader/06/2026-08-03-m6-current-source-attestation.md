# Shader06 M6 Current-Source Attestation

Plan: docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
Milestone: M6
Status: in_progress
Files: ["docs/plans/zircon_runtime/shader/06/2026-08-03-m6-current-source-attestation.md"]

## Scope Delivered

- Rebind M6's managed validation to the current source baseline without rewriting the historical M6 implementation manifest in `2026-08-01-m6-environment-only-pbr-forward-closeout.md`.
- Retain the environment-only Forward source reduction, asynchronous Base-pipeline admission, zero-direct-light startup allocation reduction, and the current PBR/IBL correctness contract as the M6 implementation under test.
- Do not include subsequent M7 source specialization work or claim that this one-file manifest represents a new production implementation slice.

## Fresh Testing Evidence

- Pending coordinator-managed M6 Windows validation. This attestation deliberately records no passed Cargo, DX12 viewer, screenshot, quantitative-image, or RenderDoc result.
- The current `Cargo.lock` contains the declared `meshopt 0.6.2` package and the `zircon_runtime` dependency edge that previously caused the M6 `--locked` materialization attempt to terminate before any Shader06 test ran.
- Historical PNG/RDC files and static source checks remain non-acceptance baselines until the matching managed current-source product gates reach a terminal result.

## Review

- The existing M6 source review remains `Critical 0 / Important 0 / Minor 0` for the recorded implementation scope. This attestation adds no acceptance claim and still requires the coordinator-recorded validation and independent milestone review before M6 can be committed.
