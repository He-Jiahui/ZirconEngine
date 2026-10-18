import { safeEntry, stableJson, VERSION } from './contract.mjs';

const utf8 = new TextEncoder();
export async function createBundle(entries, files, digest) {
  if (!files.length || files.some(file => !file.id || !Number.isSafeInteger(file.revision) || file.revision < 0)) throw new Error('Export needs actual Penpot file IDs and revisions');
  const hashes = {};
  for (const path of Object.keys(entries).sort()) {
    safeEntry(path);
    if (path === 'manifest.json') throw new Error('manifest.json is reserved for the bundle manifest');
    hashes[path] = await digest(entries[path]);
  }
  const manifest = { schema_version: 1, plugin_version: VERSION, compiler_version: VERSION,
    files, entries: hashes };
  return { manifest, entries: { ...entries, 'manifest.json': utf8.encode(stableJson(manifest) + '\n') } };
}

function crc32(bytes) {
  let crc = 0xffffffff;
  for (const byte of bytes) {
    crc ^= byte;
    for (let bit = 0; bit < 8; bit++) crc = (crc >>> 1) ^ ((crc & 1) ? 0xedb88320 : 0);
  }
  return (crc ^ 0xffffffff) >>> 0;
}

// Store-only ZIP, sorted entries, UTF-8 names, fixed 1980 timestamp. No wall clock
// or host-specific compression parameters participate in the exported bytes.
export function zipBundle(entries) {
  const chunks = [], directory = [];
  let offset = 0;
  for (const path of Object.keys(entries).sort()) {
    if (path !== 'manifest.json') safeEntry(path);
    const name = utf8.encode(path), bytes = entries[path], crc = crc32(bytes);
    if (name.length > 0xffff || bytes.length > 0xffffffff || offset > 0xffffffff) throw new Error('ZIP64 export is unsupported');
    const local = new Uint8Array(30 + name.length);
    const header = new DataView(local.buffer);
    header.setUint32(0, 0x04034b50, true); header.setUint16(4, 20, true); header.setUint16(6, 0x800, true);
    header.setUint16(12, 0x21, true); header.setUint32(14, crc, true);
    header.setUint32(18, bytes.length, true); header.setUint32(22, bytes.length, true); header.setUint16(26, name.length, true);
    local.set(name, 30);
    chunks.push(local, bytes);
    const central = new Uint8Array(46 + name.length), view = new DataView(central.buffer);
    view.setUint32(0, 0x02014b50, true); view.setUint16(4, 20, true); view.setUint16(6, 20, true); view.setUint16(8, 0x800, true);
    view.setUint16(14, 0x21, true); view.setUint32(16, crc, true); view.setUint32(20, bytes.length, true); view.setUint32(24, bytes.length, true);
    view.setUint16(28, name.length, true); view.setUint32(42, offset, true); central.set(name, 46);
    directory.push(central); offset += local.length + bytes.length;
  }
  const centralSize = directory.reduce((sum, bytes) => sum + bytes.length, 0);
  if (directory.length > 0xffff || offset + centralSize > 0xffffffff) throw new Error('ZIP64 export is unsupported');
  const end = new Uint8Array(22), footer = new DataView(end.buffer);
  footer.setUint32(0, 0x06054b50, true); footer.setUint16(8, directory.length, true); footer.setUint16(10, directory.length, true);
  footer.setUint32(12, centralSize, true); footer.setUint32(16, offset, true);
  const output = new Uint8Array(offset + centralSize + end.length);
  let cursor = 0;
  for (const chunk of [...chunks, ...directory, end]) { output.set(chunk, cursor); cursor += chunk.length; }
  return output;
}
