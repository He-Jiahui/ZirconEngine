import { shouldYieldNativeComponentWork } from './penpot-native-components';

describe('native component materialization scheduling', () => {
  it('yields after a bounded batch or immediately after creating a master', () => {
    expect(shouldYieldNativeComponentWork(0, false)).toBe(false);
    expect(shouldYieldNativeComponentWork(6, false)).toBe(false);
    expect(shouldYieldNativeComponentWork(7, false)).toBe(true);
    expect(shouldYieldNativeComponentWork(8, false)).toBe(false);
    expect(shouldYieldNativeComponentWork(0, true)).toBe(true);
  });
});
