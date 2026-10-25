import type { Board, LibraryComponent, Shape } from '@penpot/plugin-types';
import type { ZuiDocument } from './bridge/zui-document';
import { decodeZuiMetadata } from './bridge/zui-document';
import { projectZuiDocument } from './bridge/penpot-projection';
import { bindFontResources, fontResourcesForShape, selectRegisteredFont } from './bridge/penpot-font-selection';
import { planSearchFieldWidthConstraints } from './penpot-field-controls';
import { migrateSearchFieldWidthGuard } from './penpot-export-guard';
import { settleAfterLayoutRefresh } from './penpot-layout-settlement';
import { ZUI_METADATA_NAMESPACE, ZUI_METADATA_NODE_ID, ZUI_METADATA_ROLE, ZUI_ROLE_NODE } from './metadata';
import { shapeChildren } from './penpot-native-component-copy';

export interface CanonicalComponentRecord {
  version: 1;
  source: string;
  sourcePath?: string;
  libraryId: string;
  componentId: string;
  mainInstanceId: string;
  assetBoardId: string;
}

/** Resolve published identities, never a display name or a file-local imitation. */
export function resolveCanonicalComponent(
  source: string,
  libraries: ReadonlyArray<{
    id: string;
    components: ReadonlyArray<LibraryComponent>;
  }>,
): { component: LibraryComponent; record: CanonicalComponentRecord } | null {
  const matches = new Map<string, { component: LibraryComponent; record: CanonicalComponentRecord }>();
  for (const library of libraries) {
    for (const component of library.components) {
      const canonicalSource = component.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'canonical-source');
      if (!canonicalSource) continue;
      const serialized = component.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'canonical-record');
      let record: CanonicalComponentRecord;
      try { record = JSON.parse(serialized) as CanonicalComponentRecord; }
      catch { throw new Error(`Invalid canonical component record: ${component.id}`); }
      if (!record || typeof record !== 'object' || Array.isArray(record))
        throw new Error(`Invalid canonical component record: ${component.id}`);
      const name = canonicalSource.slice(canonicalSource.lastIndexOf('#') + 1);
      const authoredReference = record.sourcePath ? `${record.sourcePath}#${name}` : null;
      if (source !== canonicalSource && source !== authoredReference) continue;
      const main = component.mainInstance();
      if (!main || record.version !== 1 || record.source !== canonicalSource ||
          record.libraryId !== library.id || component.libraryId !== library.id ||
          record.componentId !== component.id || record.mainInstanceId !== main.id ||
          !main.isComponentMainInstance())
        throw new Error(`Canonical component identity changed: ${source}`);
      const identity = `${library.id}/${component.id}`;
      const previous = matches.get(identity);
      if (previous && JSON.stringify(previous.record) !== JSON.stringify(record))
        throw new Error(`Canonical component identity changed: ${source}`);
      matches.set(identity, { component, record });
    }
  }
  if (matches.size > 1)
    throw new Error(`Ambiguous canonical component source: ${source}`);
  return matches.values().next().value ?? null;
}

/** Promote the authored root itself, including its children and typed slots. */
export function createCanonicalComponentMaster(
  asset: Board,
  document: ZuiDocument,
): CanonicalComponentRecord {
  if (document.asset.kind !== 'component')
    throw new Error('A library master requires a ZUI component asset.');
  const definitions = Object.entries(document.components ?? {});
  if (definitions.length !== 1)
    throw new Error('Import one canonical component definition at a time.');
  const [name, definition] = definitions[0];
  if (!document.nodes?.[definition.root])
    throw new Error(`Missing canonical component root: ${definition.root}`);
  const source = `${document.asset.id}#${name}`;
  if (penpot.library.local.components.some((entry) =>
    entry.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'canonical-source') === source))
    throw new Error(`The library already contains ${source}; edit its existing main component.`);
  const roots: Shape[] = [];
  const visit = (shape: Shape): void => {
    if (shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, ZUI_METADATA_ROLE) === ZUI_ROLE_NODE &&
        shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, ZUI_METADATA_NODE_ID) === definition.root)
      roots.push(shape);
    for (const child of shapeChildren(shape)) visit(child);
  };
  visit(asset);
  if (roots.length !== 1 || roots[0].type !== 'board')
    throw new Error(`Expected one editable canonical root: ${definition.root}`);
  const root = roots[0];
  if (root.isComponentMainInstance() || root.isComponentCopyInstance())
    throw new Error('Import the canonical source as a new component master.');
  let component: LibraryComponent | undefined;
  try {
    component = penpot.library.local.createComponent([root]);
    const main = component.mainInstance();
    if (main.id !== root.id || !main.isComponentMainInstance())
      throw new Error('Penpot did not preserve the authored root as the component main.');
    component.name = name;
    component.path = 'Zircon / Workbench';
    const sourcePath = document['penpot_original_source_path'];
    const record: CanonicalComponentRecord = {
      version: 1, source,
      ...(typeof sourcePath === 'string' ? { sourcePath } : {}),
      libraryId: component.libraryId, componentId: component.id,
      mainInstanceId: main.id, assetBoardId: asset.id,
    };
    component.setSharedPluginData(ZUI_METADATA_NAMESPACE, 'canonical-source', source);
    component.setSharedPluginData(ZUI_METADATA_NAMESPACE, 'canonical-record', JSON.stringify(record));
    asset.setSharedPluginData(ZUI_METADATA_NAMESPACE, 'canonical-component', JSON.stringify(record));
    return record;
  } catch (error) {
    if (component && 'remove' in component && typeof component.remove === 'function') component.remove();
    throw error;
  }
}


/** Upgrade the existing local canonical main; never a linked consumer copy. */
export async function migrateCanonicalSearchFieldWidth(
  asset: Board, document: ZuiDocument,
): Promise<boolean> {
  const encoded = asset.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'canonical-component');
  if (!encoded) return false;
  const definitions = Object.entries(document.components ?? {});
  if (definitions.length !== 1 || definitions[0][0] !== 'WorkbenchSearchInput') return false;
  const [name, definition] = definitions[0];
  const canonical = resolveCanonicalComponent(`${document.asset.id}#${name}`, [penpot.library.local]);
  const saved = decodeZuiMetadata(asset.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'document'));
  if (!canonical || asset.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'role') !== 'asset' ||
      JSON.stringify(canonical.record) !== encoded || canonical.record.assetBoardId !== asset.id ||
      document.asset.kind !== 'component' || document.asset.version !== 2 ||
      canonical.record.sourcePath !== document['penpot_original_source_path'] ||
      saved['penpot_original_source_path'] !== document['penpot_original_source_path'] ||
      typeof saved['penpot_original_source'] !== 'string' ||
      saved['penpot_original_source'] !== document['penpot_original_source'] ||
      JSON.stringify(saved.nodes) !== JSON.stringify(document.nodes) ||
      JSON.stringify(saved.components) !== JSON.stringify(document.components))
    throw new Error('Canonical SearchField source owner or document changed.');
  const main = canonical.component.mainInstance();
  if (main.type !== 'board' || main.parent?.id !== asset.id ||
      main.getSharedPluginData(ZUI_METADATA_NAMESPACE, ZUI_METADATA_NODE_ID) !== definition.root ||
      main.getSharedPluginData(ZUI_METADATA_NAMESPACE, ZUI_METADATA_ROLE) !== ZUI_ROLE_NODE ||
      main.isComponentCopyInstance())
    throw new Error('Canonical SearchField main or parent changed.');
  const allocation = JSON.parse(main.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'native-layout-allocation')) as
    { parent?: string; kind?: string; width?: number; height?: number };
  if (!allocation || allocation.parent !== asset.id || allocation.kind !== 'anchored' ||
      typeof allocation.width !== 'number' || !Number.isFinite(allocation.width) ||
      typeof allocation.height !== 'number' || !Number.isFinite(allocation.height) ||
      Math.abs(allocation.width - main.width) > 0.001 || Math.abs(allocation.height - main.height) > 0.001)
    throw new Error('Canonical SearchField allocation is stale.');
  const fonts = fontResourcesForShape(asset);
  if (!fonts) throw new Error('Canonical SearchField requires its explicit source font resource.');
  bindFontResources(saved, fonts);
  const projected = projectZuiDocument(saved).shapes.find((entry) => entry.nodeId === definition.root);
  if (!projected?.field || !projected.text)
    throw new Error('Canonical SearchField source projection is incomplete.');
  const changes = planSearchFieldWidthConstraints(main, projected.field);
  const text = changes[1].shape;
  const registered = selectRegisteredFont(projected.text.fontFamily, projected.text.fontWeight ?? '400', 'normal', fonts);
  if (text.type !== 'text' || !registered || text.fontId !== registered.font.fontId ||
      text.fontVariantId !== registered.variant.fontVariantId || text.fontFamily !== projected.text.fontFamily ||
      text.fontWeight !== registered.variant.fontWeight || text.fontStyle !== 'normal' ||
      projected.text.fontSize === null || !Number.isFinite(Number(text.fontSize)) ||
      Math.abs(Number(text.fontSize) - projected.text.fontSize) > 0.001 ||
      !Number.isFinite(Number(text.lineHeight)) ||
      Math.abs(Number(text.lineHeight) - projected.text.lineHeight) > 0.001)
    throw new Error('Canonical SearchField native font differs from its admitted resource.');
  return migrateSearchFieldWidthGuard(asset, changes, () => settleAfterLayoutRefresh(
    () => {},
    () => changes.map(({ shape }) => ({ id: shape.id, horizontal: shape.constraintsHorizontal,
      vertical: shape.constraintsVertical, x: shape.x, y: shape.y, width: shape.width, height: shape.height })),
    { intervalMs: 100, quietMs: 700, timeoutMs: 10_000, numericPrecision: 3 },
  ));
}
