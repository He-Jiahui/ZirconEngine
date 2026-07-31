import { createHash } from 'node:crypto';

const data = await import('wocgit:///src/sim/data.ts');
const world = await import('wocgit:///src/sim/world.ts');
const seed = 20061;
const decorations = world.generateDecorations(seed);
const startX = -(data.WORLD_MAX_X - 14);
const startZ = data.ZONES[0].zMin + 14;

const representative = [
  find('tree', (value) => true),
  find('tree2', (value) => true),
  find('rock', (value) => value.scale >= 0.8),
  find('rock', (value) => value.scale < 0.8),
].map(withGrid);

const canonical = decorations.map((value) => ({
  kind: value.kind,
  x: value.x,
  z: value.z,
  scale: value.scale,
  variant: value.variant,
  biome: value.biome,
}));
const kindCounts = Object.fromEntries(
  ['tree', 'tree2', 'rock'].map((kind) => [
    kind,
    decorations.filter((value) => value.kind === kind).length,
  ]),
);

process.stdout.write(JSON.stringify({
  seed,
  count: decorations.length,
  kind_counts: kindCounts,
  sha256: createHash('sha256').update(JSON.stringify(canonical)).digest('hex'),
  representative,
  rejected_grid: { x: 0, z: 8 },
}));

function find(kind, predicate) {
  const value = decorations.find((candidate) => candidate.kind === kind && predicate(candidate));
  if (!value) throw new Error(`missing ${kind} decoration representative`);
  return value;
}

function withGrid(value) {
  return {
    kind: value.kind,
    grid_x: startX + Math.round((value.x - startX) / 10) * 10,
    grid_z: startZ + Math.round((value.z - startZ) / 10) * 10,
    x: value.x,
    z: value.z,
    scale: value.scale,
    variant: value.variant,
    biome: value.biome,
  };
}
