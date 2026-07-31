// Executes the locked target source through typescript_git_loader.mjs.  This
// extracts all fixed Delve module collision sets and their default chains, but
// deliberately does not model an active run's mutable module selection.

const data = await import('wocgit:///src/sim/data.ts');
const delve = await import('wocgit:///src/sim/delve_layout.ts');
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

const moduleIds = Object.keys(delve.DELVE_MODULE_LAYOUTS);
const layouts = moduleIds.map((id) => ({
  id,
  span: delve.delveModuleSpan(id),
  colliders: delve.delveModuleColliders(id).map(toCatalogCollider),
}));
const defaultChains = data.DELVE_LIST.map((definition) => ({
  id: definition.id,
  index: definition.index,
  modules: data.defaultDelveModules(definition.id).map((moduleId) => moduleIds.indexOf(moduleId)),
}));
const origin0 = data.delveOrigin(0, 0);
const origin1 = data.delveOrigin(1, 0);
const slot1 = data.delveOrigin(0, 1);
const routing = {
  delve_band_x_min: data.DELVE_BAND_X_MIN,
  yumi_band_x_min: data.YUMI_BAND_X_MIN,
  delve_origin_base_x: origin0.x,
  delve_origin_index_spacing: origin1.x - origin0.x,
  delve_origin_z0: origin0.z,
  delve_slot_count: data.DELVE_SLOT_COUNT,
  delve_slot_spacing: slot1.z - origin0.z,
  delve_module_gap: data.DELVE_MODULE_GAP,
  delve_module_z_start: data.DELVE_MODULE_Z_START,
};
const sourceVectors = [
  {
    id: 'reliquary_side_wall',
    module_index: 0,
    world_x: 4774,
    world_z: -1206,
    origin_x: 4800,
    origin_z: -1242,
  },
  {
    id: 'litany_sluice_entry_slab',
    module_index: 4,
    world_x: 5395.333333333333,
    world_z: -1256,
    origin_x: 5400,
    origin_z: -1242,
  },
].map((vector) => ({
  ...vector,
  resolved: collision.resolvePosition(20061, vector.world_x, vector.world_z, 0.5),
}));
const movementVectors = [
  {
    id: 'reliquary_side_wall_sweep',
    module_index: 0,
    origin_x: 4800,
    origin_z: -1242,
    from_x: 4770,
    from_z: -1206,
    to_x: 4780,
    to_z: -1206,
  },
  {
    id: 'litany_sluice_slab_sweep',
    module_index: 4,
    origin_x: 5400,
    origin_z: -1242,
    from_x: 5395.333333333333,
    from_z: -1248,
    to_x: 5395.333333333333,
    to_z: -1262,
  },
].map((vector) => ({
  ...vector,
  resolved: collision.resolveMovement(
    20061, vector.from_x, vector.from_z, vector.to_x, vector.to_z, 0.5,
  ),
}));
const sightVectors = [
  ['reliquary_wall', { x: 4770, z: -1206 }, { x: 4780, z: -1206 }, false],
  ['reliquary_clear', { x: 4800, z: -1220 }, { x: 4802, z: -1220 }, true],
  ['litany_slab', { x: 5395.333333333333, z: -1248 }, { x: 5395.333333333333, z: -1262 }, false],
  ['litany_clear', { x: 5400, z: -1240 }, { x: 5400, z: -1230 }, true],
  ['fallback_wall', { x: 5970, z: -1206 }, { x: 5980, z: -1206 }, false],
  ['fallback_clear', { x: 6000, z: -1220 }, { x: 6002, z: -1220 }, true],
].map(([id, from, to, expected]) => ({
  id,
  from,
  to,
  radius: 0.05,
  clear: collision.lineOfSightClear(20061, from, to, 0.05),
  expected,
}));

process.stdout.write(JSON.stringify({
  layouts, defaultChains, routing, sourceVectors, movementVectors, sightVectors,
}));
