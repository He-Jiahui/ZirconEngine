# Placement Ladder

Use this ladder before promoting a literal out of its current module.

## 1. Shared Contract

Promote a value only when multiple crates or runtime layers truly share the same contract.

Examples:
- protocol versions shared between runtime and host-facing crates
- common sentinel handles used by multiple subsystems
- shared file extensions or manifest keys

## 2. Crate Policy

Keep a value inside the owning crate when it expresses policy that is reused only inside that crate.

Examples:
- renderer batch sizes
- editor UI limits
- asset-cache capacities
- serialization retry caps

## 3. Local Helper Control

Keep a value private to one module when it only controls local flow or local implementation detail.

Examples:
- fallback capacities
- local compare tri-state values
- temporary parser depth guards
- local invalid-index sentinels

## 4. Explicit Exemption

Leave the literal in place when it is definition-bound rather than policy-bound.

Examples:
- matrix dimensions
- enum ordinals
- UTF-8 masks
- descriptor field positions
- layout-bound sentinel slots

## Decision Rule

If the scope is ambiguous, keep the value local first and promote it later only after cross-module reuse is proven.
