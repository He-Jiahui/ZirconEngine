---
title: WOS218 Chain Heal authoritative runtime closure
status: implemented
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS218 Chain Heal Authoritative Runtime

## Source Contract

The source `chain_heal` ability is a 2.5-second, friendly-target Nature cast
with one `chainHeal` effect (`min: 120`, `max: 145`, `jumps: 2`,
`falloff: 0.5`, `radius: 12`). At completion it rolls the base amount once,
including the direct-heal spell-power rider. It then selects each later hop
from living, friendly, unvisited entities within the previous hop's radius:
lowest `hp / maxHp`, then nearest squared distance, then lowest stable entity
id. Each hop receives `max(1, round(baseAmount * falloff ** hop))` and its own
ordinary heal critical-resolution draw.

`scaleEffect` scales `chainHeal.min` and `.max` by the healing talent
multiplier plus its flat modifier before this shared range roll. The initial
target remains subject to ordinary friendly-target range validation at cast
start and completion; later hops are resolved only at completion.

`applyTalentMods` independently resolves the per-ability cost as
`max(0, round(baseCost * (1 + costPct)))`. Restoration's `chain_heal`
`costPct: -0.2` therefore changes the generated cost 60 to 48. Start admission
and completion payment must share that resolved cost.

## Current WOC Surface And Gap

The full known-ability catalog exposes `chain_heal`, while the M4 projection
needs the source record retained without renumbering earlier abilities. WOC
already has stable entity ids, friendly player/pet admission, position rows,
target range checks, authoritative RNG, talent reduction and the per-heal
absorb/cap/threat/critical kernel. No Zircon engine or ZrVM capability is
missing.

## Design

1. Append Chain Heal to the generated M4 projection without renumbering its
   93 existing entries, then add catalog/profile, typed-payload,
   target-resolution, timed-cast and completion routing following the existing
   Healing Wave boundary.
2. Resolve the source's per-ability cost once for both start admission and
   completion payment. At completion consume exactly one range draw, resolve
   the source's shared talent-scaled base magnitude, choose the chain with the exact
   `(hp fraction, distance squared, entity id)` ladder, then invoke the normal
   heal kernel once per hop so crits, absorbs, caps and healer threat retain
   their existing semantics.
3. Add a focused world-state regression that proves selection tie-breaks,
   falloff, resource payment, GCD/cast lifecycle and authoritative RNG order;
   pin source and WOC structure in a static guard and add a `zr_vm:project`
   fixture. No persistence schema change is expected.

## Acceptance

- A target plus two candidates produces the source chain order and amounts;
  hostile, dead, out-of-radius and already-visited entities are excluded.
- One range draw is shared by all hops, while each reached target owns one
  normal heal critical draw; talent scaling changes only the shared base range.
- Typed and generic command routes, completion routing, the Restoration
  48-Mana cost and target/range cancellation agree with the existing friendly
  timed-cast model.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| WOS218 | M4 投影、权威链式选择、双阶段成本、夹具 | 已实现并二次审查 | 2026-08-03 | `m4_ability_codegen --check`、`m4_ability_zr_codegen --check`、`wos218_chain_heal_authoritative_runtime_static_guard.mjs` 通过；独立复审无 P1/P2；未运行 ZrVM/Cargo。 |
