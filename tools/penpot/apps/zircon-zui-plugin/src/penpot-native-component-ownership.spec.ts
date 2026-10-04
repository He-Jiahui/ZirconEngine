import type { Board, LibraryComponent, Shape } from '@penpot/plugin-types';
import {
  removeNativeComponentAsset,
  rollbackNativeComponentAssetTransfer,
  transferNativeComponentAsset,
} from './penpot-native-components';

function metadata(values: Record<string, string>) {
  return (_namespace: string, key: string) => values[key] ?? '';
}

describe('persisted native component ownership', () => {
  afterEach(() => vi.unstubAllGlobals());

  it('cleans a reloaded asset shelf and components without touching unrelated library entries', () => {
    const removed: string[] = [];
    const shelf = {
      id: 'shelf',
      type: 'board',
      parent: null,
      getSharedPluginData: metadata({ 'component-owner': 'asset' }),
      remove: () => removed.push('shelf'),
    } as unknown as Board;
    const asset = {
      id: 'asset',
      getSharedPluginData: metadata({ 'component-shelf': 'shelf' }),
      remove: () => removed.push('asset'),
    } as unknown as Board;
    const component = (id: string, parent: Shape | null) =>
      ({
        id,
        getSharedPluginData: metadata({ source: 'button.zui#Button' }),
        mainInstance: () => ({ id: `${id}-main`, parent }),
        remove: () => removed.push(id),
      }) as unknown as LibraryComponent;
    vi.stubGlobal('penpot', {
      currentPage: { getShapeById: () => shelf },
      library: {
        local: {
          components: [component('owned', shelf), component('other', null)],
        },
      },
    });
    removeNativeComponentAsset(asset);
    expect(removed).toEqual(['asset', 'owned', 'shelf']);
  });

  it('rejects a changed shelf owner before deleting the asset', () => {
    const remove = vi.fn();
    const asset = {
      id: 'asset',
      getSharedPluginData: metadata({ 'component-shelf': 'shelf' }),
      remove,
    } as unknown as Board;
    vi.stubGlobal('penpot', {
      currentPage: {
        getShapeById: () => ({
          type: 'board',
          getSharedPluginData: metadata({ 'component-owner': 'another-asset' }),
        }),
      },
    });
    expect(() => removeNativeComponentAsset(asset)).toThrow(
      'ownership differs',
    );
    expect(remove).not.toHaveBeenCalled();
  });

  it('transfers a live component shelf between review boards before final cleanup', () => {
    const removed: string[] = [];
    const shelfMetadata: Record<string, string> = { 'component-owner': 'before' };
    const componentMetadata: Record<string, string> = {
      source: 'button.zui#Button',
      'component-owner': 'before',
    };
    const assigned: Array<[string, string]> = [];
    const shelf = {
      id: 'shelf',
      type: 'board',
      parent: null,
      getSharedPluginData: metadata(shelfMetadata),
      setSharedPluginData: (_namespace: string, key: string, value: string) => {
        shelfMetadata[key] = value;
        assigned.push([`shelf:${key}`, value]);
      },
      remove: () => removed.push('shelf'),
    } as unknown as Board;
    const component = {
      id: 'owned',
      getSharedPluginData: metadata(componentMetadata),
      setSharedPluginData: (_namespace: string, key: string, value: string) => {
        componentMetadata[key] = value;
        assigned.push([`component:${key}`, value]);
      },
      mainInstance: () => ({ id: 'owned-main', parent: shelf }),
      remove: () => removed.push('owned'),
    } as unknown as LibraryComponent;
    const before = {
      id: 'before',
      getSharedPluginData: metadata({ 'component-shelf': 'shelf' }),
      remove: () => removed.push('before'),
    } as unknown as Board;
    const afterMetadata: Record<string, string> = {};
    const after = {
      id: 'after',
      setSharedPluginData: (_namespace: string, key: string, value: string) => {
        afterMetadata[key] = value;
        assigned.push([`after:${key}`, value]);
      },
      remove: () => removed.push('after'),
    } as unknown as Board;
    vi.stubGlobal('penpot', {
      currentPage: { getShapeById: () => shelf },
      library: { local: { components: [component] } },
    });

    transferNativeComponentAsset(before, after);

    expect(removed).toEqual(['before']);
    expect(assigned).toEqual(
      expect.arrayContaining([
        ['shelf:component-owner', 'after'],
        ['component:component-owner', 'after'],
        ['after:component-shelf', 'shelf'],
      ]),
    );
    removeNativeComponentAsset(after);
    expect(removed).toEqual(['before', 'after', 'owned', 'shelf']);
  });

  it('rolls back a preserved transfer without deleting the previous review board', () => {
    const removed: string[] = [];
    const shelfMetadata: Record<string, string> = {
      'component-owner': 'before-rollback',
    };
    const componentMetadata: Record<string, string> = {
      source: 'button.zui#Button',
      'component-owner': 'before-rollback',
    };
    const shelf = {
      id: 'shelf-rollback',
      type: 'board',
      parent: null,
      getSharedPluginData: metadata(shelfMetadata),
      setSharedPluginData: (_namespace: string, key: string, value: string) => {
        shelfMetadata[key] = value;
      },
      remove: () => removed.push('shelf'),
    } as unknown as Board;
    const component = {
      id: 'owned-rollback',
      getSharedPluginData: metadata(componentMetadata),
      setSharedPluginData: (_namespace: string, key: string, value: string) => {
        componentMetadata[key] = value;
      },
      mainInstance: () => ({ id: 'owned-main', parent: shelf }),
      remove: () => removed.push('owned'),
    } as unknown as LibraryComponent;
    const beforeMetadata: Record<string, string> = {
      'component-shelf': shelf.id,
    };
    const before = {
      id: 'before-rollback',
      getSharedPluginData: metadata(beforeMetadata),
      setSharedPluginData: (_namespace: string, key: string, value: string) => {
        beforeMetadata[key] = value;
      },
      remove: () => removed.push('before'),
    } as unknown as Board;
    const afterMetadata: Record<string, string> = {};
    const after = {
      id: 'after-rollback',
      setSharedPluginData: (_namespace: string, key: string, value: string) => {
        afterMetadata[key] = value;
      },
      remove: () => removed.push('after'),
    } as unknown as Board;
    vi.stubGlobal('penpot', {
      currentPage: { getShapeById: () => shelf },
      library: { local: { components: [component] } },
    });

    transferNativeComponentAsset(before, after, { removePrevious: false });
    rollbackNativeComponentAssetTransfer(before, after);

    expect(removed).toEqual(['after']);
    expect(shelfMetadata['component-owner']).toBe(before.id);
    expect(componentMetadata['component-owner']).toBe(before.id);
    expect(beforeMetadata['component-shelf']).toBe(shelf.id);
    expect(afterMetadata['component-shelf']).toBe('');
  });
});
