---
handoff_kind: fixed
status: fixed
created_at: 2026-09-07
summary_slug: validation-ticket-dedupe-closeout-runtime-identity
origin_plan: docs/plans/optimize/zircon_tooling/13-repository-codex-skill-hook-structural-audit-governance-security-currentness-review.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/optimize/zircon_tooling/13
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
failure_scope: cross_plan
related_code:
tests:
  - python -B -m unittest tools.session_coordinator.tests.test_validation_ticket_identity tools.session_coordinator.tests.test_failure_closeout tools.session_coordinator.tests.test_validation_tickets tools.session_coordinator.tests.test_validation_admission_policy -v
resolved_at: 2026-09-12
---

# Coordinator01: Validation ticket runtime identity cannot bind to closeout

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_tooling/13-repository-codex-skill-hook-structural-audit-governance-security-currentness-review.md`
- 来源执行切片：`handoff-letter-suffixed-plan-identity`, returned with request `dadfede91138438d8053e8af5db0d648`.
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：A production-issued green non-Cargo ticket is rejected before closeout can bind evidence.

## 失败现象与复现证据

Tooling13 managed ticket/run `619cb043ed934cecb2a0a292e7e9c94e`, copy job
`0acece549be744fe9bf9939ba81010e6`, passed 59 tests in 36.152 seconds with exit 0.
Preparing closeout with exact returned snapshot 2961 rejected the ticket with
`failure_closeout_validation_contract_invalid`: "Managed validation-copy immutable
identities are malformed or inconsistent". No closeout was created.

The ticket source manifest hash is
`126c28355df51791d727105f5d0ab100da30cdb5d082b020ee6c42ecb3c45e1c`.
The original fixed artifact, return receipt, ticket, and worker links remain evidence;
none may be rewritten to manufacture a different ticket identity.

## 最低共享层根因

Submission hashes nine inputs including validator version, raw runtime identity, and
the inherited environment digest. Closeout and its manually constructed fixture still
hash the previous six inputs. Submission does not persist the environment digest, and
its runtime hash cannot reconstruct the old raw runtime input. This is a new regression
after the completed 2026-09-01 baseline-identity lifecycle, not a reopening of that item.

## 架构修复验收

- One versioned identity constructor, canonical representation, and hash implementation
  serve submission and closeout. Identity and ticket persist in one transaction.
- Persist only allowed identity metadata and digests, never raw inherited environment.
- Closeout uses submission-time identity. Owner, plan, worker links, command, source,
  input manifest, source currentness, and terminal-result checks remain enforced.
- Real submission API tickets pass closeout; identity mutations, unknown or missing
  versions, incomplete legacy tickets, and existing evidence negative cases fail.
- Changed environment, runtime, or validator identity produces a distinct ticket key.
- Run managed regression and independent Critical/Important/Moderate-zero review,
  then rerun Tooling13 with a new complete-contract ticket and resume its closeout.

## 禁止临时方案

- Do not skip fingerprint comparison or infer missing historical identity inputs.
- Do not patch existing ledger evidence or use current environment to rebuild history.
- Do not remove producer inputs, weaken tests, or claim source inspection as acceptance.

## 修复结果与回传

- 根因：Closeout revalidation reconstructed a legacy ticket identity instead of consuming the immutable submission identity, so a real green validation-copy ticket was rejected before evidence binding.
- 架构修复：Submission and closeout now share one versioned canonical identity constructor and persist the immutable identity metadata; closeout revalidates owner, plan, worker links, command, source manifest, runtime and validator identity without retaining raw inherited environment data.
- 验证：Managed validation-copy ticket f2d31738af4f40ff9e7afbb2cc903e21 passed 41/41 tests with exit code 0 in job 4381a791c62547acb3c2514072f540da and run f2d31738af4f40ff9e7afbb2cc903e21; immutable source manifest hash 76bf16557327a4d43c0b1b8752df31291f4389f71e64719913a153cfc82dcc5f.
- 回传：Coordinator01 runtime identity is fixed and ready for exact closeout validation and independent review; the original failure artifact is now represented by the generated fixed record.

## Managed Identity And Diagnostics Attempt

The diagnostics author completed the existing handback before submission.
Snapshot `2993` owns `validation_copy_diagnostics.py`, its six-test module, and
only the three schema-70 changes in `migrations.py`. Its current hashes match
the author snapshot and attribution. The handback preserves separate ownership
of the surrounding ticket/copy producers and provides no independent C0/I0/M0
or managed full-closure acceptance. The `2973` identity snapshot was unchanged.

Request `failure-roll-20260908-coordinator01-identity-2973-diagnostics-2993-r1`
created ticket `58b591f228b74a4aa42115f5d224763d`, exact source manifest
`8804f4b277c9fd4cf9c216f091b893532d5bcdfa036d61d801fa29c1e5e38e00`.
It requested the original four modules plus metadata diagnostics, expecting
117 tests, with dependency root `tools/session_coordinator`. The nine declared
source paths and this document remained unchanged until the ticket terminated.

Copy job `4c10c49709db49d9b3f62d0757535643` failed at module import, exit 1:
five unittest loader errors in 0.001 seconds, no acceptance cases executed.
The immutable copy lacked `validation_ticket_inputs.py` and
`tests/test_validation_admission_policy.py`; the first missing import prevented
the remaining ticket/closeout/diagnostics modules from loading. The worker
record classifies this as a coordinator failure and excludes it from failure
cache reuse. Preparation took 639.753 seconds; run/cleanup took 1.345/0.293
seconds. The original ticket, command and diagnostic remain unchanged.

Dependency roots are extracted from the Session's pinned base commit
`585b031793088c28feef357488211f290927ec50`, not from arbitrary current worktree
contents. Thus the foundation commit `8243c817d4a2a4a84bda206fcfd0a27320209d5d`
and uncommitted policy-test dependencies were absent unless explicitly sealed.
A subsequent attempt must first establish the complete current import/test
closure and its exact allowed overlay, including separately owned producers;
adding a dependency-root label alone does not seal those files. Do not loosen
the immutable base or source-identity checks to make this ticket pass.

Tooling13's existing replacement ticket `5ad0394a2f2b4552870264ef90e35cb3`
remains queued on this open lifecycle with `validation_dependency_failed`.
It has not been resubmitted. Independent source review, complete managed
acceptance and integration of the mixed producer scope remain prerequisites.

## Current Import Closure Ownership Audit

The five requested test modules statically import 141 repository Python files.
The read-only AST audit at `.codex/tmp/coordinator01-import-closure-20260908.json`
records each current SHA-256, pinned-base/HEAD comparison and coordinator
attribution. Seventy-six differ from the pinned base. Two are the unchanged
foundation files already integrated in `8243c817`; the remaining differences
include current mixed ticket, copy, CLI and service producers. This static
closure does not certify dynamic fixture/subprocess input completeness.

The policy test still has a stale author attribution at current hash
`b08f9f5c957cf67e385ef77295cca42976c66368e02961fa7bcf97ecd84ecf1f`.
Other required files remain owned by the designated review task, App08, or
older Sessions; several current contents have no matching attribution.
The production submission API requires attribution for every explicit overlay.
No ownership was taken from a live owner, no dependency bytes were altered,
and no replacement ticket was submitted with this unresolved closure.

This failure is suspended for the specific dependency/ownership handback.
Tooling13 keeps its existing replacement ticket and owner. Independent
Frameworks01, Render07 and Render11 source repairs continue without waiting
for coordinator state changes or creating a monitoring task.
