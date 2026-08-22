use std::env;
use std::path::PathBuf;
use std::process::exit;
use sqlx::SqlitePool;
use sha2::{Digest, Sha256};

fn print_help() {
    println!("textforge-cli v3.0 — TextForge CLI Fernsteuerung");
    println!("Usage:");
    println!("  textforge-cli add \"Text\"                   Fügt Text als neuen Clipboard-Eintrag hinzu");
    println!("  textforge-cli read <id>                    Gibt Inhalt eines Eintrags auf stdout aus");
    println!("  textforge-cli list [--tag <tag>]           Listet Einträge gefiltert nach Tag");
    println!("  textforge-cli tab <name> add <id>          Fügt Item zu einem CollectionTab hinzu");
    println!("  textforge-cli sequence render <id>         Rendert eine Sequenz");
    println!("  textforge-cli --help                       Zeigt diese Hilfe an");
}

fn get_db_path() -> PathBuf {
    if let Ok(home) = env::var("HOME") {
        let p = PathBuf::from(home).join(".local/share/textforge/textforge.db");
        if p.exists() {
            return p;
        }
    }
    PathBuf::from("textforge_dev.db")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_help();
        exit(0);
    }

    let db_path = get_db_path();
    let db_url = format!("sqlite://{}", db_path.to_string_lossy());
    let pool = SqlitePool::connect(&db_url).await?;

    let command = &args[1];
    match command.as_str() {
        "add" => {
            if args.len() < 3 {
                eprintln!("Error: Text to add required");
                exit(1);
            }
            let text = &args[2];
            let id = uuid::Uuid::new_v4().to_string();
            let now = chrono::Utc::now().timestamp_millis();

            let mut hasher = Sha256::new();
            hasher.update(text.as_bytes());
            let hash = format!("{:x}", hasher.finalize());

            sqlx::query(
                "INSERT INTO clipboard_history (id, content, content_hash, content_type, captured_at) VALUES (?, ?, ?, 'plain_text', ?)"
            )
            .bind(&id).bind(text).bind(&hash).bind(now)
            .execute(&pool)
            .await?;

            println!("Added entry {} to database", id);
        }
        "read" => {
            if args.len() < 3 {
                eprintln!("Error: ID required");
                exit(1);
            }
            let id = &args[2];
            #[derive(sqlx::FromRow)]
            struct Row { content: String }

            let row = sqlx::query_as::<_, Row>("SELECT content FROM clipboard_history WHERE id = ? OR CAST(rowid AS TEXT) = ?")
                .bind(id).bind(id)
                .fetch_optional(&pool)
                .await?;

            if let Some(r) = row {
                println!("{}", r.content);
            } else {
                eprintln!("Entry not found");
                exit(1);
            }
        }
        "list" => {
            let tag_filter = args.windows(2).find(|w| w[0] == "--tag").map(|w| &w[1]);
            #[derive(sqlx::FromRow)]
            struct Row { id: String, content: String }

            let rows = if let Some(tag) = tag_filter {
                sqlx::query_as::<_, Row>(
                    "SELECT c.id, c.content FROM clipboard_history c JOIN snippet_tags t ON c.id = t.snippet_id WHERE t.tag = ? ORDER BY c.captured_at DESC LIMIT 50"
                )
                .bind(tag)
                .fetch_all(&pool)
                .await?
            } else {
                sqlx::query_as::<_, Row>(
                    "SELECT id, content FROM clipboard_history ORDER BY captured_at DESC LIMIT 50"
                )
                .fetch_all(&pool)
                .await?
            };

            for r in rows {
                let preview: String = r.content.chars().take(60).collect();
                println!("{}  {}", r.id, preview);
            }
        }
        "tab" => {
            if args.len() >= 5 && args[3] == "add" {
                let tab_name = &args[2];
                let item_id = &args[4];
                let now = chrono::Utc::now().timestamp_millis();

                sqlx::query(
                    "INSERT OR IGNORE INTO collection_tab_members (tab_id, item_kind, item_id, added_at) VALUES (?, 'snippet', ?, ?)"
                )
                .bind(tab_name).bind(item_id).bind(now)
                .execute(&pool)
                .await?;

                println!("Added item {} to tab {}", item_id, tab_name);
            } else {
                eprintln!("Usage: textforge-cli tab <name> add <id>");
                exit(1);
            }
        }
        "sequence" => {
            if args.len() >= 4 && args[2] == "render" {
                let seq_id = &args[3];
                #[derive(sqlx::FromRow)]
                struct SeqRow { name: String }

                let seq = sqlx::query_as::<_, SeqRow>("SELECT name FROM sequences WHERE id = ?")
                    .bind(seq_id)
                    .fetch_optional(&pool)
                    .await?;

                if let Some(s) = seq {
                    println!("Sequence: {}", s.name);
                } else {
                    eprintln!("Sequence not found");
                    exit(1);
                }
            } else {
                eprintln!("Usage: textforge-cli sequence render <id>");
                exit(1);
            }
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_help();
            exit(1);
        }
    }

    Ok(())
}
