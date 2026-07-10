# Feature Evidence And Test Checklist

Use this checklist before presenting a `zirconEngine` feature design or calling an implementation complete.

## 1. Evidence Capture

Fill in or state each item explicitly:

- Feature name
- Current milestone from the plan under `.codex/plans/`
- Affected `zirconEngine` crates or layers
- Foundational capability that should carry the feature
- Foundational layer that looks insufficient today, if any
- Reference languages consulted
- Exact upstream files consulted
- Shared semantic core across the references
- Deliberate `zirconEngine` divergence and reason
- Performance, memory, and ownership implications
- Upstream tests or regression files consulted

Minimum bar:

- At least 2 reference languages
- At least 1 implementation file
- At least 1 test source

Raise the bar to 3 or more reference languages when the feature affects GC, native interop, module loading, exceptions, types, or code generation.

## 2. Foundation Adequacy Review

Answer these before accepting any implementation strategy:

- Can the feature be expressed through an existing general protocol or abstraction?
- If not, which foundational abstraction must be generalized?
- Would the proposed change introduce checks on concrete type names, object names, syntax spellings, or single feature flags inside shared runtime or compiler paths?
- If yes, stop and redesign the lower layer first.
- What is the reusable contract after the redesign?
- Which future features should become easier once this foundation exists?

Reject designs that mainly rely on:

- string comparisons on type names in shared paths
- object-kind special cases added only for one feature
- duplicated runtime branches for one syntax form
- parser-only sugar that the lower layers cannot model coherently
- temporary compatibility branches with no removal plan

## 3. Test Translation Matrix

Translate upstream coverage into repository layers instead of stopping at one end-to-end example.

- Parser and diagnostics:
  - valid syntax
  - invalid syntax
  - precise diagnostic location and message shape
- Compiler, IR, or module layer:
  - symbol resolution
  - type or shape propagation
  - emitted artifact expectations
  - cross-file behavior
- Runtime layer:
  - correct result
  - structured error behavior
  - cleanup after failure
  - cache or state reuse
- Project and integration layer:
  - full crate or subsystem fixture
  - module/plugin load or entry-point behavior
  - observable host, editor, runtime, or serialization result
- Artifact layer when relevant:
  - serialized assets
  - manifests or descriptors
  - persisted state blobs
- Regression and stress layer:
  - upstream bug translation
  - large input or long-running case
  - repeated execution or repeated import

## 4. Boundary Catalog

Pick all boundary classes that apply. Omitted boundary classes must be explained.

- empty input
- null or missing values
- minimum and maximum arity
- single item vs many items
- nested or chained use
- repeated definition or shadowing
- duplicate import or cyclic import
- invalid type or invalid shape
- deep recursion or deep nesting
- overflow, truncation, or precision loss
- cache invalidation or repeated load
- cleanup after throw or partial construction
- line, column, and token accuracy for diagnostics

Add one more boundary question for this repository:

- does the feature still behave correctly without any hard-coded knowledge of a concrete type name or concrete syntax case inside shared foundations?

## 5. Extreme Or Stress Expectations

Stress coverage is mandatory when the feature can fail under load, depth, repeated transitions, or memory pressure.

- long-running loops
- repeated module import or unload
- repeated exception throw and catch
- deep call stacks
- GC during active objects or native handles
- large collections, large strings, or large source files
- re-entrant native callbacks
- hot paths that may need specialized opcodes later

If true stress infrastructure does not exist yet, add the strongest available approximation and record the gap instead of ignoring it.

## 6. Reporting Format

When reporting design or implementation results, include:

- milestone placement
- upstream evidence summary
- foundation adequacy decision
- general abstraction added or reused
- chosen `zirconEngine` behavior
- explicit divergence
- parser coverage
- compiler or artifact coverage
- runtime coverage
- project or CLI coverage
- boundary coverage
- stress coverage
- remaining gaps

If any layer is still missing, report the work as partial.
