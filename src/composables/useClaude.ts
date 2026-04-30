import { invoke } from '@tauri-apps/api/core'
import type { ClaudeConfig, ClaudeCliStatus, ClaudeSkillInfo } from '../types/persona'

export function useClaude() {
  const checkInstalled = async (): Promise<ClaudeCliStatus> => {
    return invoke<ClaudeCliStatus>('check_claude_installed')
  }

  const installCli = async (): Promise<ClaudeCliStatus> => {
    return invoke<ClaudeCliStatus>('install_claude_cli')
  }

  const getCliPath = async (): Promise<{ path: string; exists: boolean }> => {
    return invoke<{ path: string; exists: boolean }>('get_claude_cli_path')
  }

  const startSession = async (): Promise<string> => {
    return invoke<string>('start_claude_session')
  }

  const startOutput = async (sessionId: string): Promise<void> => {
    return invoke('start_claude_output', { sessionId })
  }

  const writeInput = async (sessionId: string, data: string): Promise<void> => {
    return invoke('write_claude_input', { sessionId, data })
  }

  const resizeTerminal = async (sessionId: string, rows: number, cols: number): Promise<void> => {
    return invoke('resize_claude_terminal', { sessionId, rows, cols })
  }

  const closeSession = async (sessionId: string): Promise<void> => {
    return invoke('close_claude_session', { sessionId })
  }

  const getConfig = async (): Promise<ClaudeConfig> => {
    return invoke<ClaudeConfig>('get_claude_config')
  }

  const updateConfig = async (params: {
    provider?: string
    apiKey?: string
    baseUrl?: string
    model?: string
  }): Promise<void> => {
    return invoke('update_claude_config', {
      provider: params.provider ?? null,
      apiKey: params.apiKey ?? null,
      baseUrl: params.baseUrl ?? null,
      model: params.model ?? null,
    })
  }

  const listSkills = async (): Promise<ClaudeSkillInfo[]> => {
    return invoke<ClaudeSkillInfo[]>('list_claude_skills')
  }

  const deploySkills = async (): Promise<void> => {
    return invoke('deploy_claude_skills')
  }

  return {
    checkInstalled,
    installCli,
    getCliPath,
    startSession,
    startOutput,
    writeInput,
    resizeTerminal,
    closeSession,
    getConfig,
    updateConfig,
    listSkills,
    deploySkills,
  }
}
