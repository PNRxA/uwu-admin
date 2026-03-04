import { test as setup, expect } from '@playwright/test'
import * as fs from 'node:fs'
import { fileURLToPath } from 'node:url'
import * as path from 'node:path'
import { requireEnv, openServerSelector } from './helpers'

const __dirname = path.dirname(fileURLToPath(import.meta.url))

const ADMIN_USERNAME = 'admin'
const ADMIN_PASSWORD = 'admin-e2e-test'

const AUTH_DIR = path.join(__dirname, '.auth')
const SESSION_PATH = path.join(AUTH_DIR, 'session.json')

setup.setTimeout(60000)
setup('authenticate and add server', async ({ page }) => {
  fs.mkdirSync(AUTH_DIR, { recursive: true })

  // Navigate to the app — router guard will redirect to /setup or /login
  await page.goto('/', { waitUntil: 'networkidle' })
  await page.waitForURL(/\/(setup|login)/)

  const url = page.url()

  if (url.includes('/setup')) {
    // First run — register admin account
    await page.locator('#username').waitFor({ state: 'visible' })
    await page.locator('#username').click()
    await page.locator('#username').fill(ADMIN_USERNAME)
    await page.locator('#password').click()
    await page.locator('#password').fill(ADMIN_PASSWORD)
    await page.locator('#confirm-password').click()
    await page.locator('#confirm-password').fill(ADMIN_PASSWORD)
    await page.getByRole('button', { name: 'Create Account' }).click()
  } else if (url.includes('/login')) {
    // Already set up — log in
    await page.locator('#username').waitFor({ state: 'visible' })
    await page.locator('#username').click()
    await page.locator('#username').fill(ADMIN_USERNAME)
    await page.locator('#password').click()
    await page.locator('#password').fill(ADMIN_PASSWORD)
    await page.getByRole('button', { name: 'Sign In' }).click()
  }

  // Wait for redirect to dashboard and layout to render
  await expect(page).not.toHaveURL(/\/(setup|login)/, { timeout: 15000 })
  await expect(page.locator('header')).toBeVisible({ timeout: 15000 })

  // Add a test server via ServerSelector
  const homeserver = requireEnv('TEST_HOMESERVER')
  const username = requireEnv('TEST_USERNAME')
  const password = requireEnv('TEST_PASSWORD')
  const roomId = requireEnv('TEST_ROOM_ID')

  // Open the server dropdown and click "Add Server"
  await openServerSelector(page)
  await page.getByRole('menuitem', { name: 'Add Server' }).click()

  // Fill in server details
  const dialog = page.locator('[data-slot="dialog-content"]')
  await expect(dialog).toBeVisible()
  await dialog.locator('#add-homeserver').fill(homeserver)
  await dialog.locator('#add-username').fill(username)
  await dialog.locator('#add-password').fill(password)
  await dialog.locator('#add-room-id').fill(roomId)
  await dialog.getByRole('button', { name: 'Connect' }).click()

  // Wait for success toast (server added)
  await expect(
    page.locator('[data-sonner-toast][data-type="success"]'),
  ).toBeVisible({ timeout: 30000 })

  // Extract sessionStorage token and save it (refresh token is now HttpOnly cookie)
  const session = await page.evaluate(() => {
    return {
      'uwu-admin-token': sessionStorage.getItem('uwu-admin-token'),
    }
  })
  fs.writeFileSync(SESSION_PATH, JSON.stringify(session, null, 2))
})
