use crate::config::{Config, load_config};
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};

pub async fn init_db() -> Result<(), sqlx::Error> {
    sqlx::migrate!("./migrations")
        .run(
            &get_db_pool()
                .await
                .expect("failed to connect to the database"),
        )
        .await?;
    Ok(())
}

pub async fn get_db_pool() -> Result<MySqlPool, sqlx::Error> {
    let conf: Config = load_config().expect("missing uvd config");

    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(conf.database_url.as_str())
        .await?;
    Ok(pool)
}
