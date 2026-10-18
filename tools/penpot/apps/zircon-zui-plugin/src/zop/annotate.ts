import type { Shape } from '@penpot/plugin-types';
import { NAMESPACE, parseRoleContract, stableJson, validateAnnotation, type RoleContract } from '../../../zircon-zop-compiler/src/contract.mjs';

let contract: RoleContract | null = null;

export function installZopContract(source: string): number {
  contract = parseRoleContract(source);
  return Object.keys(contract.roles).length;
}

export function zopContract(): RoleContract {
  if (!contract) throw new Error('Load the reviewed roles.contract.toml in Annotate first.');
  return contract;
}

export function annotateSelection(selection: Shape[], source: string): string {
  if (selection.length !== 1) throw new Error('Select one Penpot layer to annotate.');
  const annotation: unknown = JSON.parse(source);
  validateAnnotation(annotation, zopContract(), undefined, selection[0].name);
  selection[0].setSharedPluginData(NAMESPACE, 'binding', stableJson(annotation));
  return selection[0].name;
}
