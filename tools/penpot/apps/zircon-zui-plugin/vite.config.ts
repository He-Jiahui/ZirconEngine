/// <reference types="vitest/config" />
import { defineConfig } from 'vite';

export default defineConfig({
  root: import.meta.dirname,
  test: {
    watch: false,
    globals: true,
    environment: 'node',
    maxWorkers: 4,
    include: ['src/**/*.spec.ts'],
    reporters: ['default'],
    coverage: {
      reportsDirectory: '../coverage/zircon-zui-plugin',
      provider: 'v8',
    },
  },
});
