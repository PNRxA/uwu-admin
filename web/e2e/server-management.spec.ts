import { test, expect } from './fixtures'
import { openServerSelector, expectAnyToast, requireEnv } from './helpers'

test.describe('Server management', () => {
  test('add a second server', async ({ page }) => {
    await page.goto('/')

    const homeserver = requireEnv('TEST_HOMESERVER')
    const username = requireEnv('TEST_USERNAME')
    const password = requireEnv('TEST_PASSWORD')
    const roomId = requireEnv('TEST_ROOM_ID')

    // Open dropdown, click "Add Server"
    await openServerSelector(page)
    await page.getByRole('menuitem', { name: 'Add Server' }).click()

    // Fill and submit
    const dialog = page.locator('[data-slot="dialog-content"]')
    await expect(dialog).toBeVisible()
    await dialog.locator('#add-homeserver').fill(homeserver)
    await dialog.locator('#add-username').fill(username)
    await dialog.locator('#add-password').fill(password)
    await dialog.locator('#add-room-id').fill(roomId)
    await dialog.getByRole('button', { name: 'Connect' }).click()

    await expect(
      page.locator('[data-sonner-toast][data-type="success"]'),
    ).toBeVisible({ timeout: 30000 })
  })

  test('switch server', async ({ page }) => {
    await page.goto('/')
    await expect(page.getByRole('heading', { name: 'Overview' })).toBeVisible({ timeout: 10000 })

    // Open dropdown — there should be multiple server entries
    await openServerSelector(page)

    // Get all menu items — last one is "Add Server", the rest are server entries
    const allItems = page.getByRole('menuitem')
    const count = await allItems.count()
    expect(count, 'expected at least 3 menu items (2 servers + Add Server)').toBeGreaterThanOrEqual(3)

    // Intercept the next server-scoped API request to verify which server is active
    const requestPromise = page.waitForRequest((req) =>
      /\/api\/servers\/\d+\//.test(req.url()) && req.method() === 'POST'
    )
    await allItems.nth(1).click()

    // Execute a console command to trigger a server-scoped request
    const consoleTrigger = page.locator('button', { hasText: 'Console' }).last()
    await consoleTrigger.click()
    const consolePanel = page.locator('form', { hasText: '!admin' }).first()
    const input = consolePanel.locator('input[data-slot="input"]')
    await input.fill('server uptime')
    await input.press('Escape')
    await consolePanel.locator('button[type="submit"]').click()

    const request = await requestPromise
    // The second server added gets id=2
    expect(request.url()).toContain('/api/servers/2/')
  })

  test('remove the second server', async ({ page }) => {
    await page.goto('/')
    await expect(page.getByRole('heading', { name: 'Overview' })).toBeVisible({ timeout: 10000 })

    // Open dropdown
    await openServerSelector(page)

    // Find trash buttons inside menu items
    const trashButtons = page.locator('[role="menuitem"] button')
    const trashCount = await trashButtons.count()
    expect(trashCount, 'expected at least 1 trash button').toBeGreaterThanOrEqual(1)
    if (trashCount > 1) {
      await trashButtons.nth(1).click()
    } else {
      await trashButtons.first().click()
    }

    // Confirm removal in AlertDialog
    const alertDialog = page.locator('[data-slot="alert-dialog-content"]')
    await expect(alertDialog).toBeVisible()
    await alertDialog.getByRole('button', { name: 'Remove' }).click()

    await expectAnyToast(page)
  })
})
