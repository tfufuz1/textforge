use sqlx::sqlite::SqlitePool;
use std::path::Path;

pub async fn init_db(path: &Path) -> Result<SqlitePool, sqlx::Error> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).ok();
        }
    }

    // Create the file if it doesn't exist yet
    if !path.exists() {
        std::fs::File::create(path).ok();
    }

    let pool = SqlitePool::connect(&format!("sqlite:{}", path.display())).await?;
    
    sqlx::query("PRAGMA journal_mode = WAL").execute(&pool).await?;
    sqlx::query("PRAGMA foreign_keys = ON").execute(&pool).await?;
    
    sqlx::migrate!("./migrations").run(&pool).await?;
    
    Ok(pool)
}
