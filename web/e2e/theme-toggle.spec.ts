import { test, expect } from './fixtures'

test.describe('Theme toggle', () => {
  test('switch between light and dark mode', async ({ page }) => {
    await page.goto('/')

    const html = page.locator('html')

    // Open theme menu and switch to dark
    await page.getByRole('button', { name: 'Toggle theme' }).click()
    await page.getByRole('menuitem', { name: 'Dark' }).click()
    await expect(html).toHaveClass(/dark/)

    // Open theme menu and switch to light
    await page.getByRole('button', { name: 'Toggle theme' }).click()
    await page.getByRole('menuitem', { name: 'Light' }).click()
    await expect(html).not.toHaveClass(/dark/)

    // Switch to system (should resolve to light or dark based on prefers-color-scheme)
    await page.getByRole('button', { name: 'Toggle theme' }).click()
    await page.getByRole('menuitem', { name: 'System' }).click()

    // Verify theme persists in localStorage
    const stored = await page.evaluate(() => localStorage.getItem('theme'))
    expect(stored).toBe('system')
  })

  test('theme persists across navigation', async ({ page }) => {
    await page.goto('/')

    // Set dark mode
    await page.getByRole('button', { name: 'Toggle theme' }).click()
    await page.getByRole('menuitem', { name: 'Dark' }).click()
    await expect(page.locator('html')).toHaveClass(/dark/)

    // Navigate to another page
    await page.getByRole('link', { name: 'Users' }).click()
    await page.waitForURL('**/users')

    // Theme should still be dark
    await expect(page.locator('html')).toHaveClass(/dark/)

    // Clean up — restore light mode
    await page.getByRole('button', { name: 'Toggle theme' }).click()
    await page.getByRole('menuitem', { name: 'Light' }).click()
  })
})
