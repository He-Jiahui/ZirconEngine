---
name: zr-architecture-first-engineering
description: Design or change ZirconEngine subsystem ownership, cross module contracts, or an authorized structural migration.
---


# Architecture-First Engineering

For a new subsystem or changed cross-module contract, establish ownership, lifecycle, data flow, and the affected acceptance boundaries before implementation. Scale the design note to the change; an existing coherent abstraction needs no speculative replacement.

## Current architecture

- `zircon_app` hosts process entry, profile selection, and the main loop.
- `zircon_runtime` owns runtime-world authority and its internal `core/{runtime,framework,manager,math,resource}` spine.
- `zircon_editor` owns editor host and authoring state.
- These are primary architectural roles, not an exhaustive Cargo-member whitelist. Supporting packages in the current manifest retain their documented ownership.
- Within runtime core, `runtime` schedules lifecycle/dependencies, `manager` provides service access, `framework` defines contracts and shared DTOs, and `math/resource` provide shared foundations.
- Preserve the intended host/contract/consumer boundary. Avoid concrete cross-crate ownership that bypasses lifecycle or runtime-world authority.
- Reserve new `server` naming for actual network/service-host semantics.

Read [the baseline](references/system-architecture-baseline.md) and the relevant sections of the current convergence plans when ownership is affected. Reuse unchanged context. Reference-engine selection belongs to [reference routing](../zr-reference-engine-routing/guide.md).

## Design and implement

1. Identify the current owner and the required contract. Consider descriptors, handles, configuration, scheduling, ECS, persistence, and plugin interfaces only where the feature touches them.
2. Check lifecycle ordering, dependency direction, editor/runtime authority, failure behavior, and current callers.
3. Consult the best-fitting reference source when a design question remains. Record material divergence and its reason.
4. Implement the smallest coherent boundary and behavior that satisfies the authorized scope. Generalize only when current consumers or concrete requirements justify it.
5. For an authorized migration, update its consumers using [hard cutover](../zr-hard-cutover-migrations/guide.md). Unrelated structural debt does not automatically expand a feature task into a workspace migration.
6. Validate affected behavior and contracts under [the validation policy](../../../../docs/plans/milestone-validation-policy.md).

Do not add sibling stubs, no-op integration, speculative extension points, or a tenfold-growth architecture solely to satisfy a checklist. Do not fix a weak foundation with type-name branches or migration-only wrappers. If a required ownership change exceeds the authorized scope, explain that specific dependency and continue independent work.

A useful task note states the owner, changed contracts, lifecycle/data flow, relevant precedent, and validation scope. Create a separate design document only when a durable public or operational fact needs an owner.

## Select a specific reference

| Boundary being changed | Guide |
| --- | --- |
| Module responsibilities or a growing root file | [Module boundaries](../zr-module-boundary-discipline/guide.md) |
| Retiring a module, crate, or public path | [Hard cutover](../zr-hard-cutover-migrations/guide.md); [workspace rules](../zr-workspace-structure-hard-cutover/guide.md) for crate ownership |
| Runtime entry/module/service semantics or a convergence audit | [Runtime interfaces](../zr-runtime-interface-convergence/guide.md) and its retained audit scripts |
| Language, plugin, or runtime/editor semantics | [Feature design](../zr-language-feature-design/guide.md) |
| Choosing engine precedent | [Reference routing](../zr-reference-engine-routing/guide.md) |
| Shared constants, thresholds, or sentinels | [Constant ownership](../zr-magic-constant-convergence/guide.md) |

These are conditional references, not a required sequence. Use existing code and contracts to resolve routine choices before expanding the investigation.
