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

`tools/runtime_domain_dependency_audit.py` owns the machine-readable inventory of direct cross-domain Rust references inside `zircon_runtime` production sources. It emits a domain-edge matrix and exact path/line/source evidence so Frameworks 05 seam work can be accepted by decreasing concrete counts. The independent runtime `text` owner is a first-class domain, so `text -> ui` and `text -> graphics` dependencies cannot disappear from the report after the M3 directory hard cut.

The scanner excludes crate-root facades, inline test-only `cfg` items, support files reachable only from test module mounts, comments, and Rust string/character literals. Test-only detection parses balanced `all`/`any`/`not` expressions and asks whether the predicate can be true with `test=false`; it therefore masks nested `all(test, any(...))` without incorrectly masking `any(test, feature = "...")`. The audit counts explicit `crate::<foreign-domain>::...` paths, bare roots such as `use crate::ui;`, and only the outer domain entries of single-line or multiline `use crate::{ ... }` trees. Nested names in `core::{asset::AssetId, ...}` stay owned by the outer `core` entry. A reusable recursive use-tree lexer canonicalizes raw identifiers and exposes leaf paths plus aliases for hard-cut guards. The Frameworks05 text guard adds lexical-scope environments, fixed-point aliases, and a shared `lib.rs` namespace so block shadowing and cross-file `crate::alias` paths are distinguished. Identical path/line/source evidence is de-duplicated.

The scanner does not label every edge invalid: references into `core` are expected in the current monolith, while Frameworks 05 output records identify which edges must converge to framework DTOs, registries, or versioned handles.

Run:

```text
python tools/runtime_domain_dependency_audit.py --pretty
```

The canonical M1 baseline lives at `docs/plans/zircon_runtime/frameworks/05/baselines/2026-07-10-runtime-domain-dependencies.json`. It is anchored to the exact `f7a320904d681fb30dede6d5b222fc943cdeb3a7` source tree and contains 2,001 production references / 86 domain edges after the 2026-07-14 lexical and bare-root repair. The former 2,399/80 and 2,401/79 totals are invalid: the old scanner both omitted root imports and counted comments/string literals. Eleven focused tests cover explicit references, grouped and bare roots, recursive use-tree aliases, raw identifiers, the runtime text domain, lexical masking, lifetime preservation, nested and order-independent test-only `cfg` items, test owners, and test-only module support.

Do not suppress a reference from the scanner merely to make a count fall. All Frameworks05 baseline totals generated before the lexical/root-import repair must be regenerated at their owning milestone before they are used for acceptance. A fresh 2026-07-14 shared-worktree diagnostic produced 2,320 references / 76 edges, but that count is not frozen because multiple Sessions are actively editing runtime sources. Acceptance is based on exact source-tree identity and dependency direction, not artificially minimizing legitimate upper-to-lower references.
