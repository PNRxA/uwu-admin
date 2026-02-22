interface ConnectParams {
  homeserver: string
  username: string
  password: string
  room_id: string
}

interface StatusResponse {
  connected: boolean
  homeserver?: string
  user_id?: string
}

interface CommandResponse {
  response: string
}

interface ConnectResponse {
  connected: boolean
  user_id: string
  homeserver: string
}

interface CreateUserParams {
  username: string
  password?: string
}

async function request<T>(url: string, options?: RequestInit): Promise<T> {
  const res = await fetch(url, {
    headers: { 'Content-Type': 'application/json' },
    ...options,
  })
  if (!res.ok) {
    const body = await res.json().catch(() => ({ error: res.statusText }))
    throw new Error(body.error || res.statusText)
  }
  return res.json()
}

export const api = {
  connect(params: ConnectParams) {
    return request<ConnectResponse>('/api/connect', {
      method: 'POST',
      body: JSON.stringify(params),
    })
  },

  disconnect() {
    return request<{ connected: false }>('/api/disconnect', { method: 'POST' })
  },

  status() {
    return request<StatusResponse>('/api/status')
  },

  command(command: string) {
    return request<CommandResponse>('/api/command', {
      method: 'POST',
      body: JSON.stringify({ command }),
    })
  },

  listUsers() {
    return request<CommandResponse>('/api/users')
  },

  createUser(params: CreateUserParams) {
    return request<CommandResponse>('/api/users', {
      method: 'POST',
      body: JSON.stringify(params),
    })
  },

  listRooms() {
    return request<CommandResponse>('/api/rooms')
  },

  roomInfo(roomId: string) {
    return request<CommandResponse>(`/api/rooms/${encodeURIComponent(roomId)}`)
  },

  serverStatus() {
    return request<CommandResponse>('/api/server/status')
  },

  serverUptime() {
    return request<CommandResponse>('/api/server/uptime')
  },
}
