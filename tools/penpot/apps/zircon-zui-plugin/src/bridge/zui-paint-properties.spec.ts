import { readFileSync } from 'node:fs';
import {
  parseZuiDocument,
  normalizeZuiDocument,
  type ZuiTable,
} from './zui-document';
import {
  cloneProjectionSnapshot,
  projectZuiDocument,
  reconcileZuiDocument,
} from './penpot-projection';

const pauseSource = readFileSync(
  new URL(
    '../../../../../../zircon_runtime/assets/ui/runtime/fixtures/pause_menu.zui',
    import.meta.url,
  ),
  'utf8',
);
const pauseDocument = () => parseZuiDocument(pauseSource).document;

describe('authored structured paint projection and writeback', () => {
  it('applies an authored universal typography rule to every projected text node', () => {
    const document = pauseDocument();
    document.stylesheets = [
      {
        id: 'reactbits-ui-typography',
        rules: [
          {
            selector: '*',
            set: {
              self: {
                font: {
                  asset: 'res://fonts/editor-ui.font.toml',
                  family: 'Fira Sans',
                },
              },
            },
          },
        ],
      },
    ];
    const title = projectZuiDocument(document).shapes.find(
      ({ nodeId }) => nodeId === 'title',
    )?.text;
    expect(title).toMatchObject({ fontFamily: 'Fira Sans' });
  });

  it('projects the actual pause dialog and labels without changing source tables', () => {
    const document = pauseDocument();
    const before = normalizeZuiDocument(document);
    const projection = projectZuiDocument(document);
    expect(
      projection.shapes.find(({ nodeId }) => nodeId === 'dialog')?.paint,
    ).toMatchObject({
      fillColor: '#b13d3d',
      strokeColor: '#f6d2a6',
      strokeWidth: 3,
      borderRadius: 18,
      opacity: 0.96,
    });
    expect(
      projection.shapes.find(({ nodeId }) => nodeId === 'scrim')?.paint,
    ).toMatchObject({
      fillColor: '#10141d',
      fillOpacity: 204 / 255,
    });
    expect(
      projection.shapes.find(({ nodeId }) => nodeId === 'title')?.text?.color,
    ).toBe('#fff4e0');
    expect(
      projection.shapes.find(({ nodeId }) => nodeId === 'resume_button')?.paint,
    ).toMatchObject({
      fillColor: '#272c38',
      strokeColor: '#f7d58d',
      strokeWidth: 1,
      borderRadius: 10,
    });
    expect(normalizeZuiDocument(document)).toEqual(before);
    const result = reconcileZuiDocument(
      document,
      cloneProjectionSnapshot(projection),
    );
    expect(result.changes).toEqual([]);
    expect(normalizeZuiDocument(result.document)).toEqual(before);
  });

  it('uses nested paint and font fields before flat aliases, including zero border width', () => {
    const document = pauseDocument();
    Object.assign(document.nodes!['dialog'].props!, {
      background_color: '#000000',
      border_color: '#111111',
      border_width: 7,
      radius: 8,
      corner_radius: 9,
      border: { color: '#f6d2a6', width: 0, radius: 18 },
    });
    Object.assign(document.nodes!['title'].props!, {
      foreground_color: '#000000',
      font_size: 10,
      font_weight: 900,
      text_align: 'left',
      font: {
        size: 22,
        weight: 600,
        align: 'middle',
        family: 'Fira Sans',
        asset: 'res://fonts/editor-ui.font.toml',
      },
    });
    const projection = projectZuiDocument(document);
    expect(
      projection.shapes.find(({ nodeId }) => nodeId === 'dialog')?.paint,
    ).toMatchObject({
      fillColor: '#b13d3d',
      strokeColor: '#f6d2a6',
      strokeWidth: 0,
      borderRadius: 18,
    });
    expect(
      projection.shapes.find(({ nodeId }) => nodeId === 'title')?.text,
    ).toMatchObject({
      color: '#fff4e0',
      fontSize: 22,
      fontWeight: '600',
      align: 'center',
    });
  });

  it('preserves tokens and unknown siblings on no-edit and writes edits into the original nested fields', () => {
    const document = pauseDocument();
    document.tokens = {
      surface: '#b13d3d',
      width: 3,
      radius: 18,
      size: 22,
      weight: 600,
      ink: '#fff4e0',
    };
    const dialog = document.nodes!['dialog'];
    Object.assign(dialog.props!, {
      background: {
        color: '$surface',
        retained: { binding: '$session.color' },
      },
      border: {
        color: '#f6d2a6',
        width: '$width',
        radius: '$radius',
        extension: true,
      },
      background_color: '#111111',
      border_width: 99,
    });
    dialog.events = [
      { id: 'Pause/Click', event: 'Click', route: 'pause.click' },
    ];
    const title = document.nodes!['title'];
    Object.assign(title.props!, {
      foreground: { color: '$ink', extension: 'retained' },
      font: {
        size: '$size',
        weight: '$weight',
        align: 'right',
        family: 'Fira Sans',
        asset: 'res://fonts/editor-ui.font.toml',
      },
    });
    const before = normalizeZuiDocument(document);
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
    expect(
      normalizeZuiDocument(reconcileZuiDocument(document, snapshot).document),
    ).toEqual(before);
    const dialogShape = snapshot.shapes.find(
      ({ nodeId }) => nodeId === 'dialog',
    )!;
    Object.assign(dialogShape.current.paint, {
      fillColor: '#2468ac',
      fillOpacity: 128 / 255,
      strokeColor: '#abcdef',
      strokeWidth: 5,
      borderRadius: 12,
    });
    const titleShape = snapshot.shapes.find(
      ({ nodeId }) => nodeId === 'title',
    )!;
    Object.assign(titleShape.current.text!, {
      color: '#123456',
      fontSize: 24,
      fontWeight: 'regular',
      align: 'center',
    });
    const result = reconcileZuiDocument(document, snapshot);
    expect(result.document.nodes!['dialog'].props!['background']).toEqual({
      color: '#2468ac80',
      retained: { binding: '$session.color' },
    });
    expect(result.document.nodes!['dialog'].props!['border']).toEqual({
      color: '#abcdef',
      width: 5,
      radius: 12,
      extension: true,
    });
    expect(result.document.nodes!['dialog'].props!['background_color']).toBe(
      '#111111',
    );
    expect(result.document.nodes!['dialog'].props!['border_width']).toBe(99);
    expect(result.document.nodes!['title'].props!['foreground']).toEqual({
      color: '#123456',
      extension: 'retained',
    });
    expect(result.document.nodes!['title'].props!['font']).toEqual({
      size: 24,
      weight: 400,
      align: 'center',
      family: 'Fira Sans',
      asset: 'res://fonts/editor-ui.font.toml',
    });
    expect(result.document.nodes!['title'].props!['font_size']).toBeUndefined();
    expect(
      result.document.nodes!['title'].props!['foreground_color'],
    ).toBeUndefined();
    expect(result.document.nodes!['dialog'].events).toEqual(dialog.events);
    expect(result.document.tokens).toEqual(document.tokens);
    expect(normalizeZuiDocument(document)).toEqual(before);
    expect(result.changes).toContain('nodes.dialog.props.border.width');
    expect(result.changes).toContain('nodes.title.props.font.weight');
    const reprojection = projectZuiDocument(result.document);
    expect(
      reprojection.shapes.find(({ nodeId }) => nodeId === 'dialog')?.paint,
    ).toMatchObject({
      fillColor: '#2468ac',
      fillOpacity: 128 / 255,
      strokeWidth: 5,
      borderRadius: 12,
    });
    expect(
      reprojection.shapes.find(({ nodeId }) => nodeId === 'title')?.text,
    ).toMatchObject({
      color: '#123456',
      fontSize: 24,
      fontWeight: 'regular',
      align: 'center',
    });
  });

  it('writes through authored state and inline style tables using runtime layer priority', () => {
    const document = pauseDocument();
    const node = document.nodes!['title'];
    node.state = { foreground: { color: '#123456' } };
    node.style = { self: { font: { size: 22, weight: 600, align: 'right' } } };
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
    const shape = snapshot.shapes.find(({ nodeId }) => nodeId === 'title')!;
    expect(shape.baseline.text).toMatchObject({
      color: '#123456',
      fontSize: 22,
    });
    Object.assign(shape.current.text!, { color: '#abcdef', fontSize: 24 });
    const result = reconcileZuiDocument(document, snapshot);
    expect(result.document.nodes!['title'].state!['foreground']).toEqual({
      color: '#abcdef',
    });
    expect(
      (result.document.nodes!['title'].style!['self'] as ZuiTable)['font'],
    ).toEqual({ size: 24, weight: 600, align: 'right' });
    expect(result.document.nodes!['title'].props!['foreground']).toEqual({
      color: '#fff4e0',
    });
    expect(result.document.nodes!['title'].props!['font_size']).toBeUndefined();
  });

  it('keeps structured stylesheet paint precedence over node values of the same key', () => {
    const document = pauseDocument();
    document.stylesheets = [
      {
        rules: [
          {
            selector: '#PauseDialog',
            set: {
              self: {
                background: { color: '#123456' },
                border: { color: '#abcdef', width: 2, radius: 4 },
              },
            },
          },
        ],
      },
    ];
    expect(
      projectZuiDocument(document).shapes.find(
        ({ nodeId }) => nodeId === 'dialog',
      )?.paint,
    ).toMatchObject({
      fillColor: '#123456',
      strokeColor: '#abcdef',
      strokeWidth: 2,
      borderRadius: 4,
    });
  });

  it.each([
    { background: { color: 'rgb(255, 0, 0)' } },
    { background: { image: 'res://image.png' } },
    { background: false },
    { border: { width: 'wide' } },
    { border: { radius: -1 } },
    { font: { size: 0 } },
    { font: { weight: 550 } },
    { font: { align: 'start' } },
    { foreground: { color: '$missing' } },
  ])(
    'rejects unsupported structured mapping %j instead of showing fallback paint',
    (props) => {
      const document = pauseDocument();
      Object.assign(document.nodes!['title'].props!, props);
      expect(() => projectZuiDocument(document)).toThrow(
        'Unsupported structured paint',
      );
    },
  );

  it('roundtrips font family and line-height edits into the source font table', () => {
    const document = pauseDocument();
    document.nodes!['title'].props!['font'] = {
      family: 'Fira Mono',
      size: 16,
      weight: 500,
      line_height: 28,
      asset: 'res://fonts/editor-ui.font.toml',
      extension: { keep: true },
    };
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
    const text = snapshot.shapes.find(({ nodeId }) => nodeId === 'title')!
      .current.text!;
    expect(text).toMatchObject({ fontFamily: 'Fira Mono', lineHeight: 1.75 });
    Object.assign(text, {
      fontFamily: 'Fira Sans',
      fontSize: 20,
      lineHeight: 1.5,
    });
    const result = reconcileZuiDocument(document, snapshot);
    expect(result.document.nodes!['title'].props!['font']).toEqual({
      family: 'Fira Sans',
      size: 20,
      weight: 500,
      line_height: 30,
      asset: 'res://fonts/editor-ui.font.toml',
      extension: { keep: true },
    });
    expect(
      projectZuiDocument(result.document).shapes.find(
        ({ nodeId }) => nodeId === 'title',
      )!.text,
    ).toMatchObject({ fontFamily: 'Fira Sans', fontSize: 20, lineHeight: 1.5 });
  });

  it('writes an instance override when a stylesheet owns the edited paint and typography', () => {
    const document = pauseDocument();
    document.stylesheets = [
      {
        rules: [
          {
            selector: '#PauseDialog',
            set: {
              self: {
                border: { color: '#123456', width: 3, radius: 18 },
                opacity: 0.8,
              },
            },
          },
          {
            selector: 'Label',
            set: {
              self: {
                font: {
                  family: 'Fira Mono',
                  size: 16,
                  line_height: 24,
                  weight: 500,
                },
              },
            },
          },
        ],
      },
    ];
    const before = normalizeZuiDocument(document);
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
    const title = snapshot.shapes.find(({ nodeId }) => nodeId === 'title')!
      .current.text!;
    expect(title).toMatchObject({ fontFamily: 'Fira Mono', lineHeight: 1.5 });
    Object.assign(title, {
      fontFamily: 'Fira Sans',
      fontSize: 20,
      lineHeight: 1.4,
    });
    snapshot.shapes.find(
      ({ nodeId }) => nodeId === 'dialog',
    )!.current.paint.strokeWidth = 0;
    snapshot.shapes.find(
      ({ nodeId }) => nodeId === 'dialog',
    )!.current.paint.opacity = 0.5;
    const result = reconcileZuiDocument(document, snapshot);
    const projected = projectZuiDocument(result.document);
    expect(
      projected.shapes.find(({ nodeId }) => nodeId === 'title')!.text,
    ).toMatchObject({
      fontFamily: 'Fira Sans',
      fontSize: 20,
      lineHeight: 1.4,
      fontWeight: '500',
    });
    expect(
      projected.shapes.find(({ nodeId }) => nodeId === 'dialog')!.paint,
    ).toMatchObject({
      strokeColor: '#123456',
      strokeWidth: 0,
      borderRadius: 18,
      opacity: 0.5,
    });
    expect(result.document.stylesheets).toEqual(document.stylesheets);
    expect(result.document.nodes!['title'].props).toEqual(
      document.nodes!['title'].props,
    );
    expect(normalizeZuiDocument(document)).toEqual(before);
  });
});
