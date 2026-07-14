# Frameworks05 M1 dependency matrix and contract signatures closeout

Plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
Milestone: M1
Status: completed
Files: ["docs/plans/zircon_runtime/frameworks/05/2026-07-10-subsystem-decoupling-contracts-output-records.md", "docs/plans/zircon_runtime/frameworks/05/2026-07-14-m1-dependency-matrix-contract-signatures-closeout.md", "docs/plans/zircon_runtime/frameworks/05/baselines/2026-07-10-contract-signatures.md", "docs/plans/zircon_runtime/frameworks/05/baselines/2026-07-10-runtime-domain-dependencies.json", "docs/zircon_runtime/structure/runtime-domain-dependency-audit.md", "tools/runtime_domain_dependency_audit.py", "tools/tests/test_runtime_domain_dependency_audit.py"]

## Scope Delivered

- `tools/runtime_domain_dependency_audit.py` provides the production-owner cross-domain scanner and emits a deterministic domain matrix plus `{source_domain,target_domain,path,line,source}` evidence rows. It scans the lexical Rust code view, accepts direct and bare root imports, and parses only the outer entries of grouped `use crate::{...}` trees.
- `05/baselines/2026-07-10-runtime-domain-dependencies.json` records the corrected M1 matrix baseline anchored to the exact tree of its first commit, `f7a320904d681fb30dede6d5b222fc943cdeb3a7`: 2,001 evidence rows and 86 domain edges. The former 2,399/80 and 2,401/79 totals are explicitly invalidated because they omitted root imports and counted lexical noise.
- `05/baselines/2026-07-10-contract-signatures.md` locks the S1 text shaping, S2 importer registry, S3 render-scene extract, and S4 generation-bearing manager handle signatures. It explicitly assigns later cutover work to M2-M4 and does not claim those migrations as M1 output.
- The original implementation and baseline artifacts were committed in `f7a320904d681fb30dede6d5b222fc943cdeb3a7` and `facb719f4da98953ec83f682175389916da51b6b`; this closeout repairs their evidence completeness rather than preserving the flawed scanner for historical compatibility.

## Fresh Testing Evidence

- The focused TDD probes for bare/grouped root imports and lexical masking first failed as 3 != 8 and 5 != 1, then passed after the scanner repair.
- A second TDD probe for `type Foreign<'a> = crate::ui::UiTree<'a>;` first failed as 0 != 1, then passed after character-literal masking was restricted to valid Rust scalar/escape forms and lifetime/label tokens were preserved.
- Windows PowerShell, non-Cargo M1 testing stage: `python -m unittest tools.tests.test_runtime_domain_dependency_audit -v` passed 8/8 on 2026-07-14.
- Baseline integrity check passed: schema 1, 2,001 references, 86 matrix rows, 2,001 detailed evidence rows, every evidence row contains the required five fields, and the eight reviewer-identified bare imports are present.
- Signature integrity check passed for `TextLayoutService`, `AssetImporterRegistry`, `RenderSceneExtractSource`, and `ManagerServiceHandle<T: ?Sized>`.
- A fresh current-worktree audit completed with schema 1 and complete row cardinality (`refs=2320`, `edges=76`, `rows=76`, `evidence=2320`); key edges are asset→ui=0, graphics→ui=0, ui→graphics=29, and graphics→scene=1. These counts are diagnostic because other Sessions are actively changing runtime sources; M1 acceptance is bound to the corrected `f7a3209` tree baseline and signature list required by the plan.
- Replaying the final scanner against the exact `f7a3209` archive produced a byte-identical JSON report; both files have SHA-256 `A92305C336F08F29F3B391D4615A0FFE329C68A927E118E5EA4CB3E60E8AED1B`.
- `git diff --check` passed for the M1 scanner, tests, and baseline artifacts before this record was written. The M1 plan explicitly declares no compile gate.
- The global plan-output audit still reports six foreign-owner violations (five Editor records and one Plugins05 record); no finding targets Frameworks05 or this child directory, and this Session did not rewrite those plans.

## Review

- First independent review reported 0 Critical / 1 Important: incomplete root-import coverage and lexical false positives. The scanner, tests, baseline, signature notes, and historical archive correction now address that finding without compatibility logic.
- Second independent review confirmed the first finding closed and reported 0 Critical / 1 Important for lifetime tokens being mistaken for character literals. The valid-character lexer and lifetime regression now close that finding without changing the frozen baseline hash.
- Final independent review is submitted by a different coordinator Session after this repaired manifest fingerprint is frozen. Acceptance requires 0 Critical and 0 Important findings.

## Status And Completed Items

| Milestone | Item | Status | Evidence |
|---|---|---|---|
| M1 | Production dependency audit behavior | completed | Root imports, lexical masking, and lifetime preservation are covered; complete scanner suite passed 8/8. |
| M1 | Historical matrix baseline integrity | completed | Corrected `f7a3209` tree: 2,001 references / 86 edges / 2,001 detailed rows. |
| M1 | S1-S4 contract signature inventory | completed | Four required contract families are present and assigned to M2-M4 cutovers. |
| M1 | M1-T testing stage | completed | Plan-declared non-compile gate passed on Windows on 2026-07-14. |
