# Runtime Init-Level Services Hard Cutover Acceptance

## Scope

- Owner: `zircon_runtime::core::runtime::lifecycle`.
- Plan: Runtime 15 naming/structure priority with Runtime 01 manifest-tooling parity.
- Change: replace the ambiguous non-network `Servers` layer with `Services` without an alias or compatibility parser.

## Invariants

1. The Rust enum and its snake-case serde value are `Services` / `services`.
2. Platform, input, and asset module descriptors use the services layer.
3. Lifecycle and module-order fixtures use `ServicesModule` consistently for descriptors, dependencies, and expected order.
4. Plugin manifest validation accepts `services` and rejects retired `servers`.
5. The runtime naming audit classifies the legitimate `Editor` init level as editor-host profile vocabulary.
6. No production Rust caller, manifest validator, or lifecycle fixture preserves the retired name.

## Evidence

| Check | Result |
|---|---|
| Naming + manifest-schema Python suites | passed, 8/8 |
| Direct non-network server audit | count 0, debt 0, `classified-and-clear`, risks empty |
| Runtime naming audit | lifecycle editor rows classified; unclassified editor locations reduced 6 to 4 |
| Scoped rustfmt | passed |
| Retired symbol/value scan | no active `InitLevel::Servers`, `ServersModule`, or serialized `servers` owner |
| Runtime core-min library check | passed with existing warnings |

## Decision

The code and tooling hard cutover is accepted. Runtime 15 remains `in_progress` because active Frameworks plan mirrors still need their owner to replace the historical name and the aggregate module-convention gate retains unrelated text/editor, legacy asset/graphics, and hard-cutover debt.
