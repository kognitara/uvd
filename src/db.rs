use crate::config::{Config, load_config};
use sqlx::Row;
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

/// Récupère la liste des développeurs formatée pour le menu de sélection
pub async fn fetch_developers_list() -> Result<Vec<String>, sqlx::Error> {
    let pool = get_db_pool().await.expect("failed to conenct to the db");
    let mut response: Vec<String> = Vec::new();
    let rows = sqlx::query("SELECT name, email FROM developers ORDER BY name ASC")
        .fetch_all(&pool)
        .await?;

    // On extrait manuellement les colonnes textuelles de chaque ligne
    rows.iter().for_each(|r| {
        let name: String = r.get("name");
        let email: String = r.get("email");
        response.push(format!("{} <{}>", name, email));
    });
    Ok(response)
}

/// Récupère la liste des reviewers formatée pour le menu de sélection
pub async fn fetch_reviewers_list() -> Result<Vec<String>, sqlx::Error> {
    let pool = get_db_pool().await.expect("failed to connect");
    let rows = sqlx::query("SELECT name, email FROM reviewers ORDER BY name ASC")
        .fetch_all(&pool)
        .await?;

    let reviewers = rows
        .into_iter()
        .map(|row| {
            let name: String = row.get("name");
            let email: String = row.get("email");
            format!("{} <{}>", name, email)
        })
        .collect();
    Ok(reviewers)
}

/// Récupère la liste des managers formatée pour le menu de sélection
pub async fn fetch_managers_list() -> Result<Vec<String>, sqlx::Error> {
    let pool = get_db_pool().await.expect("failed to connect");
    let rows = sqlx::query("SELECT name, email FROM managers ORDER BY name ASC")
        .fetch_all(&pool)
        .await?;

    let reviewers = rows
        .into_iter()
        .map(|row| {
            let name: String = row.get("name");
            let email: String = row.get("email");
            format!("{} <{}>", name, email)
        })
        .collect();
    Ok(reviewers)
}
