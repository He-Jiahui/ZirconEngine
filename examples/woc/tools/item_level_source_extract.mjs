const data = await import('wocgit:///src/sim/data.ts');
const itemLevel = await import('wocgit:///src/sim/item_level.ts');

const items = Object.keys(data.ITEMS).sort().map((id) => {
  const definition = data.ITEMS[id];
  return {
    id,
    source_level: optionalInteger(itemLevel.itemSourceLevel(id)),
    from_raid: itemLevel.itemFromRaid(id),
    item_level: optionalInteger(itemLevel.itemLevel(definition)),
  };
});
const rareSource = items.find((item) =>
  item.source_level !== null && data.ITEMS[item.id].quality === 'rare');
if (!rareSource) throw new Error('item-level source contains no rare sourced item fixture');

process.stdout.write(JSON.stringify({
  items,
  rare_source_fixture: {
    id: rareSource.id,
    source_level: rareSource.source_level,
  },
}));

function optionalInteger(value) {
  if (value === undefined) return null;
  if (!Number.isInteger(value)) throw new Error('item-level source returned a non-integer: ' + value);
  return value;
}
