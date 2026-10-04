import { createHash, randomUUID } from 'node:crypto';
import {
  lstat,
  mkdir,
  readFile,
  rename,
  rm,
  writeFile,
} from 'node:fs/promises';
import {
  basename,
  dirname,
  isAbsolute,
  parse as parsePath,
  relative,
  resolve,
  sep,
} from 'node:path';
import {
  assertExpectedRootHash,
  assertFingerprintClosure,
  applyError,
  errorMessage,
  isRecord,
  parseSourceExportManifest,
  staleSourceError,
  validateSourcePath,
  type ApplyZuiExportFileToSourceRootOptions,
  type AppliedZuiSourceExportOutput,
  type AppliedZuiSourceExports,
} from './zui-export-apply-contract';
import { applyZuiExportToSourceFiles } from './zui-export-apply-sources';
import { parseZuiDocument } from './zui-document';

/**
 * Apply a downloaded `.zui` source-owner envelope to isolated staged copies of
 * only the original assets whose visual fields changed.
 */
export async function applyZuiExportFileToSourceRoot(
  options: ApplyZuiExportFileToSourceRootOptions,
): Promise<AppliedZuiSourceExports> {
  const exportedSource = await readFile(options.exportedPath, 'utf8');
  const exported = parseZuiDocument(exportedSource);
  const rawManifest = exported.document['penpot_source_exports'];
  if (rawManifest === undefined) {
    return {
      appliedPathsBySource: {},
      outputs: [],
      diagnostics: exported.diagnostics,
    };
  }

  const manifest = parseSourceExportManifest(rawManifest);
  assertExpectedRootHash(
    options.expectedRootSha256,
    manifest.sources,
    manifest.rootSourcePath,
  );
  assertFingerprintClosure(manifest.sources, options.expectedSources);
  const sourceRoot = await resolveSafeRoot(options.sourceRootPath, 'canonical');
  const canonicalSources: Record<string, string> = {};
  for (const fingerprint of manifest.sources) {
    const segments = validateSourcePath(fingerprint.sourcePath);
    const absolute = resolve(sourceRoot, ...segments);
    if (!isPathInside(sourceRoot, absolute))
      throw applyError(
        'source-path-outside-root',
        `Source path ${fingerprint.sourcePath} resolves outside the explicit canonical source root.`,
      );
    canonicalSources[fingerprint.sourcePath] = await readCanonicalSource(
      sourceRoot,
      segments,
    );
  }

  const prepared = applyZuiExportToSourceFiles(
    exportedSource,
    canonicalSources,
    options.expectedSources,
  );
  if (prepared.outputs.length === 0) return prepared;

  const outputRoot = await prepareOutputRoot(
    options.outputRootPath,
    sourceRoot,
  );
  const staged: Array<{
    output: AppliedZuiSourceExportOutput;
    temporaryPath: string;
  }> = [];
  const renamed: AppliedZuiSourceExportOutput[] = [];
  try {
    for (const output of prepared.outputs) {
      const segments = validateSourcePath(output.sourcePath);
      const outputPath = resolve(outputRoot, ...segments);
      if (!isPathInside(outputRoot, outputPath))
        throw applyError(
          'output-path-outside-root',
          `Staged output ${output.sourcePath} resolves outside the explicit output root.`,
        );
      await ensureSafeOutputParent(outputRoot, segments.slice(0, -1));
      try {
        await lstat(outputPath);
        throw applyError(
          'staged-output-exists',
          `Refusing to replace an existing staged output: ${output.sourcePath}. Choose a fresh output root.`,
        );
      } catch (error) {
        if (!isMissingFile(error)) throw error;
      }
      const temporaryPath = resolve(
        dirname(outputPath),
        `.${basename(outputPath)}.zui-apply-${process.pid}-${randomUUID()}.tmp`,
      );
      await writeFile(temporaryPath, output.source, {
        encoding: 'utf8',
        flag: 'wx',
      });
      staged.push({
        output: { ...output, outputPath },
        temporaryPath,
      });
    }

    // Recheck the complete source closure after staging and before the first
    // visible rename, so any concurrent source edit makes the export stale.
    for (const fingerprint of manifest.sources) {
      const current = await readCanonicalSource(
        sourceRoot,
        validateSourcePath(fingerprint.sourcePath),
      );
      if (sha256(current) !== fingerprint.sha256)
        throw staleSourceError(fingerprint.sourcePath);
    }

    for (const item of staged) {
      try {
        await lstat(item.output.outputPath);
        throw applyError(
          'staged-output-raced',
          `A staged output appeared while the export was being prepared: ${item.output.sourcePath}.`,
        );
      } catch (error) {
        if (!isMissingFile(error)) throw error;
      }
      await rename(item.temporaryPath, item.output.outputPath);
      renamed.push(item.output);
    }
  } catch (error) {
    for (const item of staged) await rm(item.temporaryPath, { force: true });
    // Remove files created by this attempt if their contents still match.
    for (const output of renamed) {
      try {
        const current = await readFile(output.outputPath, 'utf8');
        if (current === output.source) await rm(output.outputPath, { force: true });
      } catch {
        // Preserve the original apply failure; a missing rollback target is benign.
      }
    }
    throw error;
  }

  return {
    appliedPathsBySource: prepared.appliedPathsBySource,
    outputs: staged.map(({ output }) => output),
    diagnostics: prepared.diagnostics,
  };
}

async function resolveSafeRoot(path: string, label: string): Promise<string> {
  const absolute = resolve(path);
  await rejectAliasedPath(absolute, label, false);
  const info = await lstat(absolute).catch((error: unknown) => {
    throw applyError(
      `${label}-root-invalid`,
      `The explicit ${label} root must exist as a real directory: ${errorMessage(error)}.`,
    );
  });
  if (!info.isDirectory() || info.isSymbolicLink())
    throw applyError(
      `${label}-root-invalid`,
      `The explicit ${label} root must be a real directory, not a link or file.`,
    );
  return absolute;
}

async function prepareOutputRoot(
  path: string,
  sourceRoot: string,
): Promise<string> {
  const absolute = resolve(path);
  if (samePath(absolute, sourceRoot) || isPathInside(sourceRoot, absolute))
    throw applyError(
      'output-root-overlaps-source-root',
      'The staged output root must be outside the canonical source root.',
    );
  await rejectAliasedPath(absolute, 'output', true);
  try {
    const existing = await lstat(absolute);
    if (!existing.isDirectory() || existing.isSymbolicLink())
      throw applyError(
        'output-root-invalid',
        'The explicit staged output root must be a real directory, not a link or file.',
      );
  } catch (error) {
    if (!isMissingFile(error)) throw error;
    await mkdir(absolute, { recursive: true });
  }
  const info = await lstat(absolute);
  if (!info.isDirectory() || info.isSymbolicLink())
    throw applyError(
      'output-root-invalid',
      'The explicit staged output root must be a real directory, not a link or file.',
    );
  const root = resolve(absolute);
  await rejectAliasedPath(root, 'output', false);
  if (samePath(root, sourceRoot) || isPathInside(sourceRoot, root))
    throw applyError(
      'output-root-overlaps-source-root',
      'The staged output root must be outside the canonical source root.',
    );
  return root;
}

async function rejectAliasedPath(
  path: string,
  label: string,
  allowMissingTail: boolean,
): Promise<void> {
  const absolute = resolve(path);
  const root = parsePath(absolute).root;
  let current = root;
  const segments = relative(root, absolute).split(sep).filter(Boolean);
  for (const [index, segment] of segments.entries()) {
    current = resolve(current, segment);
    let info;
    try {
      info = await lstat(current);
    } catch (error) {
      if (allowMissingTail && isMissingFile(error)) return;
      throw applyError(
        `${label}-root-invalid`,
        `The explicit ${label} root path could not be verified: ${errorMessage(error)}.`,
      );
    }
    if (info.isSymbolicLink())
      throw applyError(
        `${label}-root-alias`,
        `The explicit ${label} root path contains a symbolic link or junction: ${current}.`,
      );
    if (index < segments.length - 1 && !info.isDirectory())
      throw applyError(
        `${label}-root-invalid`,
        `The explicit ${label} root path contains a non-directory parent: ${current}.`,
      );
  }
}

async function readCanonicalSource(
  sourceRoot: string,
  segments: string[],
): Promise<string> {
  let current = sourceRoot;
  for (const [index, segment] of segments.entries()) {
    current = resolve(current, segment);
    const info = await lstat(current).catch((error: unknown) => {
      throw applyError(
        'source-missing',
        `Canonical source ${segments.join('/')} could not be read: ${errorMessage(error)}.`,
      );
    });
    if (info.isSymbolicLink())
      throw applyError(
        'source-path-alias',
        `Canonical source path ${segments.slice(0, index + 1).join('/')} contains a symbolic link or junction.`,
      );
    if (index < segments.length - 1 && !info.isDirectory())
      throw applyError(
        'source-path-invalid',
        `Canonical source path ${segments.slice(0, index + 1).join('/')} is not a directory.`,
      );
    if (index === segments.length - 1 && !info.isFile())
      throw applyError(
        'source-path-invalid',
        `Canonical source ${segments.join('/')} is not a regular file.`,
      );
  }
  return readFile(current, 'utf8');
}

async function ensureSafeOutputParent(
  outputRoot: string,
  segments: string[],
): Promise<void> {
  let current = outputRoot;
  for (const segment of segments) {
    current = resolve(current, segment);
    try {
      const info = await lstat(current);
      if (!info.isDirectory() || info.isSymbolicLink())
        throw applyError(
          'output-path-alias',
          `Staged output directory ${relative(outputRoot, current).split(sep).join('/')} is not a real directory.`,
        );
    } catch (error) {
      if (!isMissingFile(error)) throw error;
      await mkdir(current);
    }
  }
}

function isMissingFile(error: unknown): boolean {
  return isRecord(error) && error['code'] === 'ENOENT';
}

function samePath(left: string, right: string): boolean {
  const normalizedLeft = resolve(left);
  const normalizedRight = resolve(right);
  return process.platform === 'win32'
    ? normalizedLeft.toLowerCase() === normalizedRight.toLowerCase()
    : normalizedLeft === normalizedRight;
}

function isPathInside(root: string, target: string): boolean {
  const path = relative(root, target);
  return path === '' || (!path.startsWith(`..${sep}`) && path !== '..' && !isAbsolute(path));
}

function sha256(source: string): string {
  return createHash('sha256').update(source, 'utf8').digest('hex');
}
