import 'dotenv/config'
import { defineConfig } from '@playwright/test'

// When E2E_BASE_URL is set (e.g. http://localhost:8080 from the quadlet container),
// Playwright skips launching its own webServer and tests against the running instance.
// Otherwise, it starts `npm run dev` (Vite dev server on 5173).
const baseURL = process.env.E2E_BASE_URL || 'http://localhost:5173'
const useExternalServer = !!process.env.E2E_BASE_URL

export default defineConfig({
  testDir: './e2e',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  workers: 1,
  reporter: process.env.CI ? 'html' : 'list',
  use: {
    baseURL,
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
  },
  projects: [
    {
      name: 'setup',
      testMatch: 'global-setup.ts',
    },
    {
      name: 'tests',
      testMatch: '*.spec.ts',
      dependencies: ['setup'],
    },
  ],
  ...(!useExternalServer && {
    webServer: {
      command: 'npm run dev',
      url: 'http://localhost:5173',
      reuseExistingServer: !process.env.CI,
      // In CI, the backend runs separately on :3001 — Vite's proxy config forwards /api requests to it.
    },
  }),
})
