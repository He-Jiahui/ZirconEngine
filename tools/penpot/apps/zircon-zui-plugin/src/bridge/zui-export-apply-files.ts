import { createHash, randomUUID } from 'node:crypto';
import {
  lstat,
  mkdir,
  open,
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
  type ApplyZuiExportFileToCanonicalSourceOwnersOptions,
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
  const { manifest, sourceRoot, prepared } = await prepareSourceOwnerExport(options);
  if (!manifest || prepared.outputs.length === 0) return prepared;

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
    await assertCanonicalSourceClosure(sourceRoot, manifest.sources);

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

/**
 * Replace explicitly authorized original owners after validating the whole
 * export. Each replacement is atomic; rollback restores bytes that still match
 * this attempt at its check. These checks are not a cross-process compare-and-swap
 * or a multi-file transaction.
 */
export async function applyZuiExportFileToCanonicalSourceOwners(
  options: ApplyZuiExportFileToCanonicalSourceOwnersOptions,
): Promise<AppliedZuiSourceExports> {
  if (!Array.isArray(options.writableSourcePaths) || options.writableSourcePaths.length === 0)
    throw applyError('canonical-owners-required', 'Explicit writable source owners are required.');
  const { manifest, sourceRoot, canonicalSources, prepared } = await prepareSourceOwnerExport(options);
  if (!manifest)
    throw applyError('source-exports-required', 'Canonical owner application requires a source-owner export envelope.');
  const ownerPaths = manifest.sources.map(({ sourcePath }) =>
    resolve(sourceRoot, ...validateSourcePath(sourcePath)),
  );
  const writable: string[] = [];
  for (const path of options.writableSourcePaths) {
    const target = resolve(sourceRoot, ...validateSourcePath(path));
    if (!ownerPaths.some((owner) => samePath(owner, target)))
      throw applyError('canonical-owner-outside-closure', 'A writable owner is outside the trusted source closure.');
    if (writable.some((owner) => samePath(owner, target)))
      throw applyError('canonical-owner-duplicate', 'A writable source owner was listed more than once.');
    writable.push(target);
  }
  for (const output of prepared.outputs) {
    const target = resolve(sourceRoot, ...validateSourcePath(output.sourcePath));
    if (!writable.some((path) => samePath(path, target)))
      throw applyError('canonical-owner-not-writable', 'A changed source owner is outside the explicit writable owner list.');
  }
  const staged: CanonicalOwnerStage[] = [];
  const committed: CanonicalOwnerStage[] = [];
  let failed = false;
  let failure: unknown;
  const cleanupFailures: string[] = [];
  try {
    for (const output of prepared.outputs) {
      const outputPath = resolve(sourceRoot, ...validateSourcePath(output.sourcePath));
      await readCanonicalSource(sourceRoot, validateSourcePath(output.sourcePath));
      const item: CanonicalOwnerStage = {
        output: { ...output, outputPath },
        temporaryPath: resolve(
          dirname(outputPath),
          `.${basename(outputPath)}.zui-apply-${process.pid}-${randomUUID()}.tmp`,
        ),
        temporaryOwned: false,
      };
      staged.push(item);
      await writeOwnedTemporary(item, output.source, (await lstat(outputPath)).mode);
    }
    await assertCanonicalSourceClosure(sourceRoot, manifest.sources);
    for (const item of staged) {
      const path = item.output.sourcePath;
      const current = await readCanonicalSource(sourceRoot, validateSourcePath(path));
      if (current !== canonicalSources[path]) throw staleSourceError(path);
      await assertOwnedTemporary(item, item.output.source);
      await rename(item.temporaryPath, item.output.outputPath);
      item.temporaryOwned = false;
      committed.push(item);
    }
    const expected = manifest.sources.map((fingerprint) => {
      const output = prepared.outputs.find((item) => item.sourcePath === fingerprint.sourcePath);
      return output ? { ...fingerprint, sha256: sha256(output.source) } : fingerprint;
    });
    await assertCanonicalSourceClosure(sourceRoot, expected);
  } catch (error) {
    failed = true;
    failure = error;
    const rollbackFailures: string[] = [];
    for (const item of committed.slice().reverse()) {
      const path = item.output.sourcePath;
      try {
        const current = await readCanonicalSource(sourceRoot, validateSourcePath(path));
        if (current !== item.output.source) throw staleSourceError(path);
        await writeOwnedTemporary(item, canonicalSources[path], (await lstat(item.output.outputPath)).mode);
        if (await readCanonicalSource(sourceRoot, validateSourcePath(path)) !== item.output.source)
          throw staleSourceError(path);
        await assertOwnedTemporary(item, canonicalSources[path]);
        await rename(item.temporaryPath, item.output.outputPath);
        item.temporaryOwned = false;
      } catch {
        rollbackFailures.push(path);
      }
    }
    if (rollbackFailures.length)
      failure = applyError(
        'canonical-rollback-incomplete',
        'Apply failed: ' + errorMessage(error) + '. Preserved changed or inaccessible owners: ' + rollbackFailures.join(', ') + '.',
      );
  } finally {
    for (const item of staged) {
      if (!item.temporaryOwned) continue;
      try {
        await assertOwnedTemporary(item, item.temporarySource);
        await rm(item.temporaryPath);
        item.temporaryOwned = false;
      } catch (error) {
        if (!isMissingFile(error)) cleanupFailures.push(item.temporaryPath);
      }
    }
  }
  if (failed) {
    if (failure instanceof Error && cleanupFailures.length)
      Object.defineProperty(failure, 'cleanupFailures', { value: cleanupFailures });
    throw failure;
  }
  if (cleanupFailures.length)
    throw applyError('canonical-temp-cleanup-failed', 'Owner application completed but temporary cleanup failed: ' + cleanupFailures.join(', '));
  return { ...prepared, outputs: staged.map(({ output }) => output) };
}

interface CanonicalOwnerStage {
  output: AppliedZuiSourceExportOutput;
  temporaryPath: string;
  temporaryOwned: boolean;
  temporaryIdentity?: { dev: number; ino: number };
  temporarySource?: string;
}

async function writeOwnedTemporary(item: CanonicalOwnerStage, source: string, mode: number): Promise<void> {
  const handle = await open(item.temporaryPath, 'wx', mode);
  item.temporaryOwned = true;
  item.temporarySource = source;
  try {
    const identity = await handle.stat();
    item.temporaryIdentity = { dev: identity.dev, ino: identity.ino };
    await handle.writeFile(source, 'utf8');
    await handle.sync();
  } finally {
    await handle.close();
  }
}

async function assertOwnedTemporary(item: CanonicalOwnerStage, expectedSource: string | undefined): Promise<void> {
  const matchesIdentity = (info: Awaited<ReturnType<typeof lstat>>) =>
    info.isFile() && !info.isSymbolicLink() && item.temporaryIdentity !== undefined &&
    info.dev === item.temporaryIdentity.dev && info.ino === item.temporaryIdentity.ino;
  if (!item.temporaryOwned || expectedSource === undefined || !matchesIdentity(await lstat(item.temporaryPath)))
    throw applyError('canonical-temp-changed', 'Canonical owner temporary file identity or content changed.');
  const handle = await open(item.temporaryPath, 'r');
  try {
    const identity = await handle.stat();
    if (!identity.isFile() || identity.dev !== item.temporaryIdentity!.dev ||
        identity.ino !== item.temporaryIdentity!.ino ||
        !(await handle.readFile()).equals(Buffer.from(expectedSource, 'utf8')) ||
        !matchesIdentity(await lstat(item.temporaryPath)))
      throw applyError('canonical-temp-changed', 'Canonical owner temporary file identity or content changed.');
  } finally {
    await handle.close();
  }
}

async function prepareSourceOwnerExport(options: Omit<ApplyZuiExportFileToSourceRootOptions, 'outputRootPath'>) {
  const exportedSource = await readFile(options.exportedPath, 'utf8');
  const exported = parseZuiDocument(exportedSource);
  const rawManifest = exported.document['penpot_source_exports'];
  if (rawManifest === undefined)
    return { manifest: null, sourceRoot: '', canonicalSources: {} as Record<string, string>,
      prepared: { appliedPathsBySource: {}, outputs: [], diagnostics: exported.diagnostics } as AppliedZuiSourceExports };
  const manifest = parseSourceExportManifest(rawManifest);
  assertExpectedRootHash(options.expectedRootSha256, manifest.sources, manifest.rootSourcePath);
  assertFingerprintClosure(manifest.sources, options.expectedSources);
  const sourceRoot = await resolveSafeRoot(options.sourceRootPath, 'canonical');
  const canonicalSources: Record<string, string> = {};
  for (const fingerprint of manifest.sources) {
    const segments = validateSourcePath(fingerprint.sourcePath);
    const absolute = resolve(sourceRoot, ...segments);
    if (!isPathInside(sourceRoot, absolute))
      throw applyError('source-path-outside-root', `Source path ${fingerprint.sourcePath} resolves outside the explicit canonical source root.`);
    canonicalSources[fingerprint.sourcePath] = await readCanonicalSource(sourceRoot, segments);
  }
  const prepared = applyZuiExportToSourceFiles(exportedSource, canonicalSources, options.expectedSources);
  return { manifest, sourceRoot, canonicalSources, prepared };
}

async function assertCanonicalSourceClosure(
  sourceRoot: string,
  sources: readonly { sourcePath: string; sha256: string }[],
): Promise<void> {
  await rejectAliasedPath(sourceRoot, 'canonical', false);
  for (const fingerprint of sources) {
    const current = await readCanonicalSource(sourceRoot, validateSourcePath(fingerprint.sourcePath));
    if (sha256(current) !== fingerprint.sha256) throw staleSourceError(fingerprint.sourcePath);
  }
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
  const bytes = await readFile(current);
  const source = bytes.toString('utf8');
  if (!Buffer.from(source, 'utf8').equals(bytes))
    throw applyError('source-encoding-invalid', 'Canonical source must round-trip as UTF-8 before owner application.');
  return source;
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
