import { test, expect } from './fixtures'
import {
  openActionsMenu,
  executeConfirmAction,
  executeInputAction,
  executeReadOnlyAction,
  navigateToUsers,
  resolveRoomId,
  dismissToasts,
} from './helpers'

// All tests share a single disposable user and a resolved room ID.
let testUserId: string
let resolvedRoomId: string

test.describe('User actions', () => {
  test.describe.configure({ mode: 'serial' })

  test('create test user and resolve room ID', async ({ page }) => {
    // First resolve the actual room ID (!...) from the rooms table
    resolvedRoomId = await resolveRoomId(page, 'Admin Room')
    expect(resolvedRoomId).toMatch(/^!/)

    await navigateToUsers(page)

    const timestamp = Date.now()
    const username = `e2e-test-${timestamp}`

    // Click "Create User" button
    await page.getByRole('button', { name: 'Create User' }).click()
    const dialog = page.locator('[data-slot="dialog-content"]')
    await expect(dialog).toBeVisible()
    await dialog.locator('#new-username').fill(username)
    await dialog.locator('#new-password').fill('test-password-123')
    await dialog.getByRole('button', { name: 'Create' }).click()

    // Wait for success toast
    await expect(
      page.locator('[data-sonner-toast][data-type="success"]'),
    ).toBeVisible({ timeout: 15000 })

    // Find the created user in the table by looking for the username in a user ID
    const userRow = page.locator('tr', { hasText: username })
    await expect(userRow).toBeVisible({ timeout: 10000 })

    // Extract the full user ID from the row
    testUserId = await userRow.locator('td').nth(1).innerText()
    testUserId = testUserId.trim()
    expect(testUserId).toContain(username)
  })

  test('list joined rooms', async ({ page }) => {
    await page.goto('/users')
    await openActionsMenu(page, testUserId)
    await executeReadOnlyAction(page, 'List Joined Rooms')
  })

  test('suspend', async ({ page }) => {
    await page.goto('/users')
    await openActionsMenu(page, testUserId)
    await executeConfirmAction(page, 'Suspend')
  })

  test('unsuspend', async ({ page }) => {
    await page.goto('/users')
    await openActionsMenu(page, testUserId)
    await executeConfirmAction(page, 'Unsuspend')
  })

  test('lock', async ({ page }) => {
    await page.goto('/users')
    await openActionsMenu(page, testUserId)
    await executeConfirmAction(page, 'Lock')
  })

  test('unlock', async ({ page }) => {
    await page.goto('/users')
    await openActionsMenu(page, testUserId)
    await executeConfirmAction(page, 'Unlock')
  })

  test('force logout', async ({ page }) => {
    await page.goto('/users')
    await openActionsMenu(page, testUserId)
    await executeConfirmAction(page, 'Force Logout')
  })

  test('disable login', async ({ page }) => {
    await page.goto('/users')
    await openActionsMenu(page, testUserId)
    await executeConfirmAction(page, 'Disable Login')
  })

  test('enable login', async ({ page }) => {
    await page.goto('/users')
    await openActionsMenu(page, testUserId)
    await executeConfirmAction(page, 'Enable Login')
  })

  test('reset password', async ({ page }) => {
    await page.goto('/users')
    await openActionsMenu(page, testUserId)
    await executeInputAction(page, 'Reset Password', {})
  })

  test('make admin', async ({ page }) => {
    await page.goto('/users')
    await openActionsMenu(page, testUserId)
    await executeConfirmAction(page, 'Make Admin')
  })

  test('force join room', async ({ page }) => {
    await page.goto('/users')
    await openActionsMenu(page, testUserId)
    await executeInputAction(page, 'Force Join Room', {
      room_id: resolvedRoomId,
    })
  })

  test('get room tags', async ({ page }) => {
    await page.goto('/users')
    await openActionsMenu(page, testUserId)
    await executeInputAction(page, 'Get Room Tags', {
      room_id: resolvedRoomId,
    })
  })

  test('set room tag', async ({ page }) => {
    await page.goto('/users')
    await openActionsMenu(page, testUserId)
    await executeInputAction(page, 'Set Room Tag', {
      room_id: resolvedRoomId,
      tag: 'm.test',
    })
  })

  test('delete room tag', async ({ page }) => {
    await page.goto('/users')
    await openActionsMenu(page, testUserId)
    await executeInputAction(page, 'Delete Room Tag', {
      room_id: resolvedRoomId,
      tag: 'm.test',
    })
  })

  test('force demote', async ({ page }) => {
    await page.goto('/users')
    await openActionsMenu(page, testUserId)
    await executeInputAction(page, 'Force Demote', {
      room_id: resolvedRoomId,
    })
  })

  test('force leave room', async ({ page }) => {
    await page.goto('/users')
    await openActionsMenu(page, testUserId)
    await executeInputAction(page, 'Force Leave Room', {
      room_id: resolvedRoomId,
    })
  })

  test('deactivate account', async ({ page }) => {
    await page.goto('/users')
    await dismissToasts(page)
    await openActionsMenu(page, testUserId)
    await executeConfirmAction(page, 'Deactivate Account')
  })
})
