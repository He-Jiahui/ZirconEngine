---
handoff_kind: failure
status: open
created_at: 2026-07-30
summary_slug: woc-deterministic-scalar-math-host
origin_plan: docs/plans/woc/01-woc-zrvm-one-to-one-replication.md
fixing_plan: docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md
origin_child_dir: docs/plans/woc/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/13
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/script/vm/host/builtin_host_modules.rs
  - zircon_runtime/src/script/vm/host/host_export_registry.rs
  - zircon_runtime/src/script/vm/tests/module_surface.rs
  - examples/woc/scripts/woc_game/src/combat/spell_scaling.zr
tests:
  - cargo test -p zircon_runtime --lib script --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib script::vm::tests::module_surface --locked --jobs 1 -- --nocapture --test-threads=1
  - canonical WOC zr_vm:project scalar-math vectors once Plugins08 supplies the transactional project backend
---

# Runtime13: WOC deterministic scalar math host contract

## 来源执行者

- 来源计划：`docs/plans/woc/01-woc-zrvm-one-to-one-replication.md`
- 来源执行切片：M4 current-head source rebasing and the later Eastbrook combat/MVP closure.
- 修复责任计划：`docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md`
- 交接原因：public ZrVM host-module names, scalar call semantics and capability registration are owned by Runtime13. WOC cannot provide a project-local math module without creating a second runtime contract.

## 失败现象与复现证据

`zircon_runtime/src/script/vm/host/builtin_host_modules.rs` registers
`zr.zircon.math`, but its executable surface currently contains only
`vec3_length(x, y, z)` and `vec3_dot(ax, ay, az, bx, by, bz)`. The requested
current-head WOC simulation imports the registered module identity at 36 source
sites and requires scalar `abs`, `atan2`, `ceil`, `cos`, `exp`, `floor`, `sin`,
`sqrt`, and target-compatible exponentiation. No registered host export or
published scalar precision/exception contract provides those calls.

The WOC project therefore cannot execute its source-authored scalar combat,
movement, geometry and Chronomancy paths through the required `zr_vm:project`
backend. A source-local approximation, different import module, native gameplay
calculation, or precomputed substitute would invalidate the one-authority
determinism contract.

## 最低共享层根因

Runtime13's host-function ledger and reflection system define the public
`zr.zircon.math` surface, but the module has not been extended from vector
descriptors/helpers into a versioned deterministic scalar ABI. The missing
contract includes callable exports, argument/result finite-value policy,
cross-platform precision/rounding semantics, capability declaration and
reflection documentation. It is independent of the WOC package and belongs at
the Runtime13 host boundary.

## 架构修复验收

- Publish one stable `zr.zircon.math` scalar ABI that exposes `abs`, `atan2`,
  `ceil`, `cos`, `exp`, `floor`, `sin`, `sqrt`, and `pow` with explicit float
  signatures and a documented deterministic precision and non-finite policy.
- Register the functions through the same Runtime13 host export/capability and
  reflection-ledger path as the existing vector helpers; generated host-module
  documentation and module-surface tests must include every new export.
- Add focused deterministic vectors for signed zero, negative values, quadrant
  boundaries, integer boundaries, exponent cases and rejected/non-finite values
  on every supported runtime target. Do not rely on host-specific unspecified
  libm behavior without an explicit cross-platform contract.
- Re-run the Runtime13 script/module-surface suite and then the WOC canonical
  `zr_vm:project` scalar vectors after Plugins08 returns the transactional
  project backend. The WOC vector result must match its pinned source behavior.

## 禁止临时方案

- Do not add a WOC-local `%import` math module, a native gameplay calculation,
  an approximation table, a fallback backend, or a call-site-specific branch.
- Do not expose a second scalar module identity or a platform-dependent alias.
- Do not weaken WOC source semantics, scalar vectors, finite-value checks or
  acceptance criteria to fit the current two-function surface.

## 修复结果与回传

Open state: `待修复`; no scalar-math host API or WOC execution pass is claimed.
