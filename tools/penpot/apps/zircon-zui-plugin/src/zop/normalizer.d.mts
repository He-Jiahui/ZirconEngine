import type { Board, Shape, TokenCatalog } from '@penpot/plugin-types';
import type { RoleContract } from '../../../zircon-zop-compiler/src/contract.mjs';
export function readAnnotation(shape: Shape): Record<string, unknown>;
export function normalizeBoard(board: Board, contract: RoleContract, screen: string, assetPath: string, libraryIds: string[]): Record<string, unknown>;
export function captureTokens(catalog: TokenCatalog): Record<string, unknown>;
