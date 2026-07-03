use std::fs::File;
use std::io::Write;
use std::path::Path;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize)]
pub struct Os {
    pub name: String,
    pub version: String,
    pub architecture: String,
}

#[derive(Serialize, Deserialize)]
pub struct Disk {
    pub id: Uuid,
    pub size: u64,
    pub verified: bool,
    pub os: Vec<Os>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a sample disk structure
    let disk = Disk {
        id: Uuid::new_v4(),
        size: 10737418240, // 10 GB
        verified: false,
        os: vec![
            Os {
                name: "linux".to_string(),
                version: "6.1.0".to_string(),
                architecture: "x86_64".to_string(),
            },
            Os {
                name: "linux".to_string(),
                version: "6.1.0".to_string(),
                architecture: "aarch64".to_string(),
            },
        ],
    };

    // Serialize to JSON
    let json_string = serde_json::to_string_pretty(&disk)?;
    
    // Create temporary JSON file
    let json_path = Path::new("disk.json");
    let mut json_file = File::create(json_path)?;
    json_file.write_all(json_string.as_bytes())?;
    
    println!("Created disk.json");
    println!("{}", json_string);

    // Create ZIP archive
    let file = File::create("uvd.uvd")?;
    let mut zip = zip::ZipWriter::new(file);
    
    let options = zip::write::FileOptions::default();
    zip.start_file("disk.json", options)?;
    zip.write_all(json_string.as_bytes())?;
    zip.finish()?;
    
    // Clean up temporary JSON file
    std::fs::remove_file(json_path)?;
    
    println!("Created uvd.uvd archive");
    Ok(())
}
