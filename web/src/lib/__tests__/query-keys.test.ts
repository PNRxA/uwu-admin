import { queryKeys } from '../query-keys'

describe('queryKeys', () => {
  it('users returns correct key array', () => {
    expect(queryKeys.users(1)).toEqual(['users', 1])
  })

  it('rooms returns correct key array', () => {
    expect(queryKeys.rooms(42)).toEqual(['rooms', 42])
  })

  it('serverUptime returns correct key array', () => {
    expect(queryKeys.serverUptime(3)).toEqual(['server', 'uptime', 3])
  })

  it('serverStatus returns correct key array', () => {
    expect(queryKeys.serverStatus(5)).toEqual(['server', 'status', 5])
  })

  it('federation returns correct key array', () => {
    expect(queryKeys.federation(7)).toEqual(['federation', 7])
  })
})
