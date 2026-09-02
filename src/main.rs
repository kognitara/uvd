use clap::{ArgAction, Command};
use clap_complete::{Shell, generate};
use serde::{Deserialize, Serialize};
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
use std::fs::File;
use std::io::{self, BufWriter};
use std::path::Path;
use tar::Builder;
use zstd::Encoder;

fn create_archive_zstd(chemins: Vec<String>, chemin_archive: &str) -> io::Result<()> {
    let fichier_sortie = File::create(chemin_archive)?;
    let buffer_sortie = BufWriter::new(fichier_sortie);
    let encodeur_zstd = Encoder::new(buffer_sortie, 3)?.auto_finish();
    let mut archive = Builder::new(encodeur_zstd);
    for chemin_str in chemins {
        let chemin = Path::new(&chemin_str);
        if !chemin.exists() {
            continue;
        }
        if chemin.is_dir() {
            archive.append_dir_all(&chemin, chemin)?;
        } else if chemin.is_file() {
            let mut fichier = File::open(chemin)?;
            archive.append_file(chemin, &mut fichier)?;
        }
    }
    archive.finish()?;
    Ok(())
}

fn cli() -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(Command::new("build").about("Create uvd achive from source code"))
        .subcommand(Command::new("init").about("Init the uvd config"))
        .subcommand(
            Command::new("generate")
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
        Some(("init", _)) => {
            let conf = Uvd {
                name: String::new(),
                version: String::new(),
                homepage: String::new(),
                license: String::new(),
                repository: String::new(),
                description: String::new(),
                src: Vec::new(),
                man: Vec::new(),
            };

            let uvd = File::create("uvd.yml").expect("Failed to create uvd.yml");
            serde_yml::to_writer(uvd, &conf).expect("Failed to write uvd.yml");
            println!("uvd.yml created successfully.");
        }
        Some(("generate", sub)) => {
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
            match create_archive_zstd(conf.src, &chemin_archive) {
                Ok(_) => println!("Archive created successfully: {}", chemin_archive),
                Err(e) => eprintln!("Error creating archive: {}", e),
            }
        }
        _ => app.clone().print_help().expect("failed to print help"),
    }
}
