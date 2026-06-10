use marktext_ru_patcher::*;

fn main() {
    let result = run_rollback(None);

    match result {
        Ok(()) => {
            println!();
            println!("========================================");
            println!("  ОТКАТ ВЫПОЛНЕН!");
            println!("========================================");
            println!();
            println!("MarkText восстановлен в исходное состояние.");
            println!("Перезапустите MarkText, если он запущен.");
        }
        Err(ref e) => {
            error(&format!("{}", e));
        }
    }

    press_any_key();
    std::process::exit(if result.is_ok() { 0 } else { 1 });
}
