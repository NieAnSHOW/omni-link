import { invoke } from '@tauri-apps/api/core';
import type {
  Persona,
  CreatePersona,
  TerminalSession,
  StartRewriteParams,
} from '../types/persona';

export function usePersona() {
  const scanLocalPersonas = async (): Promise<Persona[]> => {
    return await invoke<Persona[]>('scan_local_personas');
  };

  const getAllPersonas = async (): Promise<Persona[]> => {
    return await invoke<Persona[]>('get_all_personas');
  };

  const getPersonaBySkill = async (skillName: string): Promise<Persona | null> => {
    return await invoke<Persona | null>('get_persona_by_skill', { skillName });
  };

  const savePersona = async (persona: CreatePersona): Promise<number> => {
    return await invoke<number>('save_persona', { persona });
  };

  const deletePersona = async (id: number): Promise<void> => {
    await invoke('delete_persona', { id });
  };

  const startPersonaRewrite = async (params: StartRewriteParams): Promise<string> => {
    return await invoke<string>('start_persona_rewrite', { ...params });
  };

  const updateSessionStatus = async (sessionId: string, status: string): Promise<void> => {
    await invoke('update_session_status', { sessionId, status });
  };

  const closeTerminalSession = async (sessionId: string): Promise<void> => {
    await invoke('close_terminal_session', { sessionId });
  };

  const resizeTerminal = async (sessionId: string, rows: number, cols: number): Promise<void> => {
    await invoke('resize_terminal', { sessionId, rows, cols });
  };

  const getSessionInfo = async (sessionId: string): Promise<TerminalSession | null> => {
    return await invoke<TerminalSession | null>('get_session_info', { sessionId });
  };

  return {
    scanLocalPersonas,
    getAllPersonas,
    getPersonaBySkill,
    savePersona,
    deletePersona,
    startPersonaRewrite,
    updateSessionStatus,
    closeTerminalSession,
    resizeTerminal,
    getSessionInfo,
  };
}
