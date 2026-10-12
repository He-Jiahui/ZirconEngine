import type { Font, FontVariant, Shape } from '@penpot/plugin-types';
import type { ZuiDocument } from './zui-document';
import { ZUI_METADATA_NAMESPACE } from '../metadata';

/** Upload receipts identify resources; their hash strings are not load proof. */
export interface PenpotFontResourceSelection {
  readonly source: { readonly family: string; readonly weight: string; readonly style: 'normal' | 'italic' };
  readonly engine: { readonly assetPath: string; readonly assetSha256: string; readonly faceIndex: number; readonly sfntSha256: string };
  readonly penpot: { readonly fontId: string; readonly variantId: string; readonly sfntSha256: string };
}
export interface PenpotFontResourceManifest {
  readonly schema: 'zircon.penpot.font-resources.v1';
  readonly selections: readonly PenpotFontResourceSelection[];
}
const FONT_RESOURCES = Symbol('Penpot import font resource selections');
const FONT_RESOURCE_METADATA = 'font-resource-manifest';
type ImportDocument = ZuiDocument & { [FONT_RESOURCES]?: PenpotFontResourceManifest };
const record = (value: unknown): value is Record<string, unknown> => typeof value === 'object' && value !== null && !Array.isArray(value);
const string = (value: unknown): value is string => typeof value === 'string' && value.length > 0 && value.trim() === value;
const sha = (value: unknown): value is string => typeof value === 'string' && /^[a-f0-9]{64}$/i.test(value);
const tuple = (family: string, weight: string, style: string): string => JSON.stringify([family,weight,style]);

export function parseFontResourceManifest(value: unknown): PenpotFontResourceManifest {
  if (!record(value) || value['schema'] !== 'zircon.penpot.font-resources.v1' || !Array.isArray(value['selections']))
    throw new Error('Invalid font resource manifest schema');
  const seen = new Set<string>();
  const selections = value['selections'].map((entry: unknown) => {
    if (!record(entry) || !record(entry['source']) || !record(entry['engine']) || !record(entry['penpot']))
      throw new Error('Invalid font resource selection');
    const source = entry['source'], engine = entry['engine'], uploaded = entry['penpot'];
    if (!string(source['family']) || !string(source['weight']) || !/^[1-9][0-9]{0,2}$|^1000$/.test(source['weight']) || !['normal','italic'].includes(String(source['style'])))
      throw new Error('Invalid source font family, weight or style');
    if (!string(engine['assetPath']) || !sha(engine['assetSha256']) || !sha(engine['sfntSha256']) || !Number.isInteger(engine['faceIndex']) || Number(engine['faceIndex']) < 0)
      throw new Error('Invalid engine font resource receipt');
    if (!string(uploaded['fontId']) || !string(uploaded['variantId']) || !sha(uploaded['sfntSha256']) || uploaded['sfntSha256'].toLowerCase() !== engine['sfntSha256'].toLowerCase())
      throw new Error('Invalid Penpot upload receipt or mismatched SFNT hash');
    const key = tuple(source['family'],source['weight'],String(source['style']));
    if (seen.has(key)) throw new Error(`Duplicate font resource selection ${key}`);
    seen.add(key);
    return Object.freeze({ source: Object.freeze({...source}), engine: Object.freeze({...engine}), penpot: Object.freeze({...uploaded}) }) as unknown as PenpotFontResourceSelection;
  });
  return Object.freeze({ schema: 'zircon.penpot.font-resources.v1', selections: Object.freeze(selections) });
}

/** This is document-local import context, never a font collection or global registry. */
export function bindFontResources(document: ZuiDocument, manifest: PenpotFontResourceManifest | undefined): void {
  Object.defineProperty(document,FONT_RESOURCES,{value:manifest,enumerable:false,configurable:true});
}
export function fontResourcesForDocument(document: ZuiDocument): PenpotFontResourceManifest | undefined {
  return (document as ImportDocument)[FONT_RESOURCES];
}
export function copyFontResources(source: ZuiDocument, target: ZuiDocument): void {
  bindFontResources(target,fontResourcesForDocument(source));
}
export function writeFontResources(shape: Shape, manifest: PenpotFontResourceManifest | undefined): void {
  if (manifest) shape.setSharedPluginData(ZUI_METADATA_NAMESPACE,FONT_RESOURCE_METADATA,JSON.stringify(manifest));
}
export function fontResourcesForShape(shape: Shape): PenpotFontResourceManifest | undefined {
  for (let current: Shape | null = shape; current; current = current.parent) {
    const value = typeof current.getSharedPluginData === 'function'
      ? current.getSharedPluginData(ZUI_METADATA_NAMESPACE,FONT_RESOURCE_METADATA) : '';
    if (value) return parseFontResourceManifest(JSON.parse(value));
  }
  return undefined;
}

export function selectRegisteredFont(
  family: string, weight: string, style: 'normal' | 'italic',
  manifest: PenpotFontResourceManifest | undefined,
): { font: Font; variant: FontVariant } | undefined {
  if (!manifest?.selections.some((selection) => selection.source.family === family)) return undefined;
  const normalized = weight === 'regular' ? '400' : weight;
  const matches = manifest.selections.filter((selection) => tuple(selection.source.family,selection.source.weight,selection.source.style) === tuple(family,normalized,style));
  if (matches.length !== 1) throw new Error(`Expected one explicit font resource selection for ${family} ${normalized} ${style}, found ${matches.length}`);
  const selection = matches[0];
  const fonts = penpot.fonts.all.filter((font) => font.fontId === selection.penpot.fontId);
  if (fonts.length !== 1 || fonts[0].fontFamily !== family)
    throw new Error(`Font resource ID ${selection.penpot.fontId} does not identify one exact registered family ${family}`);
  const variants = fonts[0].variants.filter((variant) => variant.fontVariantId === selection.penpot.variantId);
  if (variants.length !== 1 || variants[0].fontWeight !== normalized || variants[0].fontStyle !== style)
    throw new Error(`Font variant ID ${selection.penpot.variantId} does not match ${family} ${normalized} ${style}`);
  return {font:fonts[0],variant:variants[0]};
}
