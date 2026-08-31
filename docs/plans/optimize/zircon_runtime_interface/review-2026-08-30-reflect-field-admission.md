# Reflect Field Registration Admission Review

Date: 2026-08-30
Parent: `02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md`, P1-40
State: source implemented / managed validation pending

## Current-source findings

- `TypeRegistry::validate_registration` rejects blank and duplicate field names, but native
  registrations do not parse every `value_type_path`. Only VM admission does so before publication.
  A malformed native declaration without a default value can therefore enter the registry.
- `DeclaredValueType` recursively parses `List<...>` and `Map<String, ...>` without total-byte or
  nesting-depth budgets. Its depth counter uses `saturating_add`, so adversarial wire can consume stack
  before admission fails.
- Three production `MeshRenderer` fields intentionally declare dynamic `List` values containing
  scalars or heterogeneous maps. VM tests intentionally reject bare `List`. General native reflection
  and VM ABI therefore need two policies over one parser owner, not two independent parsers.
- Default type matching exists, but enum-option uniqueness/membership, numeric-range compatibility,
  editor-hint compatibility, field/option count budgets, and canonical enum option text are not checked.
- `serializable`, editor, remote, and script visibility have independent consumers and are not field
  value-type invariants.

## Reference decision

Unreal represents property storage through an `FProperty` subtype and keeps struct/enum identity as
additional type identity. Zircon's existing `ScriptHostTypeRef` likewise separates semantic type name
from typed value kind. The MVP closure keeps the current reflection wire but treats its string as an
admitted declaration, parsed once by the runtime owner before registry publication. It does not create
a second field grammar or treat an editor hint as VM ABI authority.

## Implementation boundary

- Extend `DeclaredValueType` with general and strict-VM entry points over one bounded parser.
- General admission accepts canonical value kinds, explicit Rust aliases, validated named paths, and
  dynamic `List`/`Map`. Strict VM admission accepts only canonical typed ABI declarations.
- Keep parser byte/depth controls private to `declared_value_type.rs`. Keep per-registration field and
  per-field enum-option budgets private to `type_registry.rs`; these are local runtime admission policy,
  not cross-crate constants.
- Validate the entire registration in one pass before publication. Failures use a structured
  `InvalidFieldRegistration { type_path, field_name, reason }` error.
- Validate declaration syntax, default representation, editor-hint compatibility, numeric metadata,
  enum option text/uniqueness, and enum default membership. Reject the whole registration on any error.
- Stable schema/field identity remains a later Tooling identity revision. Field vector position must not
  be presented as a stable ID.

## Verification

- Add focused native-registry regressions for malformed declarations, duplicate enum options,
  enum-default membership, and numeric metadata on non-numeric fields.
- Preserve the existing strict-VM malformed grammar corpus and fixed native `MeshRenderer` registration.
- Run parser-only checks and source gates while managed Cargo is unavailable. The managed runtime and
  interface gates remain required before milestone acceptance.

## Current implementation evidence

- `TypeRegistry::validate_registration` admits the complete field vector before publication. It uses
  pre-sized hash sets for expected O(n) field/option uniqueness, checked total-option accumulation, and
  owner-local limits of 4,096 fields, 4,096 options per field, and 16,384 options per type.
- Field failures carry `InvalidFieldRegistration { type_path, field_name, reason }`. Declaration syntax,
  default representation, hint compatibility, numeric metadata, option text/uniqueness, and enum default
  membership now share that registration-time error boundary; runtime value writes retain `TypeMismatch`.
- VM registration and catalog sync consume the descriptor returned by one validated TypeRegistry
  preflight. The former world-level descriptor construction plus registry-level reconstruction is gone,
  reducing release validation/parser traversal and descriptor construction from two passes to one. An
  identical VM upsert returns without advancing schema catalog generation.
- Derive inference recursively maps supported `Vec<T>` to `List<T>` and rejects unknown element types
  unless the author supplies an explicit declaration. The three heterogeneous `MeshRenderer` projections
  remain explicit dynamic `List` declarations and do not weaken strict VM admission.
- The F-drive release parser harness passes 30/30 cases. Across 21 independent process samples, each
  running 1,000,000 strict parses of `List<Map<String, List<Scalar>>>`, P50 is 482.6 ns/parse and P95
  is 516.2 ns/parse (range 435.3..628.4 ns/parse). This is load-time admission evidence, not a
  frame-loop timing or whole-engine power claim.
- Focused source gates confirm one strict parser occurrence in VM admission, no world-owned
  `TypeRegistry::vm_component_descriptor` call, no retired `"Vec" => "List"` inference, and an
  infallible descriptor-to-field projection. Targeted rustfmt parsing and scoped `git diff --check`
  pass. Managed Cargo, cross-crate behavior tests, RSS/power, and product comparison remain pending.
