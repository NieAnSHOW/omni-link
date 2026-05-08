import { invoke } from '@tauri-apps/api/core'
import type { AgentConfig, PiCliStatus, AgentSkillInfo } from '../types/persona'

export function usePi() {
  const checkInstalled = async (): Promise<PiCliStatus> => {
    return invoke<PiCliStatus>('check_pi_installed')
  }

  const installPi = async (): Promise<PiCliStatus> => {
    return invoke<PiCliStatus>('install_pi')
  }

  const startSession = async (): Promise<string> => {
    return invoke<string>('start_pi_session')
  }

  const startPrintSession = async (prompt: string): Promise<string> => {
    return invoke<string>('start_persona_rewrite', {
      noteId: null,
      notePath: '',
      personaSkill: prompt,
      mode: 'manual',
    })
  }

  const startOutput = async (sessionId: string): Promise<void> => {
    return invoke('start_pi_output', { sessionId })
  }

  const writeInput = async (sessionId: string, data: string): Promise<void> => {
    return invoke('write_pi_input', { sessionId, data })
  }

  const resizeTerminal = async (sessionId: string, rows: number, cols: number): Promise<void> => {
    return invoke('resize_pi_terminal', { sessionId, rows, cols })
  }

  const closeSession = async (sessionId: string): Promise<void> => {
    return invoke('close_pi_session', { sessionId })
  }

  const getConfig = async (): Promise<AgentConfig> => {
    return invoke<AgentConfig>('get_agent_config')
  }

  const updateConfig = async (params: {
    provider?: string
    apiKey?: string
    baseUrl?: string
    model?: string
  }): Promise<void> => {
    return invoke('update_agent_config', {
      provider: params.provider ?? null,
      apiKey: params.apiKey ?? null,
      baseUrl: params.baseUrl ?? null,
      model: params.model ?? null,
    })
  }

  const listSkills = async (): Promise<AgentSkillInfo[]> => {
    return invoke<AgentSkillInfo[]>('list_pi_skills')
  }

  const deploySkills = async (): Promise<void> => {
    return invoke('deploy_pi_skills')
  }

  return {
    checkInstalled,
    installPi,
    startSession,
    startPrintSession,
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
