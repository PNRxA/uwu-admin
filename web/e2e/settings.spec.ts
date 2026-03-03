import { test, expect } from './fixtures'

test.describe('Settings', () => {
  test('navigate to settings page', async ({ page }) => {
    await page.goto('/')
    await expect(page.getByRole('heading', { name: 'Overview' })).toBeVisible({ timeout: 10000 })

    await page.getByRole('link', { name: 'Global Settings' }).click()
    await page.waitForURL('**/settings')

    await expect(page.getByRole('heading', { name: 'Global Settings' })).toBeVisible()
    await expect(page.getByText('Redact admin room messages')).toBeVisible()
  })

  test('toggle redact messages off and on', async ({ page }) => {
    await page.goto('/settings')
    await expect(page.getByText('Redact admin room messages')).toBeVisible({ timeout: 10000 })

    const toggle = page.getByRole('switch')
    await expect(toggle).toBeVisible()

    // Default should be checked (true)
    await expect(toggle).toHaveAttribute('data-state', 'checked')

    // Toggle off
    await toggle.click()
    await expect(toggle).toHaveAttribute('data-state', 'unchecked')

    // Reload to verify persistence
    await page.reload()
    await expect(page.getByText('Redact admin room messages')).toBeVisible({ timeout: 10000 })
    await expect(page.getByRole('switch')).toHaveAttribute('data-state', 'unchecked')

    // Toggle back on
    await page.getByRole('switch').click()
    await expect(page.getByRole('switch')).toHaveAttribute('data-state', 'checked')
  })
})
