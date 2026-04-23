export interface Persona {
  id: number;
  name: string;
  skill_name: string;
  category: string;
  description: string;
  is_builtin: boolean;
  is_installed: boolean;
  created_at: string;
}

export interface CreatePersona {
  name: string;
  skill_name: string;
  category: string;
  description: string;
  is_builtin: boolean;
}

export interface TerminalSession {
  id: string;
  note_id: number;
  persona_skill: string;
  mode: 'smart' | 'manual';
  status: 'running' | 'completed' | 'failed';
  created_at: string;
}

export interface CreateTerminalSession {
  note_id: number;
  persona_skill: string;
  mode: 'smart' | 'manual';
}

export interface StartRewriteParams {
  noteId: number;
  notePath: string;
  personaSkill: string;
  mode: 'smart' | 'manual';
}
