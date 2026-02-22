const parser = new DOMParser()

export function stripHtml(html: string): string {
  const doc = parser.parseFromString(html, 'text/html')
  return doc.body.textContent ?? ''
}

export type ParsedResponse =
  | { type: 'table'; header: string | null; columns: string[]; rows: string[][] }
  | { type: 'list'; header: string | null; items: string[] }
  | { type: 'kv'; header: string | null; entries: { key: string; value: string }[] }
  | { type: 'text'; text: string }

const IDENTIFIER_RE = /^[@!#]\S+:\S+$/

export function parseResponse(html: string): ParsedResponse {
  const text = stripHtml(html)
  const rawLines = text.split(/\r?\n/)
  const lines = rawLines.map((l) => l.trim()).filter(Boolean)

  if (lines.length === 0) return { type: 'text', text: text.trim() || '(empty response)' }

  // Detect optional header: "Something (N):" or "Something:" as the first line
  let header: string | null = null
  let dataLines = lines
  if (lines.length >= 2 && /^.+?(?:\s*\(\d+\))?:\s*$/.test(lines[0])) {
    header = lines[0].replace(/:\s*$/, '')
    dataLines = lines.slice(1)
  }

  if (dataLines.length === 0) return { type: 'text', text: text.trim() }

  // Table: lines with labeled fields like "!roomid  Members: N  Name: X"
  if (/^!\S+\s+[A-Z]\w*:/.test(dataLines[0])) {
    // Extract column names using "Key: Value" pairs (values end at next "Key: " or EOL)
    const fieldRe = /([A-Z]\w*):\s+(.*?)(?=\s+[A-Z]\w*:\s|$)/g
    const columns: string[] = ['ID']
    const firstRest = dataLines[0].replace(/^!\S+\s+/, '')
    let m: RegExpExecArray | null
    while ((m = fieldRe.exec(firstRest)) !== null) {
      columns.push(m[1])
    }

    const rows = dataLines
      .map((line) => {
        const idMatch = line.match(/^(!\S+)\s+/)
        if (!idMatch?.[1]) return null
        const rest = line.slice(idMatch[0].length)
        const values: string[] = [idMatch[1]]
        const fp = /([A-Z]\w*):\s+(.*?)(?=\s+[A-Z]\w*:\s|$)/g
        let fm: RegExpExecArray | null
        while ((fm = fp.exec(rest)) !== null) {
          values.push(fm[2].trim())
        }
        return values
      })
      .filter((r): r is string[] => r !== null)

    if (rows.length > 0) return { type: 'table', header, columns, rows }
  }

  // List: all lines are identifiers (@user:server, !room:server, #alias:server)
  if (dataLines.every((l) => IDENTIFIER_RE.test(l))) {
    return { type: 'list', header, items: dataLines }
  }

  // Key-value pairs: all lines match "key: value"
  const kvEntries = dataLines.map((l) => {
    const m = l.match(/^(.+?):\s+(.+)$/)
    if (!m?.[1] || !m[2]) return null
    return { key: m[1].trim(), value: m[2].trim() }
  })
  if (kvEntries.every((e): e is { key: string; value: string } => e !== null)) {
    return { type: 'kv', header, entries: kvEntries as { key: string; value: string }[] }
  }

  // Fallback: plain text
  return { type: 'text', text: text.trim() }
}
