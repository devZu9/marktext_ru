# AGENTS.md — инструкции для ИИ-ассистентов

## Правила работы

- **Пуш на GitHub только после явного подтверждения пользователя.** Никогда не пушить без разрешения.
- Все изменения согласовывать с пользователем перед выполнением.
- Код должен компилироваться без ошибок и предупреждений.

## Проект: Русская локализация для MarkText

Репозиторий содержит патчер на Rust для добавления русского языка в редактор MarkText.

### Структура

```
C:\_dev\marktext_ru\
├── src/
│   ├── lib.rs        # Общая логика: asar, патчинг, утилиты
│   ├── patch.rs      # Бинарник patch_ru_marktext
│   └── rollback.rs   # Бинарник rollback_marktext
├── ru.json           # Перевод (исходный, читаемый)
├── ru.min.json       # Перевод (минифицированный, для production)
├── patch_ru_marktext.exe
├── rollback_marktext.exe
├── Cargo.toml
└── README.md
```

## Нюансы, которые были выявлены при разработке

### 1. BOM (Byte Order Mark) в JSON

**Проблема**: PowerShell `Set-Content -Encoding UTF8` и `Out-File` добавляют BOM (`EF BB BF`) в начало файла.
Node.js `JSON.parse()` НЕ переваривает BOM — SyntaxError, fallback на английский.

**Симптомы**: 
- Выбор русского в настройках ничего не меняет (язык остаётся английским)
- В логах: `Error loading translation: ...`

**Решение в коде**: Всегда использовать `fs::write()` в Rust (не добавляет BOM) или
`[System.IO.File]::WriteAllText()` в PowerShell с `[System.Text.UTF8Encoding]::new($false)`.

### 2. Внутренние идентификаторы Muya

**Проблема**: Ключи `editor.fence`, `editor.indent`, `editor.table`, `editor.left`,
`editor.center`, `editor.right`, `editor.delete`, `store.editor.fence`,
`store.editor.indent`, `store.editor.highlightStart`, `store.editor.highlightEnd`
используются как внутренние идентификаторы движка Muya, а не как текст для отображения.
Их перевод ломает логику редактора.

**Симптомы**: Пустое окно редактора после перезапуска с русским языком (меню работают,
но содержимое файла не отображается).

**Решение**: Эти ключи должны оставаться на английском. В `ru.json` они уже исправлены.

### 3. Fullwidth pipe в quickInsert.tableBlock.subtitle

**Проблема**: vue-i18n использует обычный пайп `|` (U+007C) как разделитель
множественных форм. Если значение перевода содержит `|`, vue-i18n пытается
распарсить его как plural-правило → SyntaxError. Английская и немецкая локали
используют полноширинную черту `|` (U+FF5C), которая выглядит так же, но не
распознаётся парсером.

**Решение**: В `ru.json` `quickInsert.tableBlock.subtitle` использует `\uFF5C`
вместо обычного `|`.

### 4. ANSI-escape коды в консоли Windows

**Проблема**: Цветной вывод через `\x1b[36m` не работает в классической консоли
Windows (conhost.exe) — отображается как `←[36m`. Работает только в Windows Terminal.

**Решение**: Не использовать ANSI-коды. Весь вывод — plain text.

### 5. GitHub release body encoding

**Проблема**: PowerShell `Invoke-RestMethod` кодирует тело запроса в системную
кодировку (Windows-1251), а не в UTF-8. Из-за этого кириллица в release notes
на GitHub отображается кракозябрами (`??????`).

**Решение**: Для публикации тела релиза с кириллицей использовать `gh` CLI
через `cmd.exe`:
```powershell
# Записать тело в файл (чистый UTF-8 без BOM).
# Использовать команду write (Write tool), а не PowerShell Set-Content.

# Обновить через cmd.exe, минуя PowerShell:
cmd /c "gh release edit v1.0 --repo user/repo --notes-file C:\path\to\body.md"
```

### 6. Поиск npx на Windows

**Проблема**: `Command::new("npx")` в Rust не всегда находит `npx` на Windows,
потому что `npx` может быть `.cmd` или `.ps1` файлом.

**Решение**: Использовать `cmd /c npx ...` на Windows:
```rust
if cfg!(windows) {
    Command::new("cmd").args(["/c", "npx", ...])
}
```

### 6. Позиции патча в app.asar

Файлы внутри `app.asar`, которые都需要 модифицировать:

1. **`out/main/index.js`**: `SUPPORTED_LANGUAGES = [...]`
   - Найти строку с массивом и добавить `"ru"`
   
2. **`out/renderer/assets/index-*.js`**: 
   - `const getLanguageOptions = () => [...]` — добавить русский пункт
   - `const preferences$1 = JSON.parse(...)` — добавить `"russian":"Русский"`
   - `const editor = { ... }` — НЕ переводить `fence`, `indent`, `table`, `left`,
     `center`, `right`, `delete`

### 7. Размер бинарников

- `patch_ru_marktext.exe` в релизе ~280KB
- `rollback_marktext.exe` в релизе ~150KB
- Зависимость: `serde_json` (~70KB в релизе)

### 8. Тестирование

Для теста без переустановки MarkText:
1. Сделать бэкап `app.asar`
2. Запустить патчер
3. Проверить: `npx @electron/asar extract app.asar tmp\`
4. Проверить изменения в `tmp\out\`
5. Запустить `marktext.exe --no-sandbox --enable-logging`
6. Проверить в логах `SyntaxError` или `Error loading translation`

При возникновении SyntaxError — искать проблему в `ru.json` методом бинарного поиска
(замена отдельных секций на немецкие/английские).
