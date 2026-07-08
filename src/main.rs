use std::{fs::File, path::Path, process::ExitCode};

use clap::{Arg, Command, value_parser};
use sys_locale::get_locale;
use unic_langid::langid;

use crate::{
    config::init_config,
    db::init_db,
    manifest::{DEVELOPER_FILENAME, MANAGER_FILENAME, REVIEWER_FILENAME, extract_trust_chain},
    package::Package,
    submit::submit_archive,
    utils::{ko, ok},
};

mod config;
mod db;
mod locales;
mod manifest;
mod package;
mod submit;
mod utils;

fn cli() -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .version(env!("CARGO_PKG_VERSION"))
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
        Some(("submit", sub)) => {
            let level = sub.get_one::<u8>("level").expect("destination is required");
            let archive = sub
                .get_one::<String>("source")
                .expect("archive is required");
            submit_archive(&lang, archive, *level).await
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
        _ => {
            app.print_help().expect("failed to print help");
            ExitCode::FAILURE
        }
    }
}
