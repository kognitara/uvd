use crossterm::cursor::Hide;
use crossterm::cursor::Show;
use crossterm::execute;
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;
use serde::Serialize;
use std::fs::File;
use std::io::Read;
use std::io::stdout;

#[doc = "File containing all checksums of the developer's source code"]
pub const DEVELOPER_FILENAME: &str = "developer.json";

#[derive(Serialize, Deserialize)]
struct Manifest {
    files: Vec<FileEntry>,
}

#[derive(Serialize, Deserialize)]
struct FileEntry {
    path: String,
    hash: String,
}

#[doc = "generate the developer.json"]
pub fn generate_developer_json(src_list: &Vec<String>) -> String {
    execute!(stdout(), Hide).expect("failed to hide cursor");
    let mut files = Vec::new();
    let pb = ProgressBar::new(src_list.len() as u64);
    pb.set_style(
        ProgressStyle::with_template("{spinner:.white} [{bar}] {msg}")
            .unwrap()
            .progress_chars("=> "),
    );

    for item in src_list {
        let path = std::path::Path::new(item.as_str());
        if path.is_file() {
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
            files.push(FileEntry {
                path: item.to_string(),
                hash: hasher.finalize().to_hex().to_string(),
            });
            pb.inc(1);
        }
    }
    pb.finish_with_message("generated");
    execute!(stdout(), Show).expect("failed to show cursor");
    serde_json::to_string_pretty(&Manifest { files }).expect("failed to serialize")
}
