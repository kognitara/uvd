use std::env::consts::{ARCH, OS};
use std::fs::File;
use std::path::Path;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const UVD_ARCH: [&str; 4] = ["x86_64", "arm", "aarch64", "riscv64"];

pub const UVD_OS: &[(&str, [&str; 4])] = &[
    ("linux", UVD_ARCH),
    ("aix", UVD_ARCH),
    ("android", UVD_ARCH),
    ("apple", UVD_ARCH),
    ("freebsd", UVD_ARCH),
    ("netbsd", UVD_ARCH),
    ("openbsd", UVD_ARCH),
    ("dragonfly", UVD_ARCH),
    ("solaris", UVD_ARCH),
    ("illumos", UVD_ARCH),
    ("redox", UVD_ARCH),
    ("haiku", UVD_ARCH),
];

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

pub fn disk_os_valid(disk: &Disk) -> bool {
    let host_os = normalized_host_os();

    disk.os.iter().any(|os| {
        UVD_OS
            .iter()
            .any(|(name, arch)| name == &host_os && os.name == host_os && arch.contains(&ARCH))
    })
}

fn normalized_host_os() -> &'static str {
    OS
}

/// Reads a Disk struct from a UVD archive (ZIP format)
///
/// # Arguments
/// * `path` - Path to the .uvd file (ZIP archive containing a disk.json)
///
/// # Returns
/// * `Result<Disk, Box<dyn std::error::Error>>` - The deserialized Disk or an error
///
/// # Example
/// ```no_run
/// let disk = read_disk_from_uvd("path/to/image.uvd")?;
/// ```
pub fn read_disk_from_uvd<P: AsRef<Path>>(path: P) -> Result<Disk, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    // Find and read disk.json from the archive
    let mut disk_file = archive.by_name("disk.json")?;
    let mut contents = String::new();
    std::io::Read::read_to_string(&mut disk_file, &mut contents)?;

    // Deserialize JSON into Disk struct
    let disk: Disk = serde_json::from_str(&contents)?;
    Ok(disk)
}

#[cfg(test)]
mod test {
    use std::env::consts::ARCH;

    #[test]
    pub fn test_disk_os_valid() {
        let valid = super::Disk {
            id: uuid::Uuid::new_v4(),
            size: 64,
            verified: true,
            os: vec![super::Os {
                name: super::normalized_host_os().to_string(),
                version: "1.0".to_string(),
                architecture: ARCH.to_string(),
            }],
        };
        assert!(super::read_disk_from_uvd("uvd.uvd").is_ok());
        assert!(super::disk_os_valid(&valid));
    }
}
