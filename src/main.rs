use clap::{Arg, Command, value_parser};
use inquire::{Select, Text};
use std::{fs::File, path::Path, process::ExitCode};
use sys_locale::get_locale;
use tabled::{builder::Builder, settings::Style};
use unic_langid::langid;

use crate::{
    clone::clone_and_init,
    config::init_config,
    db::init_db,
    manifest::{DEVELOPER_FILENAME, MANAGER_FILENAME, REVIEWER_FILENAME, extract_trust_chain},
    package::Package,
    submit::submit_archive,
    teams::{
        add_role, delete_member, fetch_developers, fetch_managers, fetch_reviewers, update_member,
    },
    utils::{ko, ok},
};

mod clone;
mod config;
mod db;
mod locales;
mod manifest;
mod package;
mod submit;
mod teams;
mod utils;

fn cli() -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(Command::new("clone").about("Clone a git repository"))
        .subcommand(
            Command::new("team")
                .about("Manage team members (developers, reviewers, managers)")
                .subcommands([
                    // uvd team add [developer|reviewer|manager]
                    Command::new("add").about("Add a new member to the team"),
                    // uvd team rm [developer|reviewer|manager]
                    Command::new("remove").about("Remove a member from the team"),
                    // uvd team list [developer|reviewer|manager|all]
                    Command::new("list").about("List team members").arg(
                        Arg::new("target").required(true).value_parser([
                            "developer",
                            "reviewer",
                            "manager",
                        ]),
                    ),
                ]),
        )
        .subcommand(
            Command::new("update")
                .about("Update an existing team member's information")
                .arg(Arg::new("role").required(true).value_parser([
                    "developer",
                    "reviewer",
                    "manager",
                ]))
                .arg(
                    Arg::new("email")
                        .required(true)
                        .help("Current email of the member to update"),
                )
                .arg(
                    Arg::new("new_name")
                        .required(true)
                        .long("name")
                        .help("New name"),
                )
                .arg(
                    Arg::new("new_email")
                        .required(false)
                        .long("email")
                        .help("New email"),
                )
                .arg(
                    Arg::new("new_gpg")
                        .required(true)
                        .long("gpg")
                        .help("New GPG Key ID"),
                ),
        )
        .subcommand(
            Command::new("config")
                .about("Manage uvd config file")
                .subcommands([
                    Command::new("init").about("Init uvd configuration"),
                    Command::new("edit").about("Edit uvd configuration"),
                ]),
        )
        .subcommand(Command::new("db").about("Create teams database"))
        .subcommand(
            Command::new("submit")
                .about("Submit archive to a team's member")
                .arg(
                    Arg::new("level")
                        .short('l')
                        .required(false)
                        .default_value("0")
                        .default_missing_value("0")
                        .value_parser(value_parser!(i32))
                        .help("developer(0) reviewer(1) manager(2)"),
                )
                .arg(
                    Arg::new("source")
                        .short('s')
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("the archive source path to submit"),
                ),
        )
        .subcommand(
            Command::new("extract")
                .about("Extract trust artifacts from an archive by level")
                .long_about(
                    "Extract specific security manifests and detached cryptographic signatures \n\
         from a .tar.gz archive based on the specified trust level:\n\
         - 0: Developer artifacts (developer.json, developer.json.asc)\n\
         - 1: Reviewer artifacts (reviewer.json, reviewer.json.asc)\n\
         - 2: Manager artifacts (manager.json, manager.json.asc)",
                )
                .arg(
                    Arg::new("level")
                        .short('l')
                        .required(true)
                        .value_parser(value_parser!(i32))
                        .help("the level (0 for developer) (1 for reviewer) (2 for manager)"),
                )
                .arg(
                    Arg::new("destination")
                        .short('d')
                        .required(false)
                        .default_missing_value(".")
                        .default_value(".")
                        .value_parser(value_parser!(String))
                        .help("the destination directory"),
                )
                .arg(
                    Arg::new("source")
                        .short('s')
                        .required(true)
                        .value_parser(value_parser!(String))
                        .help("the archive file"),
                ),
        )
        .subcommand(
            Command::new("build")
                .about("Make uvd archive from source")
                .arg(
                    Arg::new("src")
                        .short('s')
                        .required(false)
                        .default_value("."),
                ),
        )
}

#[derive(Clone)]
pub enum Level {
    Developer = 0,
    Reviewer = 1,
    Manager = 2,
}

#[tokio::main]
async fn main() -> ExitCode {
    let mut app = cli();
    let matches = app.clone().get_matches();
    let locale = get_locale().unwrap_or_else(|| "en-US".to_string());
    let lang = locale.parse().unwrap_or(langid!("en-US"));
    match matches.subcommand() {
        Some(("team", sub_team)) => match sub_team.subcommand() {
            Some(("add", _)) => {
                let role = Select::new("Role to create:", vec!["developer", "reviewer", "manager"])
                    .prompt()
                    .expect("failed to get role");
                let name = Text::new("Role name:").prompt().expect("");
                let email = Text::new("Role email:").prompt().expect("");
                let gpg_key = Text::new("Gpg key:").prompt().expect("");
                if add_role(&lang, role, name.as_str(), email.as_str(), gpg_key.as_str())
                    .await
                    .is_ok()
                {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::FAILURE
                }
            }
            Some(("remove", _)) => {
                let role = Select::new("Role to remove", vec!["developer", "reviewer", "manager"])
                    .prompt()
                    .expect("failed to get role");

                let email = Text::new("Role email:")
                    .prompt()
                    .expect("failed to get email");
                ok(&lang, "removing-role");
                if delete_member(&lang, role, email.as_str()).await.is_ok() {
                    ok(&lang, "role-removed");
                    return ExitCode::SUCCESS;
                } else {
                    ko(&lang, "fail-to-remove-role");
                    return ExitCode::FAILURE;
                }
            }
            Some(("list", sub)) => {
                let target = sub.get_one::<String>("target").unwrap();
                match target.as_str() {
                    "developer" => {
                        let developers = fetch_developers().await.unwrap_or_default();
                        let mut t = Builder::default();
                        t.push_record(["id", "name", "email", "gpg"]);
                        developers.values().for_each(|x| {
                            t.push_record([
                                x.0.to_string(),
                                x.1.to_string(),
                                x.2.to_string(),
                                x.3.to_string(),
                            ]);
                        });
                        println!("{}", t.build().with(Style::modern()));
                        return ExitCode::SUCCESS;
                    }
                    "reviewer" => {
                        let reviewers = fetch_reviewers().await.unwrap_or_default();
                        let mut t = Builder::default();
                        t.push_record(["id", "name", "email", "gpg"]);
                        for x in reviewers.values() {
                            t.push_record([
                                x.0.to_string(),
                                x.1.to_string(),
                                x.2.to_string(),
                                x.3.to_string(),
                            ]);
                        }
                        println!("{}", t.build().with(Style::modern()));
                        return ExitCode::SUCCESS;
                    }
                    "manager" => {
                        let managers = fetch_managers().await.unwrap_or_default();
                        let mut t = Builder::default();
                        t.push_record(["id", "name", "email", "gpg"]);
                        for x in managers.values() {
                            t.push_record([
                                x.0.to_string(),
                                x.1.to_string(),
                                x.2.to_string(),
                                x.3.to_string(),
                            ]);
                        }
                        println!("{}", t.build().with(Style::modern()));
                        return ExitCode::SUCCESS;
                    }
                    _ => {
                        eprintln!("please use verb 'developer', 'reviewer' or 'manager'");
                        return ExitCode::FAILURE;
                    }
                }
            }
            _ => {
                app.clone().print_help().ok();
                ExitCode::FAILURE
            }
        },

        // --- BLOC UPDATE ---
        Some(("update", sub)) => {
            let role = sub.get_one::<String>("role").unwrap();
            let email = sub.get_one::<String>("email").unwrap();
            let new_name = sub.get_one::<String>("new_name").expect("");
            let new_email = sub.get_one::<String>("new_email").expect("");
            let new_gpg = sub.get_one::<String>("new_gpg").expect("");

            if update_member(
                &lang,
                role.as_str(),
                email.as_str(),
                new_name.as_str(),
                new_email.as_str(),
                new_gpg.as_str(),
            )
            .await
            .is_ok()
            {
                ok(&lang, "member-updated");
                ExitCode::SUCCESS
            } else {
                ko(&lang, "member-not-updated");
                ExitCode::FAILURE
            }
        }
        Some(("submit", sub)) => {
            let level = sub.get_one::<i32>("level").expect("level is required");
            let archive = sub
                .get_one::<String>("source")
                .expect("archive is required");
            submit_archive(&lang, archive, level).await
        }
        Some(("db", _)) => {
            if init_db().await.is_ok() {
                ok(&lang, "db-created");
                ExitCode::SUCCESS
            } else {
                ko(&lang, "db-not-created");
                ExitCode::FAILURE
            }
        }
        Some(("extract", sub)) => {
            let destination = sub
                .get_one::<String>("destination")
                .expect("destination is required");
            let archive = sub.get_one::<String>("source").expect("source is required");
            let level = sub.get_one::<i32>("level").expect("level is required");
            let (filename, end_success_message_key) = match level {
                0 => (DEVELOPER_FILENAME, "developer-files-extracted-successfully"),
                1 => (REVIEWER_FILENAME, "reviewer-files-extracted-successfully"),
                2 => (MANAGER_FILENAME, "manager-files-extracted-successfully"),
                _ => (DEVELOPER_FILENAME, "developer-files-extracted-successfully"),
            };
            extract_trust_chain(
                &lang,
                Path::new(archive.as_str()),
                Path::new(destination.as_str()),
                filename,
                end_success_message_key,
            )
        }
        Some(("build", sub)) => {
            ok(&lang, "start-building");

            let src = sub.get_one::<String>("src").expect("missing source path");

            if Path::new(src).is_dir().eq(&false) {
                ko(&lang, "src-must-be-a-directory");
                ExitCode::FAILURE
            } else if Path::new(format!("{src}/uvd.yml").as_str())
                .exists()
                .eq(&false)
            {
                ko(&lang, "src-must-be-contains-uvd-yml");
                ExitCode::FAILURE
            } else {
                let mut x: Package = serde_yaml::from_reader(
                    File::open(format!("{src}/uvd.yml").as_str()).expect("no uvd.yml"),
                )
                .expect("missing values");
                if x.archive(&lang) {
                    ok(&lang, "build-success");
                    ExitCode::SUCCESS
                } else {
                    ko(&lang, "build-failure");
                    ExitCode::FAILURE
                }
            }
        }
        Some(("config", sub)) => match sub.subcommand() {
            Some(("init", _)) => {
                if init_config(&lang) {
                    return ExitCode::SUCCESS;
                }
                ExitCode::FAILURE
            }

            Some(("edit", _)) => {
                if let Some(config) = dirs::config_dir() {
                    let p = config.join("uvd");
                    if std::process::Command::new(
                        std::env::var("EDITOR").expect("missing editor").as_str(),
                    )
                    .arg("config.toml")
                    .current_dir(&p)
                    .spawn()
                    .expect("")
                    .wait()
                    .expect("")
                    .success()
                    {
                        return ExitCode::SUCCESS;
                    } else {
                        return ExitCode::FAILURE;
                    }
                }
                ExitCode::FAILURE
            }
            _ => {
                eprintln!("please use 'edit' or 'init' verb");
                return ExitCode::FAILURE;
            }
        },
        Some(("clone", _)) => {
            let url = Text::new("Repository url:")
                .prompt()
                .expect("failed to get repo url");
            let dest = Text::new("Repository name:")
                .prompt()
                .expect("failed to get repository name");
            if clone_and_init(&lang, url.as_str(), dest.as_str()) {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }

        _ => {
            app.print_help().expect("failed to print help");
            ExitCode::FAILURE
        }
    }
}
