const [sourceModule] = process.argv.slice(2);
if (!sourceModule) {
  throw new Error('usage: talent_proc_catalog_source_extract.mjs <source-module>');
}

const { ROW_TREES, TALENTS } = await import(sourceModule);
const entries = [];
for (const [classId, tree] of Object.entries(ROW_TREES)) {
  const talents = TALENTS[classId];
  if (!talents) throw new Error(`missing talent definitions for ${classId}`);
  for (const spec of talents.specs) {
    if (spec.mastery.effect?.proc) {
      entries.push({ origin: 'spec', class_id: classId, id: spec.id, proc: spec.mastery.effect.proc });
    }
  }
  for (const row of tree) {
    for (const option of row.options) {
      if (option.effect?.proc) {
        entries.push({
          origin: 'option',
          class_id: classId,
          level: row.level,
          id: option.id,
          proc: option.effect.proc,
        });
      }
    }
  }
}

process.stdout.write(JSON.stringify({ entries }));
