import { test, expect } from './fixtures'

/**
 * Clicks through the autocomplete suggestions to build a command.
 * Each token is a suggestion name to click (e.g. 'users', 'list-users').
 */
async function clickAutocomplete(
  page: import('@playwright/test').Page,
  input: import('@playwright/test').Locator,
  tokens: string[],
) {
  await input.click()
  for (const token of tokens) {
    // The dropdown renders above the input — find the exact match
    const btn = page
      .locator('button.flex.w-full.items-center.gap-2')
      .filter({ has: page.locator('code', { hasText: new RegExp(`^${token}$`) }) })
    await expect(btn.first()).toBeVisible({ timeout: 5000 })
    await btn.first().click()
  }
}

/**
 * Waits for a console response to appear with an OK badge.
 */
async function expectConsoleResponse(
  page: import('@playwright/test').Page,
  container: import('@playwright/test').Locator,
) {
  // Wait for loading to finish (... badge disappears) and OK badge appears
  const okBadge = container.locator('code', { hasText: '!admin users list-users' })
  await expect(okBadge).toBeVisible({ timeout: 15000 })
  // Response area should have content
  const response = container.locator('.console-response').last()
  await expect(response).toBeVisible()
  // Should not be "Waiting for response..."
  await expect(response).not.toHaveText(/Waiting for response/, { timeout: 15000 })
}

test.describe('Console - bottom sheet', () => {
  test.describe.configure({ mode: 'serial' })

  test('execute command via autocomplete click', async ({ page }) => {
    await page.goto('/')

    // Open the bottom sheet console
    const consoleTrigger = page.locator('button', { hasText: 'Console' }).last()
    await consoleTrigger.click()

    // The console panel content should be visible
    const consolePanel = page.locator('form', { hasText: '!admin' }).first()
    await expect(consolePanel).toBeVisible()

    // Click through autocomplete: users -> list-users
    const input = consolePanel.locator('input[data-slot="input"]')
    await clickAutocomplete(page, input, ['users', 'list-users'])

    // Click Send
    await consolePanel.locator('button[type="submit"]').click()

    // Verify response appears
    await expectConsoleResponse(page, page)
  })

  test('execute command by typing', async ({ page }) => {
    await page.goto('/')

    // Open the bottom sheet console
    const consoleTrigger = page.locator('button', { hasText: 'Console' }).last()
    await consoleTrigger.click()

    const consolePanel = page.locator('form', { hasText: '!admin' }).first()
    await expect(consolePanel).toBeVisible()

    const input = consolePanel.locator('input[data-slot="input"]')
    await input.fill('users list-users')

    // Dismiss autocomplete if showing, then submit
    await input.press('Escape')
    await consolePanel.locator('button[type="submit"]').click()

    await expectConsoleResponse(page, page)
  })
})

test.describe('Console - fullscreen', () => {
  test.describe.configure({ mode: 'serial' })

  test('execute command via autocomplete click', async ({ page }) => {
    await page.goto('/console')

    const consoleForm = page.locator('form', { hasText: '!admin' })
    await expect(consoleForm).toBeVisible()

    const input = consoleForm.locator('input[data-slot="input"]')
    await clickAutocomplete(page, input, ['users', 'list-users'])

    await consoleForm.locator('button[type="submit"]').click()

    await expectConsoleResponse(page, page)
  })

  test('execute command by typing', async ({ page }) => {
    await page.goto('/console')

    const consoleForm = page.locator('form', { hasText: '!admin' })
    await expect(consoleForm).toBeVisible()

    const input = consoleForm.locator('input[data-slot="input"]')
    await input.fill('users list-users')

    await input.press('Escape')
    await consoleForm.locator('button[type="submit"]').click()

    await expectConsoleResponse(page, page)
  })
})
