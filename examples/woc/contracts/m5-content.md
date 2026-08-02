# WOC M5 progression and economy content contract

`m5_content.json` is the checked-in, source-pinned content inventory for the 11
M5 parity scenarios and the current authoritative economy extensions. Schema 6
is generated from Git blobs at
`5ef9f7cb21cd8875b6d2c49701015dfcd78de35a`; the nested reference checkout's
moving HEAD is never used implicitly.

## Scope

The generator parses the exact scenario factories in
`tests/parity/scenarios.ts`, extracts direct item, quest, mob, NPC, talent,
specialization and loadout-ability references, then evaluates the pinned
TypeScript content graph through `typescript_git_loader.mjs`. Quest objectives
extend the direct scope: `q_boars` adds `boar_hide`, while `q_wolves` associates
`forest_wolf` with both kill-credit and linked-quest scenarios. Quest givers and
all three banker definitions are included from the target data graph.
The selected mob's real loot table extends the item closure with
`milepost_boots` and `wolfhide_satchel` in addition to the already-direct
`wolf_fang`.
The selected Arms/Fury definitions extend the ability closure with their
signature abilities. This derives `bloodthirst` even though only
`mortal_strike` and `overpower` appear directly in the saved action bar.

The current catalog contains:

- 11 scenario names;
- 82 item definitions: the stable 67-item scenario/economy prefix followed by
  the 15 source mech-chroma armor plates;
- 2 quest definitions (`q_boars`, `q_wolves`);
- 1 mob definition (`forest_wolf`);
- 6 NPC definitions, including the market merchant, provisioner, quest giver
  and three bankers;
- 2 warrior specialization definitions and 3 scoped ability definitions;
- 2 source Delve shops with exact door positions, 18 stock rows and generated
  `autoCompanionId` mappings to Tessa and Edda;
- 15 source-ordered mech chromas with exact rank, skin index, armor-plate item
  identity and protected `noVendorSell`, `noDiscard`, and `noMarketList` flags;
- the exact 20-level XP table, prestige threshold `23200`, market cut `0.05`
  and 12 bank expansion prices.

The NPC definitions remain the complete scoped target definitions, including
Wilkes' full vendor list. Item entries are deliberately the scenario/economy
closure, not a claim that the target's complete item catalog has been ported.

`m5_camp_mob_loot.json` is a separate whole-world input projection for the 47
camp template identities: 177 source-ordered loot entries, 131 item entries,
46 copper entries, 23 quest-gated entries, 24 roll-group entries and 29
component tags. It preserves explicit optional-field presence instead of
collapsing absent values into data. The generated Zr module exposes data only;
the source-order RNG, quest-recipient filtering, heroic substitution and
inventory/corpse transaction remain M5 reducer work.

## Drift gate

Run from `examples/woc/tools`:

```text
npm run generate:m5
npm run check:m5
```

`--check` re-extracts the pinned blobs, verifies the scenario source against the
recorded source identities, asserts the exact direct and derived ID sets, and
compares the complete JSON byte-for-byte. The current catalog SHA-256 is
`b2b1f49754c5fddac3ffe8b71aa29395e4cf761f61421de1b1f12b9f485fa840`.

This inventory is content/source evidence only. It does not implement M5
transactions, scenario drivers, ZrVM world integration or any real-M2 golden
comparison.

## ZrVM projection

`m5_content_zr_codegen.mjs` converts the checked JSON into the 5332-line scalar
`generated/m5_content_catalog.zr`. It exposes category counts and IDs, selected
item/quest/mob/NPC/talent/ability fields, the complete Forest Wolf loot table,
Wilkes' vendor gate, ability ranks, XP, prestige, market-cut and bank-price
lookups without sending custom objects across the VM boundary.

The current projection has passed source-pinned generation and byte-for-byte
checks. Interpreter and binary execution remain acceptance work for the shared
`zr_vm:project` backend; no historical dynamic result is promoted to these bytes.
