import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { cloneZuiDocument, parseZuiDocument, type ZuiDocument, type ZuiNode } from '../src/bridge/zui-document';

export const UNREAL_DOCK_HEADER_SOURCE = 'zircon_editor/assets/ui/editor/unreal/components/dock_header.zui';
const HEADER_ASSET = 'res://ui/editor/unreal/components/dock_header.zui';
type RecordValue = Record<string, unknown>;
export interface UnrealDockHeaderObservation extends RecordValue {
  kind: 'document' | 'bottom' | 'floating' | 'nativeFloating';
  surfaceKey: string;
  assetId: string;
  sourcePath: string;
  sourceDocument: ZuiDocument;
  viewport: { width: number; height: number };
  mount: { region: RecordValue; header: RecordValue };
  sourceFingerprints: Array<{ sourcePath: string; sha256: string }>;
  rows: Array<RecordValue>;
}

function record(value: unknown): value is RecordValue {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}
function requireValue(condition: unknown, reason: string): asserts condition {
  if (!condition) throw new Error(reason);
}
function fields(value: RecordValue, names: readonly string[], label: string): void {
  requireValue(Object.keys(value).length === names.length &&
    names.every((name) => Object.hasOwn(value, name)), label + ' has unsupported or missing fields');
}
function rectangle(value: unknown, label: string): void {
  requireValue(record(value), label + ' must be an observed rectangle');
  fields(value, ['x', 'y', 'width', 'height'], label);
  requireValue(['x', 'y', 'width', 'height'].every((key) => typeof value[key] === 'number' && Number.isFinite(value[key])) &&
    (value['width'] as number) >= 0 && (value['height'] as number) >= 0, label + ' is invalid');
}

async function projectActualBottomTabIcons(
  repoRoot: string,
  presentation: RecordValue,
  observation: UnrealDockHeaderObservation,
  document: ZuiDocument,
): Promise<void> {
  if (observation.kind !== 'bottom') return;
  const owner = 'zircon_editor/assets/ui/editor/unreal/level_editor_window.zui';
  const fingerprint = presentation['sourceFingerprint'];
  requireValue(presentation['schema'] === 'dev.zircon.editor.workbench-presentation' &&
    presentation['version'] === 1 && record(fingerprint) && fingerprint['sourcePath'] === owner &&
    typeof presentation['stateFingerprint'] === 'string' && /^[0-9a-f]{64}$/.test(presentation['stateFingerprint']) &&
    fingerprint['sha256'] === createHash('sha256').update(await readFile(resolve(repoRoot, owner))).digest('hex'),
    'Actual bottom tab icon has no current Workbench source owner');
  requireValue(observation.surfaceKey === 'bottom' && Array.isArray(presentation['drawers']),
    'Actual bottom tab icon has no matching drawer surface');
  const drawers = presentation['drawers'].filter((drawer) => record(drawer) && drawer['slot'] === 'bottom');
  requireValue(drawers.length === 1 && record(drawers[0]), 'Actual bottom tab icon drawer is ambiguous');
  const drawer = drawers[0];
  const tabs = drawer['tabs'];
  const groups = document.nodes!['dock_tab_strip'].children!;
  requireValue(drawer['visible'] === true && Array.isArray(tabs) && tabs.length === groups.length &&
    tabs.length > 0 && tabs.length <= 3, 'Actual bottom tab icon count differs from consumed Source slots');
  const assets: Record<string, string> = {
    console: 'ionicons/terminal-outline.svg',
    'runtime-diagnostics': 'ionicons/grid-outline.svg',
    'build-export': 'ionicons/share-outline.svg',
  };
  const ids = new Set<string>();
  for (const [index, tab] of tabs.entries()) {
    requireValue(record(tab) && typeof tab['id'] === 'string' && !!tab['id'] && !ids.has(tab['id']) &&
      typeof tab['descriptorId'] === 'string' && !!tab['descriptorId'] && typeof tab['title'] === 'string' &&
      typeof tab['iconKey'] === 'string' && !!assets[tab['iconKey']],
      'Actual bottom tab icon has missing, duplicate or unsupported business identity');
    ids.add(tab['id']);
    const id = 'dock_tab_' + index;
    const node = document.nodes![id];
    const rows = observation.rows.filter((row) => row['sourceNodeId'] === id);
    const selected = drawer['activeTabId'] === tab['id'];
    requireValue(node?.component === 'Label' && node.control_id === 'DockTab' + index &&
      node.props?.['text'] === tab['title'] && node.props?.['selected'] === selected && rows.length > 0 &&
      rows.every((row) => row['sourcePath'] === UNREAL_DOCK_HEADER_SOURCE &&
        row['controlId'] === node.control_id && Array.isArray(row['instancePath']) &&
        row['instancePath'].length === 0 && row['role'] === 'Label' && row['text'] === tab['title'] &&
        row['selected'] === selected) &&
      document.nodes!['dock_tab_group_' + index].children?.filter((child) => child.node === id).length === 1,
      'Actual bottom tab icon does not match its complete consumed Source identity/title/state');
    node['penpot_review_native_tab_icon'] = {
      authority: 'actual-workbench-bottom-tab', sourcePath: UNREAL_DOCK_HEADER_SOURCE,
      sourceNodeId: id, instancePath: '[]', surfaceKey: observation.surfaceKey, drawerSlot: 'bottom',
      tabId: tab['id'], descriptorId: tab['descriptorId'], iconKey: tab['iconKey'], title: tab['title'], selected,
      ownerSha256: fingerprint['sha256'], stateFingerprint: presentation['stateFingerprint'],
      assetPath: assets[tab['iconKey']], sourceUnits: 'physical-pixels',
      // This is the real Source constructor input, not an observed row rectangle.
      sourceViewport: { ...observation.viewport },
    };
  }
  requireValue(drawer['activeTabId'] === null || ids.has(String(drawer['activeTabId'])),
    'Actual bottom tab active identity is outside the consumed Source');
}

export function readUnrealDockHeaderObservations(value: unknown): UnrealDockHeaderObservation[] {
  if (value === undefined) return []; // Missing static evidence remains unknown.
  requireValue(Array.isArray(value) && value.length <= 16, 'dockHeaders must be a bounded actual observation array');
  const identities = new Set<string>();
  for (const item of value) {
    requireValue(record(item), 'dockHeaders item must be an actual Source observation');
    fields(item, ['kind', 'surfaceKey', 'assetId', 'sourcePath', 'sourceDocument', 'viewport', 'mount', 'sourceFingerprints', 'rows'], 'dockHeaders item');
    requireValue(['document', 'bottom', 'floating', 'nativeFloating'].includes(String(item['kind'])) &&
      typeof item['surfaceKey'] === 'string' && item['surfaceKey'].length <= 1024 &&
      item['assetId'] === HEADER_ASSET && item['sourcePath'] === UNREAL_DOCK_HEADER_SOURCE,
      'dockHeaders source identity is unsupported');
    const identity = String(item['kind']) + ':' + item['surfaceKey'];
    requireValue(!identities.has(identity), 'dockHeaders contains a duplicate native surface');
    identities.add(identity);
    const viewport = item['viewport'];
    requireValue(record(viewport), 'dockHeaders viewport is missing');
    fields(viewport, ['width', 'height'], 'dockHeaders viewport');
    requireValue(typeof viewport['width'] === 'number' && typeof viewport['height'] === 'number' &&
      Number.isFinite(viewport['width']) && Number.isFinite(viewport['height']) &&
      (viewport['width'] as number) > 0 && (viewport['height'] as number) > 0 &&
      (viewport['width'] as number) <= 8192 && (viewport['height'] as number) <= 8192,
      'dockHeaders Source viewport is invalid');
    const mount = item['mount'];
    requireValue(record(mount), 'dockHeaders actual mount is missing');
    fields(mount, ['region', 'header'], 'dockHeaders mount');
    rectangle(mount['region'], 'dockHeaders region');
    rectangle(mount['header'], 'dockHeaders header');
    const document = item['sourceDocument'];
    requireValue(record(document) && record(document['asset']) && document['asset']['id'] === HEADER_ASSET &&
      document['asset']['kind'] === 'view' && document['asset']['version'] === 2 &&
      record(document['root']) && record(document['nodes']) && Object.keys(document['nodes']).length <= 64,
      'dockHeaders actual Source document is invalid');
    const fingerprints = item['sourceFingerprints'];
    requireValue(Array.isArray(fingerprints) && fingerprints.length > 0 && fingerprints.length <= 32,
      'dockHeaders loaded import closure is missing');
    const paths = new Set<string>();
    for (const fingerprint of fingerprints) {
      requireValue(record(fingerprint), 'dockHeaders source fingerprint is invalid');
      fields(fingerprint, ['sourcePath', 'sha256'], 'dockHeaders fingerprint');
      const path = fingerprint['sourcePath'];
      requireValue(typeof path === 'string' && path.startsWith('zircon_editor/assets/') &&
        !path.includes('\\') && !path.split('/').some((part) => !part || part === '.' || part === '..') &&
        typeof fingerprint['sha256'] === 'string' && /^[0-9a-f]{64}$/.test(fingerprint['sha256']) &&
        !paths.has(path), 'dockHeaders source fingerprint is invalid or duplicate');
      paths.add(path);
    }
    requireValue(paths.has(UNREAL_DOCK_HEADER_SOURCE) &&
      paths.has('zircon_editor/assets/ui/editor/unreal/starship.zui'),
      'dockHeaders must contain its real Header and Starship loaded-source closure');
    const nodes = document['nodes'] as RecordValue;
    const sourceRows = item['rows'];
    requireValue(Array.isArray(sourceRows) && sourceRows.length > 0 && sourceRows.length <= 128,
      'dockHeaders consumed rows are missing');
    for (const row of sourceRows) {
      requireValue(record(row), 'dockHeaders consumed row is invalid');
      fields(row, ['controlId', 'sourcePath', 'sourceNodeId', 'instancePath', 'role', 'text', 'selected', 'hovered', 'pressed', 'disabled', 'frame', 'renderRef'], 'dockHeaders row');
      requireValue(typeof row['controlId'] === 'string' && typeof row['sourceNodeId'] === 'string' &&
        row['sourcePath'] === UNREAL_DOCK_HEADER_SOURCE && Array.isArray(row['instancePath']) &&
        row['instancePath'].length === 0 && typeof row['role'] === 'string' &&
        typeof row['text'] === 'string' && Buffer.byteLength(row['text'], 'utf8') <= 4096 &&
        ['selected', 'hovered', 'pressed', 'disabled'].every((key) => typeof row[key] === 'boolean') &&
        record(nodes[row['sourceNodeId']]) &&
        (nodes[row['sourceNodeId']] as RecordValue)['control_id'] === row['controlId'] &&
        (row['renderRef'] === null || record(row['renderRef'])), 'dockHeaders row lacks actual standalone-view Source identity/render ref');
      rectangle(row['frame'], 'dockHeaders Source row frame');
    }
  }
  return value as UnrealDockHeaderObservation[];
}

// Rust's typed source document serializes empty default maps/arrays and optional
// nulls. They carry no authored property and are removed only for shape equality.
function normalized(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(normalized);
  if (!record(value)) return value;
  return Object.fromEntries(Object.entries(value).sort(([a], [b]) => a.localeCompare(b))
    .filter(([, child]) => child !== null &&
      !(Array.isArray(child) && child.length === 0) &&
      !(record(child) && Object.keys(child).length === 0))
    .map(([key, child]) => [key, normalized(child)])
    .filter(([, child]) => !(record(child) && Object.keys(child).length === 0)));
}
function subtree(nodes: Record<string, ZuiNode>, id: string, output: Set<string>): void {
  requireValue(nodes[id] && !output.has(id), 'Header source subtree is missing or cyclic');
  output.add(id);
  for (const child of nodes[id].children ?? []) subtree(nodes, child.node, output);
}

async function verifiedLoadedHeaderSource(
  repoRoot: string,
  fingerprints: UnrealDockHeaderObservation['sourceFingerprints'],
): Promise<ZuiDocument> {
  const documents = new Map<string, ZuiDocument>();
  const sourceByAssetId = new Map<string, string>();
  for (const fingerprint of fingerprints) {
    const bytes = await readFile(resolve(repoRoot, fingerprint.sourcePath));
    requireValue(createHash('sha256').update(bytes).digest('hex') === fingerprint.sha256,
      'Actual Header loaded source changed: ' + fingerprint.sourcePath);
    const document = parseZuiDocument(bytes.toString('utf8')).document;
    requireValue(!sourceByAssetId.has(document.asset.id),
      'Actual Header loaded import closure contains an ambiguous asset: ' + document.asset.id);
    documents.set(fingerprint.sourcePath, document);
    sourceByAssetId.set(document.asset.id, fingerprint.sourcePath);
  }
  const root = documents.get(UNREAL_DOCK_HEADER_SOURCE);
  requireValue(root, 'Actual Header loaded import closure is missing its root');
  const current = cloneZuiDocument(root);
  const queue = [UNREAL_DOCK_HEADER_SOURCE];
  const seen = new Set(queue);
  // Match file_cache.rs collect_v2_sources/build_file_store_cache_entry:
  // root first, breadth-first widgets then styles, imported tokens overwrite,
  // and stylesheets append without theme expansion or stylesheet-ID deduplication.
  for (let index = 0; index < queue.length; index++) {
    const document = documents.get(queue[index])!;
    if (index !== 0) {
      current.tokens = { ...(current.tokens ?? {}), ...(document.tokens ?? {}) };
      current.stylesheets = [...(current.stylesheets ?? []), ...(document.stylesheets ?? [])];
    }
    for (const kind of ['widgets', 'styles']) {
      const references = document.imports?.[kind] ?? [];
      requireValue(Array.isArray(references) && references.every((value) => typeof value === 'string'),
        'Actual Header loaded import references are invalid');
      for (const reference of references as string[]) {
        const source = sourceByAssetId.get(reference.split('#')[0]);
        requireValue(source, 'Actual Header loaded import closure is missing: ' + reference);
        if (!seen.has(source)) {
          seen.add(source);
          queue.push(source);
        }
      }
    }
  }
  requireValue(seen.size === documents.size,
    'Actual Header loaded import closure contains an unrelated source');
  return current;
}

export async function prepareUnrealDockHeaderViews(
  repoRoot: string,
  presentation: RecordValue | undefined,
): Promise<Array<{ observation: UnrealDockHeaderObservation; document: ZuiDocument }>> {
  const observations = readUnrealDockHeaderObservations(presentation?.['dockHeaders']);
  if (!observations.length) return [];
  const prepared = [];
  for (const observation of observations) {
    const current = await verifiedLoadedHeaderSource(repoRoot, observation.sourceFingerprints);
    const nodes = current.nodes!;
    const strip = nodes['dock_tab_strip'];
    requireValue(strip && strip.children?.length === 3, 'Authored Unreal Header slot contract changed');
    const sourceNodes = observation.sourceDocument.nodes!;
    const capturedStrip = sourceNodes['dock_tab_strip'];
    requireValue(capturedStrip && Array.isArray(capturedStrip.children) &&
      capturedStrip.children.length <= 3, 'Actual Header variant has an invalid used count');
    const used = capturedStrip.children.length;
    requireValue(capturedStrip.children.every((child, index) => child.node === 'dock_tab_group_' + index),
      'Actual Header used slots are not contiguous authored groups');
    const removed = new Set<string>();
    for (let index = used; index < 3; index++) subtree(nodes, 'dock_tab_group_' + index, removed);
    for (const id of removed) delete nodes[id];
    strip.children = strip.children!.slice(0, used);
    for (let index = 0; index < used; index++) {
      const row = observation.rows.find((row) => row['controlId'] === 'DockTab' + index);
      requireValue(row, 'Actual Header caption row is missing');
      const active = row['selected'] === true;
      for (const id of ['dock_tab_group_' + index, 'dock_tab_' + index]) {
        nodes[id].props = { ...(nodes[id].props ?? {}), selected: active,
          surface_variant: active ? 'inset' : 'transparent', text_tone: active ? 'default' : 'subtle' };
      }
      nodes['dock_tab_' + index].props!['text'] = row['text'] as string;
    }
    const subtitle = sourceNodes['dock_subtitle']?.props?.['text'];
    requireValue(typeof subtitle === 'string', 'Actual Header subtitle input is missing');
    nodes['dock_subtitle'].props = { ...(nodes['dock_subtitle'].props ?? {}), text: subtitle };
    requireValue(JSON.stringify(normalized(current)) === JSON.stringify(normalized(observation.sourceDocument)),
      'Actual Header source document differs from the current authored used-slot/caption/state variant');
    await projectActualBottomTabIcons(repoRoot, presentation!, observation, current);
    // Preserve the actual retained close filter without including descendants of
    // a selected painted group, which would wrongly restore hidden Close leaves.
    const consumed = new Set(observation.rows.map((row) => String(row['sourceNodeId'])));
    for (const [id, node] of Object.entries(nodes)) {
      if (node.control_id?.startsWith('DockTabClose') && !consumed.has(id)) {
        for (const parent of Object.values(nodes))
          if (parent.children) parent.children = parent.children.filter((child) => child.node !== id);
        delete nodes[id];
      }
    }
    for (const [id, node] of Object.entries(nodes)) {
      const row = observation.rows.find((row) => row['sourceNodeId'] === id);
      if (row) {
        node.state = { ...(node.state ?? {}), hovered: row['hovered'] as boolean,
          pressed: row['pressed'] as boolean, disabled: row['disabled'] as boolean };
      }
      node['penpot_review_source_path'] = UNREAL_DOCK_HEADER_SOURCE;
      node['penpot_review_source_node_id'] = id;
      // This is the actual standalone source-view identity, not an invented component invocation.
      node['penpot_review_instance_path'] = JSON.stringify(row?.['instancePath'] ?? []);
      node['penpot_review_runtime_surface_key'] = observation.surfaceKey;
    }
    prepared.push({ observation, document: current });
  }
  return prepared;
}
