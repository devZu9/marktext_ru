# Russian localization for MarkText

[Русская версия](README.md)

Russian translation of the [MarkText](https://github.com/marktext/marktext) editor interface.

This project was created via vibe-coding in [OpenCode](https://opencode.ai/go?ref=DHSKBMGTK0) using DeepSeek V4 Flash.

## Quick start

```powershell
patch_ru_marktext.exe
```

After restart: **Preferences → General → Language → Русский**

**Rollback:**
```powershell
rollback_marktext.exe
```

## Repository contents

| File | Purpose |
|---|---|
| `patch_ru_marktext.exe` | Patcher: backup → extract `app.asar` → patch JS → repack → copy `ru.min.json` |
| `rollback_marktext.exe` | Rollback: restore `app.asar` from backup, remove `ru.min.json` |
| `ru.json` | Full translation file (source) |
| `ru.min.json` | Production translation (copied to `resources\static\locales\`) |
| `Cargo.toml` / `src/` | Rust source code |

## What the patcher does

1. Creates a backup `app.asar` → `app.asar.backup`
2. Extracts `app.asar` via `@electron/asar`
3. Adds `"ru"` to `SUPPORTED_LANGUAGES` (main process)
4. Adds "Русский" to the language dropdown (renderer)
5. Adds `"russian":"Русский"` to the embedded English dictionary
6. Repacks `app.asar`
7. Copies `ru.min.json` to `resources\static\locales\`

## Requirements

- **Node.js** (for `npx @electron/asar`): https://nodejs.org
- Windows 10/11

## Building from source

```powershell
cargo build --release
copy target\release\patch_ru_marktext.exe .
copy target\release\rollback_marktext.exe .
```

## Known issues and pitfalls

### 1. BOM in JSON files

PowerShell `Set-Content -Encoding UTF8` adds a BOM prefix (`EF BB BF`). Node.js `JSON.parse()` **cannot handle BOM** — it throws a SyntaxError. As a result, `loadTranslations` falls back to English.

**Solution**: Rust `fs::write()` does not add BOM. `serde_json::to_string()` guarantees clean UTF-8 without BOM.

### 2. Internal Muya identifiers

Some translation keys are used as **internal identifiers** by the Muya editor engine, not as display text. Translating them breaks the editor:

| Key | Purpose | Must be |
|---|---|---|
| `editor.fence` | Code block marker | `"fence"` |
| `editor.indent` | Indent marker | `"indent"` |
| `editor.table` | Table identifier | `"table"` |
| `editor.left` | Alignment identifier | `"left"` |
| `editor.center` | Alignment identifier | `"center"` |
| `editor.right` | Alignment identifier | `"right"` |
| `editor.delete` | Delete identifier | `"delete"` |
| `store.editor.fence` | Internal marker | `"fence"` |
| `store.editor.indent` | Internal marker | `"indent"` |
| `store.editor.highlightStart` | Highlight marker | `"[highlight start]"` |
| `store.editor.highlightEnd` | Highlight marker | `"[highlight end]"` |

### 3. Pipe character (|) as vue-i18n plural separator

vue-i18n uses the pipe `|` (U+007C) as a plural separator. If a translation value contains a regular pipe, vue-i18n tries to parse it as plural rules, causing a `SyntaxError`.

**Symptom**: After restarting MarkText with Russian language selected, the file content does not render (empty window, but menus work).

**Solution**: Use the FULLWIDTH vertical bar `|` (U+FF5C) instead of a regular pipe in `quickInsert.tableBlock.subtitle`. This character looks identical but is not recognized by the vue-i18n parser.

### 4. Console encoding

ANSI escape codes (`\x1b[36m`, `\x1b[0m`, etc.) for colored output do not work in the classic Windows console (conhost.exe) and display as garbage (`←[36m`). They are removed from this project.

## For MarkText developers

If you want to include Russian locale in the main MarkText build:

1. Copy `ru.json` to `packages/desktop/static/locales/`
2. Add `'ru'` to `SUPPORTED_LANGUAGES` in `packages/desktop/src/common/i18n.ts`
3. Add Russian language option in `packages/desktop/src/renderer/src/prefComponents/general/config.ts`
4. Add `"russian": "Русский"` to `packages/desktop/static/locales/en.json` under `preferences.general.misc.language`
5. Create a minified copy `ru.min.json` for production builds
6. Make sure all JSON files are saved in UTF-8 **without BOM**

All translation rights belong to the community and can be freely used under the MIT license.

## Report a translation error

If you find an inaccuracy, typo, or error in the translation, please create an issue on GitHub:

https://github.com/mxmSx/marktext_ru/issues/new

## Translation status

All sections are translated: menus, context menus, search, sidebar, command palette, export settings, preferences, dialogs, notifications, editor, quick insert.

## License

MIT
