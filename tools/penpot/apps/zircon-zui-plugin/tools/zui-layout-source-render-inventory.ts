import { posix } from 'node:path';
import type { LayoutReviewCase } from './zui-layout-review-contract';
import type { ExpectedResourceUse } from './zui-layout-semantic-parity';
import {
  zuiRootNodeIds,
  type ZuiDocument,
  type ZuiNode,
} from '../src/bridge/zui-document';

interface SourceIdentityNode extends ZuiNode {
  penpot_review_source_path?: unknown;
  penpot_review_source_node_id?: unknown;
  penpot_review_instance_path?: unknown;
}

export interface SourceRenderInventory {
  semanticNodeIdentities: string[];
  resourceUses: ExpectedResourceUse[];
  errors: string[];
}

interface SourceFingerprint {
  sourcePath: string;
  sha256: string;
}

/** Derive expected owners and file-backed media from the case-projected host source. */
export function deriveSourceRenderInventory(
  document: ZuiDocument,
  reviewCase: LayoutReviewCase,
  fingerprints: SourceFingerprint[],
): SourceRenderInventory {
  const errors: string[] = [];
  const identities: string[] = [];
  const resources: ExpectedResourceUse[] = [];
  const fingerprintByPath = new Map<string, string>();
  for (const { sourcePath, sha256 } of fingerprints) {
    const previous = fingerprintByPath.get(sourcePath);
    if (previous && previous !== sha256)
      errors.push(`${reviewCase.id}: conflicting current source hashes ${sourcePath}`);
    fingerprintByPath.set(sourcePath, sha256);
  }
  const seenIdentity = new Set<string>();
  const seenResources = new Set<string>();
  const nodes = document.nodes ?? {};
  const roots = zuiRootNodeIds(document);
  if (!roots.length) errors.push(`${reviewCase.id}: prepared host has no root`);
  const visited = new Set<string>();
  const visiting = new Set<string>();
  const visit = (
    renderId: string,
    parentVisible: boolean | undefined,
  ): void => {
    if (visiting.has(renderId)) {
      errors.push(`${reviewCase.id}: prepared host has a node cycle ${renderId}`);
      return;
    }
    if (visited.has(renderId)) {
      errors.push(`${reviewCase.id}: prepared host mounts one node more than once ${renderId}`);
      return;
    }
    const rawNode = nodes[renderId];
    if (!rawNode) {
      errors.push(`${reviewCase.id}: prepared host mounts a missing node ${renderId}`);
      return;
    }
    visiting.add(renderId);
    visited.add(renderId);
    const node = rawNode as SourceIdentityNode;
    const sourcePath = node.penpot_review_source_path;
    const sourceNodeId = node.penpot_review_source_node_id;
    const instancePath = node.penpot_review_instance_path;
    if (
      typeof sourcePath !== 'string' ||
      !sourcePath.trim() ||
      typeof sourceNodeId !== 'string' ||
      !sourceNodeId.trim() ||
      typeof instancePath !== 'string' ||
      !isCanonicalInstancePath(instancePath)
    ) {
      errors.push(`${reviewCase.id}: unowned prepared host node ${renderId}`);
      visiting.delete(renderId);
      return;
    }
    const identity = JSON.stringify([sourcePath, sourceNodeId, instancePath]);
    if (seenIdentity.has(identity)) {
      errors.push(`${reviewCase.id}: duplicate prepared source identity ${identity}`);
      visiting.delete(renderId);
      return;
    }
    seenIdentity.add(identity);
    identities.push(identity);
    const currentSourceHash = fingerprintByPath.get(sourcePath);
    if (!currentSourceHash || !/^[0-9a-f]{64}$/.test(currentSourceHash))
      errors.push(`${reviewCase.id}: prepared source has no current file hash ${sourcePath}`);

    const props = node.props ?? {};
    const visible = isNodeVisibleForCase(node, props, parentVisible);
    if (visible === undefined) {
      errors.push(`${reviewCase.id}: source visibility is unresolved for ${identity}`);
    }
    if (visible === true) {
      const media = mediaProperties(node);
      if (media.error)
        errors.push(`${reviewCase.id}: ${media.error} ${identity}`);
      for (const { property, iconReference, value } of media.references) {
        if (value === undefined || value === null || value === '' || value === false || value === 'none')
          continue;
        if (typeof value !== 'string') {
          errors.push(
            `${reviewCase.id}: authored media property is not a static path ${sourcePath}#${sourceNodeId}.${property}`,
          );
          continue;
        }
        if (!isLocalMediaReference(value)) {
          errors.push(
            `${reviewCase.id}: non-local authored media cannot be fingerprinted ${sourcePath}#${sourceNodeId}.${property}`,
          );
          continue;
        }
        const mediaPath = resolveMediaPath(sourcePath, value, iconReference);
        if (!mediaPath) {
          errors.push(
            `${reviewCase.id}: cannot resolve authored media ${sourcePath}#${sourceNodeId}.${property}`,
          );
          continue;
        }
        const sha256 = fingerprintByPath.get(mediaPath);
        if (!sha256 || !/^[0-9a-f]{64}$/.test(sha256)) {
          errors.push(`${reviewCase.id}: authored media has no current file hash ${mediaPath}`);
          continue;
        }
        const kind = mediaPath.toLowerCase().endsWith('.svg') ? 'vector' : 'raster';
        const use: ExpectedResourceUse = {
          kind,
          path: mediaPath,
          sha256,
          sourcePath,
          sourceNodeId,
          instancePath,
        };
        const resourceKey = JSON.stringify([
          kind,
          mediaPath,
          sha256,
          sourcePath,
          sourceNodeId,
          instancePath,
        ]);
        if (!seenResources.has(resourceKey)) {
          resources.push(use);
          seenResources.add(resourceKey);
        }
      }
    }
    for (const mount of node.children ?? []) {
      if (typeof mount.node !== 'string' || !mount.node.trim()) {
        errors.push(`${reviewCase.id}: prepared host has an invalid child mount under ${renderId}`);
        continue;
      }
      visit(mount.node, visible);
    }
    visiting.delete(renderId);
  };
  for (const root of roots) visit(root, true);
  identities.sort();
  resources.sort((a, b) =>
    JSON.stringify(a).localeCompare(JSON.stringify(b)),
  );
  return { semanticNodeIdentities: identities, resourceUses: resources, errors };
}

function isCanonicalInstancePath(value: string): boolean {
  try {
    const steps = JSON.parse(value) as unknown;
    return (
      Array.isArray(steps) &&
      steps.every(
        (step) =>
          step &&
          typeof step === 'object' &&
          !Array.isArray(step) &&
          Object.keys(step).join(',') === 'sourcePath,sourceNodeId' &&
          typeof step.sourcePath === 'string' &&
          !!step.sourcePath.trim() &&
          typeof step.sourceNodeId === 'string' &&
          !!step.sourceNodeId.trim(),
      ) &&
      JSON.stringify(steps) === value
    );
  } catch {
    return false;
  }
}

function isNodeVisibleForCase(
  node: SourceIdentityNode,
  props: Record<string, unknown>,
  parentVisible: boolean | undefined,
): boolean | undefined {
  const state = node.state ?? {};
  if (Object.hasOwn(props, 'menu_items')) {
    const popupOpen = popupOpenForNode(node);
    if (popupOpen === undefined) return undefined;
    if (!popupOpen) return false;
  }
  let localVisible: boolean | undefined;
  if (typeof state['visible'] === 'boolean') localVisible = state['visible'];
  else if (state['visible'] !== undefined) return undefined;
  else if (typeof props['visible'] === 'boolean') localVisible = props['visible'];
  else if (props['visible'] !== undefined) return undefined;
  const stateVisibility = state['visibility'];
  if (localVisible === undefined && stateVisibility !== undefined) {
    if (typeof stateVisibility !== 'string') return undefined;
    if (!['hidden', 'collapsed', 'visible', 'inherit'].includes(stateVisibility))
      return undefined;
    localVisible = stateVisibility === 'inherit'
      ? parentVisible
      : stateVisibility === 'visible';
  }
  const propVisibility = props['visibility'];
  if (localVisible === undefined && typeof propVisibility === 'string') {
    if (
      ['hidden', 'collapsed', 'visible'].includes(propVisibility) ||
      propVisibility === 'inherit'
    ) {
      if (
        (state['open'] === true || state['popup_open'] === true) &&
        (propVisibility === 'collapsed' || propVisibility === 'hidden')
      )
        localVisible = true;
      else
        localVisible = propVisibility === 'inherit'
          ? parentVisible
          : propVisibility === 'visible';
      if (localVisible === undefined) return undefined;
      if (parentVisible === undefined) return undefined;
      return localVisible && parentVisible;
    }
    return undefined;
  }
  if (localVisible === undefined && propVisibility !== undefined) return undefined;
  localVisible ??= true;
  if (localVisible === false || parentVisible === false) return false;
  if (parentVisible === undefined) return undefined;
  return localVisible;
}

function popupOpenForNode(node: SourceIdentityNode): boolean | undefined {
  const state = node.state ?? {};
  const props = node.props ?? {};
  const resolve = (
    values: Record<string, unknown>,
  ): boolean | undefined => {
    const declared = ['popup_open', 'open'].filter((key) =>
      Object.hasOwn(values, key),
    );
    if (!declared.length) return undefined;
    const states = declared.map((key) => values[key]);
    if (
      states.some((value) => typeof value !== 'boolean') ||
      new Set(states).size !== 1
    )
      return undefined;
    return states[0] as boolean;
  };
  if (Object.hasOwn(state, 'popup_open') || Object.hasOwn(state, 'open'))
    return resolve(state);
  return resolve(props);
}

interface SourceMediaReference {
  property: string;
  iconReference: boolean;
  value: unknown;
}

function mediaProperties(node: SourceIdentityNode): {
  references: SourceMediaReference[];
  error?: string;
} {
  const props = node.props ?? {};
  const references: SourceMediaReference[] = [
    { property: 'icon', iconReference: true, value: props['icon'] },
    {
      property: 'background_image',
      iconReference: false,
      value: props['background_image'],
    },
  ];
  if (node.component.toLowerCase() === 'image') {
    const key = ['source', 'image', 'value'].find((candidate) =>
      Object.hasOwn(props, candidate),
    );
    if (key)
      references.push({
        property: key,
        iconReference: false,
        value: props[key],
      });
  }
  if (node.component.toLowerCase().includes('icon')) {
    const key = ['source', 'image', 'value'].find((candidate) =>
      Object.hasOwn(props, candidate),
    );
    if (key)
      references.push({
        property: key,
        iconReference: true,
        value: props[key],
      });
  }
  if (Object.hasOwn(props, 'menu_items') && popupOpenForNode(node) === true) {
    const items = props['menu_items'];
    if (!Array.isArray(items))
      return { references, error: 'open popup menu has unresolved media items' };
    for (const [index, item] of items.entries()) {
      if (typeof item !== 'string')
        return {
          references,
          error: `open popup menu item ${index} is not statically authored`,
        };
      const icon = item.match(/(?:^|[,|])\s*icon=([^,|]+)(?:[,|]|$)/)?.[1]?.trim();
      if (item.includes('icon=') && !icon)
        return {
          references,
          error: `open popup menu item ${index} has an unresolved icon`,
        };
      if (icon)
        references.push({
          property: `menu_items[${index}].icon`,
          iconReference: true,
          value: icon,
        });
    }
  }
  return { references };
}

function isLocalMediaReference(value: string): boolean {
  return !value.includes('://') || value.startsWith('res://');
}

function resolveMediaPath(
  sourcePath: string,
  reference: string,
  iconReference: boolean,
): string | undefined {
  const assetMarker = '/assets/';
  const markerIndex = sourcePath.indexOf(assetMarker);
  if (markerIndex < 0 || sourcePath.startsWith('/')) return undefined;
  let relativePath = reference
    .replace(/^res:\/\//, '')
    .split(/[?#]/, 1)[0]!;
  if (!relativePath || relativePath.startsWith('/') || relativePath.includes('\\'))
    return undefined;
  if (iconReference) {
    if (!relativePath.startsWith('icons/')) relativePath = `icons/${relativePath}`;
    if (!/\.(png|jpe?g|webp|svg)$/i.test(relativePath))
      relativePath = `${relativePath}.svg`;
  } else if (!/\.(png|jpe?g|webp|svg)$/i.test(relativePath)) {
    return undefined;
  }
  const assetRoot = `${sourcePath.slice(0, markerIndex)}/assets`;
  const resolved = posix.normalize(`${assetRoot}/${relativePath}`);
  if (
    resolved === assetRoot ||
    !resolved.startsWith(`${assetRoot}/`) ||
    resolved.split('/').some((part) => part === '..')
  )
    return undefined;
  return resolved;
}
