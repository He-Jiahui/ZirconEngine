import { dividerGeometry, projectDivider } from './zui-divider-projection';
import {
  cloneProjectionSnapshot,
  projectZuiDocument,
  reconcileZuiDocument,
} from './penpot-projection';
import { normalizeZuiDocument, type ZuiDocument } from './zui-document';

const source = (): ZuiDocument => ({
  asset: { kind: 'component', id: 'divider', version: 2 },
  components: { Divider: { root: 'root' } },
  tokens: { line: '#333333', disabled: '#363636', width: 1 },
  nodes: {
    root: {
      component: 'Divider',
      props: {
        separator_color: '$line',
        disabled_separator_color: '$disabled',
        thickness: '$width',
        inset: 8,
      },
      layout: { width: { stretch: 'Stretch' }, height: { preferred: 4 } },
    },
  },
});

describe('Runtime divider projection', () => {
  it('uses a 1px centered line and preserves the authored owner and tokens on export', () => {
    const document = source();
    const projected = projectZuiDocument(document);
    const root = projected.rootNodes[0];
    expect(root.paint.fillColor).toBeNull();
    expect(root.paint.strokeColor).toBeNull();
    expect(root.divider?.color).toBe('#333333');
    expect(dividerGeometry(root.divider!, 360, 4)).toEqual({
      x: 0,
      y: 1.5,
      width: 360,
      height: 1,
    });
    expect(dividerGeometry(root.divider!, 360, 1)).toBeNull();
    expect(
      normalizeZuiDocument(
        reconcileZuiDocument(document, cloneProjectionSnapshot(projected))
          .document,
      ),
    ).toEqual(normalizeZuiDocument(document));
  });

  it('maps vertical, inset, middle and disabled behavior without generic surfaces', () => {
    const document = source();
    const root = document.nodes!['root'];
    Object.assign(root.props!, {
      orientation: 'vertical',
      variant: 'middle',
      disabled: true,
    });
    const divider = projectDivider(document, root)!;
    expect(divider.color).toBe('#363636');
    expect(dividerGeometry(divider, 4, 40)).toEqual({
      x: 1.5,
      y: 8,
      width: 1,
      height: 24,
    });
    expect(dividerGeometry({ ...divider, variant: 'inset' }, 4, 40)).toEqual({
      x: 1.5,
      y: 8,
      width: 1,
      height: 32,
    });
    expect(dividerGeometry({ ...divider, inset: 50 }, 4, 40)).toBeNull();
  });

  it('rejects unresolved paint tokens', () => {
    const document = source();
    document.nodes!['root'].props!['separator_color'] = '$missing';
    expect(() => projectZuiDocument(document)).toThrow(
      'Unresolved divider token',
    );
  });
});
