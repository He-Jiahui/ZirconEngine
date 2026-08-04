---
title: WOS224 Adrenaline Rush resource runtime
status: implemented_static_validation_pending
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS224 Adrenaline Rush Resource Runtime

## Scope

Replicate source Rogue `adrenaline_rush` (Quickened Blood): a level-20,
zero-cost, instant, targetless, physical, off-GCD self ability with a
180-second cooldown. Its only source effect restores 60 Energy immediately.

The slice reuses the existing WOS215 `gainResource` talent resolver and WOS221
exact-resource state. It must calculate the source-resolved positive gain from
the Rogue's current talent modifiers, add it through the shared capped precise
resource accessor, and preserve fractional current/max resource values. It
does not create an aura, consume RNG, mutate the normal GCD, or add a schema
tail.

## Delivery Order

1. Append `adrenaline_rush` after Pummel in the generated M4 catalog without
   renumbering earlier entries. Regenerate both JSON and Zr projections and
   add a source-pinned static guard plus a `zr_vm:project` fixture.
2. Add exact profile and typed-payload helpers, normal known-ability/spec/level
   admission, generic-slot and typed routing, off-GCD cooldown handling, and
   direct shared resource application. Reuse the gain-resource resolver rather
   than duplicating physical talent math.
3. Cover valid restore, cap, cooldown, normal-GCD preservation, wrong class,
   level/class/resource-bar rejection, typed/slot parity, snapshot round-trip
   and zero-RNG behavior in a focused world-state fixture.
4. Run code generation, static regression and a second review. Dynamic
   execution is solely `zr_vm:project`; unavailable plugin execution delays
   acceptance but never introduces a fallback runtime.

## Boundaries

- Do not represent the source restore as an aura or a one-off integer write.
- Do not alter WOS221 resource schema, Pummel Rage semantics, WOS222 scripted
  casts, or WOS223 Lockout DR.
- Do not add engine, ZrVM host, or native gameplay code. This is WOC-owned
  deterministic gameplay state using already-retained primitives.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| WOS224 | Adrenaline Rush M4 projection, exact Energy reducer, command routing and plugin fixture | implementation complete; second static review complete; dynamic validation pending | 2026-08-03 | `wos224_adrenaline_rush_runtime_static_guard.mjs`, M4 JSON/Zr codegen checks, and `m4_ability_projection_coverage_codegen.mjs --check` passed; no ZrVM or fallback runtime executed. |
