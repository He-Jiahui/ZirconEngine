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

The scanner excludes crate-root facades and test owners (`tests/`, `tests.rs`, `*_tests.rs`, `test_*.rs`). It counts explicit `crate::<foreign-domain>::...` references and de-duplicates identical line evidence. It does not label every edge invalid: references into `core` are expected in the current monolith, while Frameworks 05 output records identify which edges must converge to framework DTOs, registries, or versioned handles.

Run:

```text
python tools/runtime_domain_dependency_audit.py --pretty
```

The current M1 baseline lives at `docs/plans/zircon_runtime/frameworks/05/baselines/2026-07-10-runtime-domain-dependencies.json`. When a seam changes, regenerate that artifact and record the before/after edge count in the numbered Frameworks 05 output archive. Do not suppress a reference from the scanner merely to make a count fall; exclusions are limited to non-production owners and are unit-tested.
