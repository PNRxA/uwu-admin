import { test, expect } from './fixtures'
import { navigateToUsers, navigateToRooms } from './helpers'

test.describe('Copy to clipboard', () => {
  test.use({
    permissions: ['clipboard-read', 'clipboard-write'],
  })

  test('copy user ID from users table', async ({ page }) => {
    await page.goto('/')
    await navigateToUsers(page)

    // Wait for user table to load — target the clickable span inside the cell
    const firstUserCell = page
      .locator('tr')
      .nth(1)
      .locator('td')
      .nth(1)
      .locator('[data-testid="copyable-cell"]')
    await expect(firstUserCell).toBeVisible({ timeout: 15000 })

    const cellText = (await firstUserCell.innerText()).trim()
    expect(cellText).toMatch(/^@/)

    await firstUserCell.click()

    const clipboardText = await page.evaluate(() =>
      navigator.clipboard.readText(),
    )
    expect(clipboardText).toBe(cellText)
  })

  test('copy room ID from rooms table', async ({ page }) => {
    await page.goto('/')
    await navigateToRooms(page)

    const firstRoomIdCell = page
      .locator('tr')
      .nth(1)
      .locator('td')
      .nth(1)
      .locator('[data-testid="copyable-cell"]')
    await expect(firstRoomIdCell).toBeVisible({ timeout: 15000 })

    const cellText = (await firstRoomIdCell.innerText()).trim()
    expect(cellText).toMatch(/^!/)

    await firstRoomIdCell.click()

    const clipboardText = await page.evaluate(() =>
      navigator.clipboard.readText(),
    )
    expect(clipboardText).toBe(cellText)
  })
})
