import { stripHtml, parseResponse } from '../response-parser'
import type { ParsedResponse } from '../response-parser'

function assertType<T extends ParsedResponse['type']>(
  r: ParsedResponse,
  type: T,
): asserts r is Extract<ParsedResponse, { type: T }> {
  expect(r.type).toBe(type)
}

describe('stripHtml', () => {
  it('strips HTML tags and returns text content', () => {
    expect(stripHtml('<b>hello</b> <i>world</i>')).toBe('hello world')
  })

  it('returns empty string for empty input', () => {
    expect(stripHtml('')).toBe('')
  })

  it('returns plain text unchanged', () => {
    expect(stripHtml('no tags here')).toBe('no tags here')
  })

  it('handles nested tags', () => {
    expect(stripHtml('<div><p>nested <b>bold</b></p></div>')).toBe('nested bold')
  })
})

describe('parseResponse', () => {
  describe('empty / text fallback', () => {
    it('returns "(empty response)" for empty string', () => {
      const r = parseResponse('')
      expect(r).toEqual({ type: 'text', text: '(empty response)' })
    })

    it('returns "(empty response)" for whitespace-only HTML', () => {
      const r = parseResponse('<p>   </p>')
      expect(r).toEqual({ type: 'text', text: '(empty response)' })
    })

    it('returns text type for plain text', () => {
      const r = parseResponse('Just a plain message')
      expect(r).toEqual({ type: 'text', text: 'Just a plain message' })
    })
  })

  describe('header detection', () => {
    it('detects "Something:" header', () => {
      const r = parseResponse('Users:\n@alice:example.com\n@bob:example.com')
      assertType(r, 'list')
      expect(r.header).toBe('Users')
    })

    it('detects "Something (N):" header', () => {
      const r = parseResponse('Users (2):\n@alice:example.com\n@bob:example.com')
      assertType(r, 'list')
      expect(r.header).toBe('Users (2)')
    })

    it('does not treat single line as header', () => {
      const r = parseResponse('@alice:example.com')
      assertType(r, 'list')
      expect(r.header).toBeNull()
    })
  })

  describe('table branch', () => {
    it('parses room-list style lines into a table', () => {
      const input = [
        'Rooms (2):',
        '!abc:example.com Members: 5 Name: General',
        '!def:example.com Members: 3 Name: Random',
      ].join('\n')
      const r = parseResponse(input)
      assertType(r, 'table')
      expect(r.header).toBe('Rooms (2)')
      expect(r.columns).toEqual(['ID', 'Members', 'Name'])
      expect(r.rows).toHaveLength(2)
      expect(r.rows[0]).toEqual(['!abc:example.com', '5', 'General'])
      expect(r.rows[1]).toEqual(['!def:example.com', '3', 'Random'])
    })
  })

  describe('list branch', () => {
    it('parses identifier-only lines as a list', () => {
      const r = parseResponse('@alice:example.com\n@bob:example.com\n#room:example.com')
      assertType(r, 'list')
      expect(r.items).toEqual([
        '@alice:example.com',
        '@bob:example.com',
        '#room:example.com',
      ])
    })

    it('recognises room IDs as identifiers', () => {
      const r = parseResponse('!room123:example.com')
      assertType(r, 'list')
      expect(r.items).toEqual(['!room123:example.com'])
    })
  })

  describe('kv branch', () => {
    it('parses "key: value" lines as kv entries', () => {
      const r = parseResponse('Name: General\nMembers: 42\nTopic: Welcome')
      assertType(r, 'kv')
      expect(r.entries).toEqual([
        { key: 'Name', value: 'General' },
        { key: 'Members', value: '42' },
        { key: 'Topic', value: 'Welcome' },
      ])
    })

    it('detects kv with a header', () => {
      const r = parseResponse('Server Info:\nVersion: 1.0\nUptime: 3d')
      assertType(r, 'kv')
      expect(r.header).toBe('Server Info')
      expect(r.entries).toEqual([
        { key: 'Version', value: '1.0' },
        { key: 'Uptime', value: '3d' },
      ])
    })
  })
})
