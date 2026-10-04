import type { Board, Shape } from '@penpot/plugin-types';
import {
  copyNativeComponentAppearance,
  nativeComponentStructure,
} from './penpot-native-component-copy';

function shape(minWidth: number | null): Shape {
  return {
    type: 'board',
    name: 'Button',
    width: 120,
    height: 32,
    children: [],
    layoutChild: { minWidth },
    getSharedPluginData: () => '',
    getSharedPluginDataKeys: () => [],
  } as unknown as Board;
}

describe('native instance size constraints', () => {
  it('does not share a master when a later instance needs a cleared bound', () => {
    expect(nativeComponentStructure(shape(100))).not.toEqual(
      nativeComponentStructure(shape(null)),
    );
    expect(nativeComponentStructure(shape(null))).toEqual(
      nativeComponentStructure(shape(null)),
    );
  });

  it('rejects a host that silently ignores a nullable constraint override', () => {
    const target = shape(100);
    Object.defineProperty(target.layoutChild, 'minWidth', {
      get: () => 100,
      set: () => undefined,
    });
    expect(() => copyNativeComponentAppearance(shape(null), target)).toThrow(
      'did not preserve native instance size constraints',
    );
  });
});
