// Executes the pinned Delve definitions and mirrors the fixed source body of
// `pickDelveModules`. The full `data.ts` aggregation is currently not loadable
// through Node 22's custom TypeScript loader, so use the direct source modules
// it imports. The generator pins `runs.ts` too, preventing this mirror from
// silently drifting from the active-run selector.

const reliquary = await import('wocgit:///src/sim/content/delves/collapsed_reliquary.ts');
const litany = await import('wocgit:///src/sim/content/delves/drowned_litany.ts');
const layouts = await import('wocgit:///src/sim/delve_layout.ts');
const rngModule = await import('wocgit:///src/sim/rng.ts');

const moduleIds = Object.keys(layouts.DELVE_MODULE_LAYOUTS);
const seeds = [0, 1, 2, 5, 42, 20061, 2147483647];
const definitions = [
  reliquary.COLLAPSED_RELIQUARY_DELVE,
  litany.DROWNED_LITANY_DELVE,
].sort((left, right) => left.index - right.index);
const delves = definitions.map((definition) => ({
  id: definition.id,
  index: definition.index,
  non_final_module_indices: definition.modules
    .filter((moduleId) => moduleId !== definition.finaleModuleId)
    .map((moduleId) => moduleIds.indexOf(moduleId)),
  finale_module_index: moduleIds.indexOf(definition.finaleModuleId),
  tier_ids: definition.tiers.map((tier) => tier.id),
  module_counts: [...definition.moduleCount],
}));

const selectionVectors = [];
for (const definition of definitions) {
  const tierIds = [...definition.tiers.map((tier) => tier.id), 'unknown'];
  for (const tierId of tierIds) {
    for (const seed of seeds) {
      selectionVectors.push({
        delve_index: definition.index,
        tier_id: tierId,
        seed,
        module_indices: pickDelveModules(definition, seed, tierId)
          .map((moduleId) => moduleIds.indexOf(moduleId)),
      });
    }
  }
}

process.stdout.write(JSON.stringify({ delves, selection_vectors: selectionVectors }));

function pickDelveModules(delve, seed, tierId) {
  const rng = new rngModule.Rng(seed);
  const pool = delve.modules.filter((id) => id !== delve.finaleModuleId);
  const shuffled = [...pool];
  for (let i = shuffled.length - 1; i > 0; i--) {
    const j = rng.int(0, i);
    [shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]];
  }
  const tierIndex = delve.tiers.findIndex((tier) => tier.id === tierId);
  const count = delve.moduleCount[tierIndex >= 0 ? tierIndex : 0] ?? delve.moduleCount[0];
  const picked = shuffled.slice(0, count);
  picked.push(delve.finaleModuleId);
  return picked;
}
