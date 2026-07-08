use crossterm::cursor::Hide;
use crossterm::cursor::Show;
use crossterm::execute;
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::fs::File;
use std::fs::create_dir_all;
use std::io::Read;
use std::io::stdout;
use std::path::Path;
use std::process::ExitCode;
use tar::Archive;
use unic_langid::LanguageIdentifier;
use zstd::Decoder;

use crate::ko;
use crate::ok;

#[doc = "File containing all checksums of the developer's source code"]
pub const DEVELOPER_FILENAME: &str = "developer.json";

#[doc = "reviewer file"]
pub const REVIEWER_FILENAME: &str = "reviewer.json";

#[doc = "manager file"]
pub const MANAGER_FILENAME: &str = "manager.json";

#[derive(Serialize, Deserialize)]
struct Manifest {
    files: Vec<FileEntry>,
}

#[derive(Serialize, Deserialize)]
struct FileEntry {
    path: String,
    hash: String,
}
pub fn extract_trust_chain(
    lang: &LanguageIdentifier,
    archive: &Path,
    dest: &Path,
    filename: &str,
    end_success_message_key: &str,
) -> ExitCode {
    if archive.exists().eq(&false) {
        ko(lang, "archive-not-exists");
        return ExitCode::FAILURE;
    }
    if dest.exists().eq(&false)
        && dest.file_name().expect("") != "."
        && dest.file_name().expect("") != ".."
    {
        create_dir_all(dest).expect("failed to create dest directory");
    }

    let file = File::open(archive).expect("failed to open archive");
    let zstd_decoder = Decoder::new(file).expect("failed to decode the flux");
    let mut archive = Archive::new(zstd_decoder);
    for entry_result in archive.entries().expect("failed to get content") {
        let mut entry = entry_result.expect("failed to get file");
        let path = entry.path().expect("failed to get the path").to_path_buf();

        if path
            .file_name()
            .expect("")
            .to_str()
            .expect("")
            .starts_with(filename)
        {
            let destination = dest.canonicalize().expect("").join(path);
            entry.unpack(destination).expect("failed to get file");
        }
    }
    ok(lang, end_success_message_key);
    ExitCode::SUCCESS
}

fn collect_files(path: &std::path::Path, files: &mut Vec<String>) {
    if path.is_file() {
        files.push(path.to_str().unwrap().to_string());
    } else if path.is_dir()
        && let Ok(entries) = std::fs::read_dir(path)
    {
        for entry in entries.flatten() {
            collect_files(&entry.path(), files);
        }
    }
}

#[doc = "generate the developer.json"]
pub fn generate_developer_json(src_list: &Vec<String>) -> String {
    if Path::new(DEVELOPER_FILENAME).is_file() {
        fs::remove_file(DEVELOPER_FILENAME).expect("Failed to remove file");
    }
    execute!(stdout(), Hide).expect("failed to hide cursor");
    let mut file_entries = Vec::new();

    let mut all_files = Vec::new();
    for item in src_list {
        collect_files(std::path::Path::new(item.as_str()), &mut all_files);
    }

    let pb = ProgressBar::new(all_files.len() as u64);
    pb.set_style(
        ProgressStyle::with_template("{spinner:.white} [{bar}] {msg}")
            .unwrap()
            .progress_chars("=> "),
    );

    // 2. On boucle sur tous les fichiers trouvés
    for item in all_files {
        let path = std::path::Path::new(item.as_str());
        let name = path.file_name().expect("").to_str().expect("");
        pb.set_message(name.to_string());

        let mut file = File::open(path).expect("failed to open file");
        let mut hasher = blake3::Hasher::new();
        let mut buffer = [0; 65536];
        while let Ok(n) = file.read(&mut buffer) {
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }

        file_entries.push(FileEntry {
            path: item,
            hash: hasher.finalize().to_hex().to_string(),
        });
        pb.inc(1);
    }

    pb.finish_with_message("generated");
    execute!(stdout(), Show).expect("failed to show cursor");
    serde_json::to_string_pretty(&Manifest {
        files: file_entries,
    })
    .expect("failed to serialize")
}
