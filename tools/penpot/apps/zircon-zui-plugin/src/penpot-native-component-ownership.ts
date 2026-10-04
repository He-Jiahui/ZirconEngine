import type { Board, LibraryComponent, Shape } from '@penpot/plugin-types';
import { ZUI_METADATA_NAMESPACE } from './metadata';

export interface NativeComponentSet {
  shelf: Board;
  components: LibraryComponent[];
}

export function resolveNativeComponentOwnership(
  asset: Board,
): NativeComponentSet | null {
  const shelfId = asset.getSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    'component-shelf',
  );
  if (!shelfId) return null;
  const shelf = penpot.currentPage?.getShapeById(shelfId);
  if (!shelf) return null;
  if (
    shelf.type !== 'board' ||
    shelf.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'component-owner') !==
      asset.id
  )
    throw new Error(`Native component shelf ownership differs: ${shelfId}`);
  const components = penpot.library.local.components.filter((component) => {
    const owner = component.getSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      'component-owner',
    );
    if (owner && owner !== asset.id) return false;
    let current: Shape | null = component.mainInstance();
    while (current && current.id !== shelf.id) current = current.parent;
    return (
      current !== null &&
      Boolean(component.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'source'))
    );
  });
  return { shelf, components };
}
