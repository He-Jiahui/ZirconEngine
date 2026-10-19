export const VERSION: string;
export const NAMESPACE: string;
export interface RoleDefinition { kind: string; screen: string; required: boolean; command?: string; slot?: string }
export interface RoleContract { schema_version: number; roles: Record<string, RoleDefinition> }
export function stableJson(value: unknown): string;
export function parseRoleContract(source: string): RoleContract;
export function validateAnnotation(annotation: unknown, contract: RoleContract, screen: string | undefined, path: string): void;
export function safeEntry(path: string): string;
