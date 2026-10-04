import { parse, stringify, type TomlTable } from 'smol-toml';
import {
  ZuiDocumentError,
  type ZuiDiagnostic,
  type ZuiDocument,
} from './zui-document';

interface DocumentChange {
  path: Array<string | number>;
  afterExists: boolean;
  after: unknown;
}

interface TextPatch {
  start: number;
  end: number;
  value: string;
}

interface NodeTableSection {
  nodeId: string;
  path: string[];
  start: number;
  end: number;
  lineEnding: string;
}

interface TomlMember {
  valueStart: number;
  valueEnd: number;
}

interface EditTrie {
  afterExists: boolean;
  after: unknown;
  children: Map<string, EditTrie>;
}

export function patchCanonicalZuiSource(
  source: string,
  canonical: ZuiDocument,
  exported: ZuiDocument,
  changedPaths: ReadonlyArray<ReadonlyArray<string | number>>,
): string {
  const changes = changedPaths.map((path) => {
    const after = getDocumentValue(exported, [...path]);
    if (!after.exists)
      throw applyError(
        'visual-property-removal-unsupported',
        'Removing mapped visual properties is not supported: ' +
          path.map(String).join('.') +
          '.',
      );
    return { path: [...path], afterExists: true, after: after.value };
  });
  return patchNodeTableSource(source, canonical, exported, changes);
}

function patchNodeTableSource(
  source: string,
  canonical: ZuiDocument,
  exported: ZuiDocument,
  changes: DocumentChange[],
): string {
  const fieldsByNode = new Map<string, Map<string, DocumentChange[]>>();
  for (const change of changes) {
    const [root, nodeId, field] = change.path;
    if (
      root !== 'nodes' ||
      typeof nodeId !== 'string' ||
      typeof field !== 'string'
    )
      throw applyError(
        'source-path-invalid',
        `Cannot apply exported path ${change.path.map(String).join('.')}.`,
      );
    const fields =
      fieldsByNode.get(nodeId) ?? new Map<string, DocumentChange[]>();
    const fieldChanges = fields.get(field) ?? [];
    fieldChanges.push(change);
    fields.set(field, fieldChanges);
    fieldsByNode.set(nodeId, fields);
  }

  const { sections } = scanNodeTables(source);
  const patches: TextPatch[] = [];
  for (const [nodeId, fields] of fieldsByNode) {
    const sectionMatches = sections.filter(
      (section) => section.nodeId === nodeId && section.path.length === 2,
    );
    if (sectionMatches.length !== 1) {
      throw applyError(
        'canonical-node-source-unmapped',
        `Canonical TOML does not have one direct [nodes.${nodeId}] table for source edits.`,
      );
    }
    const section = sectionMatches[0];
    const assignments = scanDirectAssignments(source, section);
    const canonicalNode = canonical.nodes?.[nodeId];
    const exportedNode = exported.nodes?.[nodeId];
    if (!canonicalNode || !exportedNode)
      throw applyError(
        'canonical-node-source-unmapped',
        `Canonical node ${nodeId} is missing while applying its visual edit.`,
      );

    for (const [field, fieldChanges] of fields) {
      const exportValue = exportedNode[field];
      if (exportValue === undefined) {
        throw applyError(
          'visual-property-removal-unsupported',
          `Removing nodes.${nodeId}.${field} is not supported.`,
        );
      }
      const trie = buildEditTrie(fieldChanges, exported, [
        'nodes',
        nodeId,
        field,
      ]);
      const assignment = assignments.get(field);
      if (!assignment) {
        if (canonicalNode[field] !== undefined) {
          patches.push(
            ...patchDottedTableChanges(
              source,
              sections,
              fieldChanges,
              exported,
            ),
          );
          continue;
        }
        const value = renderTomlValue(exportValue);
        patches.push({
          start: section.end,
          end: section.end,
          value: `${tableInsertionPrefix(source, section)}${field} = ${value}${section.lineEnding}`,
        });
        continue;
      }
      if (source[assignment.valueStart] !== '{') {
        throw applyError(
          'canonical-table-layout-unsupported',
          `Canonical nodes.${nodeId}.${field} is not an inline TOML table; this edit needs an explicit source mapping.`,
        );
      }
      patches.push(
        ...patchInlineValue(
          source,
          assignment.valueStart,
          assignment.valueEnd,
          trie,
          `nodes.${nodeId}.${field}`,
        ),
      );
    }
  }

  patches.sort((left, right) => right.start - left.start);
  let output = source;
  for (const patch of patches) {
    output =
      output.slice(0, patch.start) + patch.value + output.slice(patch.end);
  }
  return output;
}

function patchDottedTableChanges(
  source: string,
  sections: NodeTableSection[],
  changes: DocumentChange[],
  exported: ZuiDocument,
): TextPatch[] {
  const groups = new Map<
    string,
    { section: NodeTableSection; key: string; changes: DocumentChange[] }
  >();
  for (const change of changes) {
    const section = sections
      .filter(
        (candidate) =>
          candidate.path.length >= 3 &&
          candidate.path.length < change.path.length &&
          candidate.path.every((part, index) => part === change.path[index]),
      )
      .sort((left, right) => right.path.length - left.path.length)[0];
    if (!section)
      throw applyError(
        'canonical-field-source-unmapped',
        `Cannot locate the TOML table for ${change.path.join('.')}.`,
      );
    const key = String(change.path[section.path.length]);
    const id = JSON.stringify([...section.path, key]);
    const group = groups.get(id) ?? { section, key, changes: [] };
    group.changes.push(change);
    groups.set(id, group);
  }
  const patches: TextPatch[] = [];
  for (const { section, key, changes: edits } of groups.values()) {
    const path = [...section.path, key];
    const assignment = scanDirectAssignments(source, section).get(key);
    const trie = buildEditTrie(edits, exported, path);
    if (assignment) {
      patches.push(
        ...patchInlineValue(
          source,
          assignment.valueStart,
          assignment.valueEnd,
          trie,
          path.join('.'),
        ),
      );
    } else {
      patches.push({
        start: section.end,
        end: section.end,
        value: `${tableInsertionPrefix(source, section)}${renderTomlKey(key)} = ${renderTomlValue(trie.after)}${section.lineEnding}`,
      });
    }
  }
  return patches;
}

function tableInsertionPrefix(
  source: string,
  section: NodeTableSection,
): string {
  return section.end > 0 && source[section.end - 1] !== '\n'
    ? section.lineEnding
    : '';
}

function buildEditTrie(
  changes: DocumentChange[],
  exported: ZuiDocument,
  basePath: Array<string | number>,
): EditTrie {
  const rootAfter = getDocumentValue(exported, basePath);
  const root: EditTrie = {
    afterExists: rootAfter.exists,
    after: rootAfter.value,
    children: new Map(),
  };
  for (const change of changes) {
    let cursor = root;
    const relative = change.path.slice(basePath.length);
    const absolute = [...basePath];
    for (const segment of relative) {
      const key = String(segment);
      let child = cursor.children.get(key);
      if (!child) {
        child = { afterExists: false, after: undefined, children: new Map() };
        cursor.children.set(key, child);
      }
      cursor = child;
      absolute.push(segment);
      const value = getDocumentValue(exported, absolute);
      cursor.afterExists = value.exists;
      cursor.after = value.value;
    }
  }
  return root;
}

function patchInlineValue(
  source: string,
  valueStart: number,
  valueEnd: number,
  edit: EditTrie,
  path: string,
): TextPatch[] {
  if (edit.children.size === 0) {
    if (!edit.afterExists)
      throw applyError(
        'visual-property-removal-unsupported',
        `Removing mapped visual properties is not supported: ${path}`,
        path,
      );
    return [
      {
        start: valueStart,
        end: valueEnd,
        value: renderTomlValue(edit.after),
      },
    ];
  }
  if (source[valueStart] !== '{') {
    if (edit.afterExists) {
      return [
        {
          start: valueStart,
          end: valueEnd,
          value: renderTomlValue(edit.after),
        },
      ];
    }
    throw applyError(
      'canonical-value-source-unmapped',
      `Cannot patch nested TOML value ${path}.`,
      path,
    );
  }

  const close = findInlineTableEnd(source, valueStart, valueEnd);
  const patches: TextPatch[] = [];
  const additions: Array<[string, unknown]> = [];
  for (const [key, childEdit] of edit.children) {
    const member = findInlineTableMember(source, valueStart, close, key);
    if (!member) {
      if (!childEdit.afterExists)
        throw applyError(
          'canonical-value-source-unmapped',
          `Cannot remove missing TOML field ${path}.${key}.`,
        );
      additions.push([key, childEdit.after]);
      continue;
    }
    patches.push(
      ...patchInlineValue(
        source,
        member.valueStart,
        member.valueEnd,
        childEdit,
        `${path}.${key}`,
      ),
    );
  }
  if (additions.length > 0) {
    const closingTriviaStart = trimWhitespaceEnd(source, valueStart + 1, close);
    const content = source.slice(valueStart + 1, closingTriviaStart);
    const insertion = additions
      .map(
        ([key, value]) => `${renderTomlKey(key)} = ${renderTomlValue(value)}`,
      )
      .join(', ');
    const separator =
      content.trim() === ''
        ? ` ${insertion} `
        : /,\s*$/.test(content)
          ? ` ${insertion},`
          : `, ${insertion}`;
    patches.push({
      start: closingTriviaStart,
      end: closingTriviaStart,
      value: separator,
    });
  }
  return patches;
}

function scanNodeTables(source: string): { sections: NodeTableSection[] } {
  const sections: NodeTableSection[] = [];
  const lines = scanLines(source);
  let active: NodeTableSection | undefined;
  for (const line of lines) {
    const header = parseTableHeader(source.slice(line.start, line.contentEnd));
    if (!header) continue;
    if (active) {
      active.end = line.start;
      sections.push(active);
      active = undefined;
    }
    if (header.path.length >= 2 && header.path[0] === 'nodes') {
      active = {
        nodeId: header.path[1],
        path: header.path,
        start: line.next,
        end: source.length,
        lineEnding: line.ending || detectLineEnding(source),
      };
    }
  }
  if (active) sections.push(active);
  return { sections };
}

function scanDirectAssignments(
  source: string,
  section: NodeTableSection,
): Map<string, TomlMember> {
  const assignments = new Map<string, TomlMember>();
  const lines = scanLines(source, section.start, section.end);
  let lineIndex = 0;
  while (lineIndex < lines.length) {
    const line = lines[lineIndex];
    const text = source.slice(line.start, line.contentEnd);
    const match = /^\s*([A-Za-z0-9_-]+)\s*=/.exec(text);
    if (!match) {
      lineIndex += 1;
      continue;
    }
    const equalOffset = text.indexOf('=');
    const valueStart = skipWhitespace(
      source,
      line.start + equalOffset + 1,
      section.end,
    );
    const valueEnd = findTomlValueEnd(source, valueStart, section.end);
    assignments.set(match[1], { valueStart, valueEnd });
    while (lineIndex < lines.length && lines[lineIndex].start < valueEnd)
      lineIndex += 1;
  }
  return assignments;
}

function parseTableHeader(line: string): { path: string[] } | null {
  const trimmed = line.trim();
  if (!trimmed.startsWith('[') || trimmed.startsWith('[[')) return null;
  const close = findTableHeaderEnd(trimmed);
  if (close < 0) return null;
  const headerText = trimmed.slice(0, close + 1);
  let parsed: unknown;
  try {
    parsed = parse(headerText) as unknown;
  } catch {
    return null;
  }
  const path: string[] = [];
  let current = parsed;
  while (isRecord(current)) {
    const keys = Object.keys(current);
    if (keys.length !== 1) break;
    path.push(keys[0]);
    current = current[keys[0]];
  }
  return path.length > 0 ? { path } : null;
}

function findTableHeaderEnd(header: string): number {
  let quote = '';
  let escaped = false;
  for (let index = 1; index < header.length; index += 1) {
    const char = header[index];
    if (quote) {
      if (quote === '"' && escaped) {
        escaped = false;
        continue;
      }
      if (quote === '"' && char === '\\') {
        escaped = true;
        continue;
      }
      if (char === quote) quote = '';
    } else if (char === '"' || char === "'") {
      quote = char;
    } else if (char === ']') {
      return index;
    }
  }
  return -1;
}

function scanLines(
  source: string,
  start = 0,
  end = source.length,
): Array<{ start: number; contentEnd: number; next: number; ending: string }> {
  const lines: Array<{
    start: number;
    contentEnd: number;
    next: number;
    ending: string;
  }> = [];
  let cursor = start;
  while (cursor < end) {
    const newline = source.indexOf('\n', cursor);
    const next = newline < 0 || newline >= end ? end : newline + 1;
    let contentEnd = next;
    let ending = '';
    if (next > cursor && source[next - 1] === '\n') {
      contentEnd = next - 1;
      ending = '\n';
      if (contentEnd > cursor && source[contentEnd - 1] === '\r') {
        contentEnd -= 1;
        ending = '\r\n';
      }
    }
    lines.push({ start: cursor, contentEnd, next, ending });
    cursor = next;
  }
  return lines;
}

function findTomlValueEnd(
  source: string,
  start: number,
  limit: number,
): number {
  let quote: 'basic' | 'literal' | 'multibasic' | 'multiliteral' | null = null;
  let escaped = false;
  let braces = 0;
  let brackets = 0;
  for (let index = start; index < limit; index += 1) {
    const char = source[index];
    if (quote) {
      if (quote === 'basic' || quote === 'multibasic') {
        if (escaped) {
          escaped = false;
          continue;
        }
        if (char === '\\') {
          escaped = true;
          continue;
        }
      }
      if (quote === 'multibasic' && source.startsWith('"""', index)) {
        quote = null;
        index += 2;
      } else if (quote === 'multiliteral' && source.startsWith("'''", index)) {
        quote = null;
        index += 2;
      } else if (quote === 'basic' && char === '"') {
        quote = null;
      } else if (quote === 'literal' && char === "'") {
        quote = null;
      }
      continue;
    }
    if (char === '#') {
      if (braces === 0 && brackets === 0) return index;
      const newline = source.indexOf('\n', index);
      if (newline < 0 || newline >= limit) return limit;
      index = newline;
      continue;
    }
    if (char === '"') {
      if (source.startsWith('"""', index)) {
        quote = 'multibasic';
        index += 2;
      } else quote = 'basic';
      continue;
    }
    if (char === "'") {
      if (source.startsWith("'''", index)) {
        quote = 'multiliteral';
        index += 2;
      } else quote = 'literal';
      continue;
    }
    if (char === '{') braces += 1;
    else if (char === '}') {
      braces -= 1;
      if (braces === 0 && brackets === 0) return index + 1;
    } else if (char === '[') brackets += 1;
    else if (char === ']') {
      brackets -= 1;
      if (braces === 0 && brackets === 0) return index + 1;
    } else if (
      (char === ',' || char === '\n' || char === '#') &&
      braces === 0 &&
      brackets === 0
    ) {
      return index;
    }
  }
  return limit;
}

function findInlineTableEnd(
  source: string,
  start: number,
  limit: number,
): number {
  if (source[start] !== '{')
    throw applyError(
      'canonical-toml-invalid',
      'Expected a canonical inline TOML table.',
    );
  const end = findTomlValueEnd(source, start, limit);
  if (source[end - 1] !== '}')
    throw applyError(
      'canonical-toml-invalid',
      'Canonical inline TOML table is unterminated.',
    );
  return end - 1;
}

function findInlineTableMember(
  source: string,
  tableStart: number,
  tableEnd: number,
  desiredKey: string,
): TomlMember | null {
  let cursor = tableStart + 1;
  while (cursor < tableEnd) {
    cursor = skipTomlTrivia(source, cursor, tableEnd);
    if (cursor >= tableEnd) return null;
    const keyStart = cursor;
    const keyEnd = scanTomlKey(source, keyStart, tableEnd);
    if (keyEnd <= keyStart)
      throw applyError(
        'canonical-toml-invalid',
        'Canonical inline table has an invalid key.',
      );
    const key = decodeTomlKey(source.slice(keyStart, keyEnd));
    cursor = skipWhitespace(source, keyEnd, tableEnd);
    if (source[cursor] !== '=')
      throw applyError(
        'canonical-toml-invalid',
        `Canonical inline table field ${key} has no value.`,
      );
    const valueStart = skipWhitespace(source, cursor + 1, tableEnd);
    const valueEnd = findTomlValueEnd(source, valueStart, tableEnd);
    if (key === desiredKey) return { valueStart, valueEnd };
    cursor = skipTomlTrivia(source, valueEnd, tableEnd);
    if (source[cursor] === ',') cursor += 1;
    else if (cursor < tableEnd)
      throw applyError(
        'canonical-toml-invalid',
        'Canonical inline table fields are not comma-separated.',
      );
  }
  return null;
}

function scanTomlKey(source: string, start: number, limit: number): number {
  const first = source[start];
  if (first === '"' || first === "'") {
    let escaped = false;
    for (let index = start + 1; index < limit; index += 1) {
      const char = source[index];
      if (first === '"' && escaped) escaped = false;
      else if (first === '"' && char === '\\') escaped = true;
      else if (char === first) return index + 1;
    }
    return -1;
  }
  let index = start;
  while (index < limit && /[A-Za-z0-9_-]/.test(source[index])) index += 1;
  return index;
}

function decodeTomlKey(source: string): string {
  if (!source.startsWith('"') && !source.startsWith("'")) return source;
  try {
    const parsed = parse(`${source} = 0`) as Record<string, unknown>;
    return Object.keys(parsed)[0] ?? source;
  } catch {
    throw applyError('canonical-toml-invalid', `Invalid TOML key ${source}.`);
  }
}

function skipTomlTrivia(source: string, start: number, limit: number): number {
  let cursor = start;
  while (cursor < limit) {
    if (/\s/.test(source[cursor])) {
      cursor += 1;
      continue;
    }
    if (source[cursor] === '#') {
      const newline = source.indexOf('\n', cursor);
      if (newline < 0 || newline >= limit) return limit;
      cursor = newline + 1;
      continue;
    }
    break;
  }
  return cursor;
}

function skipWhitespace(source: string, start: number, limit: number): number {
  let cursor = start;
  while (cursor < limit && /\s/.test(source[cursor])) cursor += 1;
  return cursor;
}

function trimWhitespaceEnd(source: string, start: number, end: number): number {
  let cursor = end;
  while (cursor > start && /\s/.test(source[cursor - 1])) cursor -= 1;
  return cursor;
}

function renderTomlKey(value: string): string {
  return /^[A-Za-z0-9_-]+$/.test(value) ? value : renderTomlValue(value);
}

function renderTomlValue(value: unknown): string {
  if (Array.isArray(value)) return `[${value.map(renderTomlValue).join(', ')}]`;
  if (isRecord(value)) {
    return `{ ${Object.entries(value)
      .map(([key, item]) => `${renderTomlKey(key)} = ${renderTomlValue(item)}`)
      .join(', ')} }`;
  }
  if (
    value === null ||
    value === undefined ||
    (typeof value === 'object' && !(value instanceof Date))
  )
    throw applyError(
      'export-value-unsupported',
      'The exported value cannot be represented as a canonical TOML value.',
    );
  const encoded = stringify({ value } as unknown as TomlTable).trimEnd();
  const equals = encoded.indexOf('=');
  if (equals < 0)
    throw applyError(
      'export-value-unsupported',
      'The exported scalar could not be serialized as TOML.',
    );
  return encoded.slice(equals + 1).trim();
}

function getDocumentValue(
  document: ZuiDocument,
  path: Array<string | number>,
): { exists: boolean; value: unknown } {
  let value: unknown = document;
  for (const segment of path) {
    if (typeof segment === 'number') {
      if (!Array.isArray(value) || segment >= value.length)
        return { exists: false, value: undefined };
      value = value[segment];
      continue;
    }
    if (!isRecord(value) || !Object.hasOwn(value, segment))
      return { exists: false, value: undefined };
    value = value[segment];
  }
  return { exists: true, value };
}

function detectLineEnding(source: string): string {
  return source.includes('\r\n') ? '\r\n' : '\n';
}

function applyError(
  code: string,
  message: string,
  path?: string,
): ZuiDocumentError {
  const diagnostic: ZuiDiagnostic = {
    severity: 'error',
    code,
    message,
    ...(path ? { path } : {}),
  };
  return new ZuiDocumentError(message, [diagnostic]);
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return (
    value !== null &&
    typeof value === 'object' &&
    !Array.isArray(value) &&
    !(value instanceof Date)
  );
}
