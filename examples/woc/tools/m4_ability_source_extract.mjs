const [sourceModule, ...abilityIds] = process.argv.slice(2);
if (!sourceModule || abilityIds.length === 0) {
  throw new Error('usage: m4_ability_source_extract.mjs <source-module> <ability-id>...');
}

const { ABILITIES } = await import(sourceModule);
const definitions = abilityIds.map((id) => {
  const definition = ABILITIES[id];
  if (!definition) throw new Error(`M4 ability ${id} is missing from ABILITIES`);
  return definition;
});
process.stdout.write(JSON.stringify(definitions));
