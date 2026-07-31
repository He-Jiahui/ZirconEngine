const data = await import('wocgit:///src/sim/data.ts');
const dockLayout = await import('wocgit:///src/sim/dock_layout.ts');
const valeCup = await import('wocgit:///src/sim/vale_cup_layout.ts');
const builtin = data.BUILTIN_WORLD;

const zones = data.ZONES.map((zone) => ({
  id: zone.id,
  z_min: zone.zMin,
  z_max: zone.zMax,
  biome: zone.biome,
  hub: {
    x: zone.hub.x,
    z: zone.hub.z,
    radius: zone.hub.radius,
  },
  lakes: zone.lakes.map((lake) => ({
    x: lake.x,
    z: lake.z,
    radius: lake.radius,
  })),
}));

const camps = builtin.camps.map((camp) => {
  const template = data.MOBS[camp.mobId];
  if (!template) {
    throw new Error(`missing camp mob template ${camp.mobId}`);
  }
  return {
    mob_id: camp.mobId,
    mob_is_dummy: template.dummy === true,
    mob_min_level: template.minLevel,
    mob_max_level: template.maxLevel,
    x: camp.center.x,
    z: camp.center.z,
    radius: camp.radius,
    count: camp.count,
  };
});

const terrainEdits = (builtin.terrainEdits ?? []).map((edit) => ({
  x: edit.x,
  z: edit.z,
  radius: edit.radius,
  delta: edit.delta,
  falloff: edit.falloff,
  mode: edit.mode,
}));

const roads = builtin.roads.map((road) => road.map((point) => ({ x: point.x, z: point.z })));

const docks = builtin.props.docks.map((dock) => ({
  x: dock.x,
  z: dock.z,
  rotation: dock.rot,
  hut_local: {
    x: dock.hutLocal.x,
    z: dock.hutLocal.z,
    half_width: dock.hutLocal.hw,
    half_depth: dock.hutLocal.hd,
  },
}));

const sowfieldFlat = {
  x_min: valeCup.SOWFIELD_FLAT.xMin,
  x_max: valeCup.SOWFIELD_FLAT.xMax,
  z_min: valeCup.SOWFIELD_FLAT.zMin,
  z_max: valeCup.SOWFIELD_FLAT.zMax,
  height: valeCup.SOWFIELD_FLAT.height,
  falloff: valeCup.SOWFIELD_FLAT.falloff,
};

const sowfieldExclude = {
  x_min: valeCup.SOWFIELD_EXCLUDE.xMin,
  x_max: valeCup.SOWFIELD_EXCLUDE.xMax,
  z_min: valeCup.SOWFIELD_EXCLUDE.zMin,
  z_max: valeCup.SOWFIELD_EXCLUDE.zMax,
};

const sowfieldStands = [valeCup.STAND_NORTH, valeCup.STAND_SOUTH].map((stand) => ({
  x_min: stand.xMin,
  x_max: stand.xMax,
  z_min: stand.zMin,
  z_max: stand.zMax,
}));

const dockLayoutValues = {
  section_local_z: [...dockLayout.DOCK_SECTION_LOCAL_Z],
  section_half_width: dockLayout.DOCK_SECTION_HALF_WIDTH,
  section_half_depth: dockLayout.DOCK_SECTION_HALF_DEPTH,
  terrain_clearance: dockLayout.DOCK_SECTION_TERRAIN_CLEARANCE,
  surface_y: dockLayout.DOCK_SECTION_SURFACE_Y,
};

process.stdout.write(JSON.stringify({
  zones,
  camps,
  terrain_edits: terrainEdits,
  roads,
  docks,
  sowfield_flat: sowfieldFlat,
  sowfield_exclude: sowfieldExclude,
  sowfield_stands: sowfieldStands,
  sowfield_stand_tier_depth: valeCup.VC_STAND_TIER_DEPTH,
  sowfield_stand_tier_heights: [...valeCup.VC_STAND_TIER_HEIGHTS],
  dock_layout: dockLayoutValues,
}));
