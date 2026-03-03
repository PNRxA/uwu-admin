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

    const toggle = page.getByRole('switch', { name: 'Redact admin room messages' })
    await expect(toggle).toBeVisible()

    // Default should be checked (true)
    await expect(toggle).toHaveAttribute('data-state', 'checked')

    // Toggle off
    await toggle.click()
    await expect(toggle).toHaveAttribute('data-state', 'unchecked')

    // Reload to verify persistence
    await page.reload()
    await expect(page.getByText('Redact admin room messages')).toBeVisible({ timeout: 10000 })
    await expect(page.getByRole('switch', { name: 'Redact admin room messages' })).toHaveAttribute('data-state', 'unchecked')

    // Toggle back on
    await page.getByRole('switch', { name: 'Redact admin room messages' }).click()
    await expect(page.getByRole('switch', { name: 'Redact admin room messages' })).toHaveAttribute('data-state', 'checked')
  })

  test('switch between built-in themes', async ({ page }) => {
    await page.goto('/settings')
    await expect(page.getByRole('heading', { name: 'Appearance' })).toBeVisible({ timeout: 10000 })

    // uwu pink should be active by default
    await expect(page.getByText('uwu pink')).toBeVisible()
    await expect(page.getByText('Slate')).toBeVisible()

    // Click Slate theme swatch
    await page.getByText('Slate').click()

    // Verify the theme override style tag exists
    await expect(page.locator('#uwu-theme-overrides')).toBeAttached()

    // Click uwu pink to go back
    await page.getByText('uwu pink').click()

    // Theme overrides should be removed for default theme
    await expect(page.locator('#uwu-theme-overrides')).not.toBeAttached()
  })

  test('create and delete custom theme', async ({ page }) => {
    await page.goto('/settings')
    await expect(page.getByRole('heading', { name: 'Appearance' })).toBeVisible({ timeout: 10000 })

    // Click create theme button
    await page.getByRole('button', { name: 'Create Theme' }).click()

    // Fill in name
    await page.getByPlaceholder('My Theme').fill('Ocean Blue')

    // Save
    await page.getByRole('button', { name: 'Save' }).click()

    // Theme should appear in the picker
    await expect(page.getByText('Ocean Blue')).toBeVisible()

    // Theme overrides should be applied
    await expect(page.locator('#uwu-theme-overrides')).toBeAttached()

    // Hover the custom theme swatch to reveal the edit button, then click it
    await page.getByRole('button', { name: 'Ocean Blue' }).hover()
    await page.getByRole('button', { name: 'Edit Theme' }).click()

    // Editor should be open with the theme name
    await expect(page.getByPlaceholder('My Theme')).toHaveValue('Ocean Blue')

    // Delete the theme
    await page.getByRole('button', { name: 'Delete' }).click()

    // Theme should be gone from the picker
    await expect(page.getByText('Ocean Blue')).not.toBeVisible()

    // Should fall back to default theme (overrides removed)
    await expect(page.locator('#uwu-theme-overrides')).not.toBeAttached()
  })

  test('flavour text toggle changes logo', async ({ page }) => {
    await page.goto('/settings')
    await expect(page.getByRole('heading', { name: 'Branding' })).toBeVisible({ timeout: 10000 })

    // uwu mode should be on by default
    const toggle = page.getByRole('switch', { name: 'uwu mode' })
    await expect(toggle).toHaveAttribute('data-state', 'checked')

    // Sidebar should show "uwu" text
    await expect(page.locator('.uwu-text').first()).toBeVisible()

    // Toggle off
    await toggle.click()
    await expect(toggle).toHaveAttribute('data-state', 'unchecked')

    // Sidebar should now show "Admin" text
    await expect(page.getByText('Admin').first()).toBeVisible()

    // Toggle back on
    await toggle.click()
    await expect(toggle).toHaveAttribute('data-state', 'checked')
  })
})
