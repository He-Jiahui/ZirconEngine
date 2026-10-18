// Browser-safe semantic boundary shared by the plugin and compiler.
export const VERSION = '0.1.0';
export const NAMESPACE = 'dev.zircon.zop';
const ROLE = /^[a-z][a-z0-9_]*(\.[a-z][a-z0-9_]*){2,}$/;

export function stableJson(value) {
  if (value === null || typeof value !== 'object') return JSON.stringify(value);
  if (Array.isArray(value)) return `[${value.map(stableJson).join(',')}]`;
  return `{${Object.keys(value).sort().map(key => `${JSON.stringify(key)}:${stableJson(value[key])}`).join(',')}}`;
}

// The handwritten contract deliberately uses a small TOML grammar: version,
// quoted role tables, string and boolean scalars. Fail on syntax outside it.
export function parseRoleContract(source) {
  const result = { schema_version: 0, roles: Object.create(null) };
  let current = result;
  for (const [index, raw] of source.split(/\r?\n/).entries()) {
    const line = raw.trim();
    if (!line || line.startsWith('#')) continue;
    const table = /^\[roles\.("(?:[^"\\]|\\.)*")\]$/.exec(line);
    if (table) {
      const name = JSON.parse(table[1]);
      if (!ROLE.test(name) || Object.hasOwn(result.roles, name)) throw new Error(`roles:${index + 1}: invalid or duplicate role ${name}`);
      current = result.roles[name] = {};
      continue;
    }
    const entry = /^([a-z_]+)\s*=\s*("(?:[^"\\]|\\.)*"|true|false|1)$/.exec(line);
    if (!entry) throw new Error(`roles:${index + 1}: unsupported contract syntax`);
    const [, key, value] = entry;
    if (current === result) {
      if (key !== 'schema_version' || result.schema_version !== 0) throw new Error(`roles:${index + 1}: unexpected key ${key}`);
    } else if (Object.hasOwn(current, key) || !['kind', 'screen', 'required', 'command', 'slot'].includes(key)) {
      throw new Error(`roles:${index + 1}: unexpected or duplicate key ${key}`);
    }
    current[key] = JSON.parse(value);
  }
  if (result.schema_version !== 1 || !Object.keys(result.roles).length) throw new Error('roles: expected version 1 and nonempty roles');
  for (const [role, spec] of Object.entries(result.roles)) {
    if (!['command', 'slot', 'region', 'menu', 'dock', 'status'].includes(spec.kind)
      || role.split('.')[0] !== spec.screen || typeof spec.required !== 'boolean'
      || (spec.kind === 'command') !== Object.hasOwn(spec, 'command')
      || (spec.kind === 'slot') !== Object.hasOwn(spec, 'slot')
      || Object.hasOwn(spec, 'command') && (typeof spec.command !== 'string' || !spec.command.length)
      || Object.hasOwn(spec, 'slot') && (typeof spec.slot !== 'string' || !spec.slot.length)) {
      throw new Error(`roles: inconsistent definition ${role}`);
    }
  }
  return result;
}

export function validateAnnotation(annotation, contract, screen, path) {
  if (!annotation || typeof annotation !== 'object' || Array.isArray(annotation)) throw new Error(`${path}: binding must be an object`);
  for (const key of Object.keys(annotation)) {
    if (!['role', 'command', 'slot', 'text', 'tooltip', 'preview', 'free', 'visible_when', 'checked_from', 'repeat', 'template'].includes(key)) throw new Error(`${path}: unknown annotation ${key}`);
  }
  for (const key of ['role', 'command', 'slot', 'text', 'tooltip']) {
    if (annotation[key] !== undefined && typeof annotation[key] !== 'string') throw new Error(`${path}: ${key} must be a string`);
  }
  for (const key of ['preview', 'free']) {
    if (annotation[key] !== undefined && typeof annotation[key] !== 'boolean') throw new Error(`${path}: ${key} must be boolean`);
  }
  if (annotation.role !== undefined) {
    const spec = contract.roles[annotation.role];
    if (!spec || !ROLE.test(annotation.role)) throw new Error(`${path}: unknown role ${annotation.role}`);
    if (screen && spec.screen !== screen) throw new Error(`${path}: ${annotation.role} belongs to ${spec.screen}`);
    if ((spec.command ?? null) !== (annotation.command ?? null)) throw new Error(`${path}: unknown or mismatched command ${annotation.command ?? '(missing)'}`);
    if ((spec.slot ?? null) !== (annotation.slot ?? null)) throw new Error(`${path}: mismatched slot ${annotation.slot ?? '(missing)'}`);
  } else if (annotation.command || annotation.slot) {
    throw new Error(`${path}: command/slot needs a role`);
  }
  // No unimplemented state language is silently passed through to native UI.
  for (const key of ['visible_when', 'checked_from', 'repeat', 'template']) {
    if (annotation[key] != null) throw new Error(`${path}: ${key} is not supported by compiler ${VERSION}`);
  }
}

export function safeEntry(path) {
  if (typeof path !== 'string' || !/^[a-zA-Z0-9_.-]+(?:\/[a-zA-Z0-9_.-]+)*$/.test(path)
    || path.split('/').some(part => part === '.' || part === '..' || part.endsWith('.')
      || /^(con|prn|aux|nul|com[1-9]|lpt[1-9])(?:\.|$)/i.test(part))) throw new Error(`unsafe bundle path ${path}`);
  return path;
}
