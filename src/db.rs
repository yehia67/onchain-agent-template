use sqlx::{Pool, Postgres};

pub async fn get_db_pool() -> Option<Pool<Postgres>> {
    match std::env::var("DATABASE_URL") {
        Ok(db_url) => {
            match sqlx::PgPool::connect(&db_url).await {
                Ok(pool) => {
                    println!("Successfully connected to database");
                    Some(pool)
                },
                Err(e) => {
                    eprintln!("Failed to connect to Postgres: {}", e);
                    None
                }
            }
        },
        Err(e) => {
            eprintln!("DATABASE_URL not set: {}", e);
            None
        }
    }
}

pub async fn save_message(pool: &Pool<Postgres>, role: &str, content: &str) -> sqlx::Result<()> {
    sqlx::query("INSERT INTO messages (role, content) VALUES ($1, $2)")
        .bind(role)
        .bind(content)
        .execute(pool)
        .await?;
    Ok(())
}

