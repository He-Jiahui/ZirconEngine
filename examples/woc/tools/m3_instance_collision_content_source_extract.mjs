const layout = await import('wocgit:///src/sim/dungeon_layout.ts');
const data = await import('wocgit:///src/sim/data.ts');
const yumi = await import('wocgit:///src/sim/yumi_maze_layout.ts');
const collision = await import('wocgit:///src/sim/colliders.ts');

const toCatalogCollider = (collider) => collider.type === 'circle' ? ({
  kind: 'circle',
  x: collider.x,
  z: collider.z,
  radius: collider.r,
  half_width: 0,
  half_depth: 0,
  rotation: 0,
}) : ({
  kind: 'obb',
  x: collider.x,
  z: collider.z,
  radius: 0,
  half_width: collider.hw,
  half_depth: collider.hd,
  rotation: collider.rot,
});

const layouts = [
  ['crypt', layout.CRYPT_LAYOUT],
  ['sanctum', layout.SANCTUM_LAYOUT],
  ['temple', layout.TEMPLE_LAYOUT],
  ['nythraxis', layout.NYTHRAXIS_LAYOUT],
  ['arena', layout.ARENA_LAYOUT],
].map(([id, definition]) => ({
  id,
  colliders: layout.layoutColliders(definition).map(toCatalogCollider),
}));
layouts.push({ id: 'yumi', colliders: yumi.yumiMazeColliders().map(toCatalogCollider) });

const dungeonOrigin0 = data.instanceOrigin(0, 0);
const dungeonOrigin1 = data.instanceOrigin(1, 0);
const dungeonSlot1 = data.instanceOrigin(0, 1);
const arenaOrigin0 = data.arenaOrigin(0);
const arenaOrigin1 = data.arenaOrigin(1);
const yumiOrigin0 = data.yumiMazeOrigin(0);
const yumiOrigin1 = data.yumiMazeOrigin(1);
const routing = {
  dungeon_x_threshold: data.DUNGEON_X_THRESHOLD,
  dungeon_slot_count: data.INSTANCE_SLOT_COUNT,
  dungeon_origin_base_x: dungeonOrigin0.x,
  dungeon_origin_index_spacing: dungeonOrigin1.x - dungeonOrigin0.x,
  dungeon_origin_z0: dungeonOrigin0.z,
  dungeon_slot_spacing: dungeonSlot1.z - dungeonOrigin0.z,
  arena_x: data.ARENA_X,
  arena_x_min: data.ARENA_X_MIN,
  arena_slot_count: data.ARENA_SLOT_COUNT,
  arena_origin_z0: arenaOrigin0.z,
  arena_slot_spacing: arenaOrigin1.z - arenaOrigin0.z,
  delve_band_x_min: data.DELVE_BAND_X_MIN,
  yumi_band_x_min: data.YUMI_BAND_X_MIN,
  yumi_band_x_max: data.YUMI_BAND_X_MAX,
  yumi_maze_x: data.YUMI_MAZE_X,
  yumi_maze_slot_count: data.YUMI_MAZE_SLOT_COUNT,
  yumi_maze_origin_z0: yumiOrigin0.z,
  yumi_maze_slot_spacing: yumiOrigin1.z - yumiOrigin0.z,
  yumi_maze_seed: yumi.YUMI_MAZE_SEED,
};

const dungeons = data.DUNGEON_LIST.map((dungeon) => ({
  id: dungeon.id,
  index: dungeon.index,
  interior: dungeon.interior,
}));
const sightVectors = [
  ['crypt_wall', { x: 870, z: -1203 }, { x: 880, z: -1203 }, false],
  ['crypt_clear', { x: 900, z: -1240 }, { x: 905, z: -1240 }, true],
  ['arena_wall', { x: 4170, z: -1250 }, { x: 4230, z: -1250 }, false],
  ['arena_clear', { x: 4200, z: -1240 }, { x: 4200, z: -1230 }, true],
  ['yumi_shell', { x: 8400, z: -1250 }, { x: 8400, z: -1305 }, false],
  ['yumi_clear', { x: 8400, z: -1250 }, { x: 8400, z: -1240 }, true],
].map(([id, from, to, expected]) => ({
  id,
  from,
  to,
  radius: 0.05,
  clear: collision.lineOfSightClear(20061, from, to, 0.05),
  expected,
}));

process.stdout.write(JSON.stringify({ layouts, routing, dungeons, sightVectors }));
