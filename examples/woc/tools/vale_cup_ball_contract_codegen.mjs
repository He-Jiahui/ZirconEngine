import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const BALL_PATH = 'src/sim/vale_cup_ball.ts';
const LAYOUT_PATH = 'src/sim/vale_cup_layout.ts';
const TYPES_PATH = 'src/sim/types.ts';
const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourceRoot = resolve(projectRoot, '..', '..', 'dev', 'world-of-claudecraft');
const jsonOutput = join(projectRoot, 'reference', 'current-head', 'vale_cup_ball_contract.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'generated', 'vale_cup_ball_contract.zr');
const checkOnly = process.argv.includes('--check');

main();

function main() {
  const blobs = Object.fromEntries(
    [BALL_PATH, LAYOUT_PATH, TYPES_PATH].map((path) => [path, sourceBlob(path)]),
  );
  const ball = blobs[BALL_PATH].toString('utf8');
  const layout = blobs[LAYOUT_PATH].toString('utf8');
  const types = blobs[TYPES_PATH].toString('utf8');
  const tickRate = numberLiteral(types, 'TICK_RATE', 'tick rate');
  invariant(types.includes('export const DT = 1 / TICK_RATE;'), 'Vale Cup ball DT definition drifted');

  const physics = Object.fromEntries(
    [
      ['gravity', 'VC_BALL_GRAVITY'],
      ['radius', 'VC_BALL_RADIUS'],
      ['max_speed', 'VC_BALL_MAX_SPEED'],
      ['ground_restitution', 'VC_BALL_GROUND_RESTITUTION'],
      ['wall_restitution', 'VC_BALL_WALL_RESTITUTION'],
      ['roll_decel', 'VC_BALL_ROLL_DECEL'],
      ['slow_decel', 'VC_BALL_SLOW_DECEL'],
      ['slow_speed', 'VC_BALL_SLOW_SPEED'],
      ['bounce_min_vy', 'VC_BALL_BOUNCE_MIN_VY'],
      ['pocket_decel', 'VC_BALL_POCKET_DECEL'],
      ['trap_min_speed', 'VC_TRAP_MIN_BALL_SPEED'],
      ['trap_roll_speed', 'VC_TRAP_ROLL_SPEED'],
      ['trap_vy_damp', 'VC_TRAP_VY_DAMP'],
      ['dribble_speed_mult', 'VC_DRIBBLE_SPEED_MULT'],
      ['dribble_min_mover_speed', 'VC_DRIBBLE_MIN_MOVER_SPEED'],
    ].map(([key, name]) => [key, numberLiteral(ball, name, name)]),
  );

  for (const needle of [
    'function capSpeed(b: VcBallKinematics): void',
    'function applyRollFriction(b: VcBallKinematics, decel: number): void',
    'function integrateVertical(b: VcBallKinematics, groundY: number): void',
    'function reflectOffWall(b: VcBallKinematics, w: VcWallSegment): boolean',
    'export function stepBallPhysics(b: VcBallKinematics, groundY: number):',
    'export function settleBallInPocket(',
    'export function applyDribbleNudge(',
    'export function applyBodyTrap(',
    'export function launchBall(',
    'for (const w of PITCH_WALLS) reflectOffWall(b, w);',
  ]) {
    invariant(ball.includes(needle), `Vale Cup ball source behavior drifted: ${needle}`);
  }

  const pitch = objectLiteral(layout, 'export const PITCH', 'pitch');
  const center = objectLiteral(layout, 'export const PITCH_CENTER', 'pitch center');
  const sowfieldFlat = objectLiteral(layout, 'export const SOWFIELD_FLAT', 'Sowfield flatten');
  const sowfieldExclude = objectLiteral(layout, 'export const SOWFIELD_EXCLUDE', 'Sowfield shell');
  const goalHalfWidth = numberLiteral(layout, 'GOAL_HALF_W', 'goal half width');
  const goalDepth = numberLiteral(layout, 'GOAL_DEPTH', 'goal depth');
  const goalHeight = numberLiteral(layout, 'GOAL_HEIGHT', 'goal height');
  const practiceSlots = numberLiteral(layout, 'VC_PRACTICE_SLOTS', 'practice slot count');
  const practiceBaseX = numberLiteral(layout, 'VC_PRACTICE_BASE_X', 'practice base x');
  const practiceSlotDz = numberLiteral(layout, 'VC_PRACTICE_SLOT_DZ', 'practice slot spacing');
  invariant(
    layout.includes('export const GOAL_Z_MIN = PITCH_CENTER.z - GOAL_HALF_W;') &&
      layout.includes('export const GOAL_Z_MAX = PITCH_CENTER.z + GOAL_HALF_W;') &&
      layout.includes('export const GOAL_LINE_WEST_X = PITCH.xMin;') &&
      layout.includes('export const GOAL_LINE_EAST_X = PITCH.xMax;'),
    'Vale Cup goal layout derivation drifted',
  );
  for (const needle of [
    '{ x1: PITCH.xMin, z1: PITCH.zMax, x2: PITCH.xMax, z2: PITCH.zMax, nx: 0, nz: -1 }',
    '{ x1: PITCH.xMin, z1: PITCH.zMin, x2: PITCH.xMax, z2: PITCH.zMin, nx: 0, nz: 1 }',
    '{ x1: PITCH.xMin, z1: PITCH.zMin, x2: PITCH.xMin, z2: GOAL_Z_MIN, nx: 1, nz: 0 }',
    '{ x1: PITCH.xMin, z1: GOAL_Z_MAX, x2: PITCH.xMin, z2: PITCH.zMax, nx: 1, nz: 0 }',
    '{ x1: PITCH.xMax, z1: PITCH.zMin, x2: PITCH.xMax, z2: GOAL_Z_MIN, nx: -1, nz: 0 }',
    '{ x1: PITCH.xMax, z1: GOAL_Z_MAX, x2: PITCH.xMax, z2: PITCH.zMax, nx: -1, nz: 0 }',
  ]) {
    invariant(layout.includes(needle), `Vale Cup wall layout drifted: ${needle}`);
  }

  invariant(
    layout.includes('export function vcPracticeOrigin(slot: number): { x: number; z: number } {') &&
      layout.includes('return { x: VC_PRACTICE_BASE_X, z: slot * VC_PRACTICE_SLOT_DZ };'),
    'Vale Cup practice origin layout drifted',
  );

  const goals = {
    west_x: pitch.xMin,
    east_x: pitch.xMax,
    z_min: center.z - goalHalfWidth,
    z_max: center.z + goalHalfWidth,
    depth: goalDepth,
    height: goalHeight,
  };
  const walls = [
    [pitch.xMin, pitch.zMax, pitch.xMax, pitch.zMax, 0, -1],
    [pitch.xMin, pitch.zMin, pitch.xMax, pitch.zMin, 0, 1],
    [pitch.xMin, pitch.zMin, pitch.xMin, goals.z_min, 1, 0],
    [pitch.xMin, goals.z_max, pitch.xMin, pitch.zMax, 1, 0],
    [pitch.xMax, pitch.zMin, pitch.xMax, goals.z_min, -1, 0],
    [pitch.xMax, goals.z_max, pitch.xMax, pitch.zMax, -1, 0],
  ];
  const document = {
    schema_version: 2,
    source_commit: SOURCE_COMMIT,
    generated_by: 'examples/woc/tools/vale_cup_ball_contract_codegen.mjs',
    source_blobs: Object.fromEntries(
      Object.entries(blobs).map(([path, value]) => [path, sha256(value)]),
    ),
    simulation: { tick_rate: tickRate, dt: 1 / tickRate },
    physics,
    goals,
    layout: {
      pitch: {
        min_x: pitch.xMin,
        max_x: pitch.xMax,
        min_z: pitch.zMin,
        max_z: pitch.zMax,
      },
      sowfield_flat: sowfieldFlat,
      sowfield_exclude: sowfieldExclude,
      practice: {
        slots: practiceSlots,
        base_x: practiceBaseX,
        slot_dz: practiceSlotDz,
      },
    },
    walls,
    epsilon: { ground: 1e-3, direction: 1e-6, goal_crossing: 1e-9 },
  };
  writeOrCheck(jsonOutput, `${JSON.stringify(document, null, 2)}\n`, 'Vale Cup ball JSON contract');
  writeOrCheck(zrOutput, renderZr(document), 'Vale Cup ball Zr contract');
  process.stdout.write(`${checkOnly ? 'checked' : 'generated'} Vale Cup ball contract for ${SOURCE_COMMIT}\n`);
}

function renderZr(document) {
  const scalar = (name, value) =>
    `pub ${name}(required: bool): float { return required ? ${zrFloat(value)} : 0.0; }\n`;
  const lines = [
    `// Generated from ${SOURCE_COMMIT}; do not edit by hand.\n`,
    scalar('dt', document.simulation.dt),
    ...Object.entries(document.physics).map(([name, value]) => scalar(name, value)),
    scalar('goalWestX', document.goals.west_x),
    scalar('goalEastX', document.goals.east_x),
    scalar('goalZMin', document.goals.z_min),
    scalar('goalZMax', document.goals.z_max),
    scalar('goalDepth', document.goals.depth),
    scalar('goalHeight', document.goals.height),
    scalar('pitchMinX', document.layout.pitch.min_x),
    scalar('pitchMaxX', document.layout.pitch.max_x),
    scalar('pitchMinZ', document.layout.pitch.min_z),
    scalar('pitchMaxZ', document.layout.pitch.max_z),
    scalar('sowfieldFlatMinX', document.layout.sowfield_flat.xMin),
    scalar('sowfieldFlatMaxX', document.layout.sowfield_flat.xMax),
    scalar('sowfieldFlatMinZ', document.layout.sowfield_flat.zMin),
    scalar('sowfieldFlatMaxZ', document.layout.sowfield_flat.zMax),
    scalar('sowfieldExcludeMinX', document.layout.sowfield_exclude.xMin),
    scalar('sowfieldExcludeMaxX', document.layout.sowfield_exclude.xMax),
    scalar('sowfieldExcludeMinZ', document.layout.sowfield_exclude.zMin),
    scalar('sowfieldExcludeMaxZ', document.layout.sowfield_exclude.zMax),
    scalar('practiceSlots', document.layout.practice.slots),
    scalar('practiceBaseX', document.layout.practice.base_x),
    scalar('practiceSlotDz', document.layout.practice.slot_dz),
    scalar('groundEpsilon', document.epsilon.ground),
    scalar('directionEpsilon', document.epsilon.direction),
    scalar('goalCrossingEpsilon', document.epsilon.goal_crossing),
    `pub wallCount(required: bool): int { return required ? ${document.walls.length} : 0; }\n`,
  ];
  for (const [name, offset] of [
    ['wallX1', 0],
    ['wallZ1', 1],
    ['wallX2', 2],
    ['wallZ2', 3],
    ['wallNx', 4],
    ['wallNz', 5],
  ]) {
    lines.push(renderWallSelector(name, offset, document.walls));
  }
  return lines.join('');
}

function renderWallSelector(name, offset, walls) {
  let output = `pub ${name}(index: int): float {\n`;
  walls.forEach((wall, index) => {
    output += `    if (index == ${index}) return ${zrFloat(wall[offset])};\n`;
  });
  return output + '    return 0.0;\n}\n';
}

function zrFloat(value) {
  return Number.isInteger(value) ? `${value}.0` : String(value);
}

function numberLiteral(source, name, label) {
  const expression = new RegExp(`(?:export\\s+)?const\\s+${name}\\s*=\\s*(-?(?:\\d+(?:\\.\\d+)?|\\.\\d+))\\s*;`);
  const match = source.match(expression);
  invariant(match, `${label} is no longer a literal contract`);
  return Number(match[1]);
}

function objectLiteral(source, declaration, label) {
  const match = source.match(new RegExp(`${declaration}\\s*=\\s*\\{([^}]+)\\};`));
  invariant(match, `${label} is no longer a flat literal`);
  const values = {};
  for (const entry of match[1].split(',')) {
    const normalized = entry.replace(/\/\/[\s\S]*$/, '').trim();
    if (normalized.length === 0) continue;
    const pair = normalized.match(/^(\w+):\s*(-?(?:\d+(?:\.\d+)?|\.\d+))$/);
    invariant(pair, `${label} field is not numeric: ${entry.trim()}`);
    values[pair[1]] = Number(pair[2]);
  }
  return values;
}

function sourceBlob(path) {
  return execFileSync('git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${path}`], {
    encoding: 'buffer',
    maxBuffer: 64 * 1024 * 1024,
  });
}

function writeOrCheck(path, output, label) {
  if (checkOnly) {
    invariant(existsSync(path), `${label} is missing; run npm run generate:vale-cup-ball-contract`);
    invariant(readFileSync(path, 'utf8') === output, `${label} is stale; run npm run generate:vale-cup-ball-contract`);
    return;
  }
  writeFileSync(path, output, 'utf8');
}

function sha256(value) {
  return createHash('sha256').update(value).digest('hex');
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
