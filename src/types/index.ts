export interface Note {
  id: number;
  title: string;
  file_name: string;
  source_url: string | null;
  created_at: string;
  updated_at: string;
  file_size: number;
  word_count: number;
}

export interface NoteDetail {
  note: Note;
  content: string;
}

export interface FileEntry {
  name: string;
  path: string;
  is_dir: boolean;
  children: FileEntry[];
}

export * from './persona';
