import { test as base } from '@playwright/test'
import * as fs from 'node:fs'
import { fileURLToPath } from 'node:url'
import * as path from 'node:path'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const SESSION_PATH = path.join(__dirname, '.auth', 'session.json')

/**
 * Custom test fixture that injects sessionStorage tokens before each test.
 * Playwright's storageState only handles localStorage + cookies, not sessionStorage.
 */
export const test = base.extend({
  page: async ({ page }, use) => {
    if (fs.existsSync(SESSION_PATH)) {
      const session = JSON.parse(fs.readFileSync(SESSION_PATH, 'utf-8'))
      await page.addInitScript((tokens) => {
        for (const [key, value] of Object.entries(tokens)) {
          sessionStorage.setItem(key, value as string)
        }
      }, session)
    }
    await use(page)
  },
})

export { expect } from '@playwright/test'
