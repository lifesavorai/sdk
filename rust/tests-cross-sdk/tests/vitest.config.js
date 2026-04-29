import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    include: ['config_schema_consistency.test.js'],
  },
});
