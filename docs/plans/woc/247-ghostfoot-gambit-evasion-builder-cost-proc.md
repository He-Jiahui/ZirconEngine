---
title: WOS247 Ghostfoot Gambit Evasion builder-cost proc
status: planned
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS247 Ghostfoot Gambit Evasion builder-cost proc

## Objective

Project the source-pinned Rogue selection `rog_r17_improved_evasion` / Ghostfoot
Gambit. A successful Evasion cast restores 30 energy and grants the next Rogue
builder, within eight seconds, a 50% energy-cost multiplier.

## Source contract

- The fixed source row declares `rog_improved_evasion` as `castNth(n: 1,
  abilities: [evasion])`, followed in order by `resource(30)` and
  `empowerNext(next_cast_cheap, rogueBuilderAbilityIds, duration: 8,
  costPct: 0.5)`.
- The source cast-lifecycle admission obtains `nextCastCheapMultiplier` before
  its resource gate and uses `ceil(baseCost * multiplier)`. It consumes the
  cheap aura only after all cast gates admit a successful instant/channel cast;
  a rejected cast does not spend the aura.
- `next_cast_cheap` marks the source cast as empowered when consumed, but this
  `castNth(1)` response only observes Evasion and cannot recursively trigger
  from the subsequent builder. Neither response has chance nor internal
  cooldown, so the Evasion event takes no RNG draw.

## Existing WOC support and gap

- Generated current proc catalog already supplies selection code `68`, proc
  index `17`, one Evasion trigger, the 30-energy response, and the eight
  canonical source builder IDs with `costPct: 0.5`.
- WOC already persists and ages source-shaped `next_cast_cheap` modifiers,
  resolves their scoped target sets, and removes them only through the generic
  successful-cast consumer. The generic talent-proc runtime already exposes
  `onCastCompleted` and response planning.
- WOC currently projects Evasion and Hemorrhage but does not bridge Evasion
  into this proc or apply cheap-cost admission/consumption to Hemorrhage. This
  is an example-game wiring gap only; no ZirconEngine or `zr_vm:project`
  backend capability is missing.

## Delivery order

1. Add one Evasion completion reducer gated to the primary live Rogue and its
   row-17 selection. Rehydrate the generated proc definition, invoke generic
   `onCastCompleted` for Evasion with no prior empower consumption, and reject
   unexpected chance/RNG requests.
2. Apply planned responses in source order: cap the 30 energy refund at the
   existing resource maximum, then derive the eight known builder ability codes
   from the generated response, sort them canonically, and write the existing
   `next_cast_cheap` modifier for eight seconds at multiplier `0.5`. Use the
   source selection code as the durable modifier identity.
3. Call that reducer only after Evasion has passed every existing admission
   check, armed cooldown, and written its dodge aura; invalid, dead, cooldown,
   or unavailable casts produce neither resource nor modifier.
4. Extend the existing Hemorrhage projection to read scoped cheap-cost state
   before its resource gate, use `ceil(cost * multiplier)` for admission and
   billing, and consume the modifier only after all successful-cast work begins.
   Keep the modifier armed for rejected casts, non-builder casts, expiry, and
   unsupported targets.
5. Add a `zr_vm:project` fixture and source guard covering generated metadata,
   no RNG, Evasion gates, response order/cap, snapshot persistence, builder
   scope, rejected-cast retention, `ceil(35 * 0.5) == 18`, one-shot
   consumption, and eight-second expiry. Run static and generated-catalog
   checks; coordinator-owned dynamic acceptance is not polled.

## Non-goals

- No engine, ZrVM, input-protocol, schema, or ability-catalog change.
- No synthetic Rogue builders and no projection of unimplemented builder casts
  solely to consume the modifier.
- No resource refund for failed Evasion and no fixed-cost shortcut that bypasses
  source `ceil` semantics.

## 状态与产出记录

| 里程碑 | 状态 | 日期 | Evidence |
| --- | --- | --- | --- |
