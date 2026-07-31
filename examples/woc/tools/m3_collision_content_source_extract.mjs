const data = await import('wocgit:///src/sim/data.ts');
const valeCup = await import('wocgit:///src/sim/vale_cup_layout.ts');

const props = data.BUILTIN_WORLD.props;
const colliders = [];
const fenceSegments = [];

for (const building of props.buildings) {
  colliders.push(obb(building.x, building.z, building.w / 2, building.d / 2, building.rot));
}
for (const well of props.wells) colliders.push(circle(well.x, well.z, well.r));
for (const stall of props.stalls) colliders.push(circle(stall.x, stall.z, stall.r));
for (const mine of props.mines) {
  const mound = rotY(0, -3.4, mine.rot);
  colliders.push(circle(mine.x + mound.x, mine.z + mound.z, 5));
}
for (const dock of props.docks) {
  const hut = rotY(dock.hutLocal.x, dock.hutLocal.z, dock.rot);
  colliders.push(obb(
    dock.x + hut.x,
    dock.z + hut.z,
    dock.hutLocal.hw,
    dock.hutLocal.hd,
    dock.rot,
  ));
}
for (const tent of props.tents) colliders.push(circle(tent.x, tent.z, 1.5 * tent.scale));
for (const [x, z] of props.crates) colliders.push(circle(x, z, 0.65));
for (const [x, z] of props.campfires) colliders.push(circle(x, z, 0.85));
for (const [x, z] of props.mudHuts) colliders.push(circle(x, z, 1.1));
for (const ruin of props.ruinRings) {
  for (let index = 0; index < ruin.columns; index++) {
    const angle = (index / ruin.columns) * Math.PI * 2;
    colliders.push(circle(
      ruin.x + Math.sin(angle) * ruin.ringR,
      ruin.z + Math.cos(angle) * ruin.ringR,
      0.6,
    ));
  }
}
for (const fence of props.fences) {
  const dx = fence.x2 - fence.x1;
  const dz = fence.z2 - fence.z1;
  const length = Math.hypot(dx, dz);
  if (length < 1e-6) continue;
  fenceSegments.push({ x1: fence.x1, z1: fence.z1, x2: fence.x2, z2: fence.z2 });
  colliders.push(obb(
    (fence.x1 + fence.x2) / 2,
    (fence.z1 + fence.z2) / 2,
    length / 2 + 0.35,
    0.35,
    Math.atan2(-dz, dx),
    true,
  ));
}
for (const collider of valeCup.valeCupColliders()) {
  if (collider.type === 'circle') colliders.push(circle(collider.x, collider.z, collider.r));
  else colliders.push(obb(
    collider.x,
    collider.z,
    collider.hw,
    collider.hd,
    collider.rot,
    collider.isFence === true,
  ));
}

process.stdout.write(JSON.stringify({ colliders, fence_segments: fenceSegments }));

function circle(x, z, r) {
  return { kind: 'circle', x, z, radius: r, half_width: 0, half_depth: 0, rotation: 0, is_fence: false };
}

function obb(x, z, halfWidth, halfDepth, rotation, isFence = false) {
  return {
    kind: 'obb',
    x,
    z,
    radius: 0,
    half_width: halfWidth,
    half_depth: halfDepth,
    rotation,
    is_fence: isFence,
  };
}

function rotY(localX, localZ, rotation) {
  const cosine = Math.cos(rotation);
  const sine = Math.sin(rotation);
  return { x: localX * cosine + localZ * sine, z: -localX * sine + localZ * cosine };
}
