use crate::package::Package;
use crate::utils::{ok, tt};
use crossterm::cursor::{Hide, Show};
use crossterm::execute;
use git2::build::{CheckoutBuilder, RepoBuilder};
use git2::{FetchOptions, RemoteCallbacks};
use indicatif::{ProgressBar, ProgressStyle};
use inquire::{Confirm, Text};
use std::fs::{File, remove_dir_all};
use std::io::{Write, stdout};
use std::path::Path;
use unic_langid::LanguageIdentifier;

pub fn clone_and_init(lang: &LanguageIdentifier, url: &str, dest: &str) -> bool {
    execute!(stdout(), Hide).expect("");
    if Path::new(dest).is_dir()
        && Confirm::new(tt(lang, "repository-already-exists").as_str())
            .with_default(false)
            .prompt()
            .unwrap_or_default()
            .eq(&true)
    {
        ok(lang, "removing-repository");
        remove_dir_all(dest).expect("faield to remove dir");
        ok(lang, "repository-removed");
    }
    ok(lang, "start-clone");

    // 1. Initialisation de la barre de progression (indicatif)
    let pb = ProgressBar::new(0);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.white} [{elapsed_precise}] [{bar:40.white}] {pos}/{len} ({eta}) {msg}",
            )
            .unwrap()
            .progress_chars("=> "),
    );

    // 2. Configuration du pont entre git2 et indicatif
    let mut callbacks = RemoteCallbacks::new();
    callbacks.transfer_progress(|stats| {
        // Met à jour la taille totale et l'avancement
        pb.set_length(stats.total_objects() as u64);
        pb.set_position(stats.received_objects() as u64);
        true // Retourner true pour continuer le téléchargement
    });

    callbacks.credentials(|_url, username_from_url, allowed_types| {
        if allowed_types.is_ssh_key() {
            let user = username_from_url.unwrap_or("git");

            // Tentative 1 : Via l'agent SSH
            git2::Cred::ssh_key_from_agent(user).or_else(|_| {
                // Tentative 2 : Fallback sur le fichier de clé direct si l'agent échoue
                let home = dirs::home_dir().expect("HOME introuvable");
                let priv_key = home.join(".ssh/id_ed25519");
                git2::Cred::ssh_key(user, None, &priv_key, None)
            })
        } else {
            git2::Cred::default()
        }
    });

    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    // 3. Suivi de l'écriture sur le disque
    let mut checkout = CheckoutBuilder::new();
    checkout.progress(|path, completed_steps, total_steps| {
        // On met à jour la barre de progression pour l'extraction
        if pb.length() != Some(total_steps as u64) {
            pb.set_length(total_steps as u64);
        }
        pb.set_position(completed_steps as u64);

        // On affiche le chemin du fichier en cours d'extraction
        if let Some(p) = path {
            pb.set_message(format!("Extraction: {}", p.display()));
        }
    });

    // 4. Configuration finale du builder
    let mut builder = RepoBuilder::new();
    builder.fetch_options(fetch_options);
    builder.with_checkout(checkout); // <-- Le checkout builder est bien attaché et conservé !

    // 5. Lancement du clone
    match builder.clone(url, Path::new(dest)) {
        Ok(_) => {
            pb.finish_with_message(tt(lang, "cloned"));
            let conf_path = Path::new(dest).join("uvd.yml");

            if !conf_path.exists() {
                let default_name = Path::new(dest).file_name().unwrap().to_str().unwrap();
                let name = Text::new(tt(lang, "package-name").as_str())
                    .with_default(default_name)
                    .prompt()
                    .expect("failed to get package name");
                let pkgdesc = Text::new(tt(lang, "package-description").as_str())
                    .prompt()
                    .expect("failed to get package description");
                let pkgver = Text::new(tt(lang, "package-version").as_str())
                    .prompt()
                    .expect("failed to get package version");
                let pkg_readme_filename = Text::new(tt(lang, "package-readme-name").as_str())
                    .prompt()
                    .expect("failed to get package readme filename");
                let pkgname = Text::new(tt(lang, "packager-name").as_str())
                    .prompt()
                    .expect("failed to get packager name");
                let pkgemail = Text::new(tt(lang, "packager-email").as_str())
                    .prompt()
                    .expect("failed to get packager email");

                let src: Vec<String> = Vec::new();
                let man: Vec<String> = Vec::new();
                let license = String::new();

                let x = Package {
                    name,
                    version: pkgver,
                    description: pkgdesc,
                    pkgemail,
                    pkgname,
                    license,
                    src,
                    readme: pkg_readme_filename,
                    man,
                };

                execute!(stdout(), Show).expect("");
                let config = serde_yaml::to_string(&x).expect("failed to serialize configuration");
                let mut f = File::create_new(conf_path).expect("failed to create uvd.yml");
                f.write_all(config.as_bytes())
                    .expect("failed to write config");
                f.sync_all().expect("failed to sync config file");

                ok(lang, "uvd-yml-created-successfully");
                true
            } else {
                ok(lang, "uvd-yml-exists");
                true
            }
        }

        Err(e) => {
            pb.finish_and_clear();
            eprintln!("clone err : {e}");
            false
        }
    }
}
