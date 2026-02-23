import { sanitizeHtml } from '../sanitize'

describe('sanitizeHtml', () => {
  it('allows basic formatting tags', () => {
    const input = '<b>bold</b> <i>italic</i> <em>em</em> <strong>strong</strong>'
    expect(sanitizeHtml(input)).toBe(input)
  })

  it('allows links with href', () => {
    const input = '<a href="https://example.com">link</a>'
    expect(sanitizeHtml(input)).toBe(input)
  })

  it('allows table markup', () => {
    const input = '<table><thead><tr><th>H</th></tr></thead><tbody><tr><td>D</td></tr></tbody></table>'
    expect(sanitizeHtml(input)).toBe(input)
  })

  it('strips disallowed tags', () => {
    expect(sanitizeHtml('<img src="x">')).toBe('')
    expect(sanitizeHtml('<iframe src="x"></iframe>')).toBe('')
  })

  it('strips disallowed attributes', () => {
    const result = sanitizeHtml('<b onclick="alert(1)">text</b>')
    expect(result).toBe('<b>text</b>')
  })

  it('blocks script injection', () => {
    expect(sanitizeHtml('<script>alert(1)</script>')).toBe('')
    expect(sanitizeHtml('<b onmouseover="alert(1)">hover</b>')).toBe('<b>hover</b>')
  })
})
