use crate::{
    db::get_db_pool,
    utils::{ko, ok},
};
use sqlx::{Row, mysql::MySqlRow};
use std::{collections::HashMap, io::Error};
use unic_langid::LanguageIdentifier;

/// Supprime un membre de l'équipe selon son rôle et son email
pub async fn delete_member(
    lang: &LanguageIdentifier,
    role: &str,
    email: &str,
) -> Result<(), Error> {
    let pool = get_db_pool().await.expect("failed to connect to the db");

    // On détermine la table cible selon le rôle validé par Clap
    match role {
        "developer" => {
            sqlx::query("DELETE FROM developers WHERE email = ?")
                .bind(email)
                .execute(&pool)
                .await
                .expect("failed to remove");
            ok(lang, "developer-removed-successfully");
            Ok(())
        }

        "manager" => {
            sqlx::query("DELETE FROM  managers WHERE email = ?")
                .bind(email)
                .execute(&pool)
                .await
                .expect("failed to remove");
            ok(lang, "manager-removed-successfully");
            Ok(())
        }
        "reviewer" => {
            sqlx::query("DELETE FROM reviewers WHERE email = ?")
                .bind(email)
                .execute(&pool)
                .await
                .expect("failed to remove");
            ok(lang, "reviewer-removed-successfully");
            Ok(())
        }
        _ => Err(Error::other("no valid request")),
    }
}

pub async fn find_member(role: &str, email: &str) -> Result<Option<MySqlRow>, Error> {
    let pool = get_db_pool().await.expect("failed to connect to the db");

    match role {
        "developer" => {
            if let Ok(x) = sqlx::query("SELECT name,email FROM developers WHERE email = ?")
                .bind(email)
                .fetch_optional(&pool)
                .await
            {
                if x.is_none() {
                    return Err(Error::new(
                        std::io::ErrorKind::NotFound,
                        "developer missing",
                    ));
                }
                Ok(x)
            } else {
                Err(Error::other("db error"))
            }
        }
        "reviewer" => {
            if let Ok(x) = sqlx::query("SELECT name,email FROM reviewers WHERE email = ?")
                .bind(email)
                .fetch_optional(&pool)
                .await
            {
                if x.is_none() {
                    return Err(Error::new(std::io::ErrorKind::NotFound, "reviewer missing"));
                }
                Ok(x)
            } else {
                Err(Error::other("db error"))
            }
        }
        "manager" => {
            if let Ok(x) = sqlx::query("SELECT name,email FROM managers WHERE email = ?")
                .bind(email)
                .fetch_optional(&pool)
                .await
            {
                if x.is_none() {
                    return Err(Error::new(std::io::ErrorKind::NotFound, "manager missing"));
                }
                Ok(x)
            } else {
                Err(Error::other("db error"))
            }
        }
        _ => Err(Error::new(
            std::io::ErrorKind::InvalidInput,
            "no valid request",
        )),
    }
}

/// Met à jour les informations d'un membre de l'équipe de manière dynamique
pub async fn update_member(
    lang: &LanguageIdentifier,
    role: &str,
    current_email: &str,
    new_name: &str,
    new_email: &str,
    new_gpg: &str,
) -> Result<(), Error> {
    let pool = get_db_pool().await.expect("failed to connect to the db");

    if find_member(role, current_email).await.is_ok() {
        // On prend la nouvelle valeur si elle existe, sinon on garde l'ancienne
        if role.eq("developer") {
            sqlx::query(
                "UPDATE developers SET name = ?, email = ?, gpg_key_id = ? WHERE email = ?",
            )
            .bind(new_name)
            .bind(new_email)
            .bind(new_gpg)
            .bind(current_email)
            .execute(&pool)
            .await
            .expect("failed to update developer");
            Ok(())
        } else if role.eq("reviewer") {
            sqlx::query("UPDATE reviewers SET name = ?, email = ?, gpg_key_id = ? WHERE email = ?")
                .bind(new_name)
                .bind(new_email)
                .bind(new_gpg)
                .bind(current_email)
                .execute(&pool)
                .await
                .expect("faield to update reviewer");
            Ok(())
        } else if role.eq("manager") {
            sqlx::query("UPDATE managers SET name = ?, email = ?, gpg_key_id = ? WHERE email = ?")
                .bind(new_name)
                .bind(new_email)
                .bind(new_gpg)
                .bind(current_email)
                .execute(&pool)
                .await
                .expect("failed to update manager");
            Ok(())
        } else {
            ko(lang, "bad-role");
            Err(Error::other("bad role"))
        }
    } else {
        ko(lang, "member-not-updated");
        Err(Error::other("not-updated"))
    }
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

pub async fn add_role(
    lang: &LanguageIdentifier,
    role: &str,
    name: &str,
    email: &str,
    gpg: &str,
) -> Result<(), Error> {
    let pool = get_db_pool().await.expect("failed to connect");
    let find = find_member(role, email).await;

    // Si le membre a été trouvé, c'est qu'il existe déjà !
    if find.is_ok() {
        ko(lang, "role-exists");
        return Err(Error::other("already exists"));
    }

    if role.eq("developer")
        && sqlx::query("INSERT INTO developers VALUES (NULL,?,?,?)")
            .bind(name)
            .bind(email)
            .bind(gpg)
            .execute(&pool)
            .await
            .is_ok()
    {
        ok(lang, "role-developer-added");
        Ok(())
    } else if role.eq("reviewer")
        && sqlx::query("INSERT INTO reviewers VALUES (NULL,?,?,?)")
            .bind(name)
            .bind(email)
            .bind(gpg)
            .execute(&pool)
            .await
            .is_ok()
    {
        ok(lang, "role-reviewer-added");
        Ok(())
    } else if role.eq("manager")
        && sqlx::query("INSERT INTO managers VALUES (NULL,?,?,?)")
            .bind(name)
            .bind(email)
            .bind(gpg)
            .execute(&pool)
            .await
            .is_ok()
    {
        ok(lang, "role-manager-added");
        Ok(())
    } else {
        Err(Error::other("failed to insert"))
    }
}

/// Récupère la liste des développeurs
pub async fn fetch_developers()
-> Result<HashMap<String, (String, String, String, String)>, sqlx::Error> {
    let pool = get_db_pool()
        .await
        .expect("failed to connect to the database");
    let mut response: HashMap<String, (String, String, String, String)> = HashMap::new();

    let rows = sqlx::query("SELECT id, name, email, gpg_key_id FROM developers ORDER BY id ASC")
        .fetch_all(&pool)
        .await?;

    rows.iter().for_each(|r| {
        let id: i32 = r.get("id");
        let name: String = r.get("name");
        let email: String = r.get("email");
        let gpg: String = r.get::<Option<String>, _>("gpg_key_id").unwrap_or_default();

        response.insert(
            name.to_string(),
            (id.to_string(), name.to_string(), email, gpg),
        );
    });
    Ok(response)
}

/// Récupère la liste des managers
pub async fn fetch_managers() -> Result<HashMap<String, (i32, String, String, String)>, sqlx::Error>
{
    let pool = get_db_pool()
        .await
        .expect("failed to connect to the database");
    let mut response: HashMap<String, (i32, String, String, String)> = HashMap::new();

    // Correction : On va chercher 'gpg_key_id' au lieu de s'arrêter à email
    let rows = sqlx::query("SELECT id, name, email, gpg_key_id FROM managers ORDER BY  id ASC")
        .fetch_all(&pool)
        .await?;

    rows.iter().for_each(|r| {
        let id: i32 = r.get("id");
        let name: String = r.get("name");
        let email: String = r.get("email");
        // On récupère proprement la clé GPG
        let gpg: String = r.get::<Option<String>, _>("gpg_key_id").unwrap_or_default();

        response.insert(name.to_string(), (id, name.to_string(), email, gpg));
    });
    Ok(response)
}

/// Récupère la liste des reviewers
pub async fn fetch_reviewers() -> Result<HashMap<String, (i32, String, String, String)>, sqlx::Error>
{
    let pool = get_db_pool()
        .await
        .expect("failed to connect to the database");
    let mut response: HashMap<String, (i32, String, String, String)> = HashMap::new();

    // Correction : Idem, on ajoute 'gpg_key_id' dans le SELECT
    let rows = sqlx::query("SELECT id, name, email, gpg_key_id FROM reviewers ORDER BY id ASC")
        .fetch_all(&pool)
        .await?;

    rows.iter().for_each(|r| {
        let id: i32 = r.get("id");
        let name: String = r.get("name");
        let email: String = r.get("email");
        // On récupère proprement la clé GPG
        let gpg: String = r.get::<Option<String>, _>("gpg_key_id").unwrap_or_default();

        response.insert(name.to_string(), (id, name.to_string(), email, gpg));
    });
    Ok(response)
}
