import { createHash } from 'node:crypto';
import { parseRoleContract, safeEntry, stableJson, validateAnnotation, VERSION } from './contract.mjs';

export const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
const decode = bytes => new TextDecoder('utf-8', { fatal: true }).decode(bytes);
const encode = source => new TextEncoder().encode(source);
const INTERACTIVE = new Set(['Button', 'IconButton']);
const COMPONENTS = new Set(['Button', 'IconButton', 'Label', 'Panel', 'HorizontalGroup', 'VerticalGroup', 'Overlay', 'Slot']);
const TOKEN_BINDINGS = new Map([
  ['fill', { prop: 'background_color', types: ['color'] }],
  ['strokeColor', { prop: 'border_color', types: ['color'] }],
  ['fontFamilies', { prop: 'font_family', types: ['fontFamily'] }],
  ['fontSizes', { prop: 'font_size', types: ['number', 'dimension'] }],
  ['fontWeight', { prop: 'font_weight', types: ['number'] }],
]);
const assert = (condition, path, message) => { if (!condition) throw new Error(`${path}: ${message}`); };
const finite = (value, path) => { assert(typeof value === 'number' && Number.isFinite(value) && value >= 0, path, 'expected a finite nonnegative number'); return value; };

function toml(value) {
  if (typeof value === 'string' || typeof value === 'boolean') return JSON.stringify(value);
  if (typeof value === 'number' && Number.isFinite(value)) return String(value);
  if (Array.isArray(value)) return `[${value.map(toml).join(', ')}]`;
  if (value && typeof value === 'object') return `{ ${Object.keys(value).sort().map(key => `${JSON.stringify(key)} = ${toml(value[key])}`).join(', ')} }`;
  throw new Error('ZUI v2 cannot encode a null/undefined/nonfinite value');
}

// Uses the bridge's ZuiDocument shape; managed parity tests pass each document
// through the existing bridge serializer/runtime-schema validator as well.
export function serializeDocument(document) {
  const blocks = [];
  for (const section of ['asset', 'imports', 'tokens', 'root']) {
    if (document[section]) blocks.push(`[${section}]\n${Object.keys(document[section]).sort().map(key => `${JSON.stringify(key)} = ${toml(document[section][key])}`).join('\n')}`);
  }
  for (const id of Object.keys(document.nodes ?? {}).sort()) {
    blocks.push(`[nodes.${JSON.stringify(id)}]\n${Object.keys(document.nodes[id]).sort().map(key => `${JSON.stringify(key)} = ${toml(document.nodes[id][key])}`).join('\n')}`);
  }
  return blocks.join('\n\n') + '\n';
}

function axis(axis, path) {
  assert(axis && ['fixed', 'fill', 'hug'].includes(axis.mode), path, 'expected fixed/fill/hug sizing');
  const out = { stretch: axis.mode === 'fill' ? 'Stretch' : 'Fixed' };
  for (const key of ['min', 'max']) if (axis[key] != null) out[key] = finite(axis[key], `${path}.${key}`);
  assert(out.min == null || out.max == null || out.min <= out.max, path, 'min exceeds max');
  if (axis.mode === 'fixed') {
    const size = finite(axis.value, path);
    assert((out.min == null || size >= out.min) && (out.max == null || size <= out.max), path, 'fixed size outside min/max');
    out.min = out.max = out.preferred = size;
  }
  return out;
}

function localized(value, translations, path) {
  assert(typeof value === 'string', path, 'expected text');
  if (value.startsWith('i18n:')) {
    const key = value.slice(5);
    assert(Object.hasOwn(translations, key), path, `missing i18n key ${key}`);
    return { text_key: key, table: 'editor' };
  }
  return value;
}

function compileScreen(screen, contract, translations, includePreview, tokenTypes) {
  const path = screen.asset_path;
  safeEntry(path);
  assert(/^(shell|panels|components)\/[a-z0-9_]+\.zui$/.test(path), path, 'unsupported asset landing path');
  assert(Object.values(contract.roles).some(role => role.screen === screen.screen), path, `unknown screen ${screen.screen}`);
  const nodes = {}, bindings = [], ids = new Set(), roles = new Set(), controls = new Set();
  function visit(node, parentPath) {
    const nodePath = `${parentPath}/${node.name ?? node.id}`;
    const binding = node.binding ?? {};
    validateAnnotation(binding, contract, screen.screen, nodePath);
    if (binding.preview && !includePreview) return null;
    assert(typeof node.id === 'string' && /^[a-zA-Z0-9_-]+$/.test(node.id) && !ids.has(node.id), nodePath, 'invalid or duplicate node id');
    ids.add(node.id);
    assert(!node.shadows?.length && !node.blur, nodePath, 'shadows/blur unsupported');
    assert(!node.rotation && node.opacity === 1, nodePath, 'rotation or group opacity unsupported');
    assert(!node.image, nodePath, 'image export is not supported by this compiler version');
    assert(Array.isArray(node.fills) && node.fills.length <= 1, nodePath, 'multiple/mixed fills unsupported');
    assert(Array.isArray(node.strokes) && node.strokes.length <= 1, nodePath, 'multiple/mixed strokes unsupported');
    const component = binding.slot ? 'Slot' : node.component;
    assert(COMPONENTS.has(component), nodePath, `unsupported component ${component}`);
    if (node.component_ref) assert(/^zop\/[A-Za-z]+(?:\/md)?$/.test(node.component_ref), nodePath, 'nonlibrary or unsupported component instance');
    if (INTERACTIVE.has(component)) assert(binding.role && binding.command, nodePath, 'interactive component needs role and command');
    // A command event on an input-disabled visual compiles into an unreachable action.
    assert(!binding.command || INTERACTIVE.has(component), nodePath, 'command role requires an interactive Button or IconButton');
    const props = {};
    assert(typeof node.visible === 'boolean', nodePath, 'visibility must be explicit');
    if (!node.visible) Object.assign(props, { visible: false, visibility: 'collapsed' });
    for (const [i, fill] of node.fills.entries()) {
      assert(typeof fill.color === 'string' && /^#[a-fA-F0-9]{6}(?:[a-fA-F0-9]{2})?$/.test(fill.color) && fill.opacity === 1, nodePath, `unsupported fill ${i}`);
      props[component === 'Label' ? 'foreground_color' : 'background_color'] = fill.color;
    }
    for (const stroke of node.strokes) {
      assert(typeof stroke.color === 'string' && /^#[a-fA-F0-9]{6}(?:[a-fA-F0-9]{2})?$/.test(stroke.color) && stroke.opacity === 1, nodePath, 'unsupported stroke');
      props.border_color = stroke.color; props.border_width = finite(stroke.width, nodePath);
    }
    props.corner_radius = finite(node.radius, nodePath);
    if (node.text != null || binding.text != null) {
      const text = localized(binding.text ?? node.text, translations, nodePath);
      props[component === 'IconButton' ? 'label' : 'text'] = text;
    }
    if (binding.tooltip != null) props.tooltip = localized(binding.tooltip, translations, nodePath);
    if (node.font) {
      assert(typeof node.font.family === 'string' && node.font.family.length > 0, nodePath, 'font family required');
      props.font_family = node.font.family; props.font_size = finite(node.font.size, nodePath);
      props.font_weight = finite(node.font.weight, nodePath); props.line_height_ratio = finite(node.font.line_height, nodePath);
    }
    for (const [property, token] of Object.entries(node.tokens ?? {})) {
      const tokenBinding = TOKEN_BINDINGS.get(property);
      assert(tokenBinding && Object.hasOwn(tokenTypes, token), nodePath, `unsupported or missing token binding ${property}=${token}`);
      assert(tokenBinding.types.includes(tokenTypes[token]), nodePath,
        `token ${token} has DTCG type ${tokenTypes[token]}; ${property} requires ${tokenBinding.types.join(' or ')}`);
      const target = property === 'fill' && component === 'Label' ? 'foreground_color' : tokenBinding.prop;
      props[target] = `$${token}`;
    }
    if (component === 'Slot') props.name = binding.slot;
    // Unbound labels and containers cannot accidentally consume native input.
    props.input_interactive = INTERACTIVE.has(component); props.input_focusable = INTERACTIVE.has(component);
    const layout = node.layout;
    assert(layout && ['flex', 'leaf'].includes(layout.kind), nodePath,
      layout?.kind === 'grid' ? 'grid compilation is not supported (nonuniform tracks forbidden)' : 'free/grid layout unsupported');
    const compiledLayout = { width: axis(layout.width, `${nodePath}.width`), height: axis(layout.height, `${nodePath}.height`) };
    if (node.clip != null) { assert(typeof node.clip === 'boolean', nodePath, 'clip must be boolean'); compiledLayout.clip = node.clip; }
    const children = node.children ?? [];
    if (layout.kind === 'flex') {
      assert(['row', 'column'].includes(layout.direction) && layout.justify === 'start' && ['start', 'stretch'].includes(layout.align)
        && layout.wrap === false, nodePath, 'unsupported flex direction/alignment/wrapping');
      assert(Array.isArray(layout.padding) && layout.padding.length === 4, nodePath, 'padding needs four edges');
      const [top, right, bottom, left] = layout.padding.map(value => finite(value, nodePath));
      Object.assign(props, { layout_padding_top: top, layout_padding_right: right, layout_padding_bottom: bottom, layout_padding_left: left });
      compiledLayout.container = { kind: layout.direction === 'row' ? 'HorizontalBox' : 'VerticalBox', gap: finite(layout.gap, nodePath) };
      // Start/Fill are the currently supported cross-axis behaviors.
      for (const child of children) assert(!child.absolute, nodePath, 'free positioning inside flex requires a future explicit free-layout implementation');
    } else assert(!children.length, nodePath, 'leaf cannot contain children');
    const id = `n_${node.id}`, controlId = `zop_${node.id}`;
    assert(!controls.has(controlId), nodePath, 'duplicate control identity'); controls.add(controlId);
    const out = { component, control_id: controlId, props, layout: compiledLayout };
    const emittedChildren = children.map(child => ({ child, id: visit(child, nodePath) })).filter(child => child.id);
    if (emittedChildren.length) out.children = emittedChildren.map(({ child, id }) => ({ node: id,
      slot: { layout: { linear_size: { rule: child.layout[layout.direction === 'row' ? 'width' : 'height'].mode === 'fill' ? 'Stretch' : 'Auto', value: 1 },
        alignment: layout.direction === 'row' ? { horizontal: 'Start', vertical: layout.align === 'stretch' ? 'Fill' : 'Start' }
          : { horizontal: layout.align === 'stretch' ? 'Fill' : 'Start', vertical: 'Start' } } } }));
    if (binding.command) out.events = [{ id: `role/${binding.role}`, event: 'Click', action: { action: binding.command } }];
    if (binding.role) {
      assert(!roles.has(binding.role), nodePath, `duplicate role ${binding.role}`); roles.add(binding.role);
      bindings.push({ role: binding.role, control_id: controlId, command: binding.command ?? null, slot: binding.slot ?? null });
    }
    nodes[id] = out;
    return id;
  }
  const root = visit(screen.root, path);
  assert(root, path, 'root excluded as preview');
  const missing = Object.entries(contract.roles).filter(([id, role]) => role.screen === screen.screen && role.required && !roles.has(id)).map(([id]) => id);
  assert(!missing.length, path, `missing required roles ${missing.join(', ')}`);
  const document = { asset: { kind: 'view', id: `res://ui/operator/${path}`, version: 2 },
    imports: { styles: ['res://ui/operator/theme/zop_tokens.zui'] }, root: { node: root }, nodes };
  return { document, bindings: bindings.sort((a, b) => a.role < b.role ? -1 : a.role > b.role ? 1 : 0) };
}

function tokenDocument(tokens) {
  const flattened = {}, types = {};
  function visit(group, prefix = '', inheritedType) {
    assert(group && typeof group === 'object' && !Array.isArray(group), prefix, 'invalid DTCG group');
    if (Object.hasOwn(group, '$value')) {
      const type = group.$type ?? inheritedType, value = group.$value;
      assert(/^editor\.[a-z0-9_.]+$/.test(prefix), prefix, 'token must map to editor.*');
      if (type === 'color') assert(typeof value === 'string' && /^#[a-fA-F0-9]{6}(?:[a-fA-F0-9]{2})?$/.test(value), prefix, 'only resolved hexadecimal colors supported');
      else if (type === 'number') finite(value, prefix);
      else if (type === 'fontFamily') assert(typeof value === 'string' && value.length > 0, prefix, 'font family must be a string');
      else if (type === 'dimension') assert(value && value.unit === 'px' && finite(value.value, prefix) >= 0, prefix, 'dimensions require px');
      else throw new Error(`${prefix}: unsupported DTCG type ${type}`);
      flattened[prefix] = type === 'dimension' ? value.value : value;
      types[prefix] = type;
      return;
    }
    for (const key of Object.keys(group).sort()) if (!key.startsWith('$')) visit(group[key], prefix ? `${prefix}.${key}` : key, group.$type ?? inheritedType);
  }
  visit(tokens);
  assert(Object.keys(flattened).length, 'tokens.dtcg.json', 'at least one design token is required');
  return { document: { asset: { kind: 'style', id: 'res://ui/operator/theme/zop_tokens.zui', version: 2 }, tokens: flattened }, types };
}

export function compileBundle(bundle, contractSource, translations = {}, { includePreview = false } = {}) {
  const contract = parseRoleContract(contractSource), { manifest, entries } = bundle;
  assert(manifest.schema_version === 1 && manifest.compiler_version === VERSION && manifest.plugin_version === VERSION, 'manifest', 'unsupported schema/plugin/compiler version');
  assert(Array.isArray(manifest.files) && manifest.files.length > 0
    && manifest.files.every(file => typeof file.id === 'string' && /^[a-zA-Z0-9_-]+$/.test(file.id) && Number.isSafeInteger(file.revision) && file.revision >= 0), 'manifest', 'actual file IDs/revisions required');
  assert(manifest.entries && typeof manifest.entries === 'object', 'manifest', 'entry hashes required');
  const seen = new Set();
  for (const [path, hash] of Object.entries(manifest.entries)) {
    safeEntry(path);
    assert(!seen.has(path.toLowerCase()), path, 'case-colliding entries'); seen.add(path.toLowerCase());
    assert(/^(tokens\.dtcg\.json|screens\/[a-zA-Z0-9_-]+\.json|previews\/[a-zA-Z0-9_-]+\.png)$/.test(path), path, 'unsupported bundle entry (components/icons need the next compiler milestone)');
    assert(/^[a-f0-9]{64}$/.test(hash) && entries[path] && sha256(entries[path]) === hash, path, 'missing or corrupt entry');
  }
  for (const path of Object.keys(entries)) assert(path === 'manifest.json' || Object.hasOwn(manifest.entries, path), path, 'unmanifested entry');
  assert(manifest.entries['tokens.dtcg.json'], 'manifest', 'tokens.dtcg.json is required');
  const { document: theme, types: tokenTypes } = tokenDocument(JSON.parse(decode(entries['tokens.dtcg.json']))), files = {}, documents = {};
  const origin = manifest.files.map(file => `penpot:${file.id}@${file.revision}`).join(',');
  const provenance = `# generated by zop-compiler ${VERSION} from ${origin} — do not edit\n`;
  function add(path, document) {
    assert(!Object.hasOwn(files, path), path, 'duplicate output path');
    documents[path] = document; files[path] = encode(provenance + serializeDocument(document));
  }
  add('theme/zop_tokens.zui', theme);
  const screenPaths = Object.keys(manifest.entries).filter(path => path.startsWith('screens/')).sort();
  assert(screenPaths.length > 0, 'manifest', 'at least one screen required');
  for (const path of screenPaths) {
    const screen = JSON.parse(decode(entries[path]));
    const { document, bindings } = compileScreen(screen, contract, translations, includePreview, tokenTypes);
    add(screen.asset_path, document);
    const sidecar = screen.asset_path.replace(/\.zui$/, '.roles.json');
    files[sidecar] = encode(stableJson({ schema_version: 1, screen: screen.screen, bindings }) + '\n');
  }
  const lock = { schema_version: 1, compiler_version: VERSION, bundle_sha256: sha256(stableJson(manifest)),
    contract_sha256: sha256(contractSource), translations_sha256: sha256(stableJson(translations)), include_preview: includePreview,
    outputs: Object.fromEntries(Object.keys(files).sort().map(path => [path, sha256(files[path])])) };
  files['zop-bundle.lock.json'] = encode(stableJson(lock) + '\n');
  return { files, documents, lock };
}
