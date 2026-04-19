export interface Link {
  id: number;
  url: string;
  title: string | null;
  platform: string | null;
  source: string;
  status: 'pending' | 'parsing' | 'parsed' | 'failed';
  created_at: string;
  updated_at: string;
}

export interface Content {
  id: number;
  link_id: number;
  title: string | null;
  body_html: string | null;
  body_text: string | null;
  images: string[];
  metadata: Record<string, unknown>;
  created_at: string;
}

export interface AiResult {
  summary: string | null;
  tags: string[];
  classification: string | null;
  provider: string | null;
}

export interface LinkDetail {
  link: Link;
  content?: Content;
  ai?: AiResult;
}
