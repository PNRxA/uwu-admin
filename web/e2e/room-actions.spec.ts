import { test, expect } from './fixtures'
import {
  openActionsMenu,
  executeConfirmAction,
  executeInputAction,
  executeReadOnlyAction,
  navigateToRooms,
  gotoRooms,
  dismissToasts,
} from './helpers'

// The room table shows resolved room IDs (!...), not the alias from TEST_ROOM_ID.
// We find the admin room row by its name which contains "Admin Room".
const ROOM_ROW_TEXT = 'Admin Room'

// Shared alias between set/remove tests
const testAlias = `e2e-test-${Date.now()}`

test.describe('Room actions', () => {
  test.describe.configure({ mode: 'serial' })

  test('list members', async ({ page }) => {
    await page.goto('/')
    await navigateToRooms(page)

    // Wait for room table to load
    await expect(page.locator('tr', { hasText: ROOM_ROW_TEXT })).toBeVisible({
      timeout: 15000,
    })

    await openActionsMenu(page, ROOM_ROW_TEXT)
    await executeReadOnlyAction(page, 'List Members')
  })

  test('view topic', async ({ page }) => {
    await gotoRooms(page)
    await expect(page.locator('tr', { hasText: ROOM_ROW_TEXT })).toBeVisible({
      timeout: 15000,
    })
    await openActionsMenu(page, ROOM_ROW_TEXT)
    await executeReadOnlyAction(page, 'View Topic')
  })

  test('check exists', async ({ page }) => {
    await gotoRooms(page)
    await expect(page.locator('tr', { hasText: ROOM_ROW_TEXT })).toBeVisible({
      timeout: 15000,
    })
    await openActionsMenu(page, ROOM_ROW_TEXT)
    await executeReadOnlyAction(page, 'Check Exists')
  })

  test('list aliases', async ({ page }) => {
    await gotoRooms(page)
    await expect(page.locator('tr', { hasText: ROOM_ROW_TEXT })).toBeVisible({
      timeout: 15000,
    })
    await openActionsMenu(page, ROOM_ROW_TEXT)
    await executeReadOnlyAction(page, 'List Aliases')
  })

  test('set alias', async ({ page }) => {
    await gotoRooms(page)
    await expect(page.locator('tr', { hasText: ROOM_ROW_TEXT })).toBeVisible({
      timeout: 15000,
    })
    await openActionsMenu(page, ROOM_ROW_TEXT)
    await executeInputAction(page, 'Set Alias', {
      alias: testAlias,
    })
  })

  test('remove alias', async ({ page }) => {
    await gotoRooms(page)
    await expect(page.locator('tr', { hasText: ROOM_ROW_TEXT })).toBeVisible({
      timeout: 15000,
    })
    await openActionsMenu(page, ROOM_ROW_TEXT)
    await executeInputAction(page, 'Remove Alias', {
      alias: testAlias,
    })
  })

  test('publish', async ({ page }) => {
    await gotoRooms(page)
    await expect(page.locator('tr', { hasText: ROOM_ROW_TEXT })).toBeVisible({
      timeout: 15000,
    })
    await openActionsMenu(page, ROOM_ROW_TEXT)
    await executeConfirmAction(page, 'Publish')
  })

  test('unpublish', async ({ page }) => {
    await gotoRooms(page)
    await expect(page.locator('tr', { hasText: ROOM_ROW_TEXT })).toBeVisible({
      timeout: 15000,
    })
    await openActionsMenu(page, ROOM_ROW_TEXT)
    await executeConfirmAction(page, 'Unpublish')
  })

  test('ban room', async ({ page }) => {
    await gotoRooms(page)
    await expect(page.locator('tr', { hasText: ROOM_ROW_TEXT })).toBeVisible({
      timeout: 15000,
    })
    await dismissToasts(page)
    await openActionsMenu(page, ROOM_ROW_TEXT)

    // The admin room cannot be banned — the server returns an error like
    // "Not allowed to ban the admin room." which is expected behaviour.
    await page.getByRole('menuitem', { name: 'Ban Room', exact: true }).click()
    const dialog = page.locator('[data-slot="alert-dialog-content"]')
    await expect(dialog).toBeVisible()
    await dialog.getByRole('button', { name: 'Confirm' }).click()
    // Expect the specific error toast about not being allowed to ban the admin room
    const errorToast = page.locator('[data-sonner-toast][data-type="error"]')
    await expect(errorToast).toBeVisible({ timeout: 30000 })
    await expect(errorToast).toContainText('Not allowed to ban the admin room')
    // Dialog stays open on error — dismiss it via Cancel
    await dialog.getByRole('button', { name: 'Cancel' }).click()
    await expect(dialog).toBeHidden()
  })

  test('unban room', async ({ page }) => {
    await gotoRooms(page)
    await expect(page.locator('tr', { hasText: ROOM_ROW_TEXT })).toBeVisible({
      timeout: 15000,
    })
    await openActionsMenu(page, ROOM_ROW_TEXT)
    await executeConfirmAction(page, 'Unban Room')
  })
})
