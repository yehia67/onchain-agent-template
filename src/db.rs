use sqlx::{Pool, Postgres};

pub async fn get_db_pool() -> Pool<Postgres> {
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    sqlx::PgPool::connect(&db_url).await.expect("Failed to connect to Postgres")
}

pub async fn save_message(pool: &Pool<Postgres>, role: &str, content: &str) -> sqlx::Result<()> {
    sqlx::query("INSERT INTO messages (role, content) VALUES ($1, $2)")
        .bind(role)
        .bind(content)
        .execute(pool)
        .await?;
    Ok(())
}

