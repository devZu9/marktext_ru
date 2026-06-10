use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// ──────────────────────────────────────────────
// Error type
// ──────────────────────────────────────────────

#[derive(Debug)]
pub enum Error {
    MarkTextNotFound(String),
    NpxNotFound,
    AsarFailed(String),
    PatchFailed(String),
    Io(std::io::Error),
    InvalidJson(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MarkTextNotFound(p) => write!(f, "MarkText не найден по пути: {}", p),
            Error::NpxNotFound => write!(f, "Node.js (npx) не найден в PATH. Установите Node.js: https://nodejs.org"),
            Error::AsarFailed(msg) => write!(f, "Ошибка asar: {}", msg),
            Error::PatchFailed(msg) => write!(f, "Ошибка патча: {}", msg),
            Error::Io(e) => write!(f, "Ошибка ввода/вывода: {}", e),
            Error::InvalidJson(msg) => write!(f, "Ошибка JSON: {}", msg),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self { Error::Io(e) }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self { Error::InvalidJson(e.to_string()) }
}

pub type Result<T> = std::result::Result<T, Error>;

// ──────────────────────────────────────────────
// Console helpers (по-русски)
// ──────────────────────────────────────────────

pub fn step(msg: &str) {
    println!(">>> {}", msg);
}

pub fn success(msg: &str) {
    println!(">>> {}", msg);
}

pub fn error(msg: &str) {
    eprintln!(">>> ОШИБКА: {}", msg);
}

/// Ждать нажатия клавиши перед выходом
pub fn press_any_key() {
    use std::io::Read;
    println!();
    println!("Нажмите Enter, чтобы закрыть это окно...");
    std::io::stdin().read(&mut [0u8]).ok();
}

// ──────────────────────────────────────────────
// Path utilities
// ──────────────────────────────────────────────

pub const MARKTEXT_DIR_LOCAL: &str = r"C:\Users\mxm\AppData\Local\Programs\marktext";
pub const MARKTEXT_DIR_PROGRAM: &str = r"C:\Program Files\marktext";

pub struct MarkTextPaths {
    pub root: PathBuf,
    pub asar: PathBuf,
    pub asar_backup: PathBuf,
    pub locales_dir: PathBuf,
}

impl MarkTextPaths {
    pub fn from_root(root: PathBuf) -> Self {
        let asar = root.join("resources").join("app.asar");
        let asar_backup = root.join("resources").join("app.asar.backup");
        let locales_dir = root.join("resources").join("static").join("locales");
        MarkTextPaths { root, asar, asar_backup, locales_dir }
    }

    pub fn detect() -> Result<Self> {
        for dir in &[MARKTEXT_DIR_LOCAL, MARKTEXT_DIR_PROGRAM] {
            let p = PathBuf::from(dir);
            if p.join("marktext.exe").exists() {
                return Ok(MarkTextPaths::from_root(p));
            }
        }
        Err(Error::MarkTextNotFound(
            "проверены: %LOCALAPPDATA%\\Programs\\marktext и C:\\Program Files\\marktext".into()
        ))
    }
}

pub fn find_source_json(exe_dir: &Path) -> Result<PathBuf> {
    let candidates = [
        exe_dir.join("ru.json"),
        exe_dir.join("..").join("ru.json"),
        exe_dir.join("ru.min.json"),
        exe_dir.join("..").join("ru.min.json"),
    ];
    for c in &candidates {
        let canon = fs::canonicalize(c).ok();
        if let Some(p) = canon {
            if p.exists() { return Ok(p); }
        }
    }
    Err(Error::Io(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!("ru.json или ru.min.json не найден рядом с {:?}", exe_dir),
    )))
}

// ──────────────────────────────────────────────
// Read JSON, strip BOM, validate, return minified bytes
// ──────────────────────────────────────────────

pub fn read_and_minify_json(path: &Path) -> Result<Vec<u8>> {
    let raw = fs::read(path)?;
    let without_bom = if raw.len() >= 3 && raw[0] == 0xEF && raw[1] == 0xBB && raw[2] == 0xBF {
        &raw[3..]
    } else {
        &raw[..]
    };
    let s = std::str::from_utf8(without_bom)
        .map_err(|_| Error::InvalidJson("файл не в UTF-8".into()))?;
    let value: serde_json::Value = serde_json::from_str(s)?;
    let minified = serde_json::to_string(&value)?;
    Ok(minified.into_bytes())
}

// ──────────────────────────────────────────────
// asar operations via npx
// ──────────────────────────────────────────────

pub fn check_npx() -> bool {
    let status = if cfg!(windows) {
        Command::new("cmd")
            .args(["/c", "npx", "--version"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
    } else {
        Command::new("npx")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
    };
    status.is_ok()
}

fn run_npx(args: &[&str]) -> Result<()> {
    let status = if cfg!(windows) {
        Command::new("cmd")
            .args(["/c", "npx", "--yes", "@electron/asar"])
            .args(args)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .status()
    } else {
        Command::new("npx")
            .args(["--yes", "@electron/asar"])
            .args(args)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .status()
    };
    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(_) => Err(Error::AsarFailed("команда завершилась с ошибкой".into())),
        Err(e) => Err(Error::AsarFailed(format!("не удалось запустить npx: {}", e))),
    }
}

pub fn extract_asar(asar_path: &Path, dest: &Path) -> Result<()> {
    step("Распаковка app.asar...");
    run_npx(&["extract", &asar_path.to_string_lossy(), &dest.to_string_lossy()])?;
    success("Распаковано");
    Ok(())
}

pub fn pack_asar(src: &Path, asar_path: &Path) -> Result<()> {
    step("Упаковка app.asar обратно...");
    run_npx(&["pack", &src.to_string_lossy(), &asar_path.to_string_lossy()])?;
    success("Упаковано");
    Ok(())
}

pub fn backup_asar(asar: &Path, backup: &Path) -> Result<()> {
    if backup.exists() {
        step("Резервная копия уже существует, пропускаем");
        return Ok(());
    }
    step("Создание резервной копии app.asar...");
    fs::copy(asar, backup)?;
    success("Резервная копия создана");
    Ok(())
}

pub fn restore_backup(asar: &Path, backup: &Path) -> Result<()> {
    if !backup.exists() {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("резервная копия не найдена: {:?}", backup),
        )));
    }
    step("Восстановление app.asar из резервной копии...");
    fs::copy(backup, asar)?;
    success("Восстановлено");
    Ok(())
}

/// Прочитать ru.json/ru.min.json, минифицировать, записать без BOM
pub fn deploy_locale(src: &Path, dest_dir: &Path) -> Result<()> {
    step("Установка русского файла локализации...");
    fs::create_dir_all(dest_dir)?;
    let minified = read_and_minify_json(src)?;
    let dest = dest_dir.join("ru.min.json");
    fs::write(&dest, minified)?;
    success("Файл локализации установлен");
    Ok(())
}

pub fn remove_locale(dest_dir: &Path) -> Result<()> {
    let dest = dest_dir.join("ru.min.json");
    if dest.exists() {
        fs::remove_file(&dest)?;
        success("ru.min.json удалён");
    } else {
        step("ru.min.json не найден, удалять нечего");
    }
    Ok(())
}

// ──────────────────────────────────────────────
// JS patching
// ──────────────────────────────────────────────

pub fn patch_main_js(content: &str) -> Result<String> {
    let needle = r#"SUPPORTED_LANGUAGES = ["en", "zh-CN", "zh-TW", "es", "fr", "de", "ja", "ko", "pt"]"#;
    let replacement = r#"SUPPORTED_LANGUAGES = ["en", "zh-CN", "zh-TW", "es", "fr", "de", "ja", "ko", "pt", "ru"]"#;
    if content.contains(replacement) {
        step("  SUPPORTED_LANGUAGES уже содержит 'ru', пропускаем");
        return Ok(content.to_string());
    }
    if !content.contains(needle) {
        return Err(Error::PatchFailed("SUPPORTED_LANGUAGES не найден".into()));
    }
    Ok(content.replace(needle, replacement))
}

pub fn patch_renderer_js(content: &str) -> Result<String> {
    let mut result = content.to_string();

    // 1. Добавить русский в getLanguageOptions
    let pt_label = r#"label: t("preferences.general.misc.language.portuguese")"#;
    let pt_block = &format!("  {{\n    {},\n    value: \"pt\"\n  }}", pt_label);
    let ru_block = &format!(
        "  {{\n    label: t(\"preferences.general.misc.language.russian\"),\n    value: \"ru\"\n  }}"
    );

    if !result.contains(ru_block) {
        let inserted = if let Some(_pos) = result.find(pt_block) {
            result.replacen(pt_block, &format!("{},\n{}", ru_block, pt_block), 1)
        } else {
            let pt_alt = &format!("{{\n    {},\n    value: \"pt\"\n  }}", pt_label);
            if result.contains(pt_alt) {
                let ru_alt = &format!(
                    "{{\n    label: t(\"preferences.general.misc.language.russian\"),\n    value: \"ru\"\n  }}"
                );
                result.replacen(pt_alt, &format!("{},\n{}", ru_alt, pt_alt), 1)
            } else {
                return Err(Error::PatchFailed(
                    "португальский язык не найден в getLanguageOptions".into()
                ));
            }
        };
        result = inserted;
        success("  Русский добавлен в выпадающий список языков");
    } else {
        step("  Русский уже есть в выпадающем списке");
    }

    // 2. Добавить "russian" в en.json
    let port_key = "\"portuguese\":\"Portugu\u{ea}s\"";
    let russian_addition = ",\"russian\":\"\u{420}\u{443}\u{441}\u{441}\u{43a}\u{438}\u{439}\"";

    if result.contains(russian_addition) {
        step("  Метка \"Русский\" уже присутствует в en.json");
    } else if let Some(pos) = result.find(port_key) {
        result.insert_str(pos + port_key.len(), russian_addition);
        success("  Метка \"Русский\" добавлена в en.json");
    } else {
        let plain = "\"portuguese\":\"Portugues\"";
        if let Some(pos) = result.find(plain) {
            result.insert_str(pos + plain.len(), russian_addition);
            success("  Метка \"Русский\" добавлена в en.json (упрощённый вариант)");
        } else {
            return Err(Error::PatchFailed(
                "португальский не найден во встроенном en.json".into()
            ));
        }
    }

    Ok(result)
}

pub fn patch_renderer_assets(assets_dir: &Path) -> Result<()> {
    let entries = fs::read_dir(assets_dir)
        .map_err(|_| Error::PatchFailed("не удалось прочитать папку renderer/assets".into()))?;

    let mut found = false;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !fname.starts_with("index-") || !fname.ends_with(".js") {
            continue;
        }
        let original = fs::read_to_string(&path)
            .map_err(|_| Error::PatchFailed(format!("не удалось прочитать {:?}", path)))?;
        if !original.contains("getLanguageOptions") {
            continue;
        }
        step(&format!("  Найден файл: {}", fname));
        let patched = patch_renderer_js(&original)?;
        fs::write(&path, patched.as_bytes())?;
        success("  Файл рендерера пропатчен");
        found = true;
        break;
    }

    if !found {
        return Err(Error::PatchFailed(
            "не найден файл рендерера с getLanguageOptions".into()
        ));
    }
    Ok(())
}

// ──────────────────────────────────────────────
// High-level workflows
// ──────────────────────────────────────────────

pub fn run_patch(marktext_root: Option<PathBuf>, locale_source: Option<PathBuf>) -> Result<()> {
    let paths = match marktext_root {
        Some(r) => MarkTextPaths::from_root(r),
        None => MarkTextPaths::detect()?,
    };

    if !paths.asar.exists() {
        return Err(Error::MarkTextNotFound(
            format!("app.asar не найден: {:?}", paths.asar)
        ));
    }

    let locale_src = match locale_source {
        Some(p) => p,
        None => {
            let exe = std::env::current_exe().map_err(|e| Error::Io(e))?;
            find_source_json(exe.parent().unwrap_or(&exe))?
        }
    };

    if !check_npx() {
        return Err(Error::NpxNotFound);
    }

    backup_asar(&paths.asar, &paths.asar_backup)?;

    let tmp = std::env::temp_dir().join(format!("marktext-patcher-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    extract_asar(&paths.asar, &tmp)?;

    // main
    {
        let main = tmp.join("out").join("main").join("index.js");
        let c = fs::read_to_string(&main)
            .map_err(|_| Error::PatchFailed("не удалось прочитать main/index.js".into()))?;
        fs::write(&main, patch_main_js(&c)?.as_bytes())?;
        success("Основной процесс: SUPPORTED_LANGUAGES расширен");
    }

    // renderer
    {
        let assets = tmp.join("out").join("renderer").join("assets");
        patch_renderer_assets(&assets)?;
    }

    pack_asar(&tmp, &paths.asar)?;
    deploy_locale(&locale_src, &paths.locales_dir)?;
    let _ = fs::remove_dir_all(&tmp);
    step("Временные файлы удалены");

    Ok(())
}

pub fn run_rollback(marktext_root: Option<PathBuf>) -> Result<()> {
    let paths = match marktext_root {
        Some(r) => MarkTextPaths::from_root(r),
        None => MarkTextPaths::detect()?,
    };
    restore_backup(&paths.asar, &paths.asar_backup)?;
    remove_locale(&paths.locales_dir)?;
    Ok(())
}
