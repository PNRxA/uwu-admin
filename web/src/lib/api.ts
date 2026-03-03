interface StatusResponse {
  setup_required: boolean
}

export interface CommandResponse {
  response: string
}

interface AddServerParams {
  homeserver: string
  username: string
  password: string
  room_id: string
}

interface ServerInfo {
  id: number
  homeserver: string
  user_id: string
  connected: boolean
}

interface ListServersResponse {
  servers: ServerInfo[]
}

interface AddServerResponse {
  id: number
  homeserver: string
  user_id: string
}

interface AuthResponse {
  token: string
}

let authToken: string | null = sessionStorage.getItem('uwu-admin-token')

export function setAuthToken(token: string | null) {
  authToken = token
  if (token) {
    sessionStorage.setItem('uwu-admin-token', token)
  } else {
    sessionStorage.removeItem('uwu-admin-token')
  }
}

export function loadAuthToken(): string | null {
  authToken = sessionStorage.getItem('uwu-admin-token')
  return authToken
}

export function clearAllTokens() {
  setAuthToken(null)
}

let refreshPromise: Promise<boolean> | null = null

async function attemptRefresh(): Promise<boolean> {
  if (refreshPromise) return refreshPromise

  refreshPromise = (async () => {
    try {
      const res = await fetch('/api/auth/refresh', {
        method: 'POST',
        credentials: 'include',
      })

      if (!res.ok) {
        clearAllTokens()
        return false
      }

      const data: AuthResponse = await res.json()
      setAuthToken(data.token)
      return true
    } catch {
      clearAllTokens()
      return false
    } finally {
      refreshPromise = null
    }
  })()

  return refreshPromise
}

async function request<T>(url: string, options?: RequestInit): Promise<T> {
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  }
  if (authToken) {
    headers['Authorization'] = `Bearer ${authToken}`
  }

  const controller = new AbortController()
  const timeout = setTimeout(() => controller.abort(), 30_000)

  let res: Response
  try {
    res = await fetch(url, {
      headers,
      signal: controller.signal,
      credentials: 'include',
      ...options,
    })
  } catch (e) {
    if (e instanceof DOMException && e.name === 'AbortError') {
      throw new Error('Request timed out')
    }
    throw e
  } finally {
    clearTimeout(timeout)
  }

  if (res.status === 429) {
    const retryAfter = Math.min(parseInt(res.headers.get('retry-after') ?? '60', 10) || 60, 120)
    throw new Error(`Too many attempts. Try again in ${retryAfter}s.`)
  }

  if (res.status === 401) {
    const refreshed = await attemptRefresh()
    if (refreshed) {
      // Retry with new token
      const retryHeaders: Record<string, string> = {
        'Content-Type': 'application/json',
      }
      if (authToken) {
        retryHeaders['Authorization'] = `Bearer ${authToken}`
      }

      const retryController = new AbortController()
      const retryTimeout = setTimeout(() => retryController.abort(), 30_000)

      let retryRes: Response
      try {
        retryRes = await fetch(url, {
          ...options,
          headers: retryHeaders,
          signal: retryController.signal,
          credentials: 'include',
        })
      } catch (e) {
        if (e instanceof DOMException && e.name === 'AbortError') {
          throw new Error('Request timed out')
        }
        throw e
      } finally {
        clearTimeout(retryTimeout)
      }

      if (retryRes.status === 401) {
        clearAllTokens()
        window.location.href = '/login'
        throw new Error('Unauthorized')
      }

      if (!retryRes.ok) {
        const body = await retryRes.json().catch(() => ({ error: retryRes.statusText }))
        throw new Error(body.error || retryRes.statusText)
      }
      return retryRes.json()
    }

    clearAllTokens()
    window.location.href = '/login'
    throw new Error('Unauthorized')
  }

  if (!res.ok) {
    const body = await res.json().catch(() => ({ error: res.statusText }))
    throw new Error(body.error || res.statusText)
  }
  return res.json()
}

export const api = {
  // Auth endpoints (no server scope)
  authStatus() {
    return request<StatusResponse>('/api/auth/status')
  },

  register(username: string, password: string) {
    return request<AuthResponse>('/api/auth/setup', {
      method: 'POST',
      body: JSON.stringify({ username, password }),
    })
  },

  login(username: string, password: string) {
    return request<AuthResponse>('/api/auth/login', {
      method: 'POST',
      body: JSON.stringify({ username, password }),
    })
  },

  logout() {
    return request<{ ok: boolean }>('/api/auth/logout', {
      method: 'POST',
    })
  },

  // Server management
  addServer(params: AddServerParams) {
    return request<AddServerResponse>('/api/servers', {
      method: 'POST',
      body: JSON.stringify(params),
    })
  },

  listServers() {
    return request<ListServersResponse>('/api/servers')
  },

  removeServer(serverId: number) {
    return request<{ removed: boolean }>(`/api/servers/${serverId}`, {
      method: 'DELETE',
    })
  },

  // Server-scoped endpoints
  command(serverId: number, command: string) {
    return request<CommandResponse>(`/api/servers/${serverId}/command`, {
      method: 'POST',
      body: JSON.stringify({ command }),
    })
  },

  // Settings
  getPublicSettings() {
    return request<Record<string, string>>('/api/settings/public')
  },

  getSettings() {
    return request<Record<string, string>>('/api/settings')
  },

  updateSettings(settings: Record<string, string>) {
    return request<Record<string, string>>('/api/settings', {
      method: 'PUT',
      body: JSON.stringify(settings),
    })
  },
}
