import { test, expect } from './fixtures'

test.describe('Route protection', () => {
  test('backend API returns 401 without auth token', async ({ request }) => {
    const endpoints = ['/api/servers', '/api/auth/logout']

    for (const endpoint of endpoints) {
      const response = await request.get(endpoint)
      expect(response.status(), `${endpoint} should return 401`).toBe(401)
    }
  })

  test('backend API returns 401 with invalid auth token', async ({
    request,
  }) => {
    const response = await request.get('/api/servers', {
      headers: { Authorization: 'Bearer invalid-token-here' },
    })
    expect(response.status()).toBe(401)
  })

  test('frontend redirects unauthenticated users to login', async ({
    browser,
    baseURL,
  }) => {
    // Create a fresh context with NO stored auth state
    const context = await browser.newContext({ baseURL })
    const page = await context.newPage()

    const protectedRoutes = [
      '/users',
      '/rooms',
      '/federation',
      '/server',
      '/console',
    ]

    for (const route of protectedRoutes) {
      await page.goto(route)
      await page.waitForURL(/\/(login|setup)/, { timeout: 10000 })
      expect(
        page.url(),
        `${route} should redirect to login or setup`,
      ).toMatch(/\/(login|setup)/)
    }

    await context.close()
  })
})

test.describe('Logout', () => {
  test('clears session and redirects to login', async ({ page }) => {
    await page.goto('/')

    // Verify we're authenticated — dashboard should be visible
    await expect(page.getByRole('heading', { name: 'Overview' })).toBeVisible({
      timeout: 10000,
    })

    // Verify tokens exist in sessionStorage before logout
    const tokenBefore = await page.evaluate(() =>
      sessionStorage.getItem('uwu-admin-token'),
    )
    expect(tokenBefore).toBeTruthy()

    // Click logout
    await page.getByRole('button', { name: 'Logout' }).click()

    // Should redirect to login page
    await page.waitForURL('**/login', { timeout: 10000 })

    // Verify sessionStorage is cleared (refresh token is now HttpOnly cookie)
    const tokenAfter = await page.evaluate(() =>
      sessionStorage.getItem('uwu-admin-token'),
    )
    expect(tokenAfter).toBeNull()
  })
})
