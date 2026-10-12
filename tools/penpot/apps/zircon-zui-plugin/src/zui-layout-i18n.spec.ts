import { readFile, readdir } from 'node:fs/promises';
import { relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { parse as parseToml } from 'smol-toml';
import { beforeAll, describe, expect, it } from 'vitest';
import { prepareUnrealReviewTranslations } from '../tools/zui-layout-component-hosts';
import { LayoutDependencies } from '../tools/zui-layout-dependencies';
import {
  bytesSha256,
  dependencyFingerprints,
} from '../tools/zui-layout-evidence';
import { defaultReviewCases } from '../tools/zui-layout-review-contract';
import { projectZuiDocument } from './bridge/penpot-projection';
import { cloneZuiDocument, parseZuiDocument } from './bridge/zui-document';

const repo = resolve(
  process.env['ZUI_LAYOUT_REPO_ROOT'] ??
    resolve(fileURLToPath(new URL('.', import.meta.url)), '../../../../..'),
);
const outliner =
  'zircon_editor/assets/ui/editor/unreal/components/outliner.zui';
const details = 'zircon_editor/assets/ui/editor/unreal/components/details.zui';
const dependencies = new LayoutDependencies();

beforeAll(async () => {
  const files = await readdir(resolve(repo, 'zircon_editor/assets/ui'), {
    recursive: true,
    withFileTypes: true,
  });
  await dependencies.load(
    repo,
    files
      .filter((entry) => entry.isFile() && entry.name.endsWith('.zui'))
      .map((entry) =>
        relative(repo, resolve(entry.parentPath, entry.name)).replaceAll(
          '\\',
          '/',
        ),
      ),
  );
}, 30000);

describe('authored Unreal standalone review translations', () => {
  it.each([
    [outliner, 'WorkbenchSceneTitle', 'editor.workbench.outliner.title', 'en'],
    [
      outliner,
      'WorkbenchSceneTitle',
      'editor.workbench.outliner.title',
      'zh-CN',
    ],
    [details, 'UnrealDetailsDockTitle', 'editor.workbench.details.title', 'en'],
    [
      details,
      'UnrealDetailsDockTitle',
      'editor.workbench.details.title',
      'zh-CN',
    ],
  ])(
    'projects %s in %s without replacing authored text references',
    async (sourcePath, controlId, key, locale) => {
      const document = parseZuiDocument(
        await readFile(resolve(repo, sourcePath), 'utf8'),
      ).document;
      const cases = defaultReviewCases(
        sourcePath,
        document.asset.kind,
        document,
      );
      dependencies.embed(document, sourcePath, cases[0]?.themeSourcePath);
      const nodesBefore = structuredClone(document.nodes);
      expect(
        await prepareUnrealReviewTranslations(
          repo,
          sourcePath,
          document,
          undefined,
          { activeLocale: locale },
        ),
      ).toBe(true);
      const localePath = 'zircon_editor/assets/i18n/' + locale + '.toml';
      const localeBytes = await readFile(resolve(repo, localePath));
      const translations = parseToml(localeBytes.toString('utf8'))[
        'translations'
      ] as Record<string, string>;
      expect(
        projectZuiDocument(document).shapes.find(
          (node) => node.controlId === controlId,
        )?.text,
      ).toMatchObject({ characters: translations[key], property: null });
      expect(document.nodes).toEqual(nodesBefore);
      expect(
        (await dependencyFingerprints(repo, sourcePath, document)).find(
          (item) => item.sourcePath === localePath,
        ),
      ).toEqual({ sourcePath: localePath, sha256: bytesSha256(localeBytes) });
    },
    30000,
  );

  it('populates input and an existing consumer projection from the same real locale', async () => {
    const input = parseZuiDocument(
      await readFile(resolve(repo, outliner), 'utf8'),
    ).document;
    dependencies.embed(input, outliner);
    const consumer = cloneZuiDocument(input);
    consumer.asset.kind = 'view';
    consumer.root = { node: 'panel' };
    delete consumer.components;
    consumer['penpot_i18n_tables'] = {
      fixture: { locale: 'en', translations: {} },
    };
    input['penpot_dependency_sources'] = [outliner];
    await prepareUnrealReviewTranslations(repo, outliner, input, consumer);
    expect(input['penpot_i18n_tables']).toHaveProperty('editor.locale', 'en');
    expect(consumer['penpot_i18n_tables']).toHaveProperty(
      'editor.locale',
      'en',
    );
    expect(consumer['penpot_i18n_tables']).toHaveProperty('fixture');
    expect(input['penpot_dependency_sources']).toContain(outliner);
    expect(consumer['penpot_dependency_sources']).toContain(
      'zircon_editor/assets/i18n/en.toml',
    );
    expect(input['penpot_i18n_tables']).not.toBe(
      consumer['penpot_i18n_tables'],
    );
    for (const projection of [input, consumer])
      expect(
        projectZuiDocument(projection).shapes.find(
          (node) => node.controlId === 'WorkbenchSceneTitle',
        )?.text?.characters,
      ).toBe('Outliner');
  });

  it('keeps an unknown localized key fatal after loading real translations', async () => {
    const document = parseZuiDocument(
      await readFile(resolve(repo, details), 'utf8'),
    ).document;
    dependencies.embed(document, details);
    document.nodes!['title']!.props!['text'] = {
      text_key: 'editor.workbench.missing-test-key',
      table: 'editor',
    };
    await prepareUnrealReviewTranslations(repo, details, document);
    expect(() => projectZuiDocument(document)).toThrow(
      /Unresolved localized text editor:editor.workbench.missing-test-key/,
    );
  });

  it('rejects unsupported Unreal locales', async () => {
    const document = parseZuiDocument(
      await readFile(resolve(repo, details), 'utf8'),
    ).document;
    await expect(
      prepareUnrealReviewTranslations(repo, details, document, undefined, {
        activeLocale: 'fr',
      }),
    ).rejects.toThrow('Unreal review locale must be en or zh-CN');
    expect(document['penpot_i18n_tables']).toBeUndefined();
  });

  it('leaves unrelated catalog sources outside the Unreal locale preparation scope', async () => {
    const document = parseZuiDocument(
      await readFile(resolve(repo, details), 'utf8'),
    ).document;
    const before = cloneZuiDocument(document);
    expect(
      await prepareUnrealReviewTranslations(
        repo,
        'zircon_editor/assets/ui/editor/assets_activity.zui',
        document,
        undefined,
        { activeLocale: 'fr' },
      ),
    ).toBe(false);
    expect(document).toEqual(before);
  });
});
