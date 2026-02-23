export const queryKeys = {
  users: (serverId: number) => ['users', serverId],
  rooms: (serverId: number) => ['rooms', serverId],
  serverUptime: (serverId: number) => ['server', 'uptime', serverId],
  serverStatus: (serverId: number) => ['server', 'status', serverId],
  federation: (serverId: number) => ['federation', serverId],
} as const
