# Русская локализация для MarkText

[English version](README_EN.md)

Русский перевод интерфейса редактора [MarkText](https://github.com/marktext/marktext).

Проект сделан методом вайб-кодинга в оболочке [OpenCode](https://opencode.ai/go?ref=DHSKBMGTK0) при помощи модели DeepSeek V4 Flash.

## Быстрый старт

```powershell
patch_ru_marktext.exe
```

После перезапуска: **Настройки → Общие → Язык → Русский**

**Откат:**
```powershell
rollback_marktext.exe
```

## Состав репозитория

| Файл | Назначение |
|---|---|
| `patch_ru_marktext.exe` | Патчер: бэкап → распаковка `app.asar` → патч JS → упаковка → копирование `ru.min.json` |
| `rollback_marktext.exe` | Откат: восстановление `app.asar` из бэкапа, удаление `ru.min.json` |
| `ru.json` | Полный файл перевода (исходный) |
| `ru.min.json` | Перевод для production (копируется в `resources\static\locales\`) |
| `Cargo.toml` / `src/` | Исходный код Rust |

## Что делает патчер

1. Создаёт резервную копию `app.asar` → `app.asar.backup`
2. Распаковывает `app.asar` через `@electron/asar`
3. Добавляет `"ru"` в `SUPPORTED_LANGUAGES` (основной процесс)
4. Добавляет пункт «Русский» в выпадающий список языков (рендерер)
5. Добавляет метку `"russian":"Русский"` во встроенный английский словарь
6. Перепаковывает `app.asar`
7. Копирует `ru.min.json` в `resources\static\locales\`

## Требования

- **Node.js** (для `npx @electron/asar`), скачать: https://nodejs.org
- Windows 10/11

## Сборка из исходников

```powershell
cargo build --release
copy target\release\patch_ru_marktext.exe .
copy target\release\rollback_marktext.exe .
```

## Особенности и подводные камни

При разработке были выявлены следующие нюансы, которые важно учитывать при работе с локализацией MarkText:

### 1. BOM (Byte Order Mark) в JSON-файлах

PowerShell `Set-Content -Encoding UTF8` добавляет BOM (`EF BB BF`) в начало файла. Node.js `JSON.parse()` **не умеет работать с BOM** — он вызывает SyntaxError. В результате `loadTranslations` падает в catch-блок и загружает английский fallback.

**Решение**: Rust `fs::write()` не добавляет BOM. При генерации `ru.min.json` используется `serde_json::to_string()`, что гарантирует чистый UTF-8 без BOM.

### 2. Внутренние идентификаторы Muya

Некоторые ключи переводов используются не как текст для отображения, а как **внутренние идентификаторы** движка редактора Muya. Их перевод ломает работу редактора:

| Ключ | Описание | Должен быть |
|---|---|---|
| `editor.fence` | Маркер блока кода | `"fence"` |
| `editor.indent` | Маркер отступа | `"indent"` |
| `editor.table` | Идентификатор таблицы | `"table"` |
| `editor.left` | Идентификатор выравнивания | `"left"` |
| `editor.center` | Идентификатор выравнивания | `"center"` |
| `editor.right` | Идентификатор выравнивания | `"right"` |
| `editor.delete` | Идентификатор удаления | `"delete"` |
| `store.editor.fence` | Внутренний маркер | `"fence"` |
| `store.editor.indent` | Внутренний маркер | `"indent"` |
| `store.editor.highlightStart` | Маркер подсветки | `"[highlight start]"` |
| `store.editor.highlightEnd` | Маркер подсветки | `"[highlight end]"` |

### 3. Пайп (|) как разделитель множественных форм в vue-i18n

vue-i18n использует символ `|` (U+007C) для разделения множественных форм. Если значение перевода содержит обычный пайп, vue-i18n пытается распарсить его как plural-правило, что вызывает `SyntaxError`.

**Проявление**: При перезапуске MarkText с русским языком не отображается содержимое файла. Окно пустое, хотя меню работают.

**Решение**: Использовать полноширинную вертикальную черту `｜` (U+FF5C) вместо обычного пайпа в `quickInsert.tableBlock.subtitle`. Этот символ визуально идентичен, но не распознаётся парсером vue-i18n.

### 4. Кодировка консоли

ANSI-escape коды (`\x1b[36m`, `\x1b[0m` и т.д.) для цветного вывода не работают в классической консоли Windows (conhost.exe) и отображаются как мусор (`←[36m`). В проекте они полностью удалены.

## Для разработчиков MarkText

Если вы разработчик MarkText и хотите включить русскую локаль в основную сборку:

1. Скопируйте `ru.json` в `packages/desktop/static/locales/`
2. Добавьте `'ru'` в массив `SUPPORTED_LANGUAGES` в `packages/desktop/src/common/i18n.ts`
3. Добавьте опцию русского языка в `packages/desktop/src/renderer/src/prefComponents/general/config.ts`
4. Добавьте `"russian": "Русский"` в `packages/desktop/static/locales/en.json` в секцию `preferences.general.misc.language`
5. Создайте минифицированную копию `ru.min.json` для production-сборки
6. Убедитесь, что все JSON-файлы сохранены в UTF-8 **без BOM**

Все права на перевод принадлежат сообществу и могут быть свободно использованы в любых целях согласно лицензии MIT.

## Сообщить об ошибке в переводе

Если вы нашли неточность, опечатку или ошибку в переводе — создайте тему на GitHub:

https://github.com/devZu9/marktext_ru/issues/new

## Статус перевода

Переведены все разделы: меню, контекстные меню, поиск, боковая панель, палитра команд, настройки экспорта, параметры приложения, диалоги, уведомления, редактор, быстрая вставка элементов.

## Лицензия

MIT
