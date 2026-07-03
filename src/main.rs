mod cli;

use std::process::ExitCode;

use clap::{Arg, ArgAction, Command};

fn cli() -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(
            Command::new("verify").about("Verify a disk image").arg(
                Arg::new("image")
                    .short('i')
                    .help("Path to the disk image")
                    .required(true)
                    .action(ArgAction::Set),
            ),
        )
        .subcommand(
            Command::new("install").arg(
                Arg::new("image")
                    .short('i')
                    .required(true)
                    .action(ArgAction::Set),
            ),
        )
}
fn main() -> ExitCode {
    let app = cli().get_matches();

    match app.subcommand() {
        Some(("verify", sub)) => {
            let image_path = sub.get_one::<String>("image").unwrap();
            
            match cli::images::read_disk_from_uvd(image_path) {
                Ok(disk) => {
                    if cli::images::disk_os_valid(&disk) {
                        println!("✓ Disk image verified successfully");
                        println!("  ID: {}", disk.id);
                        println!("  Size: {} bytes", disk.size);
                        println!("  Verified: {}", disk.verified);
                        ExitCode::SUCCESS
                    } else {
                        eprintln!("✗ Disk OS is not compatible with this system");
                        ExitCode::FAILURE
                    }
                }
                Err(e) => {
                    eprintln!("✗ Failed to read disk image: {}", e);
                    ExitCode::FAILURE
                }
            }
        }
        Some(("install", sub)) => {
            let image = sub.get_one::<String>("image").unwrap();
            cli::install_disk_image(image)
        }
        _ => {
            cli().print_help().unwrap();
            ExitCode::FAILURE
        }
    }
}
