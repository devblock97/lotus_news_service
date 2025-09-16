use std::fs;

use sqlx::Pool;

pub type DbPool = Pool<sqlx::Postgres>;

pub async fn connect(url: &str) -> anyhow::Result<Pool<sqlx::Postgres>> {
    let pool = sqlx::postgres::PgPoolOptions::new().max_connections(10).connect(url).await?;
    Ok(pool)
}

pub async fn apply_sql_folder(pool: &Pool<sqlx::Postgres>, folder: &str) -> anyhow::Result<()> {
    let mut entries = tokio::fs::read_dir(folder).await?;
    let mut files = vec![];
    while let Ok(Some(e)) = entries.next_entry().await { files.push(e.path()); }
    files.sort();
    for f in files {
        if f.extension().and_then(|s| s.to_str()) == Some("sql") {
            let sql = fs::read_to_string(&f)?;
            sqlx::query(&sql).execute(pool).await?;
        }
    }

    Ok(())

}
