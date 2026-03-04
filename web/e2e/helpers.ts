import { type Page, expect } from '@playwright/test'

/**
 * Returns the value of a required environment variable, or throws with a helpful message.
 */
export function requireEnv(name: string): string {
  const value = process.env[name]
  if (!value) {
    throw new Error(
      `Missing required environment variable: ${name}. ` +
        'Set TEST_HOMESERVER, TEST_USERNAME, TEST_PASSWORD, and TEST_ROOM_ID.',
    )
  }
  return value
}

/**
 * Opens the actions dropdown menu for a table row containing the given text.
 */
export async function openActionsMenu(page: Page, rowText: string) {
  const row = page.locator('tr', { hasText: rowText })
  await row.getByRole('button', { name: 'Actions' }).click()
}

/**
 * Executes a confirm action: clicks menu item -> AlertDialog -> Confirm -> success toast.
 * Asserts the dialog closes (action succeeded) and a success toast appears.
 */
export async function executeConfirmAction(page: Page, menuItemText: string) {
  await page.getByRole('menuitem', { name: menuItemText, exact: true }).click()
  const dialog = page.locator('[data-slot="alert-dialog-content"]')
  await expect(dialog).toBeVisible()
  await dialog.getByRole('button', { name: 'Confirm' }).click()
  // Dialog stays open until command completes; toast appears on success.
  // Backend may need up to 30s for Matrix sync, so use a generous timeout.
  await Promise.all([
    expectSuccessToast(page),
    expect(dialog).toBeHidden({ timeout: 30000 }),
  ])
}

/**
 * Executes an input action: clicks menu item -> Dialog -> fills fields -> Execute -> success toast.
 * Asserts the dialog closes and a success toast appears.
 */
export async function executeInputAction(
  page: Page,
  menuItemText: string,
  fields: Record<string, string>,
) {
  await page.getByRole('menuitem', { name: menuItemText, exact: true }).click()
  const dialog = page.locator('[data-slot="dialog-content"]')
  await expect(dialog).toBeVisible()
  for (const [name, value] of Object.entries(fields)) {
    await dialog.locator(`#input-${name}`).fill(value)
  }
  await dialog.getByRole('button', { name: 'Execute' }).click()
  // Dialog stays open until command completes; toast appears on success.
  await Promise.all([
    expectSuccessToast(page),
    expect(dialog).toBeHidden({ timeout: 30000 }),
  ])
}

/**
 * Executes a read-only action: clicks menu item -> Dialog -> waits for content -> closes.
 * Asserts actual response content loaded (not just "Loading...").
 */
export async function executeReadOnlyAction(page: Page, menuItemText: string) {
  await page.getByRole('menuitem', { name: menuItemText, exact: true }).click()
  const dialog = page.locator('[data-slot="dialog-content"]')
  await expect(dialog).toBeVisible()
  // Wait for the "Loading..." text to disappear (content loaded)
  await expect(dialog.getByText('Loading...')).toBeHidden({ timeout: 15000 })
  // Verify real content appeared (not an empty dialog or error fallback)
  await expect(dialog.getByText('Failed to execute command')).not.toBeVisible()
  // Close the dialog
  await page.keyboard.press('Escape')
  await expect(dialog).toBeHidden()
}

/**
 * Waits for a success toast notification and asserts no error toast is present.
 */
export async function expectSuccessToast(page: Page) {
  const successToast = page.locator('[data-sonner-toast][data-type="success"]')
  const errorToast = page.locator('[data-sonner-toast][data-type="error"]')
  await expect(successToast).toBeVisible({ timeout: 30000 })
  await expect(errorToast).not.toBeVisible()
}

/**
 * Waits for any toast (success or error). Only for actions where failure is acceptable
 * (e.g. removing a non-existent alias).
 */
export async function expectAnyToast(page: Page) {
  await expect(page.locator('[data-sonner-toast]').first()).toBeVisible({
    timeout: 15000,
  })
}

/**
 * Navigate to the Users page via sidebar.
 */
export async function navigateToUsers(page: Page) {
  await page.getByRole('link', { name: 'Users' }).click()
  await page.waitForURL('**/users')
}

/**
 * Navigate to the Rooms page via sidebar.
 */
export async function navigateToRooms(page: Page) {
  await page.getByRole('link', { name: 'Rooms' }).click()
  await page.waitForURL('**/rooms')
}

/**
 * Navigate to the Settings page via sidebar.
 */
export async function navigateToSettings(page: Page) {
  await page.getByRole('link', { name: 'Global Settings' }).click()
  await page.waitForURL('**/settings')
}

/**
 * Navigate to the Console page via sidebar.
 */
export async function navigateToConsole(page: Page) {
  await page.getByRole('link', { name: 'Console' }).click()
  await page.waitForURL('**/console')
}

/**
 * Go to the app root and navigate to a specific page via sidebar.
 * Handles the /servers/:id redirect automatically.
 */
export async function gotoUsers(page: Page) {
  await page.goto('/')
  await navigateToUsers(page)
}

export async function gotoRooms(page: Page) {
  await page.goto('/')
  await navigateToRooms(page)
}

export async function gotoSettings(page: Page) {
  await page.goto('/settings')
  await page.waitForURL('**/settings')
}

export async function gotoConsole(page: Page) {
  await page.goto('/')
  await navigateToConsole(page)
}

/**
 * Opens the server selector dropdown in the header.
 */
export async function openServerSelector(page: Page) {
  // The ServerSelector trigger is the button between the sidebar separator and the theme toggle.
  // It contains either "No server selected" text or a homeserver URL with a chevron.
  const trigger = page.locator('header').getByRole('button').filter({
    has: page.locator('svg.lucide-chevron-down'),
  })
  await trigger.click()
  // Wait for the dropdown menu to appear
  await expect(page.getByRole('menuitem').first()).toBeVisible()
}

/**
 * Resolves the actual room ID (!...) from the rooms table by matching a row's text.
 * The rooms table shows resolved IDs, not aliases.
 */
export async function resolveRoomId(
  page: Page,
  rowText: string,
): Promise<string> {
  await page.goto('/')
  await navigateToRooms(page)
  const row = page.locator('tr', { hasText: rowText })
  await expect(row).toBeVisible({ timeout: 15000 })
  // The room ID is in the second cell (index 1), as a font-mono element
  const roomIdCell = row.locator('td').nth(1)
  const roomId = (await roomIdCell.innerText()).trim()
  return roomId
}

/**
 * Dismiss all visible toasts to clean up between actions.
 */
export async function dismissToasts(page: Page) {
  const toasts = page.locator('[data-sonner-toast]')
  const count = await toasts.count()
  for (let i = 0; i < count; i++) {
    await toasts.first().click()
  }
  // Wait for toasts to clear
  if (count > 0) {
    await expect(toasts).toHaveCount(0, { timeout: 5000 })
  }
}
