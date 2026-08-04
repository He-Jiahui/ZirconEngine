---
title: WOS216 Tiger's Fury talent snapshot runtime closure
status: completed
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS216 Tiger's Fury Talent Snapshot Runtime

## Source Contract

`scaleEffect` scales `selfBuff.kind === 'buff_ap'` as
`Math.round(value * dmgMult + flat)`. Tiger's Fury is a physical Druid
`buff_ap` of 40 for six seconds. Feral supplies `global.meleeDmgPct: 0.15`,
so its application-time aura value is 46. The source aura retains that
resolved value after a later respec.

## Current WOC Surface And Gap

WOC already persists Tiger's Fury in its source-keyed motion-aura row, but the
cast path stores the raw 40 and the validator accepts only 40. It therefore
cannot reproduce Feral's resolved buff or preserve it through a later respec
and codec round trip.

## Design

1. Add a pure flat `buff_ap` talent resolver with the source physical/spell
   multiplier, flat and endpoint rounding order.
2. Add entity-aligned Tiger's Fury resolved-value and six-row selection
   snapshot columns. An entity has at most one Tiger's Fury aura, so this
   preserves the existing generic motion-aura layout and its source order.
3. Extend WOS to 111. WOS111 writes the resolved value and selection snapshot
   as a compact entity tail. WOS110 and earlier normalize active Tiger's Fury
   rows to raw 40 with empty snapshots; historical writers reject a nonempty
   snapshot.
4. Capture on cast/refresh, clear on the shared motion-aura removal path, and
   validate the retained aura value by reconstructing it from its snapshot.
   Add respec/restore/expiry regressions, source-pinned static guard and a
   `zr_vm:project` fixture manifest.

## Acceptance

- Feral Tiger's Fury retains 46 through respec and WOS111 restore; the next
  application after respec uses the current selection.
- Expiry and all shared motion-aura removal paths clear the accompanying
  snapshot without reordering or altering unrelated aura rows.
- WOS110 data remains readable and historical encoders cannot discard a
  required Tiger's Fury talent snapshot.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| WOS216 | Tiger's Fury 天赋快照与 WOS111 | completed | 2026-08-03 | `flat_buff_talent_scaling_state.zr`、WOS216 静态守卫、`zr_vm:project` fixture；独立二次审查通过，未运行动态 ZrVM/Cargo。 |
