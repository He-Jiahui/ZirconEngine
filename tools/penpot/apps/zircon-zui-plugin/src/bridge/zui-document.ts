import { parse, stringify, TomlDate, type TomlTable } from 'smol-toml';
import { validateZuiRuntimeSchema } from './zui-runtime-schema';

export const ZUI_SCHEMA_VERSION = 2;
const ZUI_METADATA_SCHEMA = 'dev.zircon.zui.document-metadata';
const ZUI_METADATA_VERSION = 1;

type MetadataPathSegment = string | number;

interface MetadataTypedValue {
  path: MetadataPathSegment[];
  type: 'bigint' | 'date' | 'number';
  value: string;
}

export type ZuiDiagnosticSeverity = 'info' | 'warning' | 'error';

export interface ZuiDiagnostic {
  severity: ZuiDiagnosticSeverity;
  code: string;
  message: string;
  path?: string;
}

export interface ZuiAssetHeader {
  kind: 'view' | 'component' | 'style' | 'theme_tokens';
  id: string;
  version: number;
  display_name?: string;
  [key: string]: unknown;
}

export interface ZuiChildMount {
  node: string;
  slot?: ZuiTable;
  [key: string]: unknown;
}

export interface ZuiNode {
  component: string;
  control_id?: string;
  classes?: string[];
  params?: ZuiTable;
  props?: ZuiTable;
  state?: ZuiTable;
  layout?: ZuiTable;
  repeat?: ZuiTable;
  style?: ZuiTable;
  slots?: ZuiTable;
  events?: ZuiTable[];
  children?: ZuiChildMount[];
  [key: string]: unknown;
}

export interface ZuiComponent {
  root: string;
  [key: string]: unknown;
}

export interface ZuiDocument {
  asset: ZuiAssetHeader;
  imports?: ZuiTable;
  tokens?: ZuiTable;
  root?: { node: string; [key: string]: unknown };
  nodes?: Record<string, ZuiNode>;
  components?: Record<string, ZuiComponent>;
  stylesheets?: ZuiTable[];
  [key: string]: unknown;
}

export interface ParsedZuiDocument {
  document: ZuiDocument;
  diagnostics: ZuiDiagnostic[];
}

export interface ZuiTable {
  [key: string]: ZuiValue;
}
export type ZuiValue =
  string | number | bigint | boolean | Date | ZuiValue[] | ZuiTable;

export class ZuiDocumentError extends Error {
  readonly diagnostics: ZuiDiagnostic[];

  constructor(message: string, diagnostics: ZuiDiagnostic[] = []) {
    super(message);
    this.name = 'ZuiDocumentError';
    this.diagnostics = diagnostics;
  }
}

export function parseZuiDocument(source: string): ParsedZuiDocument {
  let parsed: TomlTable;
  try {
    parsed = parse(source, { integersAsBigInt: 'asNeeded' });
  } catch (error) {
    throw new ZuiDocumentError(
      `Failed to parse .zui TOML: ${errorMessage(error)}`,
      [
        {
          severity: 'error',
          code: 'toml-parse-failed',
          message: errorMessage(error),
        },
      ],
    );
  }

  const document = parsed as unknown as ZuiDocument;
  const diagnostics = validateZuiDocument(document);
  const errors = diagnostics.filter(({ severity }) => severity === 'error');
  if (errors.length > 0) {
    throw new ZuiDocumentError(
      `Invalid .zui document: ${errors.map(({ message }) => message).join('; ')}`,
      diagnostics,
    );
  }
  return { document, diagnostics };
}

export function serializeZuiDocument(document: ZuiDocument): string {
  const diagnostics = validateZuiDocument(document);
  const errors = diagnostics.filter(({ severity }) => severity === 'error');
  if (errors.length > 0) {
    throw new ZuiDocumentError(
      `Cannot serialize invalid .zui document: ${errors
        .map(({ message }) => message)
        .join('; ')}`,
      diagnostics,
    );
  }
  return `${stringify(document as unknown as TomlTable).trimEnd()}\n`;
}

export function validateZuiDocument(document: ZuiDocument): ZuiDiagnostic[] {
  const diagnostics: ZuiDiagnostic[] = [];
  if (!isRecord(document)) {
    return [
      errorDiagnostic('document-not-table', 'The TOML root must be a table.'),
    ];
  }
  if (!isRecord(document.asset)) {
    return [
      errorDiagnostic(
        'asset-missing',
        'The document must declare an [asset] table.',
      ),
    ];
  }

  diagnostics.push(...validateZuiRuntimeSchema(document));

  const { asset } = document;
  if (!['view', 'component', 'style', 'theme_tokens'].includes(asset.kind)) {
    diagnostics.push(
      errorDiagnostic(
        'asset-kind-unsupported',
        `Unsupported asset kind ${String(asset.kind)}.`,
        'asset.kind',
      ),
    );
  }
  if (typeof asset.id !== 'string' || asset.id.trim() === '') {
    diagnostics.push(
      errorDiagnostic(
        'asset-id-empty',
        'asset.id must not be empty.',
        'asset.id',
      ),
    );
  }
  if (asset.version !== ZUI_SCHEMA_VERSION) {
    diagnostics.push(
      errorDiagnostic(
        'schema-version-unsupported',
        `Unsupported schema version ${String(asset.version)}; expected ${ZUI_SCHEMA_VERSION}.`,
        'asset.version',
      ),
    );
  }
  if (document.nodes !== undefined && !isRecord(document.nodes)) {
    diagnostics.push(
      errorDiagnostic(
        'nodes-invalid',
        'The [nodes] value must be a table when present.',
        'nodes',
      ),
    );
    return diagnostics;
  }

  const nodes = zuiNodes(document);
  const nodeIds = new Set(Object.keys(nodes));
  for (const [nodeId, node] of Object.entries(nodes)) {
    if (
      !isRecord(node) ||
      typeof node.component !== 'string' ||
      node.component.trim() === ''
    ) {
      diagnostics.push(
        errorDiagnostic(
          'node-component-empty',
          `Node ${nodeId} must declare a non-empty component.`,
          `nodes.${nodeId}.component`,
        ),
      );
      continue;
    }
    if (node.children !== undefined && !Array.isArray(node.children)) {
      diagnostics.push(
        errorDiagnostic(
          'node-children-invalid',
          `Node ${nodeId} children must be an array.`,
          `nodes.${nodeId}.children`,
        ),
      );
      continue;
    }
    for (const [index, child] of (node.children ?? []).entries()) {
      if (
        !isRecord(child) ||
        typeof child.node !== 'string' ||
        child.node.trim() === ''
      ) {
        diagnostics.push(
          errorDiagnostic(
            'child-node-invalid',
            `Node ${nodeId} child ${index} must name a node.`,
            `nodes.${nodeId}.children.${index}`,
          ),
        );
      } else if (!nodeIds.has(child.node)) {
        diagnostics.push(
          errorDiagnostic(
            'child-node-missing',
            `Node ${nodeId} references missing child node ${child.node}.`,
            `nodes.${nodeId}.children.${index}.node`,
          ),
        );
      }
    }
  }

  validateProfile(document, diagnostics);
  validateTree(document, diagnostics);
  return diagnostics;
}

export function cloneZuiDocument(document: ZuiDocument): ZuiDocument {
  return cloneZuiValue(document) as ZuiDocument;
}

export function normalizeZuiDocument(document: ZuiDocument): unknown {
  return normalizeValue(document);
}

export function encodeZuiMetadata(document: ZuiDocument): string {
  const typedValues: MetadataTypedValue[] = [];
  const encodedDocument = encodeMetadataValue(document, [], typedValues);
  return JSON.stringify({
    schema: ZUI_METADATA_SCHEMA,
    version: ZUI_METADATA_VERSION,
    document: encodedDocument,
    typedValues,
  });
}

export function decodeZuiMetadata(value: string): ZuiDocument {
  let parsed: unknown;
  try {
    parsed = JSON.parse(value);
  } catch (error) {
    throw new ZuiDocumentError(
      `Invalid ZUI metadata JSON: ${errorMessage(error)}`,
    );
  }
  let document: ZuiDocument;
  try {
    document = decodeMetadataEnvelope(parsed);
  } catch (error) {
    throw new ZuiDocumentError(
      `Invalid typed value in ZUI metadata: ${errorMessage(error)}`,
    );
  }
  const diagnostics = validateZuiDocument(document);
  const errors = diagnostics.filter(({ severity }) => severity === 'error');
  if (errors.length > 0) {
    throw new ZuiDocumentError(
      `Invalid ZUI metadata: ${errors.map(({ message }) => message).join('; ')}`,
      diagnostics,
    );
  }
  return document;
}

export function zuiRootNodeIds(document: ZuiDocument): string[] {
  if (document.asset.kind === 'component') {
    const components = Object.values(document.components ?? {});
    return components.length === 1 ? [components[0].root] : [];
  }
  return document.root ? [document.root.node] : [];
}

export function zuiNodes(document: ZuiDocument): Record<string, ZuiNode> {
  return document.nodes ?? {};
}

function validateProfile(
  document: ZuiDocument,
  diagnostics: ZuiDiagnostic[],
): void {
  const nodes = zuiNodes(document);
  if (document.asset.kind === 'view') {
    if (
      !document.root ||
      typeof document.root.node !== 'string' ||
      document.root.node.trim() === ''
    ) {
      diagnostics.push(
        errorDiagnostic(
          'view-root-missing',
          'A .zui view must declare [root].node.',
          'root.node',
        ),
      );
    } else if (!nodes[document.root.node]) {
      diagnostics.push(
        errorDiagnostic(
          'view-root-node-missing',
          `View root references missing node ${document.root.node}.`,
          'root.node',
        ),
      );
    }
  }

  if (document.asset.kind === 'component') {
    if (document.root !== undefined) {
      diagnostics.push(
        errorDiagnostic(
          'component-view-root-forbidden',
          'A .zui component must not declare a [root] view entry.',
          'root',
        ),
      );
    }
    const components = Object.entries(document.components ?? {});
    if (components.length !== 1) {
      diagnostics.push(
        errorDiagnostic(
          'component-count-invalid',
          `A .zui component must declare exactly one component; found ${components.length}.`,
          'components',
        ),
      );
    } else {
      const [componentId, component] = components[0];
      if (typeof component.root !== 'string' || component.root.trim() === '') {
        diagnostics.push(
          errorDiagnostic(
            'component-root-empty',
            `Component ${componentId} must declare a non-empty root node.`,
            `components.${componentId}.root`,
          ),
        );
      } else if (!nodes[component.root]) {
        diagnostics.push(
          errorDiagnostic(
            'component-root-missing',
            `Component ${componentId} references missing root node ${component.root}.`,
            `components.${componentId}.root`,
          ),
        );
      }
    }
  }

  if (
    (document.asset.kind === 'style' ||
      document.asset.kind === 'theme_tokens') &&
    document.root &&
    !nodes[document.root.node]
  ) {
    diagnostics.push(
      errorDiagnostic(
        'style-root-missing',
        `Style root references missing node ${document.root.node}.`,
        'root.node',
      ),
    );
  }
}

function validateTree(
  document: ZuiDocument,
  diagnostics: ZuiDiagnostic[],
): void {
  const nodes = zuiNodes(document);
  const parentByNode = new Map<string, string>();
  for (const [parentId, node] of Object.entries(nodes)) {
    const localChildren = new Set<string>();
    for (const child of node.children ?? []) {
      if (typeof child.node !== 'string' || !nodes[child.node]) {
        continue;
      }
      if (localChildren.has(child.node)) {
        diagnostics.push(
          errorDiagnostic(
            'node-duplicate-child',
            `Node ${parentId} mounts child ${child.node} more than once.`,
            `nodes.${parentId}.children`,
          ),
        );
        continue;
      }
      localChildren.add(child.node);
      const previousParent = parentByNode.get(child.node);
      if (previousParent && previousParent !== parentId) {
        diagnostics.push(
          errorDiagnostic(
            'node-multiple-parents',
            `Node ${child.node} is mounted by both ${previousParent} and ${parentId}; Penpot assets require a tree.`,
            `nodes.${parentId}.children`,
          ),
        );
      } else {
        parentByNode.set(child.node, parentId);
      }
    }
  }

  const visiting = new Set<string>();
  const visited = new Set<string>();
  const visit = (nodeId: string): void => {
    if (visiting.has(nodeId)) {
      diagnostics.push(
        errorDiagnostic(
          'node-cycle',
          `Node hierarchy contains a cycle at ${nodeId}.`,
          `nodes.${nodeId}`,
        ),
      );
      return;
    }
    if (visited.has(nodeId)) {
      return;
    }
    visiting.add(nodeId);
    for (const child of nodes[nodeId]?.children ?? []) {
      if (nodes[child.node]) {
        visit(child.node);
      }
    }
    visiting.delete(nodeId);
    visited.add(nodeId);
  };
  for (const nodeId of Object.keys(nodes)) {
    visit(nodeId);
  }
}

export function cloneZuiValue(value: unknown): unknown {
  if (value instanceof Date) {
    return new TomlDate(value.toISOString());
  }
  if (Array.isArray(value)) {
    return value.map(cloneZuiValue);
  }
  if (isRecord(value)) {
    return Object.fromEntries(
      Object.entries(value).map(([key, item]) => [key, cloneZuiValue(item)]),
    );
  }
  return value;
}

function normalizeValue(value: unknown): unknown {
  if (typeof value === 'bigint') {
    return { $tomlBigInt: value.toString() };
  }
  if (value instanceof Date) {
    return { $tomlDate: value.toISOString() };
  }
  if (Array.isArray(value)) {
    return value.map(normalizeValue);
  }
  if (isRecord(value)) {
    return Object.fromEntries(
      Object.keys(value)
        .sort()
        .map((key) => [key, normalizeValue(value[key])]),
    );
  }
  return value;
}

function encodeMetadataValue(
  value: unknown,
  path: MetadataPathSegment[],
  typedValues: MetadataTypedValue[],
): unknown {
  if (typeof value === 'bigint') {
    typedValues.push({ path, type: 'bigint', value: value.toString() });
    return null;
  }
  if (value instanceof Date) {
    typedValues.push({ path, type: 'date', value: value.toISOString() });
    return null;
  }
  if (
    typeof value === 'number' &&
    (!Number.isFinite(value) || Object.is(value, -0))
  ) {
    typedValues.push({ path, type: 'number', value: encodedNumber(value) });
    return null;
  }
  if (Array.isArray(value)) {
    return value.map((item, index) =>
      encodeMetadataValue(item, [...path, index], typedValues),
    );
  }
  if (isRecord(value)) {
    return Object.fromEntries(
      Object.entries(value).map(([key, item]) => [
        key,
        encodeMetadataValue(item, [...path, key], typedValues),
      ]),
    );
  }
  return value;
}

function decodeMetadataEnvelope(value: unknown): ZuiDocument {
  if (!isRecord(value)) {
    throw new Error('metadata root must be an object');
  }
  if (value['schema'] !== ZUI_METADATA_SCHEMA) {
    throw new Error(`unsupported metadata schema ${String(value['schema'])}`);
  }
  if (value['version'] !== ZUI_METADATA_VERSION) {
    throw new Error(`unsupported metadata version ${String(value['version'])}`);
  }
  if (!isRecord(value['document'])) {
    throw new Error('metadata document must be an object');
  }
  if (!Array.isArray(value['typedValues'])) {
    throw new Error('metadata typedValues must be an array');
  }

  const document = value['document'];
  const occupiedPaths = new Set<string>();
  for (const [index, entry] of value['typedValues'].entries()) {
    const typedValue = parseMetadataTypedValue(entry, index);
    const pathKey = JSON.stringify(typedValue.path);
    if (occupiedPaths.has(pathKey)) {
      throw new Error(`duplicate typed metadata path ${pathKey}`);
    }
    occupiedPaths.add(pathKey);
    setMetadataValueAtPath(
      document,
      typedValue.path,
      decodedTypedValue(typedValue),
    );
  }
  return document as unknown as ZuiDocument;
}

function parseMetadataTypedValue(
  value: unknown,
  index: number,
): MetadataTypedValue {
  if (!isRecord(value)) {
    throw new Error(`typedValues[${index}] must be an object`);
  }
  const path = value['path'];
  const type = value['type'];
  const typedValue = value['value'];
  if (
    !Array.isArray(path) ||
    path.length === 0 ||
    path.some(
      (segment) =>
        typeof segment !== 'string' &&
        !(Number.isInteger(segment) && Number(segment) >= 0),
    )
  ) {
    throw new Error(`typedValues[${index}].path is invalid`);
  }
  if (type !== 'bigint' && type !== 'date' && type !== 'number') {
    throw new Error(`typedValues[${index}].type is invalid`);
  }
  if (typeof typedValue !== 'string') {
    throw new Error(`typedValues[${index}].value must be a string`);
  }
  return { path: path as MetadataPathSegment[], type, value: typedValue };
}

function setMetadataValueAtPath(
  root: Record<string, unknown>,
  path: MetadataPathSegment[],
  value: bigint | Date | number,
): void {
  let target: unknown = root;
  for (const segment of path.slice(0, -1)) {
    target = metadataChild(target, segment);
  }
  const leaf = path[path.length - 1];
  if (typeof leaf === 'number') {
    if (
      !Array.isArray(target) ||
      leaf >= target.length ||
      target[leaf] !== null
    ) {
      throw new Error(`typed metadata path ${JSON.stringify(path)} is missing`);
    }
    target[leaf] = value;
    return;
  }
  if (
    !isRecord(target) ||
    !Object.hasOwn(target, leaf) ||
    target[leaf] !== null
  ) {
    throw new Error(`typed metadata path ${JSON.stringify(path)} is missing`);
  }
  target[leaf] = value;
}

function decodedTypedValue(value: MetadataTypedValue): bigint | Date | number {
  if (value.type === 'bigint') return BigInt(value.value);
  if (value.type === 'date') return new TomlDate(value.value);
  switch (value.value) {
    case 'inf':
      return Number.POSITIVE_INFINITY;
    case '-inf':
      return Number.NEGATIVE_INFINITY;
    case 'nan':
      return Number.NaN;
    case '-0':
      return -0;
    default:
      throw new Error(`unsupported typed number ${value.value}`);
  }
}

function encodedNumber(value: number): string {
  if (Number.isNaN(value)) return 'nan';
  if (value === Number.POSITIVE_INFINITY) return 'inf';
  if (value === Number.NEGATIVE_INFINITY) return '-inf';
  if (Object.is(value, -0)) return '-0';
  throw new Error(`number ${String(value)} does not need metadata encoding`);
}

function metadataChild(value: unknown, segment: MetadataPathSegment): unknown {
  if (typeof segment === 'number') {
    if (!Array.isArray(value) || segment >= value.length) {
      throw new Error(`typed metadata array segment ${segment} is missing`);
    }
    return value[segment];
  }
  if (!isRecord(value) || !Object.hasOwn(value, segment)) {
    throw new Error(`typed metadata object segment ${segment} is missing`);
  }
  return value[segment];
}

function errorDiagnostic(
  code: string,
  message: string,
  path?: string,
): ZuiDiagnostic {
  return { severity: 'error', code, message, ...(path ? { path } : {}) };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return (
    typeof value === 'object' &&
    value !== null &&
    !Array.isArray(value) &&
    !(value instanceof Date)
  );
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
