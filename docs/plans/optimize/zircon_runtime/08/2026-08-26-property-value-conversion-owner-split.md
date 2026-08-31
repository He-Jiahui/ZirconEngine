# Runtime 08 property value-conversion owner split

## Scope

- Target: `zircon_runtime/src/scene/world/property_access/value_conversion.rs`.
- Baseline: clean 627-line tracked production owner before this slice.
- Priority sources: Runtime 08, Runtime 15, the engine structure convention/review findings, and the compiled property-path failure record.
- This slice changes source ownership only. It does not alter identifier normalization, numeric coercion, finite-value validation, resource parsing, enum matching, or animation-player mutation.

## Architecture review

The previous file mixed five independently changing policies: typed property error construction, canonical component/field identity, compiled-writer conversion adapters, scalar/vector/quaternion validation, and resource/animation/physics domain conversion. Every schema or error change therefore modified one shared 627-line utility owner.

The primary local Unreal reference remains `Runtime/PropertyPath/Public/PropertyPathHelpers.h`: authored path segments, resolved/cached identity, and typed get/set operations have distinct responsibilities. Bevy's `AnimatableProperty`/`AnimatedField` direction likewise keeps typed accessor identity separate from value conversion. Zircon preserves its current Rust DTO and `World` APIs while giving each policy one replaceable owner.

No hotpath optimization was attempted. The generic `World::set_property` path still normalizes identifiers as before, while stable-frame animation continues through the compiled writer. Any normalization or conversion optimization requires separate Inspector/edit and compiled-frame profiles.

## Implemented layout

| Owner | Responsibility | Current lines |
|---|---|---:|
| `value_conversion.rs` | Restricted facade and compatibility re-exports | 21 |
| `value_conversion/compiled.rs` | `World` adapters used by compiled property writers | 66 |
| `value_conversion/domain.rs` | Resource, animation, mobility, rigid-body, joint, and combine-rule conversion | 208 |
| `value_conversion/errors.rs` | Segment, missing-component, unknown-property, and type-mismatch errors | 64 |
| `value_conversion/identifiers.rs` | Canonical component identity, allocation-free matching, and axis lookup | 127 |
| `value_conversion/values.rs` | Scalar/integer/vector/quaternion extraction and finite-value validation | 200 |

Existing consumers continue importing through `property_access::value_conversion`. Free helpers retain `property_access` visibility and compiled adapters retain `scene::world` visibility.

## Preserved invariants

- Dynamic schema identifiers remain case-sensitive while established runtime components use the same canonical aliases.
- Normalized identifiers still pre-size one `String` and push ASCII alphanumeric characters directly.
- Allocation-free normalized comparisons, axis aliases, integer range checks, finite-value loops, and zero-length quaternion rejection are unchanged.
- Resource parsing and typed error sources retain their original branches and messages.
- Mobility, rigid-body, joint, and combine-rule aliases remain identical.
- Animation player-like mutation retains every no-change branch and optional weight behavior.

## Current evidence and status

- Scoped `rustfmt --edition 2021 --check` passed for six production files and three adjusted contract-test files.
- Static migration comparison retained all 43 function definitions and all 109 string literals with zero delta.
- Identifier-match calls (18), finite-array validation calls (5), type-mismatch errors (14), unsupported-value errors (4), and resource-handle construction (1) match the original owner.
- The facade/owner contract checks all five mounts, policy anchors, and a 300-line production-owner budget.
- Production files contain no new `allow`, `unwrap`, `expect`, `panic`, `todo`, or `unimplemented` escape path.
- Root size changed from 627 to 21 lines; all production child owners are at or below 208 lines.
- Managed Cargo and runtime profiling were not requested while bypassing the shared validation blocker. Status is `implemented_static_passed_managed_validation_deferred`.

No CPU, allocation, memory, latency, energy, or power improvement is claimed because the conversion algorithms and call counts were intentionally preserved.
