import type { CommandNode } from '@/lib/command-tree'

const MOCK_TREE: CommandNode[] = [
  {
    name: 'admin',
    description: 'Admin commands',
    children: [
      {
        name: 'users',
        description: 'User management',
        children: [
          {
            name: 'list',
            description: 'List all users',
          },
          {
            name: 'deactivate',
            description: 'Deactivate a user',
            args: [
              { name: 'user_id', description: 'The MXID', required: true, type: 'string' },
              { name: 'reason', description: 'Reason for deactivation', required: false, type: 'string' },
            ],
          },
        ],
      },
    ],
  },
  {
    name: 'debug',
    description: 'Debug utilities',
  },
]

vi.mock('@/lib/command-tree', () => ({
  COMMAND_TREE: MOCK_TREE,
}))

// Import after mock so the module picks up the mock tree
const { getSuggestions, applySuggestion, validateCommand } = await import('../useCommandAutocomplete')

describe('getSuggestions', () => {
  it('returns all top-level commands for empty input', () => {
    const { suggestions } = getSuggestions('')
    expect(suggestions).toHaveLength(2)
    expect(suggestions.map((s) => s.name)).toEqual(['admin', 'debug'])
  })

  it('returns top-level suggestions when typing a partial', () => {
    const { suggestions } = getSuggestions('ad')
    expect(suggestions).toHaveLength(1)
    expect(suggestions[0].name).toBe('admin')
  })

  it('filters top-level commands by partial input', () => {
    // Trailing space after nothing meaningful — parseTokens returns completed=[], partial=''
    // But actually with leading space stripped, '' → no suggestions
    const { suggestions } = getSuggestions('a')
    expect(suggestions.map((s) => s.name)).toEqual(['admin'])
  })

  it('returns children after a completed parent token', () => {
    const { suggestions } = getSuggestions('admin ')
    expect(suggestions.map((s) => s.name)).toEqual(['users'])
  })

  it('filters children by partial', () => {
    const { suggestions } = getSuggestions('admin u')
    expect(suggestions).toHaveLength(1)
    expect(suggestions[0].name).toBe('users')
  })

  it('returns grandchildren', () => {
    const { suggestions } = getSuggestions('admin users ')
    expect(suggestions.map((s) => s.name)).toEqual(['deactivate', 'list'])
  })

  it('returns empty for unknown token', () => {
    const { suggestions } = getSuggestions('nonexistent ')
    expect(suggestions).toHaveLength(0)
  })

  it('returns arg hints after a leaf command with trailing space', () => {
    const { argHints } = getSuggestions('admin users deactivate ')
    expect(argHints).toHaveLength(2)
    expect(argHints[0].name).toBe('user_id')
    expect(argHints[0].required).toBe(true)
    expect(argHints[1].name).toBe('reason')
    expect(argHints[1].required).toBe(false)
  })

  it('shows remaining arg hints after first arg provided', () => {
    const { argHints } = getSuggestions('admin users deactivate @user:example.com ')
    expect(argHints).toHaveLength(1)
    expect(argHints[0].name).toBe('reason')
  })

  it('returns no arg hints for leaf with no args', () => {
    const { argHints } = getSuggestions('admin users list ')
    expect(argHints).toHaveLength(0)
  })

  it('marks hasChildren correctly', () => {
    const { suggestions } = getSuggestions('admin users ')
    const deactivate = suggestions.find((s) => s.name === 'deactivate')
    const list = suggestions.find((s) => s.name === 'list')
    expect(deactivate?.hasChildren).toBe(false)
    expect(list?.hasChildren).toBe(false)
  })

  it('marks parent nodes as hasChildren', () => {
    const { suggestions } = getSuggestions('admin ')
    expect(suggestions[0].hasChildren).toBe(true)
  })
})

describe('applySuggestion', () => {
  it('replaces partial with suggestion and appends space', () => {
    const result = applySuggestion('ad', { name: 'admin', description: '', hasChildren: true })
    expect(result).toBe('admin ')
  })

  it('appends suggestion after completed tokens', () => {
    const result = applySuggestion('admin ', { name: 'users', description: '', hasChildren: true })
    expect(result).toBe('admin users ')
  })

  it('replaces partial in the middle of typing', () => {
    const result = applySuggestion('admin us', { name: 'users', description: '', hasChildren: true })
    expect(result).toBe('admin users ')
  })
})

describe('validateCommand', () => {
  it('returns error for empty command', () => {
    expect(validateCommand('')).toEqual({ valid: false, error: 'Empty command' })
  })

  it('returns error for incomplete command (category only)', () => {
    expect(validateCommand('admin')).toEqual({ valid: false, error: 'Incomplete command' })
  })

  it('returns error for unknown command', () => {
    expect(validateCommand('bogus')).toEqual({ valid: false, error: 'Unknown command: bogus' })
  })

  it('returns valid for leaf command with no required args', () => {
    expect(validateCommand('admin users list')).toEqual({ valid: true })
  })

  it('returns error when required args are missing', () => {
    const result = validateCommand('admin users deactivate')
    expect(result.valid).toBe(false)
    expect(result.error).toContain('user_id')
  })

  it('returns valid when required args are provided', () => {
    expect(validateCommand('admin users deactivate @user:example.com')).toEqual({ valid: true })
  })

  it('returns valid when optional args are also provided', () => {
    expect(validateCommand('admin users deactivate @user:example.com spam')).toEqual({ valid: true })
  })
})
