import { invoke } from '@tauri-apps/api/core';
import type { FileEntry } from '../types/index';

export const workspaceApi = {
  async setWorkspace(path: string): Promise<void> {
    return invoke('set_workspace', { path });
  },

  async getWorkspace(): Promise<string | null> {
    return invoke('get_workspace');
  },

  async listFiles(dir: string): Promise<FileEntry[]> {
    return invoke('list_files', { dir });
  },

  async readFile(path: string): Promise<string> {
    return invoke('read_file', { path });
  },

  async writeFile(path: string, content: string): Promise<void> {
    return invoke('write_file', { path, content });
  },
};
