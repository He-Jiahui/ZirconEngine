import type { Shape } from '@penpot/plugin-types';
import { ZUI_METADATA_NAMESPACE } from './metadata';

export interface RenderedSourceIdentity {
  sourcePath: string | null;
  sourceNodeId: string | null;
  controlId: string | null;
  instancePath: string;
  parentSourcePath: string | null;
  parentSourceNodeId: string | null;
  parentInstancePath: string;
}

/** Read the mounted shape's owner metadata without inventing missing identity. */
export function renderedSourceIdentity(
  shape: Shape,
  parent: Shape,
): RenderedSourceIdentity {
  const metadata = (target: Shape, key: string): string =>
    target.getSharedPluginData(ZUI_METADATA_NAMESPACE, key) || '';
  return {
    sourcePath: metadata(shape, 'sourcePath') || null,
    sourceNodeId: metadata(shape, 'sourceNodeId') || null,
    controlId: metadata(shape, 'controlId') || null,
    instancePath: metadata(shape, 'instancePath'),
    parentSourcePath: metadata(parent, 'sourcePath') || null,
    parentSourceNodeId: metadata(parent, 'sourceNodeId') || null,
    parentInstancePath: metadata(parent, 'instancePath'),
  };
}
