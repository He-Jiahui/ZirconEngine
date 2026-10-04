import fixtureSource from './roundtrip-fixture.zui?raw';
import {
  decodeZuiMetadata,
  encodeZuiMetadata,
  normalizeZuiDocument,
  parseZuiDocument,
  serializeZuiDocument,
  zuiNodes,
} from './zui-document';

describe('ZUI document codec', () => {
  it('round-trips the v2 document without losing semantic data', () => {
    const parsed = parseZuiDocument(fixtureSource);

    expect(
      parsed.diagnostics.filter(({ severity }) => severity === 'error'),
    ).toEqual([]);

    const reparsed = parseZuiDocument(serializeZuiDocument(parsed.document));
    expect(normalizeZuiDocument(reparsed.document)).toEqual(
      normalizeZuiDocument(parsed.document),
    );
    expect(zuiNodes(reparsed.document)['root'].events?.[0]).toMatchObject({
      id: 'Roundtrip/Changed',
      route: 'tests.roundtrip.changed',
    });
    expect(zuiNodes(reparsed.document)['root']['zircon_extension']).toEqual({
      owner: 'roundtrip-test',
      payload: { enabled: true, revision: 7 },
    });
    expect(reparsed.document.tokens?.['surface.panel']).toBe('#20242b');
  });

  it('rejects a schema version that Zircon runtime cannot load', () => {
    const invalid = fixtureSource.replace('version = 2', 'version = 3');

    expect(() => parseZuiDocument(invalid)).toThrow(
      /unsupported schema version 3/i,
    );
  });

  it('rejects missing child references before creating Penpot shapes', () => {
    const invalid = fixtureSource.replace(
      '{ node = "title" }',
      '{ node = "missing_title" }',
    );

    expect(() => parseZuiDocument(invalid)).toThrow(/missing_title/);
  });

  it('rejects mounting the same semantic child twice under one parent', () => {
    const invalid = fixtureSource.replace(
      '{ node = "title" }, { node = "actions"',
      '{ node = "title" }, { node = "title" }, { node = "actions"',
    );

    expect(() => parseZuiDocument(invalid)).toThrow(
      /mounts child title more than once/i,
    );
  });

  it('rejects fields whose types do not match the Zircon Runtime v2 loader', () => {
    const invalidCases = [
      fixtureSource.replace(
        'display_name = "Penpot Roundtrip"',
        'display_name = 123',
      ),
      fixtureSource.replace(
        'classes = ["roundtrip", "panel"]',
        'classes = "not-an-array"',
      ),
      fixtureSource.replace(
        'events = [{ id = "Roundtrip/Changed", event = "Change", route = "tests.roundtrip.changed" }]',
        'events = "not-an-array"',
      ),
    ];

    for (const source of invalidCases) {
      expect(() => parseZuiDocument(source)).toThrow(/runtime v2 loader/i);
    }
  });

  it('rejects repeat metadata the runtime compiler cannot execute', () => {
    const invalid = fixtureSource.replace(
      'authored_count = 1',
      'authored_count = 0',
    );

    expect(() => parseZuiDocument(invalid)).toThrow(
      /authored_count must be a positive runtime integer/i,
    );
  });

  it('preserves a node-less v2 theme token asset', () => {
    const source = `[asset]
kind = "theme_tokens"
id = "res://ui/theme/tokens.zui"
version = 2

[tokens]
"surface.panel" = "#20242b"
`;

    const parsed = parseZuiDocument(source);
    const serialized = serializeZuiDocument(parsed.document);
    const reparsed = parseZuiDocument(serialized);

    expect(parsed.document.nodes).toBeUndefined();
    expect(serialized).not.toContain('[nodes]');
    expect(reparsed.document.tokens?.['surface.panel']).toBe('#20242b');
  });

  it('preserves 64-bit integers through TOML and Penpot JSON metadata', () => {
    const source = fixtureSource.replace(
      'revision = 7',
      'revision = 9223372036854775807',
    );
    const parsed = parseZuiDocument(source);
    const decoded = decodeZuiMetadata(encodeZuiMetadata(parsed.document));
    const serialized = serializeZuiDocument(decoded);

    expect(serialized).toContain('revision = 9223372036854775807');
    expect(normalizeZuiDocument(decoded)).toEqual(
      normalizeZuiDocument(parsed.document),
    );
  });

  it('does not confuse legal TOML keys with typed metadata values', () => {
    const source = `${fixtureSource}
[nodes.root.bigint_tag_collision]
"$zirconTomlBigInt" = "42"

[nodes.root.date_tag_collision]
"$zirconTomlDate" = "2026-08-31"
`;
    const parsed = parseZuiDocument(source);
    const decoded = decodeZuiMetadata(encodeZuiMetadata(parsed.document));

    expect(zuiNodes(decoded)['root']['bigint_tag_collision']).toEqual({
      $zirconTomlBigInt: '42',
    });
    expect(zuiNodes(decoded)['root']['date_tag_collision']).toEqual({
      $zirconTomlDate: '2026-08-31',
    });
    expect(normalizeZuiDocument(decoded)).toEqual(
      normalizeZuiDocument(parsed.document),
    );
  });

  it('preserves non-finite and negative-zero TOML numbers in JSON metadata', () => {
    const source = `${fixtureSource}
[nodes.root.numeric_edge_cases]
positive_infinity = inf
negative_infinity = -inf
not_a_number = nan
negative_zero = -0.0
`;
    const parsed = parseZuiDocument(source);
    const decoded = decodeZuiMetadata(encodeZuiMetadata(parsed.document));
    const values = zuiNodes(decoded)['root']['numeric_edge_cases'] as Record<
      string,
      number
    >;

    expect(values['positive_infinity']).toBe(Number.POSITIVE_INFINITY);
    expect(values['negative_infinity']).toBe(Number.NEGATIVE_INFINITY);
    expect(Number.isNaN(values['not_a_number'])).toBe(true);
    expect(Object.is(values['negative_zero'], -0)).toBe(true);
    expect(normalizeZuiDocument(decoded)).toEqual(
      normalizeZuiDocument(parsed.document),
    );
  });
});
