import { sha256 } from './compiler.mjs';
import { safeEntry } from './contract.mjs';

export function generatedPath(path) {
  safeEntry(path);
  if (!/^(shell|panels|components|theme|icons)\/[a-zA-Z0-9_-]+\.(zui|roles\.json|svg)$/.test(path)) throw new Error(`unowned generated path ${path}`);
  return path;
}

export function planApply(existing, generated) {
  const decoder = new TextDecoder();
  const nextLock = JSON.parse(decoder.decode(generated['zop-bundle.lock.json']));
  const previous = existing['zop-bundle.lock.json'] ? JSON.parse(decoder.decode(existing['zop-bundle.lock.json'])) : { outputs: {} };
  if (nextLock.schema_version !== 1 || !nextLock.outputs || !previous.outputs || nextLock.include_preview) throw new Error('invalid generated lock or preview output cannot ship');
  const writes = {}, deletes = [];
  for (const [path, hash] of Object.entries(previous.outputs)) {
    generatedPath(path);
    if (!existing[path] || sha256(existing[path]) !== hash) throw new Error(`locally modified or missing generated file ${path}`);
    if (!Object.hasOwn(nextLock.outputs, path)) deletes.push(path);
  }
  for (const [path, hash] of Object.entries(nextLock.outputs)) {
    generatedPath(path);
    if (!generated[path] || sha256(generated[path]) !== hash) throw new Error(`corrupt generated file ${path}`);
    if (existing[path] && !Object.hasOwn(previous.outputs, path)) throw new Error(`refusing to overwrite unowned file ${path}`);
    writes[path] = generated[path];
  }
  for (const path of Object.keys(generated)) if (path !== 'zop-bundle.lock.json' && !Object.hasOwn(nextLock.outputs, path)) throw new Error(`unlocked generated file ${path}`);
  writes['zop-bundle.lock.json'] = generated['zop-bundle.lock.json'];
  return { writes, deletes };
}
