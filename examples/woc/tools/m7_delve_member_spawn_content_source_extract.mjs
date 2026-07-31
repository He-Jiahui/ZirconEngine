import { execFileSync } from 'node:child_process';
import ts from 'typescript';

const root = process.env.WOC_GIT_ROOT;
const commit = process.env.WOC_GIT_COMMIT;

if (!root || !commit) {
  throw new Error('WOC_GIT_ROOT and WOC_GIT_COMMIT are required');
}

const sourceText = (path) =>
  execFileSync('git', ['-C', root, 'show', `${commit}:${path}`], { encoding: 'utf8' });

const functionText = (path, name) => {
  const text = sourceText(path);
  const source = ts.createSourceFile(path, text, ts.ScriptTarget.Latest, true, ts.ScriptKind.TS);
  const declaration = source.statements.find(
    (statement) => ts.isFunctionDeclaration(statement) && statement.name?.text === name,
  );
  if (!declaration) {
    throw new Error(`${name} missing from ${path}`);
  }
  return declaration.getText(source);
};

const runs = functionText('src/sim/delves/runs.ts', 'delveMemberSpawnPos');
for (const marker of [
  'if (slotIndex <= 0) return entry;',
  'const angle = (slotIndex * Math.PI * 2) / 6;',
  'entry.x + Math.cos(angle) * DELVE_MEMBER_SPAWN_SPREAD',
  'entry.z + Math.sin(angle) * DELVE_MEMBER_SPAWN_SPREAD',
]) {
  if (!runs.includes(marker)) {
    throw new Error(`delve member spawn source drifted: ${marker}`);
  }
}

for (const path of [
  'src/sim/content/delves/collapsed_reliquary.ts',
  'src/sim/content/delves/drowned_litany.ts',
]) {
  if (!/maxPlayers:\s*2\b/.test(sourceText(path))) {
    throw new Error(`Delve party limit drifted: ${path}`);
  }
}

const spread = 2.2;
const offsetFor = (slotIndex) => {
  if (slotIndex <= 0) {
    return { offset_x: 0, offset_z: 0 };
  }
  const angle = (slotIndex * Math.PI * 2) / 6;
  return {
    offset_x: Math.cos(angle) * spread,
    offset_z: Math.sin(angle) * spread,
  };
};

process.stdout.write(
  JSON.stringify({
    party_max_members: 2,
    spawn_spread: spread,
    horizontal_slots: [
      { slot_index: 0, ...offsetFor(0) },
      { slot_index: 1, ...offsetFor(1) },
    ],
  }),
);
