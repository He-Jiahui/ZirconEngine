// Executes direct pinned Delve definition modules and mirrors rollDelveAffixes.
// runs.ts itself remains source-hash-locked by the generator because its data
// aggregate cannot currently load through Node 22's TypeScript loader.
const affixes = await import('wocgit:///src/sim/content/delves/affixes.ts');
const reliquary = await import('wocgit:///src/sim/content/delves/collapsed_reliquary.ts');
const litany = await import('wocgit:///src/sim/content/delves/drowned_litany.ts');
const rngModule = await import('wocgit:///src/sim/rng.ts');

const implementedIds = [
  'restless_graves', 'bad_air', 'candleblind', 'high_water', 'lively_choir', 'belligerent_dead',
];
const seedXor = 0x5a11c0de;
const seeds = [0, 1, 2, 5, 42, 20061, 2147483647, seedXor];
const definitions = [reliquary.COLLAPSED_RELIQUARY_DELVE, litany.DROWNED_LITANY_DELVE]
  .sort((left, right) => left.index - right.index);
const entries = Object.values(affixes.DELVE_AFFIXES);
const delves = definitions.map((definition) => ({
  id: definition.id,
  index: definition.index,
  theme: definition.theme,
  tiers: definition.tiers.map((tier) => ({ id: tier.id, affix_count: tier.affixCount })),
  pool: entries.filter((affix) => !affix.blessing && affix.themes.includes(definition.theme) &&
    implementedIds.includes(affix.id)).map((affix) => affix.id),
}));
const vectors = [];
for (const definition of definitions) {
  for (const tier of definition.tiers) {
    for (const seed of seeds) {
      vectors.push({
        delve_index: definition.index,
        tier_id: tier.id,
        seed,
        affixes: roll(definition, tier.id, seed),
      });
    }
  }
}
process.stdout.write(JSON.stringify({ seed_xor: seedXor, delves, vectors }));

function roll(delve, tierId, seed) {
  const tier = delve.tiers.find((candidate) => candidate.id === tierId) ?? delve.tiers[0];
  if (tier.affixCount <= 0) return [];
  const pool = entries.filter((affix) => !affix.blessing && affix.themes.includes(delve.theme) &&
    implementedIds.includes(affix.id));
  const rng = new rngModule.Rng(seed ^ seedXor);
  const shuffled = [...pool];
  for (let index = shuffled.length - 1; index > 0; index--) {
    const selected = rng.int(0, index);
    [shuffled[index], shuffled[selected]] = [shuffled[selected], shuffled[index]];
  }
  return shuffled.slice(0, Math.min(tier.affixCount, shuffled.length)).map((affix) => affix.id);
}
