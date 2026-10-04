import { existsSync, realpathSync, statSync } from 'node:fs';
import { dirname, resolve } from 'node:path';

/** Resolve an existing artifact directory below the approved drive-root Cargo storage. */
export function requireApprovedPenpotArtifactDirectory(
  directory: string,
  label: string,
): string {
  const candidate = resolve(directory);
  const approvedRoots = [
    'D:\\cargo-targets',
    'E:\\cargo-targets',
    'F:\\cargo-targets',
  ];
  const normalize = (value: string): string => {
    const resolved = value.replaceAll('/', '\\');
    const withoutDevicePrefix = resolved.startsWith('\\\\?\\')
      ? resolved.slice(4)
      : resolved;
    return withoutDevicePrefix.replace(/[\\/]+$/, '').toLowerCase();
  };
  const isWithinRoot = (path: string, root: string): boolean =>
    path === root || path.startsWith(`${root}\\`);
  const candidatePath = normalize(candidate);
  const lexicalRoot = approvedRoots.find((root) =>
    isWithinRoot(candidatePath, normalize(root)),
  );
  if (!lexicalRoot) {
    throw new Error(
      `${label} must be a directory below D:\\cargo-targets, E:\\cargo-targets, or F:\\cargo-targets; received ${candidate}.`,
    );
  }
  if (!existsSync(candidate) || !statSync(candidate).isDirectory()) {
    throw new Error(`${label} must be an existing directory: ${candidate}.`);
  }
  const physicalRoot = realpathSync.native(lexicalRoot);
  const physicalCandidate = realpathSync.native(candidate);
  const physicalRootPath = normalize(physicalRoot);
  const physicalCandidatePath = normalize(physicalCandidate);
  if (
    normalize(physicalRoot) !== normalize(lexicalRoot) ||
    !isWithinRoot(physicalCandidatePath, physicalRootPath)
  ) {
    throw new Error(
      `${label} resolves outside its approved physical artifact root: ${candidate}.`,
    );
  }
  return candidate;
}

/** Resolve an existing build artifact file below a physically approved root. */
export function requireApprovedPenpotArtifactFile(
  file: string,
  label: string,
): string {
  const candidate = resolve(file);
  const parent = requireApprovedPenpotArtifactDirectory(
    dirname(candidate),
    `${label} parent`,
  );
  if (!existsSync(candidate) || !statSync(candidate).isFile())
    throw new Error(`${label} must be an existing file: ${candidate}.`);
  const physicalParent = realpathSync.native(parent);
  const physicalFile = realpathSync.native(candidate);
  const normalize = (value: string): string =>
    value
      .replaceAll('/', '\\')
      .replace(/^\\\\\?\\/, '')
      .replace(/[\\/]+$/, '')
      .toLowerCase();
  if (!normalize(physicalFile).startsWith(`${normalize(physicalParent)}\\`)) {
    throw new Error(
      `${label} resolves outside its artifact directory: ${candidate}.`,
    );
  }
  return candidate;
}
