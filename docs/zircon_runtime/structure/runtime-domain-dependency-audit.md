---
related_code:
  - tools/runtime_domain_dependency_audit.py
  - tools/tests/test_runtime_domain_dependency_audit.py
implementation_files:
  - tools/runtime_domain_dependency_audit.py
  - tools/tests/test_runtime_domain_dependency_audit.py
plan_sources:
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
tests:
  - python -m unittest tools.tests.test_runtime_domain_dependency_audit
doc_type: module-detail
---

# Runtime Domain Dependency Audit

`tools/runtime_domain_dependency_audit.py` owns the machine-readable inventory of direct cross-domain Rust references inside `zircon_runtime` production sources. It emits a domain-edge matrix and exact path/line/source evidence so Frameworks 05 seam work can be accepted by decreasing concrete counts.

The scanner excludes crate-root facades, inline `cfg(test)` items, and support files reachable only from test module mounts. It counts explicit `crate::<foreign-domain>::...` references and foreign domains nested in single-line or multiline `use crate::{ ... }` groups, then de-duplicates identical line evidence. It does not label every edge invalid: references into `core` are expected in the current monolith, while Frameworks 05 output records identify which edges must converge to framework DTOs, registries, or versioned handles.

Run:

```text
python tools/runtime_domain_dependency_audit.py --pretty
```

The immutable M1 baseline lives at `docs/plans/zircon_runtime/frameworks/05/baselines/2026-07-13-runtime-domain-dependencies-production-only.json`. Current repair slices write review reports under `.codex/tmp` and record exact before/after counts in the numbered Frameworks 05 output archive; the historical baseline is not overwritten. Do not suppress a reference from the scanner merely to make a count fall. The 2026-07-13 grouped-use precision regression demonstrated why: `use crate::{plugin::ExportProfile, ...}` had been omitted, and the corrected current report increased from 2,142 / 70 to 2,213 / 75 before any further architecture claim was made.

The completed current-source repair report is `.codex/tmp/frameworks05-neutral-project-schema.json`: 2,282 production references / 72 domain edges. All Frameworks05 tracked forbidden pairs are zero (`core→{asset,graphics,scene,plugin}`, `{animation,asset,scene,script}→plugin`, and `platform→builtin`). The larger reference count is expected because canonical upper domains now import neutral `core/framework` owners directly; acceptance is based on direction, not artificially minimizing legitimate upper→lower references.
