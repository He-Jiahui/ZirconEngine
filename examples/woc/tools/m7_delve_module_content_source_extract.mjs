// Extracts active Delve module content from the pinned source definition
// modules. `data.ts` is intentionally avoided because Node 22 cannot complete
// that aggregate through the custom TypeScript loader.

const reliquary = await import('wocgit:///src/sim/content/delves/collapsed_reliquary.ts');
const litany = await import('wocgit:///src/sim/content/delves/drowned_litany.ts');
const layouts = await import('wocgit:///src/sim/delve_layout.ts');

const moduleIds = Object.keys(layouts.DELVE_MODULE_LAYOUTS);
const puzzleKinds = new Set([
  'pressure_plate',
  'sluice_valve',
  'grave_tablet',
  'corpse_candle',
  'bell_rope',
]);
const definitions = {
  ...reliquary.COLLAPSED_RELIQUARY_MODULES,
  ...litany.DROWNED_LITANY_MODULES,
};

const modules = moduleIds.map((id, moduleIndex) => {
  const definition = definitions[id];
  if (!definition) throw new Error(`missing Delve module definition for ${id}`);
  const interactables = definition.interactableSlots.flatMap((slot) =>
    slot.variants.filter((kind) => kind !== 'darkness_zone').map((kind) => ({
      kind,
      x: slot.x,
      z: slot.z,
    })));
  return {
    id,
    module_index: moduleIndex,
    spawn_sets: definition.spawnSets.map((set) => ({
      id: set.id,
      weight: set.weight,
      spawns: set.spawns.map((spawn) => ({
        mob_id: spawn.mobId,
        x: spawn.x,
        z: spawn.z,
      })),
    })),
    interactables,
    puzzle_interactable_count: interactables.filter((entry) => puzzleKinds.has(entry.kind)).length,
    puzzle_interactable_indices: interactables
      .map((entry, index) => puzzleKinds.has(entry.kind) ? index : -1)
      .filter((index) => index >= 0),
    hazards: (definition.hazards ?? []).map((hazard) => ({
      x: hazard.x,
      z: hazard.z,
      r: hazard.r,
      rx: hazard.rx ?? hazard.r,
      rz: hazard.rz ?? hazard.r,
      tier: hazard.tier ?? 'deep',
    })),
  };
});

process.stdout.write(JSON.stringify({ modules }));
