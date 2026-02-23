interface StatusResponse {
  setup_required: boolean
}

interface CommandResponse {
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

interface CreateUserParams {
  username: string
  password?: string
}

interface AuthResponse {
  token: string
}

let authToken: string | null = localStorage.getItem('uwu-admin-token')

export function setAuthToken(token: string | null) {
  authToken = token
  if (token) {
    localStorage.setItem('uwu-admin-token', token)
  } else {
    localStorage.removeItem('uwu-admin-token')
  }
}

export function loadAuthToken(): string | null {
  authToken = localStorage.getItem('uwu-admin-token')
  return authToken
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

  if (res.status === 401) {
    setAuthToken(null)
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

  listUsers(serverId: number) {
    return request<CommandResponse>(`/api/servers/${serverId}/users`)
  },

  createUser(serverId: number, params: CreateUserParams) {
    return request<CommandResponse>(`/api/servers/${serverId}/users`, {
      method: 'POST',
      body: JSON.stringify(params),
    })
  },

  listRooms(serverId: number) {
    return request<CommandResponse>(`/api/servers/${serverId}/rooms`)
  },

  roomInfo(serverId: number, roomId: string) {
    return request<CommandResponse>(
      `/api/servers/${serverId}/rooms/${encodeURIComponent(roomId)}`,
    )
  },

  serverStatus(serverId: number) {
    return request<CommandResponse>(`/api/servers/${serverId}/server/status`)
  },

  serverUptime(serverId: number) {
    return request<CommandResponse>(`/api/servers/${serverId}/server/uptime`)
  },
}
