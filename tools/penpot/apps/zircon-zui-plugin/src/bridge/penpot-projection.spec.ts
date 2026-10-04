import fixtureSource from './roundtrip-fixture.zui?raw';
import { parseZuiDocument, zuiNodes } from './zui-document';
import {
  cloneProjectionSnapshot,
  projectZuiDocument,
  reconcileZuiDocument,
} from './penpot-projection';

describe('Penpot projection bridge', () => {
  it('hides a closed root dialog and lets its explicit state override authored open props', () => {
    const { document } = parseZuiDocument(
      '[asset]\nkind = "component"\nid = "res://review/dialog.zui"\nversion = 2\n\n[components.ReviewDialog]\nroot = "dialog"\n\n[nodes.dialog]\ncomponent = "Dialog"\nprops = { open = false, popup_open = false, title = "Scene Settings" }\n',
    );
    const dialog = zuiNodes(document)['dialog'];
    expect(projectZuiDocument(document).rootNodes[0].previewHidden).toBe(true);

    dialog.state = { open: true, popup_open: true };
    expect(projectZuiDocument(document).rootNodes[0].previewHidden).toBe(false);

    dialog.props = { ...dialog.props, open: true, popup_open: true };
    dialog.state = { open: false, popup_open: false };
    expect(projectZuiDocument(document).rootNodes[0].previewHidden).toBe(true);
  });

  it.each(['DropdownPopup', 'ContextMenu', 'ContextActionMenu'])(
    'honours the authored closed visibility contract for %s',
    (component) => {
      const { document } = parseZuiDocument(
        `[asset]\nkind = "component"\nid = "res://review/${component}.zui"\nversion = 2\n\n[components.ReviewPopup]\nroot = "popup"\n\n[nodes.popup]\ncomponent = "${component}"\nprops = { open = false, popup_open = false, options = ["Scene"] }\n`,
      );
      const popup = zuiNodes(document)['popup'];
      expect(projectZuiDocument(document).rootNodes[0].previewHidden).toBe(
        true,
      );
      popup.state = { open: true, popup_open: true };
      expect(projectZuiDocument(document).rootNodes[0].previewHidden).toBe(
        false,
      );
      popup.state = { open: false, popup_open: false };
      expect(projectZuiDocument(document).rootNodes[0].previewHidden).toBe(
        true,
      );
    },
  );

  it('keeps layout-group state values as metadata while painting the child label', () => {
    const { document } = parseZuiDocument(fixtureSource);
    const root = zuiNodes(document)['root'];
    root.component = 'VerticalGroup';
    root.props = { value: 'X 128.4 Y 64.2 Z -32.7' };
    expect(projectZuiDocument(document).rootNodes[0].text).toBeNull();
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
    expect(
      reconcileZuiDocument(document, snapshot).document.nodes!['root'].props,
    ).toEqual(root.props);
    root.props['text'] = 'Transform';
    expect(projectZuiDocument(document).rootNodes[0].text?.characters).toBe(
      'Transform',
    );
    delete root.props['text'];
    root['widget'] = { value_property: 'value' };
    expect(projectZuiDocument(document).rootNodes[0].text?.characters).toBe(
      'X 128.4 Y 64.2 Z -32.7',
    );
  });

  it('projects the semantic tree and keeps detached nodes visible', () => {
    const { document } = parseZuiDocument(fixtureSource);
    const projection = projectZuiDocument(document);

    expect(projection.rootNodes.map(({ nodeId }) => nodeId)).toEqual(['root']);
    expect(
      projection.rootNodes[0].children.map(({ nodeId }) => nodeId),
    ).toEqual(['title', 'actions', 'virtual_rows']);
    expect(projection.rootNodes[0].container).toMatchObject({
      kind: 'flex',
      direction: 'column',
      gap: 12,
      padding: { left: 20, right: 20, top: 16, bottom: 16 },
    });
    expect(projection.detachedNodes.map(({ nodeId }) => nodeId)).toEqual([
      'detached_template',
    ]);
    expect(
      projection.diagnostics.some(({ code }) => code === 'metadata-preserved'),
    ).toBe(true);
    expect(projection.rootNodes[0].text).toBeNull();
    expect(projection.rootNodes[0].prefabRole).toBe('root');
    expect(
      projection.rootNodes[0].children.find(
        ({ nodeId }) => nodeId === 'actions',
      )?.text,
    ).toBeNull();
  });

  it('infers responsive and scroll containers and resolves common design tokens', () => {
    const { document } = parseZuiDocument(fixtureSource);
    const root = zuiNodes(document)['root'];
    root.component = 'ScrollableBox';
    root.layout!['container'] = {
      kind: 'ScrollableBox',
      gap: '$editor.density.gap.medium',
    };
    root.layout!['padding'] = {
      left: '$editor.density.gap.large',
      right: '$editor.density.gap.large',
      top: '$editor.density.gap.medium',
      bottom: '$editor.density.gap.medium',
    };
    const actions = zuiNodes(document)['actions'];
    actions.component = 'Stack';
    actions.props = { direction: { xs: 'column', md: 'row' } };
    actions.layout!['container'] = { kind: 'Stack', gap: '$space.medium' };

    const projection = projectZuiDocument(document);
    const projectedRoot = projection.rootNodes[0];
    const projectedActions = projectedRoot.children.find(
      ({ nodeId }) => nodeId === 'actions',
    )!;

    expect(projectedRoot.container).toMatchObject({
      kind: 'flex',
      direction: 'column',
      gap: 8,
      padding: { left: 12, right: 12, top: 8, bottom: 8 },
    });
    expect(projectedActions.container).toMatchObject({
      kind: 'flex',
      direction: 'row',
      gap: 12,
    });
  });

  it('converts implicit MUI Stack spacing props when no layout gap is authored', () => {
    const { document } = parseZuiDocument(fixtureSource);
    const root = zuiNodes(document)['root'];
    root.component = 'Stack';
    delete root.layout!['container'];
    root.props = {
      direction: { xs: 'column', md: 'row' },
      spacing: { xs: 1.5, md: 1.5 },
    };

    const projection = projectZuiDocument(document);

    expect(projection.rootNodes[0].container).toMatchObject({
      kind: 'flex',
      direction: 'row',
      gap: 12,
    });
  });

  it('keeps the authored horizontal axis for scrollable projections', () => {
    const { document } = parseZuiDocument(fixtureSource);
    const root = zuiNodes(document)['root'];
    root.component = 'ScrollableBox';
    root.layout!['container'] = {
      kind: 'ScrollableBox',
      axis: 'Horizontal',
    };

    expect(projectZuiDocument(document).rootNodes[0].container).toMatchObject({
      kind: 'flex',
      direction: 'row',
      clip: true,
    });
  });

  it('keeps native floating windows source-addressable while projecting them to the detached lane', () => {
    const { document } = parseZuiDocument(
      '[asset]\nkind = "view"\nid = "layout-demo"\nversion = 2\n\n[root]\nnode = "dock"\n\n[nodes.dock]\ncomponent = "DockHost"\nchildren = [{ node = "main" }, { node = "floating" }]\n\n[nodes.main]\ncomponent = "Window"\n\n[nodes.floating]\ncomponent = "FloatingWindow"\n',
    );

    const projection = projectZuiDocument(document);
    const dock = projection.rootNodes[0];
    const floating = dock.children.find(({ nodeId }) => nodeId === 'floating');

    expect(floating).toMatchObject({
      visualDetached: true,
      visualDetachedParentId: 'dock',
    });
    expect(projection.detachedNodes).toEqual([]);
    expect(cloneProjectionSnapshot(projection).shapes).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          nodeId: 'floating',
          parentNodeId: 'dock',
        }),
      ]),
    );
  });

  it('projects authored grid tracks instead of collapsing every child into one column', () => {
    const { document } = parseZuiDocument(fixtureSource);
    const actions = zuiNodes(document)['actions'];
    actions.component = 'GridGroup';
    actions.layout!['container'] = {
      kind: 'GridBox',
      columns: 3,
      rows: 2,
      column_gap: 6,
      row_gap: 10,
    };

    const grid = projectZuiDocument(document).shapes.find(
      ({ nodeId }) => nodeId === 'actions',
    )!;

    expect(grid.container).toMatchObject({
      kind: 'grid',
      columns: 3,
      rows: 2,
      columnGap: 6,
      rowGap: 10,
    });
  });

  it('uses unified prefab paint and typography defaults for common controls', () => {
    const { document } = parseZuiDocument(fixtureSource);
    const projection = projectZuiDocument(document);
    const cancel = projection.shapes.find(({ nodeId }) => nodeId === 'cancel')!;
    const row = projection.shapes.find(
      ({ nodeId }) => nodeId === 'row_template',
    )!;

    expect(cancel.prefabRole).toBe('button');
    expect(cancel.paint).toMatchObject({
      fillColor: null,
      strokeColor: '#2fbc9a',
      borderRadius: 6,
    });
    expect(cancel.text).toMatchObject({
      characters: 'Cancel',
      color: '#2fbc9a',
      fontSize: 11,
      fontWeight: '600',
      align: 'center',
    });
    expect(row.prefabRole).toBe('label');
    expect(row.paint.fillColor).toBeNull();
    expect(row.text).toMatchObject({ color: '#f1f4f2', fontSize: 12 });
  });

  it('maps semantic surface classes onto visible reusable prefabs', () => {
    const { document } = parseZuiDocument(fixtureSource);
    const nodes = zuiNodes(document);
    nodes['cancel'].component = 'Space';
    nodes['cancel'].classes = ['woc-bar-rail'];
    nodes['actions'].classes = ['woc-tracker'];
    nodes['row_template'].component = 'Space';
    nodes['row_template'].classes = ['woc-minimap'];

    const projection = projectZuiDocument(document);

    expect(
      projection.shapes.find(({ nodeId }) => nodeId === 'cancel')?.prefabRole,
    ).toBe('progress');
    expect(
      projection.shapes.find(({ nodeId }) => nodeId === 'actions')?.prefabRole,
    ).toBe('panel');
    expect(
      projection.shapes.find(({ nodeId }) => nodeId === 'row_template')
        ?.prefabRole,
    ).toBe('canvas');
  });

  it('uses owning Editor tokens without losing action and disabled semantics', () => {
    const { document } = parseZuiDocument(fixtureSource);
    document.tokens = {
      ...document.tokens,
      'editor.accent': '#60aeff',
      'editor.surface.3': '#383838',
      'editor.text.primary': '#e8e8e8',
      'editor.text.disabled': '#737373',
      'editor.control.radius.control': 4,
      'editor.typography.body.size': 14,
    };
    const shapes = projectZuiDocument(document).shapes;
    expect(shapes.find(({ nodeId }) => nodeId === 'cancel')).toMatchObject({
      paint: { fillColor: null, strokeColor: '#60aeff', borderRadius: 4 },
      text: { color: '#60aeff', fontSize: 14 },
    });
    expect(shapes.find(({ nodeId }) => nodeId === 'submit')).toMatchObject({
      paint: { fillColor: '#60aeff', strokeColor: null },
    });
    zuiNodes(document)['cancel'].props!['disabled'] = true;
    expect(
      projectZuiDocument(document).shapes.find(
        ({ nodeId }) => nodeId === 'cancel',
      ),
    ).toMatchObject({ text: { color: '#737373' } });
  });

  it('classifies toggle buttons with the toggle prefab before the generic button rule', () => {
    const { document } = parseZuiDocument(fixtureSource);
    zuiNodes(document)['cancel'].component = 'ToggleButton';

    const toggle = projectZuiDocument(document).shapes.find(
      ({ nodeId }) => nodeId === 'cancel',
    );

    expect(toggle?.prefabRole).toBe('toggle');
  });

  it('classifies workbench table rows without mistaking table for tab', () => {
    const { document } = parseZuiDocument(fixtureSource);
    const nodes = zuiNodes(document);
    nodes['cancel'].component = 'WorkbenchTableRow';
    nodes['row_template'].component = 'WorkbenchTab';

    const projection = projectZuiDocument(document);

    expect(
      projection.shapes.find(({ nodeId }) => nodeId === 'cancel')?.prefabRole,
    ).toBe('row');
    expect(
      projection.shapes.find(({ nodeId }) => nodeId === 'row_template')
        ?.prefabRole,
    ).toBe('tab');
  });

  it('keeps component roots at their semantic prefab size', () => {
    const { document } = parseZuiDocument(
      '[asset]\nkind = "component"\nid = "button-component"\nversion = 2\n\n[components.Button]\nroot = "root"\n\n[nodes.root]\ncomponent = "WorkbenchButton"\nprops = { text = "Run" }\n',
    );

    const root = projectZuiDocument(document).rootNodes[0];

    expect(root.prefabRole).toBe('button');
    expect(root.geometry).toMatchObject({ width: 120, height: 32 });
  });

  it('gives transparent layout component roots a reversible preview canvas', () => {
    const { document } = parseZuiDocument(
      '[asset]\nkind = "component"\nid = "layout-component"\nversion = 2\n\n[components.Layout]\nroot = "root"\n\n[nodes.root]\ncomponent = "HorizontalBox"\nlayout = { container = { kind = "HorizontalBox", gap = 8 } }\n',
    );

    const projection = projectZuiDocument(document);

    expect(projection.rootNodes[0].prefabRole).toBe('layout');
    expect(projection.rootNodes[0].paint.fillColor).toBe('#151719');
    expect(
      reconcileZuiDocument(document, cloneProjectionSnapshot(projection))
        .document,
    ).toEqual(document);
  });

  it('uses a wrapping desktop row for Stack when no responsive direction is authored', () => {
    const { document } = parseZuiDocument(fixtureSource);
    const actions = zuiNodes(document)['actions'];
    actions.component = 'Stack';
    actions.layout!['container'] = { kind: 'Stack', gap: 8, wrap: true };
    delete actions.props?.['direction'];

    const projected = projectZuiDocument(document).shapes.find(
      ({ nodeId }) => nodeId === 'actions',
    );

    expect(projected?.container).toMatchObject({
      kind: 'flex',
      direction: 'row',
      wrap: true,
    });
  });

  it('keeps collapsed states and tiny live regions semantic but out of the visual preview', () => {
    const { document } = parseZuiDocument(fixtureSource);
    const nodes = zuiNodes(document);
    nodes['cancel'].props!['visibility'] = 'collapsed';
    nodes['live_region'] = {
      component: 'Label',
      classes: ['screen-reader-live-region'],
      props: { text: '' },
      layout: {
        width: { min: 1, preferred: 1, max: 1, stretch: 'Fixed' },
        height: { min: 1, preferred: 1, max: 1, stretch: 'Fixed' },
      },
    };
    nodes['root'].children!.push({ node: 'live_region' });
    nodes['responsive_parent'] = {
      component: 'Container',
      props: { visibility: { xs: 'visible', md: 'collapsed' } },
      layout: { container: { kind: 'VerticalBox', gap: 4 } },
      children: [{ node: 'responsive_child' }],
    };
    nodes['responsive_child'] = {
      component: 'Label',
      props: { text: 'Hidden with parent' },
      layout: {
        width: { preferred: 120, stretch: 'Fixed' },
        height: { preferred: 24, stretch: 'Fixed' },
      },
    };
    nodes['root'].children!.push({ node: 'responsive_parent' });

    const projection = projectZuiDocument(document);
    const root = projection.shapes.find(({ nodeId }) => nodeId === 'root')!;
    const cancel = projection.shapes.find(({ nodeId }) => nodeId === 'cancel')!;
    const liveRegion = projection.shapes.find(
      ({ nodeId }) => nodeId === 'live_region',
    )!;
    const responsiveParent = projection.shapes.find(
      ({ nodeId }) => nodeId === 'responsive_parent',
    )!;
    const responsiveChild = projection.shapes.find(
      ({ nodeId }) => nodeId === 'responsive_child',
    )!;

    expect(root.previewHidden).toBe(false);
    expect(cancel.previewHidden).toBe(true);
    expect(liveRegion.text).toBeNull();
    expect(liveRegion.previewHidden).toBe(true);
    expect(responsiveParent.previewHidden).toBe(true);
    expect(responsiveChild.previewHidden).toBe(true);
  });

  it('hides descendants of a clipped zero-sized dynamic host in the visual projection', () => {
    const { document } = parseZuiDocument(fixtureSource);
    const nodes = zuiNodes(document);
    nodes['zero_host'] = {
      component: 'Overlay',
      layout: {
        clip: true,
        container: { kind: 'Overlay' },
        width: { min: 0, preferred: 0, max: 0, stretch: 'Fixed' },
        height: { min: 0, preferred: 0, max: 0, stretch: 'Fixed' },
      },
      children: [{ node: 'zero_child' }],
    };
    nodes['zero_child'] = {
      component: 'Label',
      props: { text: 'Deferred host content' },
    };
    nodes['root'].children!.push({ node: 'zero_host' });

    const projection = projectZuiDocument(document);
    expect(
      projection.shapes.find(({ nodeId }) => nodeId === 'zero_host')
        ?.previewHidden,
    ).toBe(true);
    expect(
      projection.shapes.find(({ nodeId }) => nodeId === 'zero_child')
        ?.previewHidden,
    ).toBe(true);
  });

  it('leaves the source semantic document unchanged when no shape changed', () => {
    const { document } = parseZuiDocument(fixtureSource);
    const projection = projectZuiDocument(document);
    const result = reconcileZuiDocument(
      document,
      cloneProjectionSnapshot(projection),
    );

    expect(result.document).toEqual(document);
    expect(result.changes).toEqual([]);
  });

  it('round-trips a zero-sized runtime placeholder through its Penpot proxy', () => {
    const { document } = parseZuiDocument(fixtureSource);
    const rootWidth = zuiNodes(document)['root'].layout?.['width'] as Record<
      string,
      unknown
    >;
    rootWidth['min'] = 0;
    rootWidth['preferred'] = 0;
    rootWidth['max'] = 0;
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));

    expect(
      snapshot.shapes.find(({ nodeId }) => nodeId === 'root')?.baseline.geometry
        .width,
    ).toBe(0);
    expect(reconcileZuiDocument(document, snapshot).document).toEqual(document);
  });

  it('applies supported edits while preserving runtime-only semantics', () => {
    const { document } = parseZuiDocument(fixtureSource);
    const projection = projectZuiDocument(document);
    const snapshot = cloneProjectionSnapshot(projection);
    const root = snapshot.shapes.find(({ nodeId }) => nodeId === 'root')!;
    const title = snapshot.shapes.find(({ nodeId }) => nodeId === 'title')!;
    const actions = snapshot.shapes.find(({ nodeId }) => nodeId === 'actions')!;

    root.current.geometry.width = 512;
    root.current.paint.fillColor = '#303640';
    title.current.text!.characters = 'Edited in Penpot';
    actions.childNodeIds = ['submit', 'cancel'];

    const result = reconcileZuiDocument(document, snapshot);
    const resultNodes = zuiNodes(result.document);
    const sourceNodes = zuiNodes(document);

    expect(resultNodes['root'].layout?.['width']).toEqual({
      min: 512,
      preferred: 512,
      max: 512,
      stretch: 'Fixed',
    });
    expect(resultNodes['root'].props?.['background_color']).toBe('#303640');
    expect(resultNodes['title'].props?.['text']).toBe('Edited in Penpot');
    expect(resultNodes['actions'].children?.map(({ node }) => node)).toEqual([
      'submit',
      'cancel',
    ]);
    expect(resultNodes['root'].events).toEqual(sourceNodes['root'].events);
    expect(resultNodes['root']['zircon_extension']).toEqual(
      sourceNodes['root']['zircon_extension'],
    );
    expect(resultNodes['title'].props?.['runtime_only']).toEqual(
      sourceNodes['title'].props?.['runtime_only'],
    );
  });

  it('preserves unknown keys inside edited dimension and padding tables', () => {
    const { document } = parseZuiDocument(fixtureSource);
    const rootNode = zuiNodes(document)['root'];
    const width = rootNode.layout?.['width'] as Record<string, unknown>;
    const padding = rootNode.layout?.['padding'] as Record<string, unknown>;
    width['runtime_hint'] = 'keep-width';
    padding['runtime_hint'] = 'keep-padding';

    const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
    const root = snapshot.shapes.find(({ nodeId }) => nodeId === 'root')!;
    root.current.geometry.width = 640;
    root.current.container.padding.left = 28;

    const resultRoot = zuiNodes(
      reconcileZuiDocument(document, snapshot).document,
    )['root'];
    expect(resultRoot.layout?.['width']).toMatchObject({
      preferred: 640,
      runtime_hint: 'keep-width',
    });
    expect(resultRoot.layout?.['padding']).toMatchObject({
      left: 28,
      runtime_hint: 'keep-padding',
    });
  });

  it('writes clipping back to the source property used by the projection', () => {
    const { document } = parseZuiDocument(fixtureSource);
    const rootNode = zuiNodes(document)['root'];
    delete rootNode.layout!['clip'];
    rootNode.props!['clip_content'] = true;
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
    const root = snapshot.shapes.find(({ nodeId }) => nodeId === 'root')!;
    root.current.container.clip = false;

    const resultRoot = zuiNodes(
      reconcileZuiDocument(document, snapshot).document,
    )['root'];
    expect(resultRoot.props?.['clip_content']).toBe(false);
    expect(resultRoot.layout).not.toHaveProperty('clip');
  });

  it('preserves token siblings when one position axis or padding side changes', () => {
    const { document } = parseZuiDocument(fixtureSource);
    const rootNode = zuiNodes(document)['root'];
    const position = rootNode.layout!['position'] as Record<string, unknown>;
    const padding = rootNode.layout!['padding'] as Record<string, unknown>;
    position['x'] = '$layout.origin.x';
    padding['left'] = '$space.panel.left';
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
    const root = snapshot.shapes.find(({ nodeId }) => nodeId === 'root')!;
    root.current.geometry.y += 12;
    root.current.container.padding.right += 4;

    const resultRoot = zuiNodes(
      reconcileZuiDocument(document, snapshot).document,
    )['root'];
    expect(resultRoot.layout?.['position']).toMatchObject({
      x: '$layout.origin.x',
      y: 72,
    });
    expect(resultRoot.layout?.['padding']).toMatchObject({
      left: '$space.panel.left',
      right: 24,
    });
  });

  it('projects eight-digit hex colors through Penpot opacity without losing alpha', () => {
    const { document } = parseZuiDocument(fixtureSource);
    zuiNodes(document)['root'].props!['background_color'] = '#11223380';
    const projection = projectZuiDocument(document);
    const snapshot = cloneProjectionSnapshot(projection);
    const root = snapshot.shapes.find(({ nodeId }) => nodeId === 'root')!;

    expect(root.current.paint.fillColor).toBe('#112233');
    expect(root.current.paint.fillOpacity).toBeCloseTo(128 / 255);
    expect(reconcileZuiDocument(document, snapshot).document).toEqual(document);

    root.current.paint.fillOpacity = 0.25;
    const edited = reconcileZuiDocument(document, snapshot);
    expect(zuiNodes(edited.document)['root'].props?.['background_color']).toBe(
      '#11223340',
    );
  });

  it('resolves declared font weights to valid Penpot variants', () => {
    const { document } = parseZuiDocument(fixtureSource);
    document.tokens = {
      ...document.tokens,
      'editor.typography.strong.weight': 600,
    };
    zuiNodes(document)['title'].props!['font_weight'] =
      '$editor.typography.strong.weight';

    const projection = projectZuiDocument(document);
    const title = projection.shapes.find(({ nodeId }) => nodeId === 'title')!;

    expect(title.text?.fontWeight).toBe('600');
    expect(
      projection.diagnostics.some(
        ({ code, path }) =>
          code === 'token-value-preserved' &&
          path === 'nodes.title.props.font_weight',
      ),
    ).toBe(false);
    expect(
      reconcileZuiDocument(document, cloneProjectionSnapshot(projection))
        .document,
    ).toEqual(document);
  });

  it('projects inline strong content into one marker-free text shape', () => {
    const { document } = parseZuiDocument(fixtureSource);
    const title = zuiNodes(document)['title'];
    title.props!['text'] = 'Verify **rich-text** runs';
    title.props!['rich_text_format'] = 'markdown_inline_v1';

    const projection = projectZuiDocument(document);
    const projectedTitle = projection.shapes.find(
      ({ nodeId }) => nodeId === 'title',
    )!;

    expect(projectedTitle.text).toMatchObject({
      characters: 'Verify rich-text runs',
      richTextRuns: [{ start: 7, end: 16, fontWeight: '700' }],
    });
    expect(projectedTitle.text?.characters).not.toContain('**');

    const snapshot = cloneProjectionSnapshot(projection);
    const titleSnapshot = snapshot.shapes.find(
      ({ nodeId }) => nodeId === 'title',
    )!;
    expect(titleSnapshot.baseline.text).not.toHaveProperty('richTextRuns');
    expect(reconcileZuiDocument(document, snapshot).changes).toEqual([]);
  });

  it('blocks export when a semantic node disappeared from the Penpot asset', () => {
    const { document } = parseZuiDocument(fixtureSource);
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
    snapshot.shapes = snapshot.shapes.filter(
      ({ nodeId }) => nodeId !== 'submit',
    );

    expect(() => reconcileZuiDocument(document, snapshot)).toThrow(
      /missing semantic node submit/i,
    );
  });

  it('blocks export when semantic component metadata changed', () => {
    const { document } = parseZuiDocument(fixtureSource);
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
    const title = snapshot.shapes.find(({ nodeId }) => nodeId === 'title')!;
    title.component = 'Button';

    expect(() => reconcileZuiDocument(document, snapshot)).toThrow(
      /component metadata for node title/i,
    );
  });

  it('blocks export when detached-node metadata disagrees with hierarchy', () => {
    const { document } = parseZuiDocument(fixtureSource);
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
    snapshot.detachedNodeIds = [];

    expect(() => reconcileZuiDocument(document, snapshot)).toThrow(
      /detached node metadata/i,
    );
  });

  it('blocks export when editable text metadata is removed or retargeted', () => {
    const { document } = parseZuiDocument(fixtureSource);
    const missingText = cloneProjectionSnapshot(projectZuiDocument(document));
    const missingTitle = missingText.shapes.find(
      ({ nodeId }) => nodeId === 'title',
    )!;
    missingTitle.current.text = null;

    expect(() => reconcileZuiDocument(document, missingText)).toThrow(
      /editable text metadata for node title/i,
    );

    const retargetedText = cloneProjectionSnapshot(
      projectZuiDocument(document),
    );
    const retargetedTitle = retargetedText.shapes.find(
      ({ nodeId }) => nodeId === 'title',
    )!;
    retargetedTitle.current.text!.property = 'label';

    expect(() => reconcileZuiDocument(document, retargetedText)).toThrow(
      /text property metadata for node title/i,
    );
  });

  it('does not project container identifiers as display-only body text', () => {
    const { document } = parseZuiDocument(fixtureSource);
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
    const root = snapshot.shapes.find(({ nodeId }) => nodeId === 'root')!;
    expect(root.current.text).toBeNull();
    expect(reconcileZuiDocument(document, snapshot).document).toEqual(document);
  });
});
