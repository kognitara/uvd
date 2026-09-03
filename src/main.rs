use clap::{Arg, ArgAction, Command};
use clap_complete::{Shell, generate};
use inquire::Text;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, BufWriter};
use std::path::Path;
use tar::Builder;
use zstd::Encoder;

#[derive(Deserialize, Serialize)]
pub struct Uvd {
    name: String,
    version: String,
    homepage: String,
    license: String,
    repository: String,
    description: String,
    src: Vec<String>,
    man: Vec<String>,
}

fn create_archive_zstd(
    sources: Vec<String>,
    manuals: Vec<String>,
    archive: &str,
) -> io::Result<()> {
    let output = File::create(archive)?;
    let buffer = BufWriter::new(output);
    let encoder_zstd = Encoder::new(buffer, 3)?.auto_finish();
    let mut archive = Builder::new(encoder_zstd);
    for chemin_str in &sources {
        let chemin = Path::new(&chemin_str);
        if !chemin.exists() {
            continue;
        }
        if chemin.is_dir() {
            archive.append_dir_all(chemin, chemin)?;
        } else if chemin.is_file() {
            let mut fichier = File::open(chemin)?;
            archive.append_file(chemin, &mut fichier)?;
        }
    }
    for manual in &manuals {
        let man = Path::new(&manual);
        if !man.exists() {
            continue;
        }
        if man.is_dir() {
            archive.append_dir_all(man, man)?;
        } else if man.is_file() {
            let mut f = File::open(man)?;
            archive.append_file(man, &mut f)?;
        }
    }
    archive.finish()?;
    Ok(())
}

fn cli() -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(Command::new("build").about("Create uvd achive from source code"))
        .subcommand(
            Command::new("init").about("Init the uvd config").arg(
                Arg::new("interactive")
                    .long("interactive")
                    .short('i')
                    .required(false)
                    .action(ArgAction::SetTrue),
            ),
        )
        .subcommand(
            Command::new("completion")
                .about("Generate shell completion scripts")
                .arg(
                    clap::Arg::new("shell")
                        .action(ArgAction::Set)
                        .help("Shell to generate completion for")
                        .required(true),
                ),
        )
}
fn main() {
    let app = cli();
    let matches = app.clone().get_matches();
    match matches.subcommand() {
        Some(("init", sub)) => {
            let conf = if sub.get_flag("interactive") {
                let name = Text::new("Enter the name of the project:")
                    .prompt()
                    .expect("Failed to read input");
                let version = Text::new("Enter the version of the project:")
                    .prompt()
                    .expect("Failed to read input");
                let homepage = Text::new("Enter the homepage of the project:")
                    .prompt()
                    .expect("Failed to read input");
                let license = Text::new("Enter the license of the project:")
                    .prompt()
                    .expect("Failed to read input");
                let repository = Text::new("Enter the repository of the project:")
                    .prompt()
                    .expect("Failed to read input");
                let description = Text::new("Enter the description of the project:")
                    .prompt()
                    .expect("Failed to read input");
                Uvd {
                    name,
                    version,
                    homepage,
                    license,
                    repository,
                    description,
                    src: Vec::new(),
                    man: Vec::new(),
                }
            } else {
                Uvd {
                    name: String::new(),
                    version: String::new(),
                    homepage: String::new(),
                    license: String::new(),
                    repository: String::new(),
                    description: String::new(),
                    src: Vec::new(),
                    man: Vec::new(),
                }
            };
            let uvd = File::create("uvd.yml").expect("Failed to create uvd.yml");
            serde_yml::to_writer(uvd, &conf).expect("Failed to write uvd.yml");
            println!("uvd.yml created successfully.");
        }
        Some(("completion", sub)) => {
            let bin_name = sub
                .get_one::<String>("shell")
                .expect("bin_name is required");
            let serde_shell = match bin_name.as_str() {
                "bash" => Shell::Bash,
                "zsh" => Shell::Zsh,
                "fish" => Shell::Fish,
                _ => {
                    eprintln!("Unsupported shell: {}", bin_name);
                    std::process::exit(1);
                }
            };
            let mut cmd = cli();
            // On génère le  et on l'envoie sur la sortie standard (stdout)
            generate(serde_shell, &mut cmd, bin_name, &mut io::stdout());
        }
        Some(("build", _)) => {
            let conf = serde_yml::from_str::<Uvd>(
                &std::fs::read_to_string("uvd.yml").expect("Failed to read uvd.yml"),
            )
            .expect("Failed to parse uvd.yml");
            let chemin_archive = format!("{}_{}.tar.zst", conf.name, conf.version);
            match create_archive_zstd(conf.src, conf.man, &chemin_archive) {
                Ok(_) => println!("Archive created successfully: {chemin_archive}"),
                Err(e) => eprintln!("Error creating archive: {e}"),
            }
        }
        _ => app.clone().print_help().expect("failed to print help"),
    }
}
