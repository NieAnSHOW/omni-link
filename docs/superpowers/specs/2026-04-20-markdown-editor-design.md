# Markdown Editor for ContentView

**Date:** 2026-04-20
**Status:** Draft

## Goal

Allow users to edit the parsed content (title + body) in ContentView.vue using a Markdown editor, then save changes back to the database.

## Design Decisions

- **Editor library:** MdEditorV3 — purpose-built for Vue 3, toolbar + preview built-in, Chinese docs
- **Component structure:** Extract `ContentEditor.vue` as a child component, `ContentView.vue` toggles between read/edit modes
- **Editable fields:** Title (`string`) and body text (`body_text` — markdown source)
- **Edit mode toggle:** Click "编辑" to enter edit, "保存" to persist and return to read-only, "取消" to discard

## Architecture

```
ContentView.vue
  ├── Read-only view (existing, conditional on !isEditing)
  │     ├── Title display
  │     └── Rendered markdown (v-html via marked)
  └── ContentEditor.vue (conditional on isEditing)
        ├── Title input
        ├── MdEditorV3 for body_text
        └── Save / Cancel buttons
```

## Interaction Flow

1. User clicks "编辑" button in ContentView header actions
2. `isEditing = true`, read-only content hides, ContentEditor shows
3. ContentEditor receives `initialTitle` and `initialBody` as props, manages local editing state
4. User edits title and body in the editor
5. "保存" → ContentEditor emits `save(title, bodyText)` → parent calls API → refresh detail → `isEditing = false`
6. "取消" → parent sets `isEditing = false` without saving

## Backend Changes

### 1. Migration: add `updated_at` to contents table

File: `src-tauri/src/db/schema.rs`

```sql
ALTER TABLE contents ADD COLUMN updated_at TEXT;
```

Add migration in the existing `migrate()` function.

### 2. Model: `UpdateContentInput`

File: `src-tauri/src/models.rs`

```rust
#[derive(Deserialize)]
pub struct UpdateContentInput {
    pub id: i64,
    pub title: Option<String>,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
}
```

### 3. Repository: `update_content`

File: `src-tauri/src/repositories/content_repo.rs`

```rust
pub fn update_content(conn: &Connection, input: &UpdateContentInput) -> Result<()> {
    conn.execute(
        "UPDATE contents SET title = ?, body_text = ?, body_html = ?, updated_at = datetime('now') WHERE id = ?",
        params![input.title, input.body_text, input.body_html, input.id],
    )?;
    Ok(())
}
```

### 4. Command: `update_content_cmd`

File: `src-tauri/src/commands/content_commands.rs`

```rust
#[tauri::command]
pub fn update_content_cmd(state: State<AppState>, input: UpdateContentInput) -> Result<bool, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    content_repo::update_content(&conn, &input).map_err(|e| e.to_string())?;
    Ok(true)
}
```

### 5. Register command

File: `src-tauri/src/lib.rs`

Add `commands::content_commands::update_content_cmd` to `tauri::generate_handler![]`.

## Frontend Changes

### 1. New dependency

```bash
npm install md-editor-v3
```

### 2. API method

File: `src/composables/useApi.ts`

```typescript
updateContent: (id: number, title: string | null, bodyText: string) =>
  invoke<boolean>('update_content_cmd', {
    input: { id, title, body_text: bodyText, body_html: null },
  }),
```

### 3. New component: `ContentEditor.vue`

File: `src/components/ContentEditor.vue`

Props:
- `initialTitle: string | null`
- `initialBody: string`

Emits:
- `save(title: string | null, bodyText: string)`
- `cancel()`

Contains:
- Title `<input>` bound to local `editTitle`
- MdEditorV3 bound to local `editBody`
- "保存" and "取消" buttons

### 4. Modify `ContentView.vue`

- Import `ContentEditor`
- Add `isEditing` ref
- Add "编辑" button in `.actions` (shown when `detail.content` exists and `!isEditing`)
- Conditionally render ContentEditor vs read-only content
- Handle `save` event: call `api.updateContent()` → refresh → exit edit mode
- Handle `cancel` event: set `isEditing = false`

### 5. Body HTML regeneration

When saving, the frontend passes `body_html: null` to the backend. The backend will keep the existing `body_html` unchanged (since the SQL only updates non-null fields, we'll adjust the query to use COALESCE or handle it differently).

**Revised approach:** Generate `body_html` from `body_text` on the frontend using `marked()` before sending to backend. This way both fields stay in sync.

```typescript
// In ContentView.vue save handler
const html = marked(bodyText);
await api.updateContent(contentId, title, bodyText, html);
```

Update `useApi.ts` accordingly:

```typescript
updateContent: (id: number, title: string | null, bodyText: string, bodyHtml: string) =>
  invoke<boolean>('update_content_cmd', {
    input: { id, title, body_text: bodyText, body_html: bodyHtml },
  }),
```

## Scope

- Only affects ContentView and its new child component
- No changes to link list, tags, categories, or settings
- No AI re-analysis triggered on manual edit (user can re-run AI analysis separately)
