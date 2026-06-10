use marktext_ru_patcher::*;

fn main() {
    let locale = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("ru.json")));

    let result = run_patch(None, locale);

    match result {
        Ok(()) => {
            println!();
            println!("========================================");
            println!("  РУССКАЯ ЛОКАЛИЗАЦИЯ УСТАНОВЛЕНА!");
            println!("========================================");
            println!();
            println!("Что дальше:");
            println!("  1. Перезапустите MarkText, если он запущен");
            println!("  2. Откройте Настройки > Общие > Язык");
            println!("  3. Выберите Русский");
        }
        Err(ref e) => {
            error(&format!("{}", e));
        }
    }

    press_any_key();
    std::process::exit(if result.is_ok() { 0 } else { 1 });
}
