import { setAuthToken, setRefreshToken, loadAuthToken, clearAllTokens, api } from '../api'

// Prevent actual navigation
const locationMock = { href: '' }
vi.stubGlobal('location', locationMock)

function mockFetch(response: unknown, options: { status?: number; ok?: boolean } = {}) {
  const { status = 200, ok = true } = options
  return vi.fn().mockResolvedValue({
    ok,
    status,
    statusText: 'Error',
    json: () => Promise.resolve(response),
  })
}

beforeEach(() => {
  sessionStorage.clear()
  clearAllTokens()
  locationMock.href = ''
})

describe('token management', () => {
  it('setAuthToken stores token in sessionStorage', () => {
    setAuthToken('abc123')
    expect(sessionStorage.getItem('uwu-admin-token')).toBe('abc123')
  })

  it('setAuthToken(null) removes token from sessionStorage', () => {
    setAuthToken('abc123')
    setAuthToken(null)
    expect(sessionStorage.getItem('uwu-admin-token')).toBeNull()
  })

  it('setRefreshToken stores refresh token', () => {
    setRefreshToken('refresh123')
    expect(sessionStorage.getItem('uwu-admin-refresh-token')).toBe('refresh123')
  })

  it('loadAuthToken reads from sessionStorage', () => {
    sessionStorage.setItem('uwu-admin-token', 'stored-token')
    expect(loadAuthToken()).toBe('stored-token')
  })

  it('clearAllTokens removes both tokens', () => {
    setAuthToken('a')
    setRefreshToken('b')
    clearAllTokens()
    expect(sessionStorage.getItem('uwu-admin-token')).toBeNull()
    expect(sessionStorage.getItem('uwu-admin-refresh-token')).toBeNull()
  })
})

describe('api.authStatus', () => {
  it('makes GET request to /api/auth/status', async () => {
    const fetchMock = mockFetch({ setup_required: false })
    vi.stubGlobal('fetch', fetchMock)

    const res = await api.authStatus()
    expect(res).toEqual({ setup_required: false })
    expect(fetchMock).toHaveBeenCalledWith('/api/auth/status', expect.objectContaining({
      headers: expect.objectContaining({ 'Content-Type': 'application/json' }),
    }))
  })
})

describe('api.login', () => {
  it('sends credentials and returns token', async () => {
    const fetchMock = mockFetch({ token: 'tk', refresh_token: 'rt' })
    vi.stubGlobal('fetch', fetchMock)

    const res = await api.login('user', 'pass')
    expect(res).toEqual({ token: 'tk', refresh_token: 'rt' })
    expect(fetchMock).toHaveBeenCalledWith('/api/auth/login', expect.objectContaining({
      method: 'POST',
      body: JSON.stringify({ username: 'user', password: 'pass' }),
    }))
  })
})

describe('request error handling', () => {
  it('throws on non-ok response', async () => {
    vi.stubGlobal('fetch', mockFetch({ error: 'Bad request' }, { status: 400, ok: false }))
    await expect(api.authStatus()).rejects.toThrow('Bad request')
  })

  it('falls back to statusText when no error body', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
      ok: false,
      status: 500,
      statusText: 'Internal Server Error',
      json: () => Promise.reject(new Error('no json')),
    }))
    await expect(api.authStatus()).rejects.toThrow('Internal Server Error')
  })
})

describe('auth header injection', () => {
  it('includes Authorization header when token is set', async () => {
    setAuthToken('my-token')
    const fetchMock = mockFetch({ setup_required: true })
    vi.stubGlobal('fetch', fetchMock)

    await api.authStatus()
    expect(fetchMock).toHaveBeenCalledWith('/api/auth/status', expect.objectContaining({
      headers: expect.objectContaining({ Authorization: 'Bearer my-token' }),
    }))
  })

  it('omits Authorization header when no token', async () => {
    clearAllTokens()
    const fetchMock = mockFetch({ setup_required: true })
    vi.stubGlobal('fetch', fetchMock)

    await api.authStatus()
    const callHeaders = fetchMock.mock.calls[0][1].headers
    expect(callHeaders).not.toHaveProperty('Authorization')
  })
})

describe('401 refresh flow', () => {
  it('attempts token refresh on 401 and retries', async () => {
    setAuthToken('expired')
    setRefreshToken('valid-refresh')

    let callCount = 0
    vi.stubGlobal('fetch', vi.fn().mockImplementation((url: string) => {
      callCount++
      if (url === '/api/auth/refresh') {
        return Promise.resolve({
          ok: true,
          status: 200,
          json: () => Promise.resolve({ token: 'new-token', refresh_token: 'new-refresh' }),
        })
      }
      if (callCount === 1) {
        // First call returns 401
        return Promise.resolve({ ok: false, status: 401, statusText: 'Unauthorized', json: () => Promise.resolve({}) })
      }
      // Retry succeeds
      return Promise.resolve({ ok: true, status: 200, json: () => Promise.resolve({ setup_required: false }) })
    }))

    const res = await api.authStatus()
    expect(res).toEqual({ setup_required: false })
    expect(sessionStorage.getItem('uwu-admin-token')).toBe('new-token')
  })

  it('redirects to /login when refresh fails', async () => {
    setAuthToken('expired')
    setRefreshToken('bad-refresh')

    vi.stubGlobal('fetch', vi.fn().mockImplementation((url: string) => {
      if (url === '/api/auth/refresh') {
        return Promise.resolve({ ok: false, status: 401, statusText: 'Unauthorized', json: () => Promise.resolve({}) })
      }
      return Promise.resolve({ ok: false, status: 401, statusText: 'Unauthorized', json: () => Promise.resolve({}) })
    }))

    await expect(api.authStatus()).rejects.toThrow('Unauthorized')
    expect(locationMock.href).toBe('/login')
  })

  it('redirects to /login when no refresh token exists', async () => {
    setAuthToken('expired')
    // No refresh token

    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
      ok: false,
      status: 401,
      statusText: 'Unauthorized',
      json: () => Promise.resolve({}),
    }))

    await expect(api.authStatus()).rejects.toThrow('Unauthorized')
    expect(locationMock.href).toBe('/login')
  })
})

describe('timeout handling', () => {
  it('throws "Request timed out" when fetch is aborted', async () => {
    vi.stubGlobal('fetch', vi.fn().mockImplementation(() => {
      const error = new DOMException('The operation was aborted.', 'AbortError')
      return Promise.reject(error)
    }))

    await expect(api.authStatus()).rejects.toThrow('Request timed out')
  })
})
