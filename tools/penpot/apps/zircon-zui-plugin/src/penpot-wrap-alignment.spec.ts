import { validateCapturedWrapAlignment } from './penpot-capture-validation';
import { parseZuiDocument } from './bridge/zui-document';
import { createPenpotBridgeAsset } from './bridge/penpot-asset';
import { reconcileZuiDocument } from './bridge/penpot-projection';
it('preserves source alignment ignored by the native Wrap parent style', () => {
  const { document } = parseZuiDocument(
    '[asset]\nkind="view"\nid="wrap"\nversion=2\n[root]\nnode="root"\n[nodes.root]\ncomponent="WrapBox"\nlayout={container={kind="WrapBox",align_items="center"}}\n',
  );
  const asset = createPenpotBridgeAsset(document, 'wrap.zui');
  const root = asset.snapshot.shapes.find((s) => s.nodeId === 'root')!;
  root.baseline.container.alignItems = 'start';
  root.current.container.alignItems = 'start';
  validateCapturedWrapAlignment('root', true, 'start', 'start');
  const result = reconcileZuiDocument(document, asset.snapshot);
  expect(result.changes).toEqual([]);
  expect(result.document.nodes!['root'].layout).toEqual(
    document.nodes!['root'].layout,
  );
});
it.each(['center', 'end', 'stretch'])(
  'rejects unsupported authored parent alignItems %s before native reflow',
  (align) => {
    expect(() =>
      validateCapturedWrapAlignment('root', true, align, 'start'),
    ).toThrow(/native Wrap parent alignment/);
  },
);
it('rejects authored line alignment while leaving linear parent alignment editable', () => {
  expect(() =>
    validateCapturedWrapAlignment('root', true, 'start', 'center'),
  ).toThrow(/native Wrap parent alignment/);
  expect(() =>
    validateCapturedWrapAlignment('root', false, 'center', 'center'),
  ).not.toThrow();
});
