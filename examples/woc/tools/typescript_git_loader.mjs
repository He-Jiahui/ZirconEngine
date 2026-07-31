import { execFileSync } from 'node:child_process';
import path from 'node:path';
import ts from 'typescript';

const repository = process.env.WOC_GIT_ROOT;
const commit = process.env.WOC_GIT_COMMIT;
if (!repository || !commit) {
  throw new Error('WOC_GIT_ROOT and WOC_GIT_COMMIT are required');
}

export async function resolve(specifier, context, nextResolve) {
  if (specifier.startsWith('wocgit:')) return { url: specifier, shortCircuit: true };
  if (!context.parentURL?.startsWith('wocgit:') ||
      (!specifier.startsWith('./') && !specifier.startsWith('../'))) {
    return nextResolve(specifier, context);
  }
  const parent = gitPath(context.parentURL);
  const base = path.posix.normalize(path.posix.join(path.posix.dirname(parent), specifier));
  if (base.startsWith('../') || path.posix.isAbsolute(base)) {
    throw new Error(`Git TypeScript import escapes source root: ${specifier}`);
  }
  const candidates = path.posix.extname(base) ? [base] : [`${base}.ts`, `${base}/index.ts`];
  for (const candidate of candidates) {
    if (gitPathExists(candidate)) return { url: gitUrl(candidate), shortCircuit: true };
  }
  throw new Error(`cannot resolve ${specifier} from ${context.parentURL}`);
}

export async function load(url, context, nextLoad) {
  if (!url.startsWith('wocgit:')) return nextLoad(url, context);
  const sourcePath = gitPath(url);
  const source = gitShow(sourcePath);
  const output = ts.transpileModule(source, {
    compilerOptions: {
      target: ts.ScriptTarget.ES2022,
      module: ts.ModuleKind.ESNext,
      verbatimModuleSyntax: true,
    },
    fileName: sourcePath,
  }).outputText;
  return { format: 'module', source: output, shortCircuit: true };
}

function gitPath(url) {
  return decodeURIComponent(new URL(url).pathname).replace(/^\/+/, '');
}

function gitUrl(sourcePath) {
  return `wocgit:///${sourcePath.split('/').map(encodeURIComponent).join('/')}`;
}

function gitPathExists(sourcePath) {
  try {
    execFileSync(
      'git',
      ['-C', repository, 'cat-file', '-e', `${commit}:${sourcePath}`],
      { stdio: 'ignore' },
    );
    return true;
  } catch {
    return false;
  }
}

function gitShow(sourcePath) {
  return execFileSync(
    'git',
    ['-C', repository, 'show', `${commit}:${sourcePath}`],
    { encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 },
  );
}
