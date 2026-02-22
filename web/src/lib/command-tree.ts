export interface CommandArg {
  name: string
  description: string
  required: boolean
  type: string
}

export interface CommandNode {
  name: string
  description: string
  children?: CommandNode[]
  args?: CommandArg[]
}

import data from '@shared/command-tree.json'

export const COMMAND_TREE: CommandNode[] = data
