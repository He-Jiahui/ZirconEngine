import type { Board, LibraryComponent, Shape } from '@penpot/plugin-types';
import type { ZuiDocument } from './bridge/zui-document';
import {
  ZUI_METADATA_NAMESPACE,
  ZUI_METADATA_NODE_ID,
  ZUI_METADATA_ROLE,
  ZUI_ROLE_NODE,
} from './metadata';
import {
  copyNativeComponentAppearance,
  nativeComponentStructure,
  shapeChildren,
  supportsNativeComponentCopy,
} from './penpot-native-component-copy';
import {
  resolveNativeComponentOwnership,
  type NativeComponentSet,
} from './penpot-native-component-ownership';
const componentSets = new Map<string, NativeComponentSet>();
const NATIVE_COMPONENT_CACHE_KEY = 'component-cache-key';
const NATIVE_COMPONENT_INSTANCE_BATCH = 8;

export type NativeComponentMaterializationMode = 'full' | 'leaf' | 'semantic';

export function prefabSourceForNode(
  document: ZuiDocument,
  nodeId: string,
): string | null {
  const node = document.nodes?.[nodeId];
  if (!node || node.children?.length || node.component === 'Slot') return null;
  if (typeof node['penpot_prefab_source'] === 'string')
    return node['penpot_prefab_source'];
  if (document.asset.kind !== 'component') return null;
  const definitions = Object.entries(document.components ?? {}).filter(
    ([, definition]) => definition.root === nodeId,
  );
  return definitions.length === 1
    ? `${document.asset.id}#${definitions[0][0]}`
    : null;
}

export async function materializeNativeComponents(
  asset: Board,
  document: ZuiDocument,
  onProgress?: (progress: {
    index: number;
    total: number;
    nodeId: string;
    source: string;
    createdMaster: boolean;
  }) => void,
  options: {
    mode?: NativeComponentMaterializationMode;
  } = {},
): Promise<void> {
  if (options.mode === 'semantic') {
    // A large dynamic host can contain hundreds of component instances. Keep
    // the complete semantic projection and its `penpot_prefab_source`
    // metadata, but defer Penpot library promotion to an explicit component
    // review. This is a design-only performance policy; product imports use
    // the default full mode and therefore never lose native instances.
    asset.setSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      'native-components-deferred',
      'semantic-review',
    );
    return;
  }
  const boards: Array<{ board: Board; source: string }> = [];
  const visit = (shape: Shape): void => {
    if (
      shape.type === 'board' &&
      !shape.hidden &&
      shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, ZUI_METADATA_ROLE) ===
        ZUI_ROLE_NODE
    ) {
      const source = prefabSourceForNode(
        document,
        shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, ZUI_METADATA_NODE_ID),
      );
      if (source && supportsNativeComponentCopy(shape)) {
        if (options.mode === 'leaf' && shapeChildren(shape).length > 0) {
          // Composite prefab roots remain ordinary semantic boards in the
          // review projection. Their authored source and child mappings stay
          // intact; only primitive leaves are promoted to native instances.
          // This is intentionally opt-in for the visual harness because a
          // large retained shell can otherwise make the official Penpot SVG
          // renderer materialize the same deep tree several times.
          shape.setSharedPluginData(
            ZUI_METADATA_NAMESPACE,
            'native-component-deferred',
            'true',
          );
        } else {
          boards.push({ board: shape, source });
        }
      }
    }
    for (const child of shapeChildren(shape)) visit(child);
  };
  visit(asset);
  if (!boards.length) return;
  const owned =
    componentSets.get(asset.id) ?? createNativeComponentSet(asset, document);
  const cache = nativeComponentCache(owned);
  // Do not read Board.height here. The official plugin host exposes that
  // getter through a lazy proxy which can throw a TDZ error while the shelf is
  // still being registered. The original fixed 32px top inset is stable, and
  // existing masters provide a safe bottom edge when a transferred shelf is
  // extended with a new structure.
  let nextY = nativeShelfNextY(owned.shelf);
  for (const [index, { board, source }] of boards.entries()) {
    const key = nativeComponentCacheKey(source, board);
    let component = cache.get(key);
    let createdMaster = false;
    const nodeId = board.getSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      ZUI_METADATA_NODE_ID,
    );
    onProgress?.({
      index,
      total: boards.length,
      nodeId,
      source,
      createdMaster: false,
    });
    if (!component) {
      const master = board.clone();
      if (master.type !== 'board')
        throw new Error(`Native component master is not a board: ${source}`);
      owned.shelf.appendChild(master);
      if (master.layoutChild) {
        master.layoutChild.horizontalSizing = 'fix';
        master.layoutChild.verticalSizing = 'fix';
      }
      master.x = owned.shelf.x + 24;
      master.y = nextY;
      nextY += master.height + 48;
      owned.shelf.resize(
        Math.max(owned.shelf.width, master.width + 48),
        nextY - owned.shelf.y,
      );
      component = penpot.library.local.createComponent([master]);
      if (!component || !component.mainInstance()?.isComponentMainInstance())
        throw new Error(`Penpot did not create a native component: ${source}`);
      component.name = source.split('#').pop()!;
      component.path = source.includes('workbench')
        ? 'Zircon / Workbench'
        : 'Zircon';
      component.setSharedPluginData(ZUI_METADATA_NAMESPACE, 'source', source);
      component.setSharedPluginData(
        ZUI_METADATA_NAMESPACE,
        'component-owner',
        asset.id,
      );
      component.setSharedPluginData(
        ZUI_METADATA_NAMESPACE,
        NATIVE_COMPONENT_CACHE_KEY,
        key,
      );
      owned.components.push(component);
      cache.set(key, component);
      createdMaster = true;
    }
    const parent = board.parent;
    if (parent?.type !== 'board')
      throw new Error(`Native component parent is not a board: ${board.name}`);
    const insertionIndex = parent.children.findIndex(
      (child) => child.id === board.id,
    );
    const instance = component.instance();
    if (instance.type !== 'board' || !instance.isComponentCopyInstance())
      throw new Error(
        `Penpot did not create a native component instance: ${source}`,
      );
    parent.insertChild(insertionIndex, instance);
    copyNativeComponentAppearance(board, instance);
    instance.setSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      'native-component-source',
      source,
    );
    instance.setSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      'native-component-id',
      component.id,
    );
    board.remove();
    onProgress?.({
      index,
      total: boards.length,
      nodeId,
      source,
      createdMaster,
    });
    // Let Penpot process component-library updates after creating a master,
    // then in bounded batches while inserting ordinary copies. Yielding after
    // every instance makes a dense product page exceed the review timeout.
    if (shouldYieldNativeComponentWork(index, createdMaster))
      await new Promise((resolve) => setTimeout(resolve, 0));
  }
  // The shelf is an implementation detail of the editable component library,
  // not part of the authored asset viewport.  Keep it in the document so
  // native instances and source mappings remain editable, but hide the
  // off-canvas masters from Penpot's retained SVG/layout tree.  Large shells
  // otherwise pay the cost of painting every master on every review frame.
  owned.shelf.hidden = true;
}

export function shouldYieldNativeComponentWork(
  index: number,
  createdMaster: boolean,
): boolean {
  return createdMaster || (index + 1) % NATIVE_COMPONENT_INSTANCE_BATCH === 0;
}

/**
 * Replace a review board without rebuilding the same native component
 * library. The next board receives the existing shelf and rematerializes its
 * semantic nodes as fresh instances, so authored state can still differ per
 * review case while Penpot avoids recreating every master on every viewport.
 */
export function transferNativeComponentAsset(
  previous: Board,
  next: Board,
  options: { removePrevious?: boolean } = {},
): void {
  const owned =
    componentSets.get(previous.id) ?? resolveNativeComponentOwnership(previous);
  if (options.removePrevious !== false) previous.remove();
  componentSets.delete(previous.id);
  if (!owned) return;
  owned.shelf.setSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    'component-owner',
    next.id,
  );
  for (const component of owned.components)
    component.setSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      'component-owner',
      next.id,
    );
  next.setSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    'component-shelf',
    owned.shelf.id,
  );
  componentSets.set(next.id, owned);
}

/**
 * Undo a transfer performed with `removePrevious: false`. Review-case
 * rendering can fail after native instances are materialized; in that case
 * the authored board and its component shelf must remain usable for the next
 * case instead of being deleted with the failed projection.
 */
export function rollbackNativeComponentAssetTransfer(
  previous: Board,
  next: Board,
): void {
  const owned =
    componentSets.get(next.id) ?? resolveNativeComponentOwnership(next);
  if (!owned) {
    next.remove();
    return;
  }
  componentSets.delete(next.id);
  owned.shelf.setSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    'component-owner',
    previous.id,
  );
  for (const component of owned.components)
    component.setSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      'component-owner',
      previous.id,
    );
  previous.setSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    'component-shelf',
    owned.shelf.id,
  );
  next.setSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    'component-shelf',
    '',
  );
  next.remove();
  componentSets.set(previous.id, owned);
}

export function removeNativeComponentAsset(asset: Board): void {
  const owned =
    componentSets.get(asset.id) ?? resolveNativeComponentOwnership(asset);
  asset.remove();
  if (!owned) return;
  for (const component of owned.components)
    if ('remove' in component && typeof component.remove === 'function')
      component.remove();
  owned.shelf.remove();
  componentSets.delete(asset.id);
}

function createNativeComponentSet(
  asset: Board,
  document: ZuiDocument,
): NativeComponentSet {
  const shelf = penpot.createBoard();
  shelf.name = `ZUI components / ${document.asset.display_name ?? document.asset.id}`;
  shelf.x = asset.x + asset.width + 48;
  shelf.y = asset.y;
  shelf.resize(480, 48);
  shelf.fills = [];
  shelf.clipContent = false;
  shelf.setSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    'component-owner',
    asset.id,
  );
  const owned: NativeComponentSet = { shelf, components: [] };
  componentSets.set(asset.id, owned);
  asset.setSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    'component-shelf',
    shelf.id,
  );
  return owned;
}

function nativeComponentCache(
  owned: NativeComponentSet,
): Map<string, LibraryComponent> {
  const cache = new Map<string, LibraryComponent>();
  for (const component of owned.components) {
    const key = component.getSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      NATIVE_COMPONENT_CACHE_KEY,
    );
    if (key) cache.set(key, component);
  }
  return cache;
}

function nativeComponentCacheKey(source: string, board: Board): string {
  return JSON.stringify([source, nativeComponentStructure(board)]);
}

function nativeShelfNextY(shelf: Board): number {
  let nextY = shelf.y + 32;
  for (const child of shapeChildren(shelf))
    nextY = Math.max(nextY, child.y + child.height + 48);
  return nextY;
}
