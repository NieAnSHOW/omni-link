import { defineStore } from 'pinia';
import { ref } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { workspaceApi } from '../composables/useWorkspaceApi';
import type { FileEntry } from '../types/index';

export const useWorkspaceStore = defineStore('workspace', () => {
  const workspacePath = ref<string | null>(null);
  const fileTree = ref<FileEntry[]>([]);
  const currentFilePath = ref<string | null>(null);
  const currentContent = ref<string>('');
  const loadingTree = ref(false);
  const loadingFile = ref(false);
  const error = ref<string | null>(null);

  async function initWorkspace() {
    try {
      const path = await workspaceApi.getWorkspace();
      if (path) {
        workspacePath.value = path;
        await loadFileTree();
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : '初始化工作区失败';
    }
  }

  async function selectWorkspace() {
    loadingTree.value = true;
    error.value = null;
    try {
      const selected = await open({ directory: true, multiple: false });
      if (!selected) return;

      const path = selected as string;
      await workspaceApi.setWorkspace(path);
      workspacePath.value = path;
      await loadFileTree();
    } catch (e) {
      error.value = e instanceof Error ? e.message : '选择工作区失败';
      throw e;
    } finally {
      loadingTree.value = false;
    }
  }

  async function loadFileTree() {
    if (!workspacePath.value) return;
    loadingTree.value = true;
    error.value = null;
    try {
      fileTree.value = await workspaceApi.listFiles(workspacePath.value);
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载文件树失败';
      throw e;
    } finally {
      loadingTree.value = false;
    }
  }

  async function openFile(path: string) {
    loadingFile.value = true;
    error.value = null;
    try {
      currentContent.value = await workspaceApi.readFile(path);
      currentFilePath.value = path;
    } catch (e) {
      error.value = e instanceof Error ? e.message : '打开文件失败';
      throw e;
    } finally {
      loadingFile.value = false;
    }
  }

  async function saveCurrentFile() {
    if (!currentFilePath.value) return;
    try {
      await workspaceApi.writeFile(currentFilePath.value, currentContent.value);
    } catch (e) {
      error.value = e instanceof Error ? e.message : '保存文件失败';
      console.error('Save failed:', e);
    }
  }

  async function reloadCurrentFile() {
    if (!currentFilePath.value) return;
    try {
      currentContent.value = await workspaceApi.readFile(currentFilePath.value);
    } catch (e) {
      error.value = e instanceof Error ? e.message : '重新加载文件失败';
      console.error('Reload failed:', e);
    }
  }

  function closeWorkspace() {
    workspacePath.value = null;
    fileTree.value = [];
    currentFilePath.value = null;
    currentContent.value = '';
  }

  return {
    workspacePath,
    fileTree,
    currentFilePath,
    currentContent,
    loadingTree,
    loadingFile,
    error,
    initWorkspace,
    selectWorkspace,
    loadFileTree,
    openFile,
    saveCurrentFile,
    reloadCurrentFile,
    closeWorkspace,
  };
});
