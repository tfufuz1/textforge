use serde::{Serialize, Deserialize};
use tauri::State;
use crate::AppState;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TagInfoDto {
    pub name: String,
    pub color: Option<String>,
    pub usage_count: u32,
    pub last_used_at: i64,
    pub created_at: i64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagRenameResultDto {
    pub old_name: String,
    pub new_name: String,
    pub affected_items: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagMergeResultDto {
    pub source_tags: Vec<String>,
    pub target_tag: String,
    pub affected_items: u32,
}

#[tauri::command]
pub async fn suggest_tags(
    prefix: String,
    limit: u32,
    state: State<'_, AppState>,
) -> Result<Vec<TagInfoDto>, String> {
    let clean_prefix = format!("{}%", prefix.trim().to_lowercase());
    let lim = if limit == 0 { 10 } else { limit };

    #[derive(sqlx::FromRow)]
    struct Row {
        tag: String,
        color: Option<String>,
        cnt: i64,
        tag_created_at: Option<i64>,
    }

    let rows = sqlx::query_as::<_, Row>(
        "SELECT t.tag, tc.color, tc.created_at as tag_created_at, COUNT(*) as cnt
         FROM (
           SELECT tag FROM snippet_tags
           UNION ALL
           SELECT tag FROM script_tags
           UNION ALL
           SELECT tag FROM pipeline_tags
         ) t
         LEFT JOIN tag_colors tc ON t.tag = tc.tag_name
         WHERE t.tag LIKE ?
         GROUP BY t.tag
         ORDER BY cnt DESC, t.tag ASC
         LIMIT ?"
    )
    .bind(&clean_prefix)
    .bind(lim)
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    let now = chrono::Utc::now().timestamp_millis();
    Ok(rows.into_iter().map(|r| TagInfoDto {
        name: r.tag,
        color: r.color,
        usage_count: r.cnt as u32,
        last_used_at: now,
        created_at: r.tag_created_at.unwrap_or(now),
    }).collect())
}

#[tauri::command]
pub async fn rename_tag(
    old_name: String,
    new_name: String,
    state: State<'_, AppState>,
) -> Result<TagRenameResultDto, String> {
    let old_tag = old_name.trim().to_lowercase();
    let new_tag = new_name.trim().to_lowercase();

    if old_tag == new_tag {
        return Ok(TagRenameResultDto { old_name: old_tag, new_name: new_tag, affected_items: 0 });
    }

    let mut tx = state.db.begin().await.map_err(|e| e.to_string())?;

    // Schritt 1: Duplikate auf 3 Tabellen bereinigen
    // Für Items, die BEREITS den new_tag haben: old_tag entfernen (statt rename versuchen)
    for table in ["snippet_tags", "script_tags", "pipeline_tags"] {
        let id_col = match table {
            "snippet_tags"  => "snippet_id",
            "script_tags"   => "script_id",
            _               => "pipeline_id",
        };
        sqlx::query(&format!(
            "DELETE FROM {table} WHERE tag = ? AND {id_col} IN (
               SELECT {id_col} FROM {table} WHERE tag = ?
             )"
        ))
        .bind(&old_tag).bind(&new_tag)
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;
    }

    // Schritt 2: Umbenennen (jetzt ohne Konflikte möglich)
    let mut total_affected = 0u32;
    for table in ["snippet_tags", "script_tags", "pipeline_tags"] {
        let res = sqlx::query(&format!("UPDATE {table} SET tag = ? WHERE tag = ?"))
            .bind(&new_tag).bind(&old_tag)
            .execute(&mut *tx).await.map_err(|e| e.to_string())?;
        total_affected += res.rows_affected() as u32;
    }

    // Schritt 3: Tag-Color migrieren
    sqlx::query("UPDATE tag_colors SET tag_name = ? WHERE tag_name = ?")
        .bind(&new_tag).bind(&old_tag)
        .execute(&mut *tx).await.ok();

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(TagRenameResultDto {
        old_name: old_tag,
        new_name: new_tag,
        affected_items: total_affected,
    })
}

#[tauri::command]
pub async fn merge_tags(
    source_tags: Vec<String>,
    target_tag: String,
    state: State<'_, AppState>,
) -> Result<TagMergeResultDto, String> {
    let target = target_tag.trim().to_lowercase();
    let mut total_affected = 0u32;

    for src in &source_tags {
        let res = rename_tag(src.clone(), target.clone(), state.clone()).await?;
        total_affected += res.affected_items;
    }

    Ok(TagMergeResultDto {
        source_tags,
        target_tag: target,
        affected_items: total_affected,
    })
}

#[tauri::command]
pub async fn set_tag_color(
    tag_name: String,
    color: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let name = tag_name.trim().to_lowercase();
    let now = chrono::Utc::now().timestamp_millis();

    if let Some(col) = color {
        sqlx::query("INSERT INTO tag_colors (tag_name, color, created_at) VALUES (?, ?, ?) ON CONFLICT(tag_name) DO UPDATE SET color = ?")
            .bind(&name).bind(&col).bind(now).bind(&col)
            .execute(&state.db)
            .await
            .map_err(|e| e.to_string())?;
    } else {
        sqlx::query("DELETE FROM tag_colors WHERE tag_name = ?")
            .bind(&name)
            .execute(&state.db)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}
