import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const EXPECTED_ZONE_IDS = ['eastbrook_vale', 'mirefen_marsh', 'thornpeak_heights'];
const EXPECTED_BIOMES = ['vale', 'marsh', 'peaks'];
const EXPECTED_LAKE_COUNT = 5;
const EXPECTED_CAMP_COUNT = 67;
const EXPECTED_CAMP_MOB_TOTAL = 307;
const EXPECTED_NON_DUMMY_CAMP_MOB_COUNT = 306;
const EXPECTED_FIRST_CAMP = { mob_id: 'forest_wolf', count: 7, min_level: 1, max_level: 2 };
const EXPECTED_LAST_CAMP = { mob_id: 'grix_the_tunnelking', count: 1, min_level: 7, max_level: 7 };
const EXPECTED_DUMMY_CAMP = {
  index: 36,
  mob_id: 'training_dummy',
  count: 1,
  min_level: 20,
  max_level: 20,
};
const EXPECTED_TERRAIN_EDIT_COUNT = 1;
const EXPECTED_DOCK_COUNT = 2;
const EXPECTED_SOWFIELD_FLAT = {
  x_min: -56,
  x_max: 34,
  z_min: -141,
  z_max: -83,
  height: -2.6,
  falloff: 8,
};
const EXPECTED_SOWFIELD_STANDS = [
  { x_min: -42, x_max: 20, z_min: -96, z_max: -87 },
  { x_min: -42, x_max: 20, z_min: -138, z_max: -129 },
];
const EXPECTED_SOWFIELD_STAND_TIER_DEPTH = 4.6;
const EXPECTED_SOWFIELD_STAND_TIER_HEIGHTS = [0.55, 1.28];
const EXPECTED_SOWFIELD_STAND_RAMP = 1.4;
const EXPECTED_SOWFIELD_EXCLUDE = { x_min: -66, x_max: 44, z_min: -151, z_max: -73 };
const EXPECTED_ROAD_LENGTHS = [4, 3, 4, 4, 4, 4, 4, 5, 3, 4, 4, 3, 3, 3];
const EXPECTED_ROADS_SHA256 = 'fb63b62216ff93c7b4fe7fccfe9bbdc7b5c9bc8a4549ff44dd4390fde78bc0e3';
const EXPECTED_DECORATION_EXCLUSION_RADIUS = 1.2;
const EXPECTED_DECORATION_EXCLUSIONS = [{ x: 2.456450840458274, z: 211.33819991815835 }];
const EXPECTED_DECORATION_MAX_SLOPE = 1.5;
const EXPECTED_DOCK_LAYOUT = {
  section_local_z: [-1.05, -3.18, -5.31],
  section_half_width: 0.98,
  section_half_depth: 1.07,
  terrain_clearance: 0.15,
  surface_y: 0.36,
};
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const sourceManifestPath = join(projectRoot, 'reference', 'current-head', 'source_manifest.json');
const extractorPath = join(scriptDirectory, 'm3_terrain_content_source_extract.mjs');
const loaderUrl = pathToFileURL(join(scriptDirectory, 'typescript_git_loader.mjs')).href;
const contractPath = join(projectRoot, 'contracts', 'm3_terrain_content.json');
const zrPath = join(projectRoot, 'scripts', 'woc_game', 'src', 'world', 'terrain_content.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  execFileSync('git', ['-C', sourceRoot, 'cat-file', '-e', `${SOURCE_COMMIT}^{commit}`]);
  const manifest = JSON.parse(readFileSync(sourceManifestPath, 'utf8'));
  assert(manifest.source_commit === SOURCE_COMMIT, 'WOC source manifest commit drifted');

  const extracted = extractZones();
  assert(extracted.zones.length === EXPECTED_ZONE_IDS.length, 'builtin zone count drifted');
  assert(
    JSON.stringify(extracted.zones.map((zone) => zone.id)) === JSON.stringify(EXPECTED_ZONE_IDS),
    'builtin zone identity or order drifted',
  );
  assert(
    JSON.stringify(extracted.zones.map((zone) => zone.biome)) === JSON.stringify(EXPECTED_BIOMES),
    'builtin biome identity or order drifted',
  );
  const lakes = extracted.zones.flatMap((zone) => zone.lakes);
  assert(lakes.length === EXPECTED_LAKE_COUNT, 'builtin lake count drifted');
  assert(extracted.camps.length === EXPECTED_CAMP_COUNT, 'builtin camp count drifted');
  assert(
    extracted.camps.every((camp) => typeof camp.mob_id === 'string' && camp.mob_id.length > 0),
    'builtin camp mob identity is missing',
  );
  assert(
    extracted.camps.every((camp) => Number.isInteger(camp.count) && camp.count > 0),
    'builtin camp mob count is missing or invalid',
  );
  assert(
    extracted.camps.every((camp) => typeof camp.mob_is_dummy === 'boolean'),
    'builtin camp dummy flag is missing or invalid',
  );
  assert(
    extracted.camps.every(
      (camp) =>
        Number.isInteger(camp.mob_min_level) &&
        Number.isInteger(camp.mob_max_level) &&
        camp.mob_min_level >= 1 &&
        camp.mob_max_level >= camp.mob_min_level,
    ),
    'builtin camp mob level bounds are missing or invalid',
  );
  const campMobTotal = extracted.camps.reduce((total, camp) => total + camp.count, 0);
  const nonDummyCampMobCount = extracted.camps.reduce(
    (total, camp) => total + (camp.mob_is_dummy ? 0 : camp.count),
    0,
  );
  assert(campMobTotal === EXPECTED_CAMP_MOB_TOTAL, 'builtin camp mob total drifted');
  assert(
    nonDummyCampMobCount === EXPECTED_NON_DUMMY_CAMP_MOB_COUNT,
    'builtin non-dummy camp mob total drifted',
  );
  const dummyCamps = extracted.camps.filter((camp) => camp.mob_is_dummy);
  assert(
    dummyCamps.length === 1 &&
      extracted.camps[EXPECTED_DUMMY_CAMP.index].mob_id === EXPECTED_DUMMY_CAMP.mob_id &&
      extracted.camps[EXPECTED_DUMMY_CAMP.index].count === EXPECTED_DUMMY_CAMP.count &&
      extracted.camps[EXPECTED_DUMMY_CAMP.index].mob_min_level === EXPECTED_DUMMY_CAMP.min_level &&
      extracted.camps[EXPECTED_DUMMY_CAMP.index].mob_max_level === EXPECTED_DUMMY_CAMP.max_level,
    'builtin dummy camp identity or order drifted',
  );
  assert(
    extracted.camps[0].mob_id === EXPECTED_FIRST_CAMP.mob_id &&
      extracted.camps[0].count === EXPECTED_FIRST_CAMP.count &&
      extracted.camps[0].mob_min_level === EXPECTED_FIRST_CAMP.min_level &&
      extracted.camps[0].mob_max_level === EXPECTED_FIRST_CAMP.max_level &&
      extracted.camps[0].x === -15 &&
      extracted.camps[0].z === 55 &&
      extracted.camps[0].radius === 22 &&
      extracted.camps.at(-1).mob_id === EXPECTED_LAST_CAMP.mob_id &&
      extracted.camps.at(-1).count === EXPECTED_LAST_CAMP.count &&
      extracted.camps.at(-1).mob_min_level === EXPECTED_LAST_CAMP.min_level &&
      extracted.camps.at(-1).mob_max_level === EXPECTED_LAST_CAMP.max_level &&
      extracted.camps.at(-1).x === -95 &&
      extracted.camps.at(-1).z === -78 &&
      extracted.camps.at(-1).radius === 4,
    'builtin camp source order or sentinel identities drifted',
  );
  assert(
    extracted.terrain_edits.length === EXPECTED_TERRAIN_EDIT_COUNT,
    'builtin terrain edit count drifted',
  );
  assert(extracted.docks.length === EXPECTED_DOCK_COUNT, 'builtin dock count drifted');
  assert(
    JSON.stringify(extracted.roads.map((road) => road.length)) === JSON.stringify(EXPECTED_ROAD_LENGTHS) &&
      sha256(JSON.stringify(extracted.roads)) === EXPECTED_ROADS_SHA256,
    'builtin road content/order drifted',
  );
  assert(
    JSON.stringify(extracted.sowfield_flat) === JSON.stringify(EXPECTED_SOWFIELD_FLAT),
    'Sowfield flatten footprint drifted',
  );
  assert(
    JSON.stringify(extracted.sowfield_exclude) === JSON.stringify(EXPECTED_SOWFIELD_EXCLUDE),
    'Sowfield exclusion footprint drifted',
  );
  assert(
    JSON.stringify(extracted.sowfield_stands) === JSON.stringify(EXPECTED_SOWFIELD_STANDS),
    'Sowfield stand footprint drifted',
  );
  assert(
    extracted.sowfield_stand_tier_depth === EXPECTED_SOWFIELD_STAND_TIER_DEPTH &&
      JSON.stringify(extracted.sowfield_stand_tier_heights) ===
        JSON.stringify(EXPECTED_SOWFIELD_STAND_TIER_HEIGHTS),
    'Sowfield stand tier layout drifted',
  );
  assert(
    JSON.stringify(extracted.dock_layout) === JSON.stringify(EXPECTED_DOCK_LAYOUT),
    'dock surface layout drifted',
  );

  const worldText = gitShow('src/sim/world.ts');
  const dataText = gitShow('src/sim/data.ts');
  const zone3Text = gitShow('src/sim/content/zone3.ts');
  const valeCupText = gitShow('src/sim/vale_cup_layout.ts');
  const sowfieldStandRamp = localNumberConstant(valeCupText, 'VC_STAND_RAMP');
  const decorationExclusionRadius = localNumberConstant(worldText, 'DECORATION_EXCLUSION_RADIUS');
  const decorationExclusions = pointArrayConstant(worldText, 'DECORATION_EXCLUSIONS');
  const decorationMaxSlope = numberConstant(worldText, 'DECORATION_MAX_SLOPE');
  const constants = {
    water_level: numberConstant(worldText, 'WATER_LEVEL'),
    lake_blend_radius_multiplier: numberConstant(worldText, 'LAKE_BLEND_RADIUS_MULT'),
    world_size: numberConstant(dataText, 'WORLD_SIZE'),
    dungeon_x_threshold: numberConstant(dataText, 'DUNGEON_X_THRESHOLD'),
    dungeon_floor_y: numberConstant(dataText, 'DUNGEON_FLOOR_Y'),
  };
  assert(constants.world_size === 360, 'WORLD_SIZE must stay 360 for this projection');
  assert(sowfieldStandRamp === EXPECTED_SOWFIELD_STAND_RAMP, 'Sowfield stand ramp drifted');
  assert(
    decorationExclusionRadius === EXPECTED_DECORATION_EXCLUSION_RADIUS &&
      JSON.stringify(decorationExclusions) === JSON.stringify(EXPECTED_DECORATION_EXCLUSIONS) &&
      decorationMaxSlope === EXPECTED_DECORATION_MAX_SLOPE,
    'decoration exclusion constants drifted',
  );
  assert(
    /training_dummy:\s*\{[\s\S]*?dummy:\s*true,/.test(zone3Text),
    'training dummy source flag drifted',
  );

  const catalog = {
    schema_version: 6,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/m3_terrain_content_codegen.mjs',
    source_sha256: {
      'src/sim/world.ts': sha256(worldText),
      'src/sim/data.ts': sha256(dataText),
      'src/sim/content/zone1.ts': sha256(gitShow('src/sim/content/zone1.ts')),
      'src/sim/content/zone2.ts': sha256(gitShow('src/sim/content/zone2.ts')),
      'src/sim/content/zone3.ts': sha256(zone3Text),
      'src/sim/dock_layout.ts': sha256(gitShow('src/sim/dock_layout.ts')),
      'src/sim/vale_cup_layout.ts': sha256(gitShow('src/sim/vale_cup_layout.ts')),
    },
    constants: {
      water_level: constants.water_level,
      lake_blend_radius_multiplier: constants.lake_blend_radius_multiplier,
      world_min_x: -constants.world_size / 2,
      world_max_x: constants.world_size / 2,
      world_min_z: extracted.zones[0].z_min,
      world_max_z: extracted.zones.at(-1).z_max,
      dungeon_x_threshold: constants.dungeon_x_threshold,
      dungeon_floor_y: constants.dungeon_floor_y,
      camp_mob_total: campMobTotal,
      non_dummy_camp_mob_count: nonDummyCampMobCount,
    },
    zones: extracted.zones,
    lakes,
    camps: extracted.camps,
    terrain_edits: extracted.terrain_edits,
    roads: extracted.roads,
    docks: extracted.docks,
    sowfield_flat: extracted.sowfield_flat,
    sowfield_exclude: extracted.sowfield_exclude,
    sowfield_stands: extracted.sowfield_stands,
    sowfield_stand_tier_depth: extracted.sowfield_stand_tier_depth,
    sowfield_stand_tier_heights: extracted.sowfield_stand_tier_heights,
    sowfield_stand_ramp: sowfieldStandRamp,
    dock_layout: extracted.dock_layout,
    decoration_exclusion_radius: decorationExclusionRadius,
    decoration_exclusions: decorationExclusions,
    decoration_max_slope: decorationMaxSlope,
  };
  catalog.catalog_sha256 = sha256(JSON.stringify({
    constants: catalog.constants,
    zones: catalog.zones,
    lakes: catalog.lakes,
    camps: catalog.camps,
    terrain_edits: catalog.terrain_edits,
    roads: catalog.roads,
    docks: catalog.docks,
    sowfield_flat: catalog.sowfield_flat,
    sowfield_exclude: catalog.sowfield_exclude,
    sowfield_stands: catalog.sowfield_stands,
    sowfield_stand_tier_depth: catalog.sowfield_stand_tier_depth,
    sowfield_stand_tier_heights: catalog.sowfield_stand_tier_heights,
    sowfield_stand_ramp: catalog.sowfield_stand_ramp,
    dock_layout: catalog.dock_layout,
    decoration_exclusion_radius: catalog.decoration_exclusion_radius,
    decoration_exclusions: catalog.decoration_exclusions,
    decoration_max_slope: catalog.decoration_max_slope,
  }));

  const contractText = `${JSON.stringify(catalog, null, 2)}\n`;
  const zrText = renderZr(catalog);
  verifyOrWrite(contractPath, contractText);
  verifyOrWrite(zrPath, zrText);
}

function extractZones() {
  const child = spawnSync(process.execPath, [
    '--no-warnings',
    '--experimental-loader',
    loaderUrl,
    extractorPath,
  ], {
    encoding: 'utf8',
    maxBuffer: 16 * 1024 * 1024,
    env: { ...process.env, WOC_GIT_ROOT: sourceRoot, WOC_GIT_COMMIT: SOURCE_COMMIT },
  });
  assert(child.status === 0, child.stderr || `terrain source extractor exited ${child.status}`);
  return JSON.parse(child.stdout);
}

function gitShow(sourcePath) {
  return execFileSync(
    'git',
    ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${sourcePath}`],
    { encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 },
  );
}

function numberConstant(text, name) {
  const match = text.match(new RegExp(`export const ${name} = (-?(?:\\d+\\.?\\d*|\\.\\d+));`));
  assert(match, `source constant ${name} is missing or non-literal`);
  return Number(match[1]);
}

function localNumberConstant(text, name) {
  const match = text.match(new RegExp(`const ${name} = (-?(?:\\d+\\.?\\d*|\\.\\d+));`));
  assert(match, `source local constant ${name} is missing or non-literal`);
  return Number(match[1]);
}

function pointArrayConstant(text, name) {
  const match = text.match(new RegExp(`const ${name} = \\[(.*?)\\];`, 's'));
  assert(match, `source point array ${name} is missing`);
  const points = [...match[1].matchAll(
    /\{\s*x:\s*(-?(?:\d+\.?\d*|\.\d+)),\s*z:\s*(-?(?:\d+\.?\d*|\.\d+))\s*\}/g,
  )].map((entry) => ({ x: Number(entry[1]), z: Number(entry[2]) }));
  assert(points.length > 0, `source point array ${name} is empty or non-literal`);
  return points;
}

function renderZr(catalog) {
  const {
    constants,
    zones,
    lakes,
    camps,
    terrain_edits: terrainEdits,
    roads,
    docks,
    sowfield_flat: sowfieldFlat,
    sowfield_exclude: sowfieldExclude,
    sowfield_stands: sowfieldStands,
    sowfield_stand_tier_depth: sowfieldStandTierDepth,
    sowfield_stand_tier_heights: sowfieldStandTierHeights,
    sowfield_stand_ramp: sowfieldStandRamp,
    dock_layout: dockLayout,
    decoration_exclusion_radius: decorationExclusionRadius,
    decoration_exclusions: decorationExclusions,
    decoration_max_slope: decorationMaxSlope,
  } = catalog;
  const lines = [
    '// Generated by tools/m3_terrain_content_codegen.mjs from the pinned WOC source.',
    '// Source-pinned builtin WorldContent values used by src/sim/world.ts. This is',
    '// data only: terrain shape and collision resolution remain separate reducers.',
    '// ZrVM represents the source\'s no-water `-Infinity` outcome as an explicit',
    '// hasWaterAt predicate, never as a finite gameplay sentinel.',
    '',
    'zoneIndexIsValid(index: int): bool {',
    `    return index >= 0 && index < ${zones.length};`,
    '}',
    '',
    'lakeIndexIsValid(index: int): bool {',
    `    return index >= 0 && index < ${lakes.length};`,
    '}',
    '',
    'campIndexIsValid(index: int): bool {',
    `    return index >= 0 && index < ${camps.length};`,
    '}',
    '',
    'terrainEditIndexIsValid(index: int): bool {',
    `    return index >= 0 && index < ${terrainEdits.length};`,
    '}',
    '',
    'roadIndexIsValid(index: int): bool {',
    `    return index >= 0 && index < ${roads.length};`,
    '}',
    '',
    'decorationExclusionIndexIsValid(index: int): bool {',
    `    return index >= 0 && index < ${decorationExclusions.length};`,
    '}',
    '',
    'dockIndexIsValid(index: int): bool {',
    `    return index >= 0 && index < ${docks.length};`,
    '}',
    '',
    'sowfieldStandIndexIsValid(index: int): bool {',
    `    return index >= 0 && index < ${sowfieldStands.length};`,
    '}',
    '',
    'sowfieldStandTierIndexIsValid(index: int): bool {',
    `    return index >= 0 && index < ${sowfieldStandTierHeights.length};`,
    '}',
    '',
    'dockSectionIndexIsValid(index: int): bool {',
    `    return index >= 0 && index < ${dockLayout.section_local_z.length};`,
    '}',
    '',
    requiredFloat('waterLevel', 'water level', constants.water_level),
    '',
    requiredFloat('lakeBlendRadiusMultiplier', 'lake blend radius', constants.lake_blend_radius_multiplier),
    '',
    requiredFloat('worldMaxX', 'world maximum x', constants.world_max_x),
    '',
    requiredFloat('worldMinZ', 'world minimum z', constants.world_min_z),
    '',
    requiredFloat('worldMaxZ', 'world maximum z', constants.world_max_z),
    '',
    requiredFloat('dungeonXThreshold', 'dungeon threshold', constants.dungeon_x_threshold),
    '',
    requiredFloat('dungeonFloorY', 'dungeon floor', constants.dungeon_floor_y),
    '',
    requiredFloat('sowfieldMinX', 'Sowfield minimum x', sowfieldFlat.x_min),
    '',
    requiredFloat('sowfieldMaxX', 'Sowfield maximum x', sowfieldFlat.x_max),
    '',
    requiredFloat('sowfieldMinZ', 'Sowfield minimum z', sowfieldFlat.z_min),
    '',
    requiredFloat('sowfieldMaxZ', 'Sowfield maximum z', sowfieldFlat.z_max),
    '',
    requiredFloat('sowfieldHeight', 'Sowfield height', sowfieldFlat.height),
    '',
    requiredFloat('sowfieldFalloff', 'Sowfield falloff', sowfieldFlat.falloff),
    '',
    requiredFloat('sowfieldExcludeMinX', 'Sowfield exclusion minimum x', sowfieldExclude.x_min),
    '',
    requiredFloat('sowfieldExcludeMaxX', 'Sowfield exclusion maximum x', sowfieldExclude.x_max),
    '',
    requiredFloat('sowfieldExcludeMinZ', 'Sowfield exclusion minimum z', sowfieldExclude.z_min),
    '',
    requiredFloat('sowfieldExcludeMaxZ', 'Sowfield exclusion maximum z', sowfieldExclude.z_max),
    '',
    requiredFloat('decorationExclusionRadius', 'decoration exclusion radius', decorationExclusionRadius),
    '',
    requiredFloat('decorationMaxSlope', 'decoration maximum slope', decorationMaxSlope),
    '',
    renderIndexedFloat('sowfieldStandMinX', 'Sowfield stand minimum x', sowfieldStands.map((stand) => stand.x_min), 'sowfieldStandIndexIsValid'),
    '',
    renderIndexedFloat('sowfieldStandMaxX', 'Sowfield stand maximum x', sowfieldStands.map((stand) => stand.x_max), 'sowfieldStandIndexIsValid'),
    '',
    renderIndexedFloat('sowfieldStandMinZ', 'Sowfield stand minimum z', sowfieldStands.map((stand) => stand.z_min), 'sowfieldStandIndexIsValid'),
    '',
    renderIndexedFloat('sowfieldStandMaxZ', 'Sowfield stand maximum z', sowfieldStands.map((stand) => stand.z_max), 'sowfieldStandIndexIsValid'),
    '',
    renderCount('sowfieldStandCount', 'Sowfield stand count', sowfieldStands.length),
    '',
    requiredFloat('sowfieldStandTierDepth', 'Sowfield stand tier depth', sowfieldStandTierDepth),
    '',
    requiredFloat('sowfieldStandRamp', 'Sowfield stand ramp', sowfieldStandRamp),
    '',
    renderIndexedFloat('sowfieldStandTierHeight', 'Sowfield stand tier height', sowfieldStandTierHeights, 'sowfieldStandTierIndexIsValid'),
    '',
    renderCount('sowfieldStandTierCount', 'Sowfield stand tier count', sowfieldStandTierHeights.length),
    '',
    renderIndexedFloat('dockSectionLocalZ', 'dock section local z', dockLayout.section_local_z, 'dockSectionIndexIsValid'),
    '',
    renderCount('dockSectionCount', 'dock section count', dockLayout.section_local_z.length),
    '',
    requiredFloat('dockSectionHalfWidth', 'dock section half width', dockLayout.section_half_width),
    '',
    requiredFloat('dockSectionHalfDepth', 'dock section half depth', dockLayout.section_half_depth),
    '',
    requiredFloat('dockTerrainClearance', 'dock terrain clearance', dockLayout.terrain_clearance),
    '',
    requiredFloat('dockSectionSurfaceY', 'dock section surface y', dockLayout.surface_y),
    '',
    'pub isFlatInstanceX(x: float, required: bool): bool {',
    '    if (!required) {',
    '        throw "woc terrain instance query is required";',
    '    }',
    '    return x > dungeonXThreshold(true);',
    '}',
    '',
    'pub zoneCount(required: bool): int {',
    '    if (!required) {',
    '        throw "woc terrain zone count is required";',
    '    }',
    `    return ${zones.length};`,
    '}',
    '',
    '// Mirrors data.ts zoneAt: points beyond either end clamp to the first or last',
    '// source zone rather than producing an out-of-world biome code.',
    'pub zoneIndexAt(z: float, required: bool): int {',
    '    if (!required) {',
    '        throw "woc terrain zone query is required";',
    '    }',
  ];
  for (let index = 0; index + 1 < zones.length; index++) {
    lines.push(`    if (z < ${formatNumber(zones[index].z_max)}) {`);
    lines.push(`        return ${index};`);
    lines.push('    }');
  }
  lines.push(`    return ${zones.length - 1};`, '}', '');
  lines.push(renderIndexedFloat('zoneZMin', 'zone minimum', zones.map((zone) => zone.z_min), 'zoneIndexIsValid'));
  lines.push('', renderIndexedFloat('zoneZMax', 'zone maximum', zones.map((zone) => zone.z_max), 'zoneIndexIsValid'));
  lines.push('', renderIndexedInt('zoneBiomeCode', 'zone biome', zones.map((zone) => EXPECTED_BIOMES.indexOf(zone.biome) + 1), 'zoneIndexIsValid'));
  lines.push('', renderIndexedFloat('zoneHubX', 'zone hub x', zones.map((zone) => zone.hub.x), 'zoneIndexIsValid'));
  lines.push('', renderIndexedFloat('zoneHubZ', 'zone hub z', zones.map((zone) => zone.hub.z), 'zoneIndexIsValid'));
  lines.push('', renderIndexedFloat('zoneHubRadius', 'zone hub radius', zones.map((zone) => zone.hub.radius), 'zoneIndexIsValid'));
  lines.push('', 'pub lakeCount(required: bool): int {', '    if (!required) {', '        throw "woc terrain lake count is required";', '    }', `    return ${lakes.length};`, '}', '');
  lines.push(renderIndexedFloat('lakeX', 'lake x', lakes.map((lake) => lake.x), 'lakeIndexIsValid'));
  lines.push('', renderIndexedFloat('lakeZ', 'lake z', lakes.map((lake) => lake.z), 'lakeIndexIsValid'));
  lines.push('', renderIndexedFloat('lakeRadius', 'lake radius', lakes.map((lake) => lake.radius), 'lakeIndexIsValid'));
  lines.push('', renderIndexedString('campMobId', 'camp mob identity', camps.map((camp) => camp.mob_id), 'campIndexIsValid'));
  lines.push('', renderIndexedBool('campMobIsDummy', 'camp mob dummy flag', camps.map((camp) => camp.mob_is_dummy), 'campIndexIsValid'));
  lines.push('', renderIndexedInt('campMobMinLevel', 'camp mob minimum level', camps.map((camp) => camp.mob_min_level), 'campIndexIsValid'));
  lines.push('', renderIndexedInt('campMobMaxLevel', 'camp mob maximum level', camps.map((camp) => camp.mob_max_level), 'campIndexIsValid'));
  lines.push('', renderIndexedInt('campMobCount', 'camp mob count', camps.map((camp) => camp.count), 'campIndexIsValid'));
  lines.push('', renderIndexedFloat('campX', 'camp x', camps.map((camp) => camp.x), 'campIndexIsValid'));
  lines.push('', renderIndexedFloat('campZ', 'camp z', camps.map((camp) => camp.z), 'campIndexIsValid'));
  lines.push('', renderIndexedFloat('campRadius', 'camp radius', camps.map((camp) => camp.radius), 'campIndexIsValid'));
  lines.push('', renderCount('campCount', 'camp count', camps.length));
  lines.push('', renderCount('campMobTotal', 'camp mob total', constants.camp_mob_total));
  lines.push('', renderCount('campNonDummyMobCount', 'camp non-dummy mob count', constants.non_dummy_camp_mob_count));
  lines.push('', renderIndexedFloat('terrainEditX', 'terrain edit x', terrainEdits.map((edit) => edit.x), 'terrainEditIndexIsValid'));
  lines.push('', renderIndexedFloat('terrainEditZ', 'terrain edit z', terrainEdits.map((edit) => edit.z), 'terrainEditIndexIsValid'));
  lines.push('', renderIndexedFloat('terrainEditRadius', 'terrain edit radius', terrainEdits.map((edit) => edit.radius), 'terrainEditIndexIsValid'));
  lines.push('', renderIndexedFloat('terrainEditDelta', 'terrain edit delta', terrainEdits.map((edit) => edit.delta), 'terrainEditIndexIsValid'));
  lines.push('', renderIndexedInt('terrainEditFalloffCode', 'terrain edit falloff', terrainEdits.map((edit) => falloffCode(edit.falloff)), 'terrainEditIndexIsValid'));
  lines.push('', renderIndexedInt('terrainEditModeCode', 'terrain edit mode', terrainEdits.map((edit) => modeCode(edit.mode)), 'terrainEditIndexIsValid'));
  lines.push('', renderCount('terrainEditCount', 'terrain edit count', terrainEdits.length));
  lines.push('', renderRoadCount(roads));
  lines.push('', renderRoadPointCount(roads));
  lines.push('', renderRoadPointCoordinate('roadPointX', 'road point x', roads, 'x'));
  lines.push('', renderRoadPointCoordinate('roadPointZ', 'road point z', roads, 'z'));
  lines.push('', renderIndexedFloat('decorationExclusionX', 'decoration exclusion x', decorationExclusions.map((point) => point.x), 'decorationExclusionIndexIsValid'));
  lines.push('', renderIndexedFloat('decorationExclusionZ', 'decoration exclusion z', decorationExclusions.map((point) => point.z), 'decorationExclusionIndexIsValid'));
  lines.push('', renderCount('decorationExclusionCount', 'decoration exclusion count', decorationExclusions.length));
  lines.push('', renderIndexedFloat('dockX', 'dock x', docks.map((dock) => dock.x), 'dockIndexIsValid'));
  lines.push('', renderIndexedFloat('dockZ', 'dock z', docks.map((dock) => dock.z), 'dockIndexIsValid'));
  lines.push('', renderIndexedFloat('dockRotation', 'dock rotation', docks.map((dock) => dock.rotation), 'dockIndexIsValid'));
  lines.push('', renderIndexedFloat('dockHutLocalX', 'dock hut local x', docks.map((dock) => dock.hut_local.x), 'dockIndexIsValid'));
  lines.push('', renderIndexedFloat('dockHutLocalZ', 'dock hut local z', docks.map((dock) => dock.hut_local.z), 'dockIndexIsValid'));
  lines.push('', renderIndexedFloat('dockHutHalfWidth', 'dock hut half width', docks.map((dock) => dock.hut_local.half_width), 'dockIndexIsValid'));
  lines.push('', renderIndexedFloat('dockHutHalfDepth', 'dock hut half depth', docks.map((dock) => dock.hut_local.half_depth), 'dockIndexIsValid'));
  lines.push('', renderCount('dockCount', 'dock count', docks.length));
  lines.push('',
    'pub hasWaterAt(x: float, z: float, required: bool): bool {',
    '    if (!required) {',
    '        throw "woc terrain water query is required";',
    '    }',
    '    var index = 0;',
    '    while (index < lakeCount(true)) {',
    '        var dx = x - lakeX(index, true);',
    '        var dz = z - lakeZ(index, true);',
    '        var radius = lakeRadius(index, true) * lakeBlendRadiusMultiplier(true);',
    '        if (dx * dx + dz * dz < radius * radius) {',
    '            return true;',
    '        }',
    '        index = index + 1;',
    '    }',
    '    return false;',
    '}',
    '',
    renderContractTest(catalog),
    '',
  );
  return lines.join('\n');
}

function requiredFloat(name, label, value) {
  return [
    `pub ${name}(required: bool): float {`,
    '    if (!required) {',
    `        throw "woc terrain ${label} is required";`,
    '    }',
    `    return ${formatNumber(value)};`,
    '}',
  ].join('\n');
}

function renderCount(name, label, value) {
  return [
    `pub ${name}(required: bool): int {`,
    '    if (!required) {',
    `        throw "woc terrain ${label} is required";`,
    '    }',
    `    return ${value};`,
    '}',
  ].join('\n');
}

function renderIndexedFloat(name, label, values, validName) {
  return renderIndexed(name, label, values, validName, formatNumber, 'float');
}

function renderIndexedInt(name, label, values, validName) {
  return renderIndexed(name, label, values, validName, String, 'int');
}

function renderIndexedString(name, label, values, validName) {
  return renderIndexed(name, label, values, validName, JSON.stringify, 'string');
}

function renderIndexedBool(name, label, values, validName) {
  return renderIndexed(name, label, values, validName, (value) => (value ? 'true' : 'false'), 'bool');
}

function renderIndexed(name, label, values, validName, format, type) {
  const lines = [
    `pub ${name}(index: int, required: bool): ${type} {`,
    `    if (!required || !${validName}(index)) {`,
    `        throw "woc terrain ${label} is invalid";`,
    '    }',
  ];
  for (let index = 0; index + 1 < values.length; index++) {
    lines.push(`    if (index == ${index}) {`);
    lines.push(`        return ${format(values[index])};`);
    lines.push('    }');
  }
  lines.push(`    return ${format(values.at(-1))};`, '}');
  return lines.join('\n');
}

function renderRoadCount(roads) {
  return [
    'pub roadCount(required: bool): int {',
    '    if (!required) {',
    '        throw "woc terrain road count is required";',
    '    }',
    `    return ${roads.length};`,
    '}',
  ].join('\n');
}

function renderRoadPointCount(roads) {
  return renderIndexed('roadPointCount', 'road point count', roads.map((road) => road.length), 'roadIndexIsValid', String, 'int');
}

function renderRoadPointCoordinate(name, label, roads, field) {
  const lines = [
    `pub ${name}(roadIndex: int, pointIndex: int, required: bool): float {`,
    '    if (!required || !roadIndexIsValid(roadIndex)) {',
    `        throw "woc terrain ${label} is invalid";`,
    '    }',
  ];
  for (let roadIndex = 0; roadIndex < roads.length; roadIndex++) {
    const road = roads[roadIndex];
    lines.push(`    if (roadIndex == ${roadIndex}) {`);
    lines.push(`        if (pointIndex < 0 || pointIndex >= ${road.length}) {`);
    lines.push(`            throw "woc terrain ${label} is invalid";`);
    lines.push('        }');
    for (let pointIndex = 0; pointIndex + 1 < road.length; pointIndex++) {
      lines.push(`        if (pointIndex == ${pointIndex}) {`);
      lines.push(`            return ${formatNumber(road[pointIndex][field])};`);
      lines.push('        }');
    }
    lines.push(`        return ${formatNumber(road.at(-1)[field])};`);
    lines.push('    }');
  }
  lines.push(`    throw "woc terrain ${label} is invalid";`, '}');
  return lines.join('\n');
}

function renderContractTest(catalog) {
  const {
    constants,
    zones,
    lakes,
    camps,
    terrain_edits: terrainEdits,
    roads,
    docks,
    sowfield_flat: sowfieldFlat,
    sowfield_exclude: sowfieldExclude,
    sowfield_stands: sowfieldStands,
    sowfield_stand_tier_depth: sowfieldStandTierDepth,
    sowfield_stand_tier_heights: sowfieldStandTierHeights,
    sowfield_stand_ramp: sowfieldStandRamp,
    dock_layout: dockLayout,
    decoration_exclusion_radius: decorationExclusionRadius,
    decoration_exclusions: decorationExclusions,
    decoration_max_slope: decorationMaxSlope,
  } = catalog;
  const firstLake = lakes[0];
  const edgeX = firstLake.x + firstLake.radius * constants.lake_blend_radius_multiplier;
  const justInsideX = edgeX - 0.001;
  return [
    'pub contractTest(): int {',
    `    if (waterLevel(true) != ${formatNumber(constants.water_level)} || lakeBlendRadiusMultiplier(true) != ${formatNumber(constants.lake_blend_radius_multiplier)} ||`,
    `        worldMaxX(true) != ${formatNumber(constants.world_max_x)} || worldMinZ(true) != ${formatNumber(constants.world_min_z)} ||`,
    `        worldMaxZ(true) != ${formatNumber(constants.world_max_z)} || dungeonXThreshold(true) != ${formatNumber(constants.dungeon_x_threshold)} ||`,
    `        dungeonFloorY(true) != ${formatNumber(constants.dungeon_floor_y)} || !isFlatInstanceX(${formatNumber(constants.dungeon_x_threshold + 0.1)}, true) ||`,
    `        isFlatInstanceX(${formatNumber(constants.dungeon_x_threshold)}, true)) {`,
    '        return -1;',
    '    }',
    `    if (zoneCount(true) != ${zones.length} || zoneIndexAt(-999.0, true) != 0 ||`,
    `        zoneIndexAt(${formatNumber(zones[0].z_max - 0.001)}, true) != 0 || zoneIndexAt(${formatNumber(zones[0].z_max)}, true) != 1 ||`,
    `        zoneIndexAt(${formatNumber(zones[1].z_max - 0.001)}, true) != 1 || zoneIndexAt(${formatNumber(zones[1].z_max)}, true) != 2 ||`,
    `        zoneIndexAt(999.0, true) != ${zones.length - 1} || zoneZMin(0, true) != ${formatNumber(zones[0].z_min)} ||`,
    `        zoneZMax(${zones.length - 1}, true) != ${formatNumber(zones.at(-1).z_max)} || zoneBiomeCode(0, true) != 1 ||`,
    `        zoneBiomeCode(1, true) != 2 || zoneBiomeCode(2, true) != 3 ||`,
    `        zoneHubRadius(0, true) != ${formatNumber(zones[0].hub.radius)} || zoneHubZ(1, true) != ${formatNumber(zones[1].hub.z)} ||`,
    `        zoneHubZ(2, true) != ${formatNumber(zones[2].hub.z)}) {`,
    '        return -2;',
    '    }',
    `    if (lakeCount(true) != ${lakes.length} || lakeX(0, true) != ${formatNumber(lakes[0].x)} ||`,
    `        lakeZ(1, true) != ${formatNumber(lakes[1].z)} || lakeRadius(2, true) != ${formatNumber(lakes[2].radius)} ||`,
    `        lakeX(${lakes.length - 1}, true) != ${formatNumber(lakes.at(-1).x)} || lakeZ(${lakes.length - 1}, true) != ${formatNumber(lakes.at(-1).z)} ||`,
    `        lakeRadius(${lakes.length - 1}, true) != ${formatNumber(lakes.at(-1).radius)}) {`,
    '        return -3;',
    '    }',
    `    if (!hasWaterAt(${formatNumber(firstLake.x)}, ${formatNumber(firstLake.z)}, true) || hasWaterAt(${formatNumber(edgeX)}, ${formatNumber(firstLake.z)}, true) ||`,
    `        !hasWaterAt(${formatNumber(justInsideX)}, ${formatNumber(firstLake.z)}, true) || !hasWaterAt(${formatNumber(lakes[2].x)}, ${formatNumber(lakes[2].z)}, true) ||`,
    '        hasWaterAt(0.0, 0.0, true)) {',
    '        return -4;',
    '    }',
    ...renderCampRosterContractCheck(camps, constants),
    `    if (terrainEditCount(true) != ${terrainEdits.length} || terrainEditX(0, true) != ${formatNumber(terrainEdits[0].x)} ||`,
    `        terrainEditZ(0, true) != ${formatNumber(terrainEdits[0].z)} || terrainEditRadius(0, true) != ${formatNumber(terrainEdits[0].radius)} ||`,
    `        terrainEditDelta(0, true) != ${formatNumber(terrainEdits[0].delta)} || terrainEditFalloffCode(0, true) != ${falloffCode(terrainEdits[0].falloff)} ||`,
    `        terrainEditModeCode(0, true) != ${modeCode(terrainEdits[0].mode)}) {`,
    '        return -6;',
    '    }',
    `    if (dockCount(true) != ${docks.length} || dockX(0, true) != ${formatNumber(docks[0].x)} ||`,
    `        dockZ(1, true) != ${formatNumber(docks[1].z)} || dockRotation(0, true) != ${formatNumber(docks[0].rotation)} ||`,
    `        dockHutLocalX(1, true) != ${formatNumber(docks[1].hut_local.x)} ||`,
    `        dockHutHalfDepth(1, true) != ${formatNumber(docks[1].hut_local.half_depth)}) {`,
    '        return -7;',
    '    }',
    `    if (sowfieldMinX(true) != ${formatNumber(sowfieldFlat.x_min)} || sowfieldMaxX(true) != ${formatNumber(sowfieldFlat.x_max)} ||`,
    `        sowfieldMinZ(true) != ${formatNumber(sowfieldFlat.z_min)} || sowfieldMaxZ(true) != ${formatNumber(sowfieldFlat.z_max)} ||`,
    `        sowfieldHeight(true) != ${formatNumber(sowfieldFlat.height)} || sowfieldFalloff(true) != ${formatNumber(sowfieldFlat.falloff)}) {`,
    '        return -8;',
    '    }',
    `    if (sowfieldStandCount(true) != ${sowfieldStands.length} || sowfieldStandMinX(0, true) != ${formatNumber(sowfieldStands[0].x_min)} ||`,
    `        sowfieldStandMaxZ(1, true) != ${formatNumber(sowfieldStands[1].z_max)} || sowfieldStandTierDepth(true) != ${formatNumber(sowfieldStandTierDepth)} ||`,
    `        sowfieldStandRamp(true) != ${formatNumber(sowfieldStandRamp)} ||`,
    `        sowfieldStandTierCount(true) != ${sowfieldStandTierHeights.length} || sowfieldStandTierHeight(1, true) != ${formatNumber(sowfieldStandTierHeights[1])}) {`,
    '        return -9;',
    '    }',
    `    if (dockSectionCount(true) != ${dockLayout.section_local_z.length} || dockSectionLocalZ(0, true) != ${formatNumber(dockLayout.section_local_z[0])} ||`,
    `        dockSectionLocalZ(${dockLayout.section_local_z.length - 1}, true) != ${formatNumber(dockLayout.section_local_z.at(-1))} ||`,
    `        dockSectionHalfWidth(true) != ${formatNumber(dockLayout.section_half_width)} || dockSectionHalfDepth(true) != ${formatNumber(dockLayout.section_half_depth)} ||`,
    `        dockTerrainClearance(true) != ${formatNumber(dockLayout.terrain_clearance)} || dockSectionSurfaceY(true) != ${formatNumber(dockLayout.surface_y)}) {`,
    '        return -10;',
    '    }',
    `    if (roadCount(true) != ${roads.length} || roadPointCount(0, true) != ${roads[0].length} ||`,
    `        roadPointX(0, 0, true) != ${formatNumber(roads[0][0].x)} || roadPointZ(0, 0, true) != ${formatNumber(roads[0][0].z)} ||`,
    `        roadPointX(${roads.length - 1}, ${roads.at(-1).length - 1}, true) != ${formatNumber(roads.at(-1).at(-1).x)} ||`,
    `        roadPointZ(${roads.length - 1}, ${roads.at(-1).length - 1}, true) != ${formatNumber(roads.at(-1).at(-1).z)}) {`,
    '        return -11;',
    '    }',
    `    if (sowfieldExcludeMinX(true) != ${formatNumber(sowfieldExclude.x_min)} || sowfieldExcludeMaxX(true) != ${formatNumber(sowfieldExclude.x_max)} ||`,
    `        sowfieldExcludeMinZ(true) != ${formatNumber(sowfieldExclude.z_min)} || sowfieldExcludeMaxZ(true) != ${formatNumber(sowfieldExclude.z_max)}) {`,
    '        return -12;',
    '    }',
    `    if (decorationExclusionRadius(true) != ${formatNumber(decorationExclusionRadius)} || decorationMaxSlope(true) != ${formatNumber(decorationMaxSlope)} ||`,
    `        decorationExclusionCount(true) != ${decorationExclusions.length} || decorationExclusionX(0, true) != ${formatNumber(decorationExclusions[0].x)} ||`,
    `        decorationExclusionZ(0, true) != ${formatNumber(decorationExclusions[0].z)}) {`,
    '        return -13;',
    '    }',
    '    return 1;',
    '}',
  ].join('\n');
}

function renderCampRosterContractCheck(camps, constants) {
  const lines = [
    `    if (campCount(true) != ${camps.length} || campMobTotal(true) != ${constants.camp_mob_total} ||`,
    `        campNonDummyMobCount(true) != ${constants.non_dummy_camp_mob_count} ||`,
  ];
  for (let index = 0; index < camps.length; index++) {
    const camp = camps[index];
    const suffix = index + 1 < camps.length ? ' ||' : ') {';
    lines.push(
      `        campMobId(${index}, true) != ${JSON.stringify(camp.mob_id)} || campMobIsDummy(${index}, true) != ${camp.mob_is_dummy ? 'true' : 'false'} ||`,
      `        campMobMinLevel(${index}, true) != ${camp.mob_min_level} || campMobMaxLevel(${index}, true) != ${camp.mob_max_level} ||`,
      `        campMobCount(${index}, true) != ${camp.count} ||`,
      `        campX(${index}, true) != ${formatNumber(camp.x)} || campZ(${index}, true) != ${formatNumber(camp.z)} ||`,
      `        campRadius(${index}, true) != ${formatNumber(camp.radius)}${suffix}`,
    );
  }
  lines.push('        return -5;', '    }');
  return lines;
}

function falloffCode(value) {
  if (value === 'flat') return 1;
  if (value === 'smooth') return 2;
  throw new Error(`unknown terrain edit falloff ${value}`);
}

function modeCode(value) {
  if (value === 'level') return 1;
  if (value === 'add') return 2;
  throw new Error(`unknown terrain edit mode ${value}`);
}

function verifyOrWrite(path, text) {
  if (checkOnly) {
    assert(existsSync(path), `generated output is missing: ${path}`);
    assert(readFileSync(path, 'utf8') === text, `generated output drifted: ${path}`);
    return;
  }
  writeFileSync(path, text);
}

function formatNumber(value) {
  assert(Number.isFinite(value), `cannot emit non-finite number ${value}`);
  if (Number.isInteger(value)) return `${value}.0`;
  return value.toString();
}

function sha256(text) {
  return createHash('sha256').update(text).digest('hex');
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}
