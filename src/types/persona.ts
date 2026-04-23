export interface Persona {
  id: number;
  name: string;
  skillName: string;
  category: string;
  description: string;
  isBuiltin: boolean;
  isInstalled: boolean;
  createdAt: string;
}

export interface CreatePersona {
  name: string;
  skillName: string;
  category: string;
  description: string;
  isBuiltin: boolean;
}

export interface TerminalSession {
  id: string;
  noteId: number;
  personaSkill: string;
  mode: 'smart' | 'manual';
  status: 'running' | 'completed' | 'failed';
  createdAt: string;
}

export interface CreateTerminalSession {
  noteId: number;
  personaSkill: string;
  mode: 'smart' | 'manual';
}

export interface StartRewriteParams {
  noteId: number;
  notePath: string;
  personaSkill: string;
  mode: 'smart' | 'manual';
}
