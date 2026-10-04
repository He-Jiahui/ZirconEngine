import { readFile, realpath } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { resolve, relative, isAbsolute, sep } from 'node:path';
import { parse } from 'smol-toml';
import { parseZuiLayoutSource } from '../src/bridge/zui-layout-optimizer';
import { editorHostColors } from './zui-layout-host-theme';
import { FIELD_ICON_PATHS, isEditorField } from '../src/bridge/zui-field-projection';
import {
  cloneZuiDocument,
  type ZuiDocument,
  type ZuiNode,
  type ZuiChildMount,
  type ZuiTable,
} from '../src/bridge/zui-document';
import {
  prepareDynamicReviewState,
  type DynamicReviewContext,
} from './zui-layout-dynamic-hosts';

export class LayoutDependencies {
  private readonly documents = new Map<string, ZuiDocument>();
  private readonly sourceById = new Map<string, string[]>();
  private readonly parseFailures = new Map<string, unknown>();
  private readonly iconSources = new Map<string, string>();
  private readonly fieldIcons: Record<string, string> = {};

  async load(repoRoot: string, paths: string[]): Promise<void> {
    const fieldIcons = await Promise.all(
      Object.entries(FIELD_ICON_PATHS).map(async ([key, path]) => ({
        key,
        source: existsSync(resolve(repoRoot, path))
          ? await readFile(resolve(repoRoot, path), 'utf8')
          : undefined,
      })),
    );
    for (const { key, source } of fieldIcons) {
      if (source !== undefined) this.fieldIcons[key] = source;
    }
    const sources = await Promise.all(
      paths.map(async (path) => {
        try {
          const text = await readFile(resolve(repoRoot, path), 'utf8');
          let document = parse(text) as unknown as ZuiDocument;
          if (document.asset.version === 1)
            document = parseZuiLayoutSource(text, path).document;
          return { path, document };
        } catch (error) {
          return { path, error };
        }
      }),
    );
    for (const source of sources) {
      const { path } = source;
      if (!('document' in source) || !source.document) {
        const { error } = source;
        this.parseFailures.set(path, error);
        continue;
      }
      const document: ZuiDocument = source.document;
      this.documents.set(path, document);
      const assetRoot = ownerAssetRoot(path);
      for (const node of Object.values(document.nodes ?? {})) {
        const value =
          node.props?.['icon'] ??
          (String(node.component).toLowerCase().includes('icon')
            ? node.props?.['source']
            : undefined);
        if (typeof value !== 'string' || !value.endsWith('.svg')) continue;
        const locator = value.replace(/^res:\/\/(icons\/)?/, '');
        const iconRoot = resolve(repoRoot, assetRoot, 'icons');
        const iconPath = resolve(iconRoot, locator);
        if (!contained(iconRoot, iconPath))
          throw new Error(`Icon escapes asset root: ${value} in ${path}`);
        if (existsSync(iconPath)) {
          if (!contained(await realpath(iconRoot), await realpath(iconPath)))
            throw new Error(
              `Icon symlink escapes asset root: ${value} in ${path}`,
            );
          this.iconSources.set(
            `${assetRoot}:${value}`,
            await readFile(iconPath, 'utf8'),
          );
        }
      }
      const id = document.asset?.id;
      if (id)
        this.sourceById.set(id, [...(this.sourceById.get(id) ?? []), path]);
      const fixtureParts = path.split('/ui_zui/');
      if (fixtureParts.length === 2) {
        const uri = `res://ui/${fixtureParts[1]}`;
        if (uri !== id)
          this.sourceById.set(uri, [...(this.sourceById.get(uri) ?? []), path]);
      }
    }
    const fixture =
      'zircon_runtime/src/dynamic_api/session/tests/runtime_ui_surface.rs';
    if (existsSync(resolve(repoRoot, fixture))) {
      const rust = await readFile(resolve(repoRoot, fixture), 'utf8');
      for (const name of [
        'PENPOT_ROUNDTRIP_BUTTON_COMPONENT',
        'PENPOT_ROUNDTRIP_STYLE',
      ]) {
        const literal = rust.match(
          new RegExp(`const ${name}: &str = r(#+)"([\\s\\S]*?)"\\1;`),
        );
        if (!literal)
          throw new Error(`Missing embedded fixture ${fixture}#${name}`);
        const document = parse(literal[2]) as unknown as ZuiDocument;
        const path = `${fixture}#${name}`;
        this.documents.set(path, document);
        this.sourceById.set(document.asset.id, [
          ...(this.sourceById.get(document.asset.id) ?? []),
          path,
        ]);
      }
    }
  }

  embed(
    document: ZuiDocument,
    sourcePath: string,
    themeSourcePath?: string,
    dynamicReviewContext: DynamicReviewContext = {},
  ): string[] {
    const tokens: ZuiTable = {};
    const styles: ZuiTable[] = [];
    const visited = new Set<string>();
    const ownerByNode = new Map(
      Object.keys(document.nodes ?? {}).map((id) => [id, sourcePath]),
    );
    // Keep authored ownership on every direct source node before expansion.
    // The metadata is projection-only and is stripped by the source exporter;
    // it gives browser/native review selectors an exact source identity even
    // when control ids repeat across component instances.
    for (const [id, node] of Object.entries(document.nodes ?? {})) {
      // Product-model rows are materialized from factual case data, not from
      // authored .zui callsites. Keep their explicit marker and never invent
      // an authored source key from the generated projection node id.
      if (node['penpot_review_generated_product_row']) continue;
      annotateSourceIdentity(node, sourcePath, id, '[]');
    }
    const sourceFor = (uri: string, path: string) => {
      const candidates = this.sourceById.get(uri.split('#')[0]) ?? [];
      const fixtureRoot = path.includes('/ui_zui/')
        ? path.split('/ui_zui/')[0] + '/ui_zui/'
        : null;
      const owner = path.split('/assets/')[0];
      const local = candidates.filter((candidate) =>
        candidate.startsWith(`${owner}/assets/`),
      );
      const applicable = fixtureRoot
        ? candidates.filter((candidate) => candidate.startsWith(fixtureRoot))
        : candidates.filter(
            (candidate) => !candidate.includes('/tests/fixtures/'),
          );
      const matches =
        fixtureRoot && applicable.length
          ? applicable
          : local.length
            ? local
            : applicable.length
              ? applicable
              : candidates;
      if (matches.length > 1)
        throw new Error(`Ambiguous asset ${uri} imported from ${path}`);
      if (matches[0]) return matches[0];
      const expected = `${ownerAssetRoot(path)}/${uri.split('#')[0].replace(/^res:\/\//, '')}`;
      const failure = this.parseFailures.get(expected);
      if (failure)
        throw new Error(`Invalid dependency ${expected}: ${String(failure)}`);
      throw new Error(`Missing dependency ${uri} imported from ${path}`);
    };
    const resolveComponent = (componentName: string, ownerPath: string) => {
      const current =
        ownerPath === sourcePath ? document : this.documents.get(ownerPath)!;
      const candidates: Array<{
        document: ZuiDocument;
        path: string;
        name: string;
      }> = [];
      if (current.components?.[componentName])
        candidates.push({
          document: current,
          path: ownerPath,
          name: componentName,
        });
      else
        for (const uri of componentName.includes('#')
          ? [componentName]
          : strings(current.imports?.['widgets'])) {
          const [assetId, fragment] = uri.split('#');
          const name = componentName.includes('#') ? fragment : componentName;
          if (fragment && name !== fragment) continue;
          const path = sourceFor(assetId, ownerPath);
          const dependency = path ? this.documents.get(path) : undefined;
          if (path && dependency?.components?.[name])
            candidates.push({ document: dependency, path, name });
        }
      const unique = [
        ...new Map(
          candidates.map((candidate) => [
            `${candidate.path}#${candidate.name}`,
            candidate,
          ]),
        ).values(),
      ];
      if (unique.length > 1)
        throw new Error(`Ambiguous component ${componentName} in ${ownerPath}`);
      const match = unique[0];
      if (!match) return undefined;
      const definition = match.document.components![match.name];
      return {
        ...match,
        definition,
        root: definition.root,
        classes: strings(definition['default_classes']),
      };
    };
    const dependencies: string[] = [];
    const visit = (current: ZuiDocument, path: string) => {
      if (visited.has(path)) return;
      visited.add(path);
      for (const uri of [
        ...strings(current.imports?.['styles']),
        ...strings(current.imports?.['widgets']),
      ]) {
        const dependency = sourceFor(uri, path);
        if (!dependency || visited.has(dependency)) continue;
        dependencies.push(dependency);
        visit(this.documents.get(dependency)!, dependency);
      }
      Object.assign(tokens, themeTokens(current), current.tokens);
      for (const sheet of current.stylesheets ?? []) {
        const existing = styles.findIndex(
          (value) => value['id'] === sheet['id'],
        );
        if (existing >= 0) styles[existing] = sheet;
        else styles.push(sheet);
      }
    };
    if (themeSourcePath) {
      const theme = this.documents.get(themeSourcePath);
      if (!theme)
        throw new Error(`Missing review host theme: ${themeSourcePath}`);
      if (themeSourcePath !== sourcePath) dependencies.push(themeSourcePath);
      visit(theme, themeSourcePath);
      document['penpot_host_colors'] = editorHostColors(theme);
      document['penpot_host_palette'] = structuredClone(theme['palette']);
      document['penpot_host_theme_source'] = themeSourcePath;
    }
    visit(document, sourcePath);
    document.tokens = tokens;
    document.stylesheets = styles;
    document['penpot_dependency_sources'] = dependencies;
    let expanded = 0;
    const nodes = document.nodes ?? {};
    const expand = (id: string, stack: string[]) => {
      const instance = nodes[id];
      if (!instance) return;
      const prefab = resolveComponent(
        instance.component,
        ownerByNode.get(id) ?? sourcePath,
      );
      if (!prefab) return;
      const prefabKey = `${prefab.path}#${prefab.name}`;
      if (stack.includes(prefabKey))
        throw new Error(
          `Recursive component ${[...stack, instance.component].join(' -> ')}`,
        );
      if (++expanded > 10000)
        throw new Error('Component expansion exceeds 10000 instances');
      const source = cloneZuiDocument(prefab.document);
      // Dynamic retained hosts carry inactive authored branches for the
      // runtime router.  A Penpot projection has a deterministic review
      // state, so apply that state before recursively expanding the prefab.
      // This keeps the inactive branch semantically authored in the source
      // while preventing a composite shell (notably EditorMainFrame) from
      // materializing every workspace just to collapse it afterwards.
      prepareDynamicReviewState(source, prefab.path, dynamicReviewContext);
      const params: ZuiTable = {};
      for (const name of Object.keys(instance.params ?? {})) {
        if (!(name in object(prefab.definition['params'])))
          throw new Error(`Unknown param ${prefabKey}.${name}`);
      }
      for (const [name, schemaValue] of Object.entries(
        object(prefab.definition['params']),
      )) {
        const schema = object(schemaValue);
        const value = instance.params?.[name] ?? schema['default'];
        if (value === undefined)
          throw new Error(`Missing required param ${name} in ${prefabKey}`);
        if (!validParamType(String(schema['type'] ?? 'any'), value))
          throw new Error(
            `Invalid param type ${prefabKey}.${name}: expected ${schema['type']}`,
          );
        params[name] = value;
      }
      const suppliedSlots = new Map<string, ZuiChildMount[]>();
      for (const child of instance.children ?? []) {
        const name = String(
          child.slot?.['name'] ?? child.slot?.['slot_name'] ?? 'default',
        );
        suppliedSlots.set(name, [...(suppliedSlots.get(name) ?? []), child]);
      }
      const slots = object(prefab.definition['slots']);
      for (const [name, children] of suppliedSlots) {
        const schema = object(slots[name]);
        if (
          !slots[name] &&
          !(name === 'default' && Object.keys(slots).length === 0)
        )
          throw new Error(`Unknown slot ${name} in ${prefabKey}`);
        if (slots[name] && schema['multiple'] !== true && children.length > 1)
          throw new Error(
            `Multiple children in single slot ${prefabKey}.${name}`,
          );
        const accepts = strings(schema['accepts']);
        for (const child of children) {
          if (!nodes[child.node])
            throw new Error(
              `Missing slot child ${child.node} in ${prefabKey}.${name}`,
            );
          const original = nodes[child.node]?.component.split('#').pop();
          if (
            accepts.length &&
            original &&
            !accepts.includes(original) &&
            !accepts.includes('*')
          )
            throw new Error(`Slot ${prefabKey}.${name} rejects ${original}`);
        }
      }
      for (const [name, schema] of Object.entries(slots)) {
        if (object(schema)['required'] === true && !suppliedSlots.has(name))
          throw new Error(`Missing required slot ${prefabKey}.${name}`);
      }
      const root = source.nodes?.[prefab.root];
      if (!root) throw new Error(`Missing component root in ${prefab.path}`);
      const callerIdentity = sourceIdentity(instance, ownerByNode.get(id) ?? sourcePath, id);
      const instancePath = appendInstancePath(callerIdentity);
      const mapping = new Map<string, string>([[prefab.root, id]]);
      const collect = (nodeId: string) => {
        const node = source.nodes?.[nodeId];
        if (!node) return;
        for (const child of node.children ?? []) {
          if (source.nodes?.[child.node]?.component === 'Slot') continue;
          if (mapping.has(child.node)) continue;
          const qualified = `${id}__${child.node}`;
          if (nodes[qualified])
            throw new Error(`Duplicate expanded node ${qualified}`);
          mapping.set(child.node, qualified);
          collect(child.node);
        }
      };
      collect(prefab.root);
      const next = [...stack, prefabKey];
      const consumed = new Set<string>();
      for (const [from, to] of mapping) {
        const template = resolveParamValues(
          source.nodes![from],
          params,
        ) as ZuiNode;
        const remapped: ZuiNode = {
          ...template,
          children: template.children?.flatMap((child) => {
            const placeholder = source.nodes?.[child.node];
            if (placeholder?.component !== 'Slot')
              return [{ ...child, node: mapping.get(child.node)! }];
            const name = String(
              placeholder.props?.['name'] ??
                placeholder.props?.['slot_name'] ??
                'default',
            );
            consumed.add(name);
            return (suppliedSlots.get(name) ?? []).map((mount) => ({
              ...mount,
              slot: mergeLayout(
                { layout: placeholder.layout ?? {} },
                mount.slot,
              ),
            }));
          }),
        };
        if (!remapped.children) delete remapped.children;
        nodes[to] =
          to === id
            ? {
                ...remapped,
                ...instance,
                component: template.component,
                props: { ...template.props, ...instance.props },
                layout: mergeLayout(template.layout, instance.layout),
                classes: [
                  ...new Set([
                    ...(template.classes ?? []),
                    ...prefab.classes,
                    ...(instance.classes ?? []),
                  ]),
                ],
                state: { ...template.state, ...instance.state },
                style: {
                  self: {
                    ...object(template.style?.['self']),
                    ...object(instance.style?.['self']),
                  },
                  slot: {
                    ...object(template.style?.['slot']),
                    ...object(instance.style?.['slot']),
                  },
                },
                slots: { ...template.slots, ...instance.slots },
                events: [
                  ...(template.events ?? []),
                  ...(instance.events ?? []),
                ],
                children: remapped.children ?? [],
                penpot_prefab_source: `${prefab.path}#${prefab.name}`,
              }
            : remapped;
        setSourceIdentity(
          nodes[to],
          prefab.path,
          from,
          instancePath,
        );
        ownerByNode.set(to, prefab.path);
        delete nodes[to].params;
      }
      for (const to of mapping.values()) expand(to, next);
    };
    for (const id of Object.keys(nodes)) expand(id, []);
    for (const definition of Object.values(document.components ?? {})) {
      const defaults = Object.fromEntries(
        Object.entries(object(definition['params'])).flatMap(([key, value]) => {
          const fallback = object(value)['default'];
          return fallback === undefined ? [] : [[key, fallback]];
        }),
      ) as ZuiTable;
      const seen = new Set<string>();
      const apply = (id: string) => {
        if (seen.has(id) || !nodes[id]) return;
        seen.add(id);
        nodes[id] = resolveParamValues(nodes[id], defaults) as ZuiNode;
        for (const child of nodes[id].children ?? []) apply(child.node);
      };
      apply(definition.root);
    }
    for (const [id, parent] of Object.entries(nodes)) {
      const icon =
        parent.props?.['icon'] ??
        (parent.component.toLowerCase().includes('icon')
          ? parent.props?.['source']
          : undefined);
      const iconKey = `${ownerAssetRoot(ownerByNode.get(id) ?? sourcePath)}:${icon}`;
      if (typeof icon === 'string' && this.iconSources.has(iconKey))
        parent['penpot_icon_svg'] = this.iconSources.get(iconKey);
      for (const mount of parent.children ?? []) {
        const child = nodes[mount.node];
        const slot = object(mount.slot?.['layout']);
        if (!child || Object.keys(slot).length === 0) continue;
        // Native slot padding surrounds the child; it is not child content padding.
        const nodeSlot = { ...slot };
        delete nodeSlot['padding'];
        child.layout = mergeLayout(child.layout, nodeSlot);
        const alignment = object(slot['alignment']);
        if (String(parent.component).toLowerCase() === 'overlay') {
          for (const [axis, key] of [
            ['x', 'horizontal'],
            ['y', 'vertical'],
          ] as const) {
            const value = alignment[key];
            const factor =
              value === 'End' ? 1 : value === 'Center' ? 0.5 : null;
            if (factor !== null) {
              child.layout['anchor'] = {
                ...object(child.layout['anchor']),
                [axis]: factor,
              };
              child.layout['pivot'] = {
                ...object(child.layout['pivot']),
                [axis]: factor,
              };
            }
          }
        }
      }
    }
    document['penpot_original_imports'] = document.imports ?? {};
    if (themeSourcePath && Object.values(nodes).some(isEditorField)) {
      document['penpot_field_icons'] = { ...this.fieldIcons };
      dependencies.push(...Object.values(FIELD_ICON_PATHS));
    }
    document.imports = { ...document.imports, widgets: [], styles: [] };
    return [
      ...(dependencies.length ? ['resolve-imported-design-foundation'] : []),
      ...(expanded > 0 ? ['materialize-shared-prefab-instances'] : []),
    ];
  }
}

function resolveParamValues(
  value: unknown,
  params: ZuiTable,
  key = '',
): unknown {
  if (typeof value === 'string') {
    const reference = value.match(/^\$param\.([A-Za-z_][\w]*)$/);
    if (reference) {
      if (params[reference[1]] === undefined)
        throw new Error(`Missing component param ${reference[1]}`);
      return params[reference[1]];
    }
    const expression = value.match(/^(=?)param\.([A-Za-z_][\w]*)$/);
    if (expression && (expression[1] === '=' || key === 'expression')) {
      const resolved = params[expression[2]];
      if (resolved === undefined)
        throw new Error(`Missing binding param ${expression[2]}`);
      return expression[1] === '=' ? resolved : JSON.stringify(resolved);
    }
    if (
      (value.trimStart().startsWith('=') || key === 'expression') &&
      /\bparam\./.test(value)
    )
      throw new Error(
        `Compound component binding requires runtime expression compilation: ${value}`,
      );
    return value;
  }
  if (Array.isArray(value))
    return value.map((item) => resolveParamValues(item, params, key));
  if (value && typeof value === 'object')
    return Object.fromEntries(
      Object.entries(value).map(([name, item]) => [
        name,
        resolveParamValues(item, params, name),
      ]),
    );
  return value;
}

function contained(root: string, path: string): boolean {
  const value = relative(root, path);
  return !isAbsolute(value) && value !== '..' && !value.startsWith(`..${sep}`);
}

function ownerAssetRoot(path: string): string {
  return path.split('/assets/')[0] + '/assets';
}

function validParamType(type: string, value: unknown): boolean {
  const kind = type.trim().toLowerCase();
  if (kind === 'any') return true;
  if (['bool', 'boolean'].includes(kind)) return typeof value === 'boolean';
  if (['int', 'integer'].includes(kind)) return Number.isSafeInteger(value);
  if (['float', 'number'].includes(kind))
    return typeof value === 'number' && Number.isFinite(value);
  if (
    [
      'string',
      'text',
      'color',
      'asset_ref',
      'assetref',
      'asset',
      'instance_ref',
      'instanceref',
      'instance',
      'enum',
    ].includes(kind)
  )
    return typeof value === 'string';
  if (['array', 'collection'].includes(kind)) return Array.isArray(value);
  if (kind === 'flags')
    return (
      Array.isArray(value) && value.every((item) => typeof item === 'string')
    );
  if (['map', 'object'].includes(kind))
    return !!value && typeof value === 'object' && !Array.isArray(value);
  if (/^vec[234]$/.test(kind))
    return (
      Array.isArray(value) &&
      value.length === Number(kind.slice(3)) &&
      value.every((item) => typeof item === 'number' && Number.isFinite(item))
    );
  return false;
}

function mergeLayout(
  base: ZuiTable | undefined,
  patch: ZuiTable | undefined,
): ZuiTable {
  const result = { ...base };
  for (const [key, value] of Object.entries(patch ?? {})) {
    const original = result[key];
    result[key] =
      typeof value === 'object' && !Array.isArray(value) && value !== null
        ? mergeLayout(object(original), object(value))
        : value;
  }
  return result;
}

function strings(value: unknown): string[] {
  return Array.isArray(value)
    ? value.filter((item): item is string => typeof item === 'string')
    : [];
}

function object(value: unknown): ZuiTable {
  return value && typeof value === 'object' && !Array.isArray(value)
    ? (value as ZuiTable)
    : {};
}

interface SourceIdentity {
  sourcePath: string;
  sourceNodeId: string;
  instancePath: string;
}

function annotateSourceIdentity(
  node: ZuiNode,
  sourcePath: string,
  sourceNodeId: string,
  instancePath: string,
): void {
  if (!node['penpot_review_source_path'])
    node['penpot_review_source_path'] = sourcePath;
  if (!node['penpot_review_source_node_id'])
    node['penpot_review_source_node_id'] = sourceNodeId;
  if (!node['penpot_review_instance_path'])
    node['penpot_review_instance_path'] = instancePath;
}

function setSourceIdentity(
  node: ZuiNode,
  sourcePath: string,
  sourceNodeId: string,
  instancePath: string,
): void {
  node['penpot_review_source_path'] = sourcePath;
  node['penpot_review_source_node_id'] = sourceNodeId;
  node['penpot_review_instance_path'] = instancePath;
}

function sourceIdentity(
  node: ZuiNode,
  fallbackSourcePath: string,
  fallbackSourceNodeId: string,
): SourceIdentity {
  if (node['penpot_review_generated_product_row'])
    return { sourcePath: '', sourceNodeId: '', instancePath: '' };
  const sourcePath =
    typeof node['penpot_review_source_path'] === 'string' &&
    node['penpot_review_source_path']
      ? node['penpot_review_source_path']
      : fallbackSourcePath;
  const sourceNodeId =
    typeof node['penpot_review_source_node_id'] === 'string' &&
    node['penpot_review_source_node_id']
      ? node['penpot_review_source_node_id']
      : fallbackSourceNodeId;
  const instancePath =
    typeof node['penpot_review_instance_path'] === 'string'
      ? node['penpot_review_instance_path']
      : '[]';
  return { sourcePath, sourceNodeId, instancePath };
}

function appendInstancePath(caller: SourceIdentity): string {
  if (!caller.instancePath) return '';
  let steps: unknown;
  try {
    steps = JSON.parse(caller.instancePath) as unknown;
  } catch {
    return '';
  }
  if (!Array.isArray(steps)) return '';
  return JSON.stringify([
    ...steps,
    { sourcePath: caller.sourcePath, sourceNodeId: caller.sourceNodeId },
  ]);
}

function themeTokens(document: ZuiDocument): ZuiTable {
  if (document.asset.kind !== 'theme_tokens') return {};
  const tokens: ZuiTable = {};
  const palette = object(document['palette']);
  // EditorDesignTokens::to_theme_document and UiThemeRegistry::resolve_token_name.
  if (Array.isArray(palette['surface']))
    palette['surface'].forEach((color, index) => {
      tokens[`theme.palette.surface.${index}`] = color;
    });
  for (const [source, target] of Object.entries({
    text_primary: 'text.primary',
    text_secondary: 'text.secondary',
    text_disabled: 'text.disabled',
    accent: 'accent',
    success: 'success',
    info: 'info',
    warning: 'warning',
    error: 'error',
    border: 'separator',
  })) {
    if (palette[source] !== undefined)
      tokens[`theme.palette.${target}`] = palette[source];
  }
  const names = object(document['names']);
  if (Object.keys(names).length > 0) {
    for (const [section, mappings] of Object.entries(names)) {
      const values = object(document[section]);
      for (const [key, target] of Object.entries(object(mappings))) {
        const surfaceIndex = key.match(/^surface_(\d+)$/);
        const value =
          surfaceIndex && Array.isArray(values['surface'])
            ? values['surface'][Number(surfaceIndex[1])]
            : values[key];
        if (typeof target === 'string' && value !== undefined)
          tokens[target] = value;
      }
    }
    return tokens;
  }
  for (const [key, value] of Object.entries(palette)) {
    if (key === 'surface' && Array.isArray(value)) {
      value.forEach((color, index) => {
        tokens[`editor.surface.${index}`] = color;
      });
    } else {
      const semantic = /^(success|info|warning|error)(_|$)/.test(key)
        ? 'semantic.'
        : '';
      tokens[`editor.${semantic}${key.replaceAll('_', '.')}`] = value;
    }
  }
  for (const [key, value] of Object.entries(object(document['typography']))) {
    const suffix = /_(size|weight)$/.test(key)
      ? key.replace(/_(size|weight)$/, '.$1')
      : key;
    tokens[`editor.typography.${suffix}`] = value;
  }
  for (const [key, value] of Object.entries(object(document['controls']))) {
    const [name, unit] = key.split('_');
    const suffix =
      unit === 'height' || unit === 'radius' ? `${unit}.${name}` : key;
    tokens[`editor.control.${suffix}`] = value;
  }
  for (const section of ['chrome', 'density']) {
    for (const [key, value] of Object.entries(object(document[section]))) {
      tokens[`editor.${section}.${key}`] = value;
    }
  }
  return tokens;
}
