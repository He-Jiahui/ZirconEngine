import type { Board, Shape } from '@penpot/plugin-types';
import { createBundle, zipBundle } from '../../../zircon-zop-compiler/src/bundle.mjs';
import { stableJson } from '../../../zircon-zop-compiler/src/contract.mjs';
import { captureTokens, normalizeBoard, readAnnotation } from './normalizer.mjs';
import { zopContract } from './annotate';

function selectedBoard(screen: string): Board {
  let shape: Shape | null = penpot.selection[0] ?? null;
  let candidate: Board | null = null;
  const seen = new Set<string>();
  while (shape) {
    if (seen.has(shape.id)) throw new Error('Cyclic Penpot selection ancestry');
    seen.add(shape.id);
    const role = readAnnotation(shape)['role'];
    if (shape.type === 'board' && typeof role === 'string') {
      const definition = zopContract().roles[role];
      if (definition?.screen === screen && definition.kind === 'region') candidate = shape;
    }
    shape = shape.parent;
  }
  if (!candidate) throw new Error('Select a board annotated with the screen root role, or one of its layers.');
  return candidate;
}

export async function exportZop(screen: string, assetPath: string): Promise<{ fileName: string; bytes: Uint8Array }> {
  const file = penpot.currentFile;
  if (!file) throw new Error('Save the Penpot file before export.');
  const board = selectedBoard(screen);
  const revision = file.revn;
  const tree = normalizeBoard(board, zopContract(), screen, assetPath, [file.id]);
  const tokens = captureTokens(penpot.library.local.tokens);
  const png = await board.export({ type: 'png', scale: 1 });
  if (penpot.currentFile?.id !== file.id || penpot.currentFile.revn !== revision) throw new Error('Penpot file changed during export; retry on a stable revision.');
  if (!(png instanceof Uint8Array) || png.length < 24 || png[0] !== 0x89 || png[1] !== 0x50) throw new Error('Penpot did not return a valid board PNG.');
  const encode = (value: unknown) => new TextEncoder().encode(stableJson(value) + '\n');
  const entries = { [`screens/${board.id}.json`]: encode(tree), 'tokens.dtcg.json': encode(tokens), [`previews/${board.id}.png`]: png };
  const bundle = await createBundle(entries, [{ id: file.id, revision }], async bytes => {
    const owned = new Uint8Array(bytes);
    const hash = await crypto.subtle.digest('SHA-256', owned.buffer);
    return [...new Uint8Array(hash)].map(value => value.toString(16).padStart(2, '0')).join('');
  });
  return { fileName: `zop-bundle-${file.id}-${revision}.zip`, bytes: zipBundle(bundle.entries) };
}
