# WOC Weapon Skin Contract

`reference/current-head/weapon_skin_contract.json` is the source-pinned Season 1
Armory catalog used by the WOC replication. It is generated from source commit
`5ef9f7cb21cd8875b6d2c49701015dfcd78de35a`; do not maintain a second handwritten
skin list in runtime code.

The contract contains 29 skin identities in authored `WEAPON_SKINS` order, eight
ordered skin types (`sword`, `axe`, `mace`, `dagger`, `staff`, `wand`, `bow`,
`crossbow`) and 92 item-to-weapon-type mappings. `polearm` remains an explicit
non-skinnable item type. The generated Zr projection provides stable one-based
skin and type codes plus exact UTF-8 id accessors.

Runtime rules are account-scoped:

- Apply accepts only a known skin owned by the account. Non-hunter players
  must match the requesting player's currently displayed mainhand type.
  Hunters must have a mainhand equipped, but may apply bow or crossbow skins
  independently of that mainhand's concrete type.
- Detach accepts one canonical weapon type and removes only an existing entry.
- Hunter bow and crossbow entries are mutually exclusive. Active resolution
  checks crossbow before bow; other types stay parked for compatible gear swaps.
- A successful account loadout change re-resolves every live player row. The
  active per-entity skin is derived from class, mainhand and account loadout.
- The command has no dead-state gate and consumes no simulation RNG.

For a fresh offline world, the host supplies this account state in
`OfflineSessionBootstrap` v2 before Tick 1: 29 source-ordered ownership markers
followed by eight source-ordered one-based loadout codes (`0` means no selected
skin). The Rust transport rejects non-boolean markers, unknown codes and selected
codes that are not owned; the Zr reducer additionally rejects a code placed in
the wrong weapon-type slot. This is account-data hydration only: Claudium grants
and purchases remain outside the simulation.

The active code is retained as the authoritative per-entity projection. The
generic `zr_vm:project` transaction bridge still lacks the lossless dual-output
path needed to return `Entity.weaponSkinId` as the native
`ActorAppearance.weapon_skin_id` in the same tick. That shared capability is
tracked by `docs/plans/zircon_plugins/08/failure-2026-07-18-woc-zrvm-deterministic-bulk-cross-platform-runtime.md`;
no WOC-specific VM adapter or alternate runtime is used.

The command payload is a discriminated binary union: mode `1` carries a bounded
`u32` little-endian UTF-8 skin id; mode `0` carries a generated one-byte weapon
type code. This omits the source request's redundant `wtype` during apply because
the authoritative catalog derives it from the skin id.

Regenerate or verify the projection from `examples/woc/tools`:

```text
npm run generate:weapon-skin-contract
npm run check:weapon-skin-contract
```

Claudium purchasing and account grants are host economy-service operations. The
WOC world persists and enforces ownership supplied at that boundary; this
contract does not define a local purchase substitute.
