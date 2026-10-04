import { readFile, writeFile } from 'node:fs/promises';
import { basename } from 'node:path';
import { applyZuiExportFileToCanonicalSource } from './bridge/zui-export-apply';
import {
  applyZuiExportFileToSourceRoot,
  type ZuiSourceFingerprint,
} from './bridge/zui-export-apply';
import {
  createPenpotBridgeAsset,
  parsePenpotBridgeAsset,
  reconcilePenpotBridgeAsset,
  serializePenpotBridgeAsset,
} from './bridge/penpot-asset';
import { parseZuiDocument, serializeZuiDocument } from './bridge/zui-document';

const USAGE = `Usage:
  pnpm bridge project <input.zui> <output.penpot.json>
  pnpm bridge reconcile <input.penpot.json> <output.zui>
  pnpm bridge roundtrip <input.zui> <output.zui>
  pnpm bridge apply-export <exported.zui> <canonical.zui> <output.zui> --expected-sha256 <baseline-source-sha256>
  pnpm bridge apply-export-sources <exported.zui> <canonical-repo-root> <staged-output-root> --expected-root-sha256 <baseline-root-sha256> --expected-sources <source-fingerprints.json>`;

async function main(): Promise<void> {
  const [command, ...args] = process.argv.slice(2);
  if (!command) throw new Error(USAGE);

  if (command === 'apply-export') {
    const [exportedPath, canonicalPath, outputPath, hashFlag, expectedSha256] =
      args;
    if (
      args.length !== 5 ||
      !exportedPath ||
      !canonicalPath ||
      !outputPath ||
      hashFlag !== '--expected-sha256' ||
      !expectedSha256
    )
      throw new Error(USAGE);
    const result = await applyZuiExportFileToCanonicalSource({
      exportedPath,
      canonicalPath,
      outputPath,
      expectedCanonicalSha256: expectedSha256,
    });
    console.log(
      `Applied ${result.appliedPaths.length} visual source edits to ${outputPath}.`,
    );
    for (const path of result.appliedPaths) console.log(`  ${path}`);
    return;
  }

  if (command === 'apply-export-sources') {
    const [
      exportedPath,
      sourceRootPath,
      outputRootPath,
      rootHashFlag,
      expectedRootSha256,
      sourcesFlag,
      expectedSourcesPath,
    ] = args;
    if (
      args.length !== 7 ||
      !exportedPath ||
      !sourceRootPath ||
      !outputRootPath ||
      rootHashFlag !== '--expected-root-sha256' ||
      !expectedRootSha256 ||
      sourcesFlag !== '--expected-sources' ||
      !expectedSourcesPath
    )
      throw new Error(USAGE);
    let expectedSources: unknown;
    try {
      expectedSources = JSON.parse(
        await readFile(expectedSourcesPath, 'utf8'),
      );
    } catch (error) {
      throw new Error(
        `Expected source fingerprints could not be read: ${error instanceof Error ? error.message : String(error)}`,
      );
    }
    if (
      !Array.isArray(expectedSources) ||
      expectedSources.some(
        (item) =>
          typeof item !== 'object' ||
          item === null ||
          typeof item['sourcePath'] !== 'string' ||
          typeof item['sha256'] !== 'string',
      )
    )
      throw new Error(
        'Expected source fingerprints must be a JSON array of { sourcePath, sha256 } objects.',
      );
    const result = await applyZuiExportFileToSourceRoot({
      exportedPath,
      sourceRootPath,
      outputRootPath,
      expectedRootSha256,
      expectedSources: expectedSources as ZuiSourceFingerprint[],
    });
    console.log(
      `Staged ${result.outputs.length} visual owner source update${result.outputs.length === 1 ? '' : 's'} under ${outputRootPath}.`,
    );
    for (const [sourcePath, paths] of Object.entries(result.appliedPathsBySource))
      for (const path of paths)
        console.log(`${sourcePath}: ${path}`);
    return;
  }

  const [inputPath, outputPath] = args;
  if (!inputPath || !outputPath || args.length !== 2) throw new Error(USAGE);

  if (command === 'project') {
    const source = await readFile(inputPath, 'utf8');
    const { document } = parseZuiDocument(source);
    const asset = createPenpotBridgeAsset(document, basename(inputPath));
    await writeFile(outputPath, serializePenpotBridgeAsset(asset), 'utf8');
    return;
  }

  if (command === 'reconcile') {
    const source = await readFile(inputPath, 'utf8');
    const result = reconcilePenpotBridgeAsset(parsePenpotBridgeAsset(source));
    await writeFile(
      outputPath,
      result.source ?? serializeZuiDocument(result.document),
      'utf8',
    );
    return;
  }

  if (command === 'roundtrip') {
    const source = await readFile(inputPath, 'utf8');
    const { document } = parseZuiDocument(source);
    const asset = createPenpotBridgeAsset(document, basename(inputPath));
    const result = reconcilePenpotBridgeAsset(asset);
    await writeFile(
      outputPath,
      result.source ?? serializeZuiDocument(result.document),
      'utf8',
    );
    return;
  }

  throw new Error(`Unknown bridge command ${command}.\n\n${USAGE}`);
}

void main().catch((error: unknown) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
});
