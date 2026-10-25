import type { Board, LibraryComponent } from '@penpot/plugin-types';
import type { ZuiDocument } from './bridge/zui-document';
import { createCanonicalComponentMaster, resolveCanonicalComponent, migrateCanonicalSearchFieldWidth } from './penpot-canonical-library';
import { encodeZuiMetadata } from './bridge/zui-document';

describe('canonical component library mains', () => {
  afterEach(() => vi.unstubAllGlobals());
  function fixture() {
    const data = (values: Record<string, string> = {}) => ({
      getSharedPluginData: (_ns: string, key: string) => values[key] ?? '',
      setSharedPluginData: (_ns: string, key: string, value: string) => { values[key] = value; },
    });
    const slot = { id: 'slot-shape', type: 'board', children: [], ...data({role: 'node', 'node-id': 'value_slot'}) };
    let isMain = false;
    const root = { id: 'authored-root', type: 'board', children: [slot],
      ...data({role: 'node', 'node-id': 'root'}),
      isComponentMainInstance: () => isMain, isComponentCopyInstance: () => false };
    const asset = { id: 'asset', type: 'board', children: [root], ...data() } as unknown as Board;
    const component = { id: 'actual-component-id', libraryId: 'actual-file-id', ...data(),
      mainInstance: () => root, remove: vi.fn() } as unknown as LibraryComponent & {remove: ReturnType<typeof vi.fn>};
    const create = vi.fn(() => { isMain = true; return component; });
    const components: LibraryComponent[] = [];
    vi.stubGlobal('penpot', {library: {local: {components, createComponent: create}}});
    const document: ZuiDocument = {
      asset: { kind: 'component', id: 'res://ui/property_row.zui', version: 2 },
      components: {PropertyEditorRow: {root: 'root', slots: {value: {required: true, accepts: ['NumberField']}}}},
      nodes: {root: {component: 'PropertyRow', children: [{node: 'value_slot'}]},
        value_slot: {component: 'Slot', props: {name: 'value'}}},
      penpot_original_source_path: 'zircon_editor/assets/ui/property_row.zui',
    };
    return {asset, root, slot, component, create, components, document};
  }

  function migrationFixture() {
    const current = fixture();
    current.document.components = { WorkbenchSearchInput: { root: 'root' } };
    current.document.nodes!.root.component = 'SearchField';
    current.document.penpot_original_source = 'unchanged source input';
    const record = createCanonicalComponentMaster(current.asset, current.document);
    current.components.push(current.component);
    Object.assign(penpot.library.local, { id: record.libraryId });
    current.asset.setSharedPluginData('dev.zircon.zui', 'role', 'asset');
    current.asset.setSharedPluginData('dev.zircon.zui', 'document', encodeZuiMetadata(current.document));
    Object.assign(current.root, { parent: current.asset, width: 240, height: 32 });
    current.root.setSharedPluginData('dev.zircon.zui', 'native-layout-allocation', JSON.stringify({
      kind: 'anchored', parent: current.asset.id, width: 240, height: 32,
    }));
    return current;
  }

  it('does not admit a source-less foreign copy to the canonical migration', async () => {
    const current = fixture();
    const before = JSON.stringify(current.document);
    await expect(migrateCanonicalSearchFieldWidth(current.asset, current.document)).resolves.toBe(false);
    expect(JSON.stringify(current.document)).toBe(before);
    expect(current.create).not.toHaveBeenCalled();
  });

  it('rejects source-owner drift before geometry or native constraints are written', async () => {
    const current = migrationFixture();
    current.document.penpot_original_source_path = 'another-owner/search.zui';
    await expect(migrateCanonicalSearchFieldWidth(current.asset, current.document)).rejects.toThrow('source owner');
    expect(current.root.children).toEqual([current.slot]);
    expect(current.component.remove).not.toHaveBeenCalled();
  });

  it('rejects the copied allocation parent instead of rebasing it into admission', async () => {
    const current = migrationFixture();
    const allocation = JSON.stringify({ kind: 'anchored', parent: 'original-main-board', width: 240, height: 32 });
    current.root.setSharedPluginData('dev.zircon.zui', 'native-layout-allocation', allocation);
    await expect(migrateCanonicalSearchFieldWidth(current.asset, current.document)).rejects.toThrow('allocation is stale');
    expect(current.root.getSharedPluginData('dev.zircon.zui', 'native-layout-allocation')).toBe(allocation);
  });

  it('requires the explicit font resource before any field-part migration', async () => {
    const current = migrationFixture();
    await expect(migrateCanonicalSearchFieldWidth(current.asset, current.document)).rejects.toThrow('explicit source font');
    expect(current.root.children).toEqual([current.slot]);
    expect(current.component.remove).not.toHaveBeenCalled();
  });

  it('keeps a composite root and typed slot intact and records actual library identities', () => {
    const f = fixture();
    const original = JSON.stringify(f.document);
    const record = createCanonicalComponentMaster(f.asset, f.document);
    expect(f.create).toHaveBeenCalledWith([f.root]);
    expect(f.root.children).toEqual([f.slot]);
    expect(JSON.stringify(f.document)).toBe(original);
    expect(record).toMatchObject({libraryId: 'actual-file-id', componentId: 'actual-component-id',
      mainInstanceId: 'authored-root', source: 'res://ui/property_row.zui#PropertyEditorRow'});
    expect(JSON.parse(f.asset.getSharedPluginData('dev.zircon.zui', 'canonical-component'))).toEqual(record);
  });

  it('rejects a duplicate canonical source before mutating the document', () => {
    const f = fixture();
    f.components.push({getSharedPluginData: () => 'res://ui/property_row.zui#PropertyEditorRow'} as unknown as LibraryComponent);
    expect(() => createCanonicalComponentMaster(f.asset, f.document)).toThrow('already contains');
    expect(f.create).not.toHaveBeenCalled();
  });

  it('rejects ambiguous definitions and non-component assets', () => {
    const f = fixture();
    f.document.components!['Another'] = {root: 'root'};
    expect(() => createCanonicalComponentMaster(f.asset, f.document)).toThrow('one canonical component');
    f.document.asset.kind = 'view';
    expect(() => createCanonicalComponentMaster(f.asset, f.document)).toThrow('component asset');
    expect(f.create).not.toHaveBeenCalled();
  });

  it('removes a newly created invalid main instead of reporting it as published', () => {
    const f = fixture();
    f.component.mainInstance = () => ({id: 'unexpected-wrapper', isComponentMainInstance: () => true}) as unknown as Board;
    expect(() => createCanonicalComponentMaster(f.asset, f.document)).toThrow('preserve the authored root');
    expect(f.component.remove).toHaveBeenCalledOnce();
    expect(f.asset.getSharedPluginData('dev.zircon.zui', 'canonical-component')).toBe('');
  });

  it('resolves an expanded repository reference to the original foreign component UUID', () => {
    const f = fixture();
    const record = createCanonicalComponentMaster(f.asset, f.document);
    const resolved = resolveCanonicalComponent(
      'zircon_editor/assets/ui/property_row.zui#PropertyEditorRow',
      [{id: record.libraryId, components: [f.component]}],
    );
    expect(resolved?.component).toBe(f.component);
    expect(resolved?.record).toEqual(record);
    expect(f.component.remove).not.toHaveBeenCalled();
  });

  it('does not substitute a component with the same name from another authored owner', () => {
    const f = fixture();
    const record = createCanonicalComponentMaster(f.asset, f.document);
    expect(resolveCanonicalComponent('res://ui/another.zui#PropertyEditorRow',
      [{id: record.libraryId, components: [f.component]}])).toBeNull();
  });

  it('rejects stale main and library identities instead of falling back to a local master', () => {
    const f = fixture();
    const record = createCanonicalComponentMaster(f.asset, f.document);
    expect(() => resolveCanonicalComponent(record.source,
      [{id: 'another-file-id', components: [f.component]}])).toThrow('identity changed');
    f.component.mainInstance = () => ({id: 'recreated-main', isComponentMainInstance: () => true}) as unknown as Board;
    expect(() => resolveCanonicalComponent(record.source,
      [{id: record.libraryId, components: [f.component]}])).toThrow('identity changed');
  });

  it('deduplicates the same native identity exposed in local and connected libraries', () => {
    const f = fixture();
    const record = createCanonicalComponentMaster(f.asset, f.document);
    const library = {id: record.libraryId, components: [f.component]};
    expect(resolveCanonicalComponent(record.source, [library, library])?.record).toEqual(record);
  });

  it('rejects distinct canonical owners for the same source', () => {
    const f = fixture();
    const record = createCanonicalComponentMaster(f.asset, f.document);
    const otherRecord = {...record, libraryId: 'another-file-id', componentId: 'another-component-id'};
    const other = {...f.component, id: otherRecord.componentId, libraryId: otherRecord.libraryId,
      getSharedPluginData: (_ns: string, key: string) => key === 'canonical-source'
        ? record.source : key === 'canonical-record' ? JSON.stringify(otherRecord) : ''};
    expect(() => resolveCanonicalComponent(record.source, [
      {id: record.libraryId, components: [f.component]},
      {id: otherRecord.libraryId, components: [other]},
    ])).toThrow('Ambiguous');
  });

  it('rejects corrupt records and missing mains before materializing an instance', () => {
    const f = fixture();
    const record = createCanonicalComponentMaster(f.asset, f.document);
    f.component.setSharedPluginData('dev.zircon.zui', 'canonical-record', 'null');
    expect(() => resolveCanonicalComponent(record.source,
      [{id: record.libraryId, components: [f.component]}])).toThrow('Invalid canonical');
    f.component.setSharedPluginData('dev.zircon.zui', 'canonical-record', JSON.stringify(record));
    f.component.mainInstance = () => undefined as unknown as Board;
    expect(() => resolveCanonicalComponent(record.source,
      [{id: record.libraryId, components: [f.component]}])).toThrow('identity changed');
  });
});
