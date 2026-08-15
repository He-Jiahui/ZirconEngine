---
related_code:
  - deny.toml
  - .github/workflows/ci.yml
  - tools/tests/test_frameworks_06_dependency_governance_contract.py
  - tools/check_conventions.py
  - tools/convention_exemptions.py
  - tools/check-conventions.ps1
  - tools/tests/test_check_conventions.py
  - docs/cli-and-tooling/check-conventions.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/zircon_runtime/structure/module-convention.md
  - docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/zircon_runtime/frameworks/development-conventions.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - python -B -m unittest tools.tests.test_frameworks_06_dependency_governance_contract -v
  - python -B -m unittest tools.tests.test_frameworks_06_ci_toolchain_contract tools.tests.test_check_conventions -v
  - python -B tools/check_conventions.py --only docs --only guards --only exemptions --json
  - cargo deny --manifest-path Cargo.toml --all-features check advisories bans licenses sources
  - cargo deny --manifest-path zircon_plugins/Cargo.toml --all-features check advisories bans licenses sources
---

# Frameworks06 M3 convention and dependency governance

Plan: docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md

Milestone: M3

Status: validation_pending

Files: [".github/workflows/ci.yml","deny.toml","docs/cli-and-tooling/check-conventions.md","docs/plans/engine-code-review-findings-2026-06.md","docs/plans/engine-code-structure-convention.md","docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md","docs/plans/zircon_runtime/frameworks/06/2026-08-08-m3-dependency-governance.md","tools/check-conventions.ps1","tools/check_conventions.py","tools/convention_exemptions.py","tools/tests/test_check_conventions.py","tools/tests/test_frameworks_06_dependency_governance_contract.py"]

Current status: `implementation_secondary_review_green_managed_contract_queued_cargo_deny_acceptance_pending`

## Scope Delivered

- Extended the single convention runner from six to seven gates with one owned Rust exemption
  scanner. The scanner inventories tracked, dirty, and non-ignored untracked Rust sources, masks
  strings, characters, and nested comments, recognizes multiline `allow` attributes, covers
  explicit Cargo target paths, and assigns nested members to the longest resolved owner root.
- Enforced adjacent `// EXEMPT(<known MUST rule>): <reason>` markers for the first strict members
  without adding an allowlist, compatibility scanner, or legacy path. Non-strict members remain
  visible as quantified migration debt rather than being silently accepted.
- Extended the G7 frontmatter audit to concrete `tests:` file references while excluding commands,
  globs, templates, and generated `target`/`build` artifacts. Absolute paths and repository escapes
  remain hard failures.
- Removed stale owner references from the two user-priority plans, the runtime module-convention
  document, and the completed Runtime05 closeout record. The Runtime05 metadata repair does not
  reopen its accepted implementation milestone, and foreign Text04 body edits were preserved.
- Added the root `deny.toml` as the single dependency-governance policy for both Rust workspaces.
- Allowed the current permissive license families globally while limiting `MPL-2.0` to the 18
  existing crate owners that require it. No GPL/LGPL family or blanket copyleft allowance was added.
- Kept the advisory ignore list empty, denied unknown registries and Git sources, and allowed only
  the canonical crates.io index plus workspace path dependencies.
- Made duplicate versions visible as warnings during the existing convergence period and rejected
  wildcard dependency requirements. No crate, duplicate subtree, or source is skipped.
- Added a dedicated CI matrix for the root and `zircon_plugins` manifests using Rust 1.94.1 and the
  maintained cargo-deny v2 action.
- Added a standard-library-only contract that parses `deny.toml` and both lockfiles, verifies the
  CI projection, and prevents the policy from widening silently. The dependency-governance job
  executes this contract before invoking cargo-deny, so the guard cannot disappear silently.

## Fresh Testing Evidence

- Current combined convention contracts are GREEN at 27/27; Python AST/compile checks are GREEN
  for the runner, scanner, and contract tests; exact-scope `git diff --check` is GREEN apart from
  existing Windows line-ending notices.
- Review-driven TDD reproduced both G7 path-classification defects before the repair: nested
  crate `target`/`build` artifacts expanded the checked set from 5 to 7, while two unsafe paths
  containing spaces disappeared from the expected six-path safety audit. The same two focused
  contracts are GREEN after safety classification was moved before command exclusion and generated
  directory filtering was expanded to path components.
- The current exemption inventory uses the real Git source path and reports 426 candidate files,
  148 actual attributes, 0 strict-member attributes, and 0 violations. These inventory counts are
  a current shared-tree observation, not a frozen global acceptance threshold.
- The six exact-scope convention/plan documents report 0 owned G7 violations. The four cleanup
  owners changed from 186, 185, 103, and 35 stale references respectively to 0. The shared docs
  tree remains RED and must be repaired by its actual owners; this record does not convert that
  global debt into success.
- TDD RED was observed before implementation: all four focused contracts failed because the policy
  and CI job did not exist.
- Focused static verification is GREEN: 4 tests passed, Python compile and exact-scope
  `git diff --check` passed, and a structured scan found zero wildcard requirements across 402
  Cargo manifests.
- Exact-scope independent review is GREEN at Critical 0 / Important 0 / Minor 0.
- The first managed exact5 copy terminated with 3/4 passing because its immutable input manifest
  omitted both lockfiles; this was a validation-manifest defect, not a policy source failure, and
  that job is not reused.
- The corrected exact7 managed copy `b302f30719fc4001a81e4e1581650ba7`, run
  `0afdc09f80954541a5cde4b3f8b14ff1`, is GREEN at 4/4 with both lockfiles frozen as read-only
  inputs.
- The post-fix exact11 convention candidate was submitted as managed ticket
  `ed638b6864f8459aa65fef2e090daa9e` with source manifest
  `1d981d38339619b6f7817b160da6641d3014d7f0bf4fd24d44c7a2301d364667`. The ticket is
  coordinator-owned and pending; this record does not poll it or treat the receipt as acceptance.
- Real cargo-deny/CI acceptance is recorded after it completes; this record does not mark
  Frameworks06 M3 or the parent plan complete in advance.

## Review

- The first G7/exemption review reported C0/I1/M2; all three coverage findings were repaired
  with clean/dirty/untracked Git inventory, template/build exclusions, fake-comment attributes,
  explicit Cargo roots, and nested-member ownership regressions.
- The first immutable exact11 review reported C0/I0/M1; its record-completeness finding is repaired
  by the scope and evidence above. A second frozen review reported C0/I1/M1 for the whitespace
  safety bypass and nested generated-directory false positives. Both findings now have focused
  RED-to-GREEN regressions and minimal parser repairs. The final post-fix review reported
  C0/I0/M0 and confirmed stable pre/post SHA-256 values for the runner, extracted scanner, and
  regression owner. A later exact12 record audit reported C0/I1/M1 because the canonical `Files:`
  list still described exact11 and the shared-tree inventory text still said 421 candidates. Both
  record findings are repaired above. The fresh immutable exact12 re-review returned C0/I0/M0 with
  stable pre/post fingerprint `c990ebf3eda504b6d3c0ae2433d9d72c5d7569a3019c91a64701e6a739beef16`.
  Managed validation and cargo-deny/CI acceptance remain pending.
