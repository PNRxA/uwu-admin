import { COMMAND_TREE, type CommandNode } from '@/lib/command-tree'

export interface Suggestion {
  /** The text to insert */
  name: string
  /** Human-readable description */
  description: string
  /** Whether this node has children (i.e. it's a category, not a leaf) */
  hasChildren: boolean
}

export interface ArgHint {
  name: string
  description: string
  required: boolean
}

export interface AutocompleteResult {
  suggestions: Suggestion[]
  argHints: ArgHint[]
}

/**
 * Parse the input into completed tokens and a partial token being typed.
 * A trailing space means the last token is complete.
 */
function parseTokens(input: string): { completed: string[]; partial: string } {
  const trimmedLeft = input.replace(/^\s+/, '')
  if (!trimmedLeft) return { completed: [], partial: '' }

  const parts = trimmedLeft.split(/\s+/).filter(Boolean)
  if (input.endsWith(' ')) {
    return { completed: parts, partial: '' }
  }
  return { completed: parts.slice(0, -1), partial: parts[parts.length - 1] ?? '' }
}

/**
 * Walk the command tree consuming completed tokens, then filter
 * the current level by the partial token.
 */
export function getSuggestions(input: string): AutocompleteResult {
  const { completed, partial } = parseTokens(input)

  let currentLevel: CommandNode[] = COMMAND_TREE
  let currentNode: CommandNode | null = null

  // Walk down the tree consuming completed tokens
  for (const token of completed) {
    const match = currentLevel.find((n) => n.name === token.toLowerCase())
    if (!match) {
      // Unknown token — can't suggest anything
      return { suggestions: [], argHints: [] }
    }
    currentNode = match
    if (match.children) {
      currentLevel = match.children
    } else {
      // Reached a leaf command — show arg hints for remaining positions
      const argHints = getArgHints(match, completed, token)
      return { suggestions: [], argHints }
    }
  }

  // If we reached a leaf node and there's a trailing space (partial is empty),
  // show arg hints
  if (currentNode && !currentNode.children && partial === '') {
    const argHints = getArgHints(currentNode, completed, completed[completed.length - 1] ?? '')
    return { suggestions: [], argHints }
  }

  // Filter current level by the partial token
  const lowerPartial = partial.toLowerCase()
  const suggestions: Suggestion[] = currentLevel
    .filter((n) => n.name.startsWith(lowerPartial))
    .map((n) => ({
      name: n.name,
      description: n.description,
      hasChildren: !!n.children,
    }))

  return { suggestions, argHints: [] }
}

function getArgHints(
  node: CommandNode,
  completed: string[],
  _matchedToken: string,
): ArgHint[] {
  if (!node.args) return []

  // Count how many tokens came after the leaf command
  // Find which token matched this node
  const nodeDepth = findNodeDepth(node.name, completed)
  const argsProvided = completed.length - nodeDepth - 1

  return node.args.slice(argsProvided).map((a) => ({
    name: a.name,
    description: a.description,
    required: a.required,
  }))
}

function findNodeDepth(name: string, completed: string[]): number {
  for (let i = completed.length - 1; i >= 0; i--) {
    if (completed[i]!.toLowerCase() === name) return i
  }
  return completed.length - 1
}

/**
 * Replace the partial last token with the selected suggestion,
 * appending a trailing space so the user can continue typing.
 */
export function applySuggestion(input: string, suggestion: Suggestion): string {
  const { completed } = parseTokens(input)
  return [...completed, suggestion.name].join(' ') + ' '
}
