import 'dotenv/config'
import { defineConfig } from '@playwright/test'

// When E2E_BASE_URL is set (e.g. http://localhost:8080 from the quadlet container),
// Playwright skips launching its own webServer and tests against the running instance.
// In CI, we build and use Vite preview (production build) on :4173.
// Otherwise, we start `npm run dev` (Vite dev server on :5173).
const useExternalServer = !!process.env.E2E_BASE_URL
const isCI = !!process.env.CI

const baseURL = process.env.E2E_BASE_URL || (isCI ? 'http://localhost:4173' : 'http://localhost:5173')

export default defineConfig({
  testDir: './e2e',
  fullyParallel: false,
  forbidOnly: isCI,
  retries: isCI ? 1 : 0,
  workers: 1,
  reporter: isCI ? 'html' : 'list',
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
      command: isCI ? 'npm run build-only && npm run preview' : 'npm run dev',
      url: baseURL,
      reuseExistingServer: !isCI,
      // The backend runs separately on :3001 — Vite's proxy config forwards /api requests to it.
    },
  }),
})
