import { fileURLToPath } from 'node:url'
import { mergeConfig, defineConfig } from 'vitest/config'
import viteConfig from './vite.config'

export default mergeConfig(
  viteConfig({ mode: 'test', command: 'serve', isSsrBuild: false, isPreview: false }),
  defineConfig({
    test: {
      environment: 'jsdom',
      globals: true,
      include: ['src/**/__tests__/**/*.test.ts'],
      setupFiles: [fileURLToPath(new URL('./src/__tests__/setup.ts', import.meta.url))],
    },
  }),
)
