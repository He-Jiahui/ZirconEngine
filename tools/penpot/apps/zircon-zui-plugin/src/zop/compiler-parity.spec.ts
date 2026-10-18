import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { compileBundle, serializeDocument } from '../../../zircon-zop-compiler/src/compiler.mjs';
import { fixtureBundle } from '../../../zircon-zop-compiler/src/fixture.mjs';
import { parseZuiDocument, serializeZuiDocument } from '../bridge/zui-document';

describe('operator compiler native ZUI v2 schema boundary', () => {
  it('passes every synthetic fixture document through the existing bridge parser and serializer', async () => {
    const contract = readFileSync(new URL('../../../../../../zircon_editor/assets/ui/operator/contract/roles.contract.toml', import.meta.url), 'utf8');
    const result = compileBundle(await fixtureBundle(), contract);
    for (const document of Object.values(result.documents)) {
      const existing = parseZuiDocument(serializeZuiDocument(document));
      const generated = parseZuiDocument(serializeDocument(document));
      expect(generated.document).toEqual(existing.document);
      expect(generated.diagnostics.filter(diagnostic => diagnostic.severity === 'error')).toEqual([]);
    }
  });
});
