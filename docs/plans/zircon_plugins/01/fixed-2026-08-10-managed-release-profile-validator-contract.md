---
handoff_kind: fixed
status: fixed
created_at: 2026-08-10
resolved_at: 2026-08-10
summary_slug: managed-release-profile-validator-contract
origin_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
fixing_plan: docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
origin_child_dir: docs/plans/zircon_plugins/01
fixing_child_dir: docs/plans/zircon_runtime/frameworks/06
plan_link_mode: child_record_only
related_code:
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1
  - .codex/skills/zircon-dev/scripts/validate-matrix.Tests.ps1
  - zircon_plugins/Cargo.toml
tests:
  - default and development profile cargo-argument contract
  - release profile compatibility-key and cargo-argument contract
  - profiling profile compatibility-key, cargo-argument, and workspace-manifest contract
  - invalid profile rejection before cargo acquisition
  - release artifact publication path contract
---


# Frameworks06: managed release profile validator contract is absent

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 来源执行切片：Plugins01 structural performance research M0 measurement prerequisite
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md`
- 交接原因：the managed validator owns Cargo command construction, coordinator compatibility identity, and build artifact publication for every plan. Plugins01 must not add a local release-mode bypass before a source-bound performance baseline.

## 失败现象与复现证据

Current validator source exposes no allow-listed `CargoProfile` parameter. Its compatibility document records feature/export modes and environment profile debug settings but not the selected Cargo profile. `Get-CargoArgs` builds Cargo commands without `--release`, and `Publish-BuildArtifacts` unconditionally reads `<target-dir>/debug`.

The current Pester seams cover `New-CargoCompatibilityJson`, `Get-CargoArgs`, and artifact publication, but contain no profile-selection contract. Consequently a release-equivalent benchmark cannot prove that its command, coordinator lane, and artifact directory used one profile.

## 最低共享层根因

`validate-matrix` treats the Cargo profile as an implicit development default instead of an explicit managed validation input. This leaves the profile absent from the coordinator reuse key and separates compilation from artifact publication semantics.

## 架构修复验收

- Add one allow-listed profile contract with development as the existing default plus release and the workspace-defined symbolized profiling profile.
- Include the selected profile in `New-CargoCompatibilityJson`, so development, release, and profiling never share a coordinator compatibility lane.
- Every Cargo command builder that compiles code adds exactly one `--release` for release or exactly one `--profile profiling` for profiling; cleanup remains profile-independent.
- `Publish-BuildArtifacts` resolves the selected profile directory as `debug`, `release`, or `profiling` from the same selection.
- Keep the root and plugin workspace manifests on the same profiling definition: inherit release, retain debug symbols, and disable stripping.
- Add RED-then-GREEN Pester contracts for default/development behavior, release/profiling build and test arguments, invalid-profile rejection before Cargo acquisition, compatibility-key separation, workspace profile parity, and artifact publication.
- Run the focused validator contract suite and return this handoff before Plugins01 obtains a release baseline reservation.

## 禁止临时方案

- Do not add a Plugins01-only `--release` flag, manually choose a `target/release` artifact, or create a second unmanaged benchmark runner.
- Do not infer release mode from environment variables, target directories, or artifact names.
- Do not omit the profile from coordinator compatibility identity or weaken profile contract tests to preserve lane reuse.

## 修复结果与回传

- 根因：Managed validator treated Cargo profile as an implicit development default and duplicated an explicit subworkspace manifest after changing Cargo working directory.
- 架构修复：Thread one allow-listed development/release/profiling selection through coordinator identity, all compiling command builders, artifact publication, root/plugin profiling manifests, and split manifest identity from invocation path.
- 验证：Profile contracts 19/19; fresh compatibility/workspace/publication/CLI Pester acceptance 17/17; parsing 2/2; diff-check clean; independent review C0/I0/M0.
- 回传：Plugins01 M1 source-bound release/profiling benchmark baseline may start; no performance or ETW result is claimed.
