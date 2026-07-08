use clap::{Arg, Command, value_parser};
use crossterm::{
    execute,
    style::{Print, Stylize},
    terminal::size,
};
use fluent_templates::{Loader, static_loader};
use std::{fs::File, io::stdout, path::Path, process::ExitCode};
use sys_locale::get_locale;
use unic_langid::{LanguageIdentifier, langid};

use crate::{
    manifest::{DEVELOPER_FILENAME, MANAGER_FILENAME, REVIEWER_FILENAME, extract_trust_chain},
    package::Package,
};

mod manifest;
mod package;

static_loader! {
  pub static LOCALES = {
        locales: "./locales",
        fallback_language: "en-US",
    };
}

fn cli() -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .version(env!("CARGO_PKG_VERSION"))
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

fn ok(lang: &LanguageIdentifier, key: &str) {
    let (x, _) = size().expect("failed to get term size");
    let description = LOCALES.try_lookup(lang, key).unwrap_or_default();
    let star = " * ";
    let ok = " ok ";
    let padding = x
        - description.chars().count() as u16
        - ok.chars().count() as u16
        - star.chars().count() as u16
        - 2;
    execute!(
        stdout(),
        Print(" * ".green().bold()),
        Print(description.white()),
        Print(" ".repeat(padding as usize)),
        Print("[ ".white().bold()),
        Print("ok".green().bold()),
        Print(" ]".white().bold()),
    )
    .expect("failed to print");
}

fn ko(lang: &LanguageIdentifier, key: &str) {
    let (x, _) = size().expect("failed to get term size");
    let description = LOCALES.try_lookup(lang, key).unwrap_or_default();
    let star = " * ";
    let ok = " ko ";
    let padding = x
        - description.chars().count() as u16
        - ok.chars().count() as u16
        - star.chars().count() as u16
        - 2;
    execute!(
        stdout(),
        Print(" ! ".red().bold()),
        Print(description.white()),
        Print(" ".repeat(padding as usize)),
        Print("[ ".white().bold()),
        Print("ko".red().bold()),
        Print(" ]".white().bold()),
    )
    .expect("faield to print");
}
#[derive(Clone)]
pub enum Level {
    Developer = 0,
    Reviewer = 1,
    Manager = 2,
}

fn main() -> ExitCode {
    let mut app = cli();
    let matches = app.clone().get_matches();
    let locale = get_locale().unwrap_or_else(|| "en-US".to_string());
    let lang = locale.parse().unwrap_or(langid!("en-US"));
    match matches.subcommand() {
        Some(("extract", sub)) => {
            let destination = sub
                .get_one::<String>("destination")
                .expect("destination is required");
            let archive = sub
                .get_one::<String>("source")
                .expect("archive is required");
            let level = sub.get_one::<i32>("level").expect("level is required");
            let (filename, end_success_message_key) = match level {
                0 => (DEVELOPER_FILENAME, "developer-files-extracted-successfully"),
                1 => (REVIEWER_FILENAME, "rewiever-files-extracted-successfully"),
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
        _ => {
            app.print_help().expect("failed to print help");
            ExitCode::FAILURE
        }
    }
}
