import type { Board, ImageData } from '@penpot/plugin-types';
import type { ZuiDocument, ZuiNode } from './bridge/zui-document';
import {
  ZUI_METADATA_NAMESPACE,
  ZUI_METADATA_ROLE,
  ZUI_ROLE_AUXILIARY,
  ZUI_METADATA_NODE_ID,
} from './metadata';

interface EmbeddedImage {
  uri: string;
  name: string;
  mime: string;
  base64: string;
}

export function createImageContent(board: Board, node: ZuiNode): void {
  if (!node['penpot_image'] && !node['penpot_image_svg']) return;
  const image =
    typeof node['penpot_image_svg'] === 'string'
      ? penpot.createShapeFromSvg(node['penpot_image_svg'])
      : penpot.createRectangle();
  if (!image) throw new Error('Unable to create image content');
  image.name = 'Image';
  if (!node['penpot_image_svg']) image.fills = [];
  image.resize(board.width, board.height);
  board.appendChild(image);
  image.x = board.x;
  image.y = board.y;
  if (image.layoutChild) image.layoutChild.absolute = true;
  image.setSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    ZUI_METADATA_ROLE,
    ZUI_ROLE_AUXILIARY,
  );
  image.setSharedPluginData(ZUI_METADATA_NAMESPACE, 'image-content', 'true');
}

export async function loadAssetImages(
  asset: Board,
  document: ZuiDocument,
): Promise<void> {
  const cache = new Map<string, ImageData>();
  const visit = async (board: Board): Promise<void> => {
    const id = board.getSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      ZUI_METADATA_NODE_ID,
    );
    const embedded = document.nodes?.[id]?.['penpot_image'] as unknown as
      EmbeddedImage | undefined;
    for (const child of board.children) {
      if (
        embedded &&
        child.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'image-content')
      ) {
        let image = cache.get(embedded.uri);
        if (!image) {
          const binary = atob(embedded.base64);
          image = await penpot.uploadMediaData(
            embedded.name,
            Uint8Array.from(binary, (character) => character.charCodeAt(0)),
            embedded.mime,
          );
          cache.set(embedded.uri, image);
        }
        child.fills = [{ fillImage: image, fillOpacity: 1 }];
      }
      if (child.type === 'board') await visit(child);
    }
  };
  await visit(asset);
}

export function refreshImageContent(board: Board): void {
  for (const child of board.children) {
    if (!child.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'image-content'))
      continue;
    child.resize(board.width, board.height);
    child.x = board.x;
    child.y = board.y;
  }
}
