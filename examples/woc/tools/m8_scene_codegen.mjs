import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const toolDir = dirname(fileURLToPath(import.meta.url));
const wocRoot = resolve(toolDir, "..");
const contractPath = resolve(wocRoot, "contracts", "m8_eastbrook_scene.json");
const assetManifestPath = resolve(wocRoot, "contracts", "m8_assets.json");
const generatedManifestPath = resolve(
  wocRoot,
  "contracts",
  "m8_eastbrook_scene.generated.json",
);
const scenePath = resolve(wocRoot, "assets", "scenes", "eastbrook_mvp.scene.toml");
const groundPath = resolve(wocRoot, "assets", "models", "eastbrook_ground.model.toml");
const checkOnly = process.argv.includes("--check");
const ZERO_GUID = "00000000-0000-0000-0000-000000000000";

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function parseGlb(bytes, assetPath) {
  assert(bytes.length >= 20, `${assetPath} is too short to be a GLB`);
  assert(bytes.toString("ascii", 0, 4) === "glTF", `${assetPath} has no GLB magic`);
  assert(bytes.readUInt32LE(4) === 2, `${assetPath} is not GLB version 2`);
  const jsonLength = bytes.readUInt32LE(12);
  const jsonType = bytes.readUInt32LE(16);
  assert(jsonType === 0x4e4f534a, `${assetPath} does not begin with a JSON chunk`);
  assert(20 + jsonLength <= bytes.length, `${assetPath} JSON chunk escapes the file`);
  return JSON.parse(bytes.subarray(20, 20 + jsonLength).toString("utf8").trimEnd());
}

function identityMatrix() {
  return [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1];
}

function multiplyMatrices(a, b) {
  const out = Array(16).fill(0);
  for (let column = 0; column < 4; column++) {
    for (let row = 0; row < 4; row++) {
      for (let k = 0; k < 4; k++) {
        out[column * 4 + row] += a[k * 4 + row] * b[column * 4 + k];
      }
    }
  }
  return out;
}

function matrixFromTrs(translation, rotation, scale) {
  const [x, y, z, w] = rotation;
  const [sx, sy, sz] = scale;
  const xx = x * x;
  const yy = y * y;
  const zz = z * z;
  const xy = x * y;
  const xz = x * z;
  const yz = y * z;
  const wx = w * x;
  const wy = w * y;
  const wz = w * z;
  return [
    (1 - 2 * (yy + zz)) * sx,
    (2 * (xy + wz)) * sx,
    (2 * (xz - wy)) * sx,
    0,
    (2 * (xy - wz)) * sy,
    (1 - 2 * (xx + zz)) * sy,
    (2 * (yz + wx)) * sy,
    0,
    (2 * (xz + wy)) * sz,
    (2 * (yz - wx)) * sz,
    (1 - 2 * (xx + yy)) * sz,
    0,
    translation[0],
    translation[1],
    translation[2],
    1,
  ];
}

function nodeMatrix(node) {
  if (node.matrix) return [...node.matrix];
  return matrixFromTrs(
    node.translation ?? [0, 0, 0],
    node.rotation ?? [0, 0, 0, 1],
    node.scale ?? [1, 1, 1],
  );
}

function transformPoint(matrix, point) {
  const [x, y, z] = point;
  return [
    matrix[0] * x + matrix[4] * y + matrix[8] * z + matrix[12],
    matrix[1] * x + matrix[5] * y + matrix[9] * z + matrix[13],
    matrix[2] * x + matrix[6] * y + matrix[10] * z + matrix[14],
  ];
}

function yawQuaternion(yaw) {
  return [0, Math.sin(yaw / 2), 0, Math.cos(yaw / 2)];
}

function sceneNodeGraph(document) {
  const sceneIndex = document.scene ?? 0;
  const scene = document.scenes?.[sceneIndex];
  assert(scene, `GLB scene ${sceneIndex} is missing`);
  const selected = new Set();
  const parent = new Map();
  function visit(index, parentIndex) {
    assert(document.nodes?.[index], `GLB node ${index} is missing`);
    if (selected.has(index)) return;
    selected.add(index);
    if (parentIndex !== null) parent.set(index, parentIndex);
    for (const child of document.nodes[index].children ?? []) visit(child, index);
  }
  for (const root of scene.nodes ?? []) visit(root, null);
  return { roots: [...(scene.nodes ?? [])], selected, parent };
}

function normalizedAccessorComponent(value, componentType) {
  if (componentType === 5120) return Math.max(value / 127, -1);
  if (componentType === 5121) return value / 255;
  if (componentType === 5122) return Math.max(value / 32767, -1);
  if (componentType === 5123) return value / 65535;
  if (componentType === 5125) return value / 4294967295;
  throw new Error(`unsupported normalized accessor component type ${componentType}`);
}

function accessorBounds(accessor) {
  if (!accessor.normalized) return { min: accessor.min, max: accessor.max };
  return {
    min: accessor.min.map((value) => normalizedAccessorComponent(value, accessor.componentType)),
    max: accessor.max.map((value) => normalizedAccessorComponent(value, accessor.componentType)),
  };
}

function modelBounds(document, graph, preYaw, hiddenNodes = []) {
  const hidden = new Set(hiddenNodes);
  const worlds = new Map();
  function worldFor(index) {
    if (worlds.has(index)) return worlds.get(index);
    const local = nodeMatrix(document.nodes[index]);
    const parent = graph.parent.get(index);
    const world = parent === undefined ? local : multiplyMatrices(worldFor(parent), local);
    worlds.set(index, world);
    return world;
  }
  const pre = matrixFromTrs([0, 0, 0], yawQuaternion(preYaw), [1, 1, 1]);
  const min = [Infinity, Infinity, Infinity];
  const max = [-Infinity, -Infinity, -Infinity];
  for (const index of graph.selected) {
    const node = document.nodes[index];
    if (hidden.has(node.name)) continue;
    if (node.mesh === undefined) continue;
    const mesh = document.meshes?.[node.mesh];
    assert(mesh, `GLB mesh ${node.mesh} is missing`);
    const transform = multiplyMatrices(pre, worldFor(index));
    for (const primitive of mesh.primitives ?? []) {
      const accessorIndex = primitive.attributes?.POSITION;
      const accessor = document.accessors?.[accessorIndex];
      assert(accessor?.min && accessor?.max, `mesh ${node.mesh} POSITION bounds are missing`);
      const accessorBox = accessorBounds(accessor);
      for (const x of [accessorBox.min[0], accessorBox.max[0]]) {
        for (const y of [accessorBox.min[1], accessorBox.max[1]]) {
          for (const z of [accessorBox.min[2], accessorBox.max[2]]) {
            const point = transformPoint(transform, [x, y, z]);
            for (let axis = 0; axis < 3; axis++) {
              min[axis] = Math.min(min[axis], point[axis]);
              max[axis] = Math.max(max[axis], point[axis]);
            }
          }
        }
      }
    }
  }
  assert(min.every(Number.isFinite), "GLB scene has no bounded visible mesh");
  return { min, max, size: max.map((value, index) => value - min[index]) };
}

function hash2(x, y, seed) {
  let hash = seed >>> 0;
  hash = Math.imul(hash ^ (x * 374761393), 668265263);
  hash = Math.imul(hash ^ (y * 1274126177), 461845907);
  hash ^= hash >>> 13;
  hash = Math.imul(hash, 1274126177);
  hash ^= hash >>> 16;
  return (hash >>> 0) / 4294967296;
}

function propRand(x, z, stream) {
  return hash2(Math.round(x * 37), Math.round(z * 37) + stream * 7919, 0x517cc1);
}

function keyRand(key, stream) {
  return hash2(Math.round(key * 97), stream * 7919, 0x9e3779);
}

function resolvedYaw(placement) {
  const base = placement.yaw ?? 0;
  const rule = placement.yaw_rule;
  if (!rule) return base;
  const [x, , z] = placement.translation;
  if (rule.kind === "prop_rand") {
    return base + propRand(x, z, rule.stream) * rule.multiplier;
  }
  if (rule.kind === "key_rand_jitter") {
    const key = x * 7.7 + z * 2.3;
    return base + (keyRand(key, rule.stream) - 0.5) * rule.amplitude;
  }
  throw new Error(`unknown yaw rule ${rule.kind}`);
}

function expandFenceRuns(contract) {
  const placements = [];
  for (const run of contract.fence_runs) {
    const [x1, z1] = run.from;
    const [x2, z2] = run.to;
    const length = Math.hypot(x2 - x1, z2 - z1);
    const count = Math.max(1, Math.round(length / 2.35));
    const directionX = (x2 - x1) / length;
    const directionZ = (z2 - z1) / length;
    const yaw = Math.atan2(-directionZ, directionX);
    for (let index = 0; index < count; index++) {
      const ax = x1 + (x2 - x1) * (index / count);
      const az = z1 + (z2 - z1) * (index / count);
      const bx = x1 + (x2 - x1) * ((index + 1) / count);
      const bz = z1 + (z2 - z1) * ((index + 1) / count);
      const x = (ax + bx) / 2;
      const z = (az + bz) / 2;
      placements.push({
        id: `${run.id}_${index + 1}`,
        asset_path: "assets/m8/models/props/fence.glb",
        translation: [x, -0.05, z],
        yaw,
        pre_yaw: 0,
        direct_scale: [3, 2.9 + (propRand(x, z, 1) - 0.5) * 0.5, 3],
        source: `ZONE1_PROPS.fences:${run.id}`,
      });
    }
  }
  return placements;
}

function fmtNumber(value) {
  assert(Number.isFinite(value), `cannot format non-finite number ${value}`);
  const clean = Math.abs(value) < 0.0000005 ? 0 : value;
  if (Number.isInteger(clean)) return `${clean}.0`;
  return clean.toFixed(7).replace(/0+$/, "").replace(/\.$/, ".0");
}

function fmtArray(values) {
  return `[${values.map(fmtNumber).join(", ")}]`;
}

function quote(value) {
  return JSON.stringify(value);
}

function projectRef(assetPath, sub) {
  const fields = [
    'kind = "project"',
    `guid = "${ZERO_GUID}"`,
    `path_hint = ${quote(assetPath)}`,
  ];
  if (sub !== undefined) fields.push(`sub = ${quote(sub)}`);
  return `{ ${fields.join(", ")} }`;
}

function builtinRef(locator) {
  return `{ kind = "builtin", locator = ${quote(locator)} }`;
}

function emitBaseEntity(lines, entity) {
  lines.push("[[entities]]");
  lines.push(`entity = ${entity.entity}`);
  lines.push(`name = ${quote(entity.name)}`);
  if (entity.parent !== undefined) lines.push(`parent = ${entity.parent}`);
  lines.push(`active = ${entity.active ?? true}`);
  lines.push("render_layer_mask = 1");
  lines.push(`mobility = ${quote(entity.mobility ?? "Static")}`);
  lines.push("");
  lines.push("[entities.transform]");
  lines.push(`translation = ${fmtArray(entity.translation ?? [0, 0, 0])}`);
  lines.push(`rotation = ${fmtArray(entity.rotation ?? [0, 0, 0, 1])}`);
  lines.push(`scale = ${fmtArray(entity.scale ?? [1, 1, 1])}`);
  lines.push("");
}

function emitProjectReferenceTable(lines, table, assetPath, sub) {
  lines.push(`[${table}]`);
  lines.push('kind = "project"');
  lines.push(`guid = "${ZERO_GUID}"`);
  lines.push(`path_hint = ${quote(assetPath)}`);
  if (sub !== undefined) lines.push(`sub = ${quote(sub)}`);
  lines.push("");
}

function emitGltfMesh(lines, document, node, assetPath) {
  if (node.mesh === undefined) return;
  const mesh = document.meshes[node.mesh];
  const primitiveRows = mesh.primitives.map((primitive, primitiveIndex) => {
    const material =
      primitive.material === undefined ? "DefaultMaterial" : `Material${primitive.material}`;
    return `{ mesh = ${projectRef(assetPath, `Mesh${node.mesh}/Primitive${primitiveIndex}`)}, material = ${projectRef(assetPath, material)} }`;
  });
  const firstMaterial =
    mesh.primitives[0]?.material === undefined
      ? "DefaultMaterial"
      : `Material${mesh.primitives[0].material}`;
  lines.push("[entities.mesh]");
  lines.push("primitives = [");
  for (const row of primitiveRows) lines.push(`  ${row},`);
  lines.push("]");
  lines.push("");
  emitProjectReferenceTable(lines, "entities.mesh.model", assetPath, `Mesh${node.mesh}`);
  emitProjectReferenceTable(lines, "entities.mesh.material", assetPath, firstMaterial);
}

function emitScriptBinding(lines, actor) {
  lines.push("[[entities.script_bindings]]");
  lines.push('package = "woc_game"');
  lines.push('module = "main"');
  lines.push("enabled = false");
  lines.push("update = false");
  lines.push("fixed_update = false");
  lines.push("");
  lines.push("[entities.script_bindings.properties]");
  lines.push(`role = ${quote(actor.role)}`);
  lines.push(`source_id = ${quote(actor.source_id)}`);
  lines.push(`position_semantics = ${quote(actor.position_semantics)}`);
  lines.push("");
}

async function checkedGlb(assetPath, entries) {
  const entry = entries.get(assetPath);
  assert(entry?.gltf, `scene asset is absent from the checked M8 GLB closure: ${assetPath}`);
  const absolute = resolve(wocRoot, ...assetPath.split("/"));
  const bytes = await readFile(absolute);
  assert(bytes.length === entry.byte_length, `scene GLB byte length drift: ${assetPath}`);
  assert(sha256(bytes) === entry.sha256, `scene GLB digest drift: ${assetPath}`);
  return { entry, document: parseGlb(bytes, assetPath) };
}

function generateGround(contract) {
  const [halfX, halfZ] = contract.ground.half_extents;
  const [tilesX, tilesZ] = contract.ground.uv_tiles;
  return Buffer.from(
    [
      '# Generated by tools/m8_scene_codegen.mjs. Do not edit by hand.',
      'uri = "res://models/eastbrook_ground.model.toml"',
      "",
      "[[primitives]]",
      "indices = [0, 1, 2, 0, 2, 3]",
      "",
      "[[primitives.vertices]]",
      `position = ${fmtArray([-halfX, 0, -halfZ])}`,
      "normal = [0.0, 1.0, 0.0]",
      "uv = [0.0, 0.0]",
      "",
      "[[primitives.vertices]]",
      `position = ${fmtArray([halfX, 0, -halfZ])}`,
      "normal = [0.0, 1.0, 0.0]",
      `uv = ${fmtArray([tilesX, 0])}`,
      "",
      "[[primitives.vertices]]",
      `position = ${fmtArray([halfX, 0, halfZ])}`,
      "normal = [0.0, 1.0, 0.0]",
      `uv = ${fmtArray([tilesX, tilesZ])}`,
      "",
      "[[primitives.vertices]]",
      `position = ${fmtArray([-halfX, 0, halfZ])}`,
      "normal = [0.0, 1.0, 0.0]",
      `uv = ${fmtArray([0, tilesZ])}`,
      "",
    ].join("\n"),
    "utf8",
  );
}

const contractBytes = await readFile(contractPath);
const contract = JSON.parse(contractBytes.toString("utf8"));
const assetManifest = JSON.parse(await readFile(assetManifestPath, "utf8"));
assert(contract.schema_version === 1, "unsupported M8 Eastbrook scene contract schema");
assert(contract.source_commit === assetManifest.source_commit, "scene and asset commits differ");
const assetEntries = new Map(assetManifest.entries.map((entry) => [entry.asset_path, entry]));
const glbCache = new Map();
async function glb(assetPath) {
  if (!glbCache.has(assetPath)) glbCache.set(assetPath, await checkedGlb(assetPath, assetEntries));
  return glbCache.get(assetPath);
}

const lines = ["# Generated by tools/m8_scene_codegen.mjs. Do not edit by hand.", ""];
let nextEntity = 1;
let gltfNodeEntities = 0;
let propInstances = 0;

emitBaseEntity(lines, {
  entity: nextEntity++,
  name: "Eastbrook MVP Camera",
  mobility: "Dynamic",
  translation: contract.camera.translation,
  rotation: contract.camera.rotation,
});
lines.push("[entities.camera]");
lines.push(`fov_y_radians = ${fmtNumber(contract.camera.fov_y_radians)}`);
lines.push(`z_near = ${fmtNumber(contract.camera.z_near)}`);
lines.push(`z_far = ${fmtNumber(contract.camera.z_far)}`);
lines.push("hdr = true");
lines.push(`exposure_ev100 = ${fmtNumber(contract.camera.exposure_ev100)}`);
lines.push("");
lines.push("[entities.camera.post_process_settings.bloom]");
lines.push("threshold = 0.72");
lines.push("intensity = 0.32");
lines.push("radius = 0.62");
lines.push("");
lines.push("[entities.camera.post_process_settings.color_grading]");
lines.push("exposure = 1.0");
lines.push("contrast = 1.08");
lines.push("saturation = 1.04");
lines.push("gamma = 1.0");
lines.push("tint = [1.0, 0.97, 0.9]");
lines.push("");
lines.push("[entities.camera.post_process_settings.effect_stack.tonemap]");
lines.push('operator = "aces"');
lines.push("exposure_bias = 0.0");
lines.push("white_point = 1.0");
lines.push("");

emitBaseEntity(lines, { entity: nextEntity++, name: "Vale Day Ambient" });
lines.push("[entities.ambient_light]");
lines.push(`color = ${fmtArray(contract.lighting.ambient_color)}`);
lines.push(`intensity = ${fmtNumber(contract.lighting.ambient_intensity)}`);
lines.push("affects_lightmapped_meshes = true");
lines.push("");

emitBaseEntity(lines, { entity: nextEntity++, name: "Vale Day Sun" });
lines.push("[entities.directional_light]");
lines.push(`direction = ${fmtArray(contract.lighting.direction)}`);
lines.push(`color = ${fmtArray(contract.lighting.directional_color)}`);
lines.push(`intensity = ${fmtNumber(contract.lighting.directional_intensity)}`);
lines.push("volumetric = false");
lines.push("");

emitBaseEntity(lines, { entity: nextEntity++, name: "Eastbrook Flat MVP Ground" });
lines.push("[entities.mesh]");
lines.push(
  `primitives = [{ mesh = ${projectRef("assets/models/eastbrook_ground.model.toml", "Mesh0/Primitive0")}, material = ${builtinRef("builtin://material/default")} }]`,
);
lines.push("");
emitProjectReferenceTable(
  lines,
  "entities.mesh.model",
  "assets/models/eastbrook_ground.model.toml",
  undefined,
);
lines.push("[entities.mesh.material]");
lines.push('kind = "builtin"');
lines.push('locator = "builtin://material/default"');
lines.push("");

async function emitAssetScene(placement, actor) {
  const { document } = await glb(placement.asset_path);
  const graph = sceneNodeGraph(document);
  const hiddenNodes = actor ? placement.hidden_nodes : [];
  const preYaw = actor ? 0 : placement.pre_yaw ?? 0;
  const bounds = modelBounds(document, graph, preYaw, hiddenNodes);
  const placementEntity = nextEntity++;
  if (actor) {
    const uniformScale = placement.target_height / bounds.size[1];
    emitBaseEntity(lines, {
      entity: placementEntity,
      name: `Actor:${placement.id}`,
      mobility: "Dynamic",
      translation: placement.translation,
      rotation: yawQuaternion(placement.facing),
    });
    const animationIndex = document.animations?.findIndex(
      (animation) => animation.name === placement.animation,
    );
    assert(animationIndex >= 0, `${placement.id} animation ${placement.animation} is missing`);
    assert(document.skins?.[placement.skeleton_index], `${placement.id} skeleton is missing`);
    emitProjectReferenceTable(
      lines,
      "entities.animation_skeleton.skeleton",
      placement.asset_path,
      `Skin${placement.skeleton_index}/Skeleton`,
    );
    lines.push("[entities.animation_player]");
    lines.push("playback_speed = 1.0");
    lines.push("time_seconds = 0.0");
    lines.push("weight = 1.0");
    lines.push("looping = true");
    lines.push("playing = true");
    lines.push("");
    emitProjectReferenceTable(
      lines,
      "entities.animation_player.clip",
      placement.asset_path,
      `Animation${animationIndex}`,
    );
    emitScriptBinding(lines, placement);
    const normalizationEntity = nextEntity++;
    emitBaseEntity(lines, {
      entity: normalizationEntity,
      parent: placementEntity,
      name: `ActorNormalization:${placement.id}`,
      mobility: "Dynamic",
      translation: [0, -bounds.min[1] * uniformScale, 0],
      scale: [uniformScale, uniformScale, uniformScale],
    });
    emitGltfNodes(document, graph, placement, normalizationEntity, hiddenNodes, "Dynamic");
    return;
  }

  const scale = placement.target_size
    ? placement.target_size.map((value, index) => value / bounds.size[index])
    : placement.direct_scale ?? [placement.uniform_scale, placement.uniform_scale, placement.uniform_scale];
  assert(scale.every((value) => Number.isFinite(value) && value > 0), `${placement.id} scale is invalid`);
  const translation = [...placement.translation];
  translation[1] += placement.local_y_offset ?? 0;
  emitBaseEntity(lines, {
    entity: placementEntity,
    name: `Prop:${placement.id}`,
    translation,
    rotation: yawQuaternion(resolvedYaw(placement)),
    scale,
  });
  const normalizationEntity = nextEntity++;
  emitBaseEntity(lines, {
    entity: normalizationEntity,
    parent: placementEntity,
    name: `PropNormalization:${placement.id}`,
    translation: [
      -(bounds.min[0] + bounds.max[0]) / 2,
      -bounds.min[1],
      -(bounds.min[2] + bounds.max[2]) / 2,
    ],
  });
  const preYawEntity = nextEntity++;
  emitBaseEntity(lines, {
    entity: preYawEntity,
    parent: normalizationEntity,
    name: `PropPreYaw:${placement.id}`,
    rotation: yawQuaternion(preYaw),
  });
  emitGltfNodes(document, graph, placement, preYawEntity, [], "Static");
  propInstances++;
}

function emitGltfNodes(document, graph, placement, rootParent, hiddenNodes, mobility) {
  const hidden = new Set(hiddenNodes);
  const remap = new Map();
  for (const index of [...graph.selected].sort((a, b) => a - b)) remap.set(index, nextEntity++);
  for (const index of [...graph.selected].sort((a, b) => a - b)) {
    const node = document.nodes[index];
    const parentIndex = graph.parent.get(index);
    emitBaseEntity(lines, {
      entity: remap.get(index),
      parent: parentIndex === undefined ? rootParent : remap.get(parentIndex),
      name: `${placement.id}:Node${index}:${node.name ?? `Node${index}`}`,
      active: !hidden.has(node.name),
      mobility,
      translation: node.translation ?? [0, 0, 0],
      rotation: node.rotation ?? [0, 0, 0, 1],
      scale: node.scale ?? [1, 1, 1],
    });
    assert(!node.matrix, `${placement.asset_path} Node${index} uses an unsupported matrix transform`);
    emitGltfMesh(lines, document, node, placement.asset_path);
    gltfNodeEntities++;
  }
}

for (const actor of contract.actors) await emitAssetScene(actor, true);
const props = [...contract.props, ...expandFenceRuns(contract)];
for (const prop of props) await emitAssetScene(prop, false);

const bonfire = contract.props.find((prop) => prop.id === "eastbrook_bonfire");
assert(bonfire, "bonfire placement is missing");
emitBaseEntity(lines, {
  entity: nextEntity++,
  name: "Eastbrook Bonfire Light",
  translation: [bonfire.translation[0], 1.2, bonfire.translation[2]],
});
lines.push("[entities.point_light]");
lines.push(`color = ${fmtArray(contract.lighting.bonfire_color)}`);
lines.push(`intensity = ${fmtNumber(contract.lighting.bonfire_intensity)}`);
lines.push(`range = ${fmtNumber(contract.lighting.bonfire_range)}`);
lines.push("volumetric = false");
lines.push("");

const sceneBytes = Buffer.from(`${lines.join("\n")}\n`, "utf8");
const groundBytes = generateGround(contract);
const referencedEntries = [...glbCache.entries()]
  .map(([assetPath, value]) => ({
    asset_path: assetPath,
    sha256: value.entry.sha256,
    extensions_required: value.entry.gltf.extensions_required,
  }))
  .sort((a, b) => a.asset_path.localeCompare(b.asset_path));
const generatedManifest = {
  schema_version: 1,
  source_commit: contract.source_commit,
  scene_contract_sha256: sha256(contractBytes),
  asset_selection_sha256: assetManifest.selection_sha256,
  scene_sha256: sha256(sceneBytes),
  ground_sha256: sha256(groundBytes),
  totals: {
    entities: nextEntity - 1,
    gltf_node_entities: gltfNodeEntities,
    actors: contract.actors.length,
    prop_instances: propInstances,
    referenced_glbs: referencedEntries.length,
    fence_modules: props.length - contract.props.length,
  },
  coordinate_contract: contract.coordinate_contract,
  referenced_assets: referencedEntries,
};
const generatedManifestBytes = Buffer.from(`${JSON.stringify(generatedManifest, null, 2)}\n`, "utf8");

async function writeOrCheck(path, bytes, label) {
  if (checkOnly) {
    let current;
    try {
      current = await readFile(path);
    } catch {
      throw new Error(`${label} is missing`);
    }
    assert(current.equals(bytes), `${label} is stale`);
    return;
  }
  await mkdir(dirname(path), { recursive: true });
  await writeFile(path, bytes);
}

await writeOrCheck(scenePath, sceneBytes, "generated Eastbrook scene");
await writeOrCheck(groundPath, groundBytes, "generated Eastbrook ground model");
await writeOrCheck(
  generatedManifestPath,
  generatedManifestBytes,
  "generated Eastbrook scene manifest",
);

console.log(
  JSON.stringify({
    mode: checkOnly ? "check" : "generate",
    ...generatedManifest.totals,
    scene_sha256: generatedManifest.scene_sha256,
    ground_sha256: generatedManifest.ground_sha256,
  }),
);
