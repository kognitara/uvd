pub mod images;

use std::{path::Path, process::ExitCode};

pub fn verify_disk_image(image_path: &str) -> ExitCode {
    // Placeholder for the actual verification logic
    println!("Verifying disk image at path: {image_path}");
    if Path::new(image_path).exists()
        && Path::new(image_path).is_file()
        && Path::new(image_path)
            .extension()
            .map_or(false, |ext| ext == "uvd")
    {
        println!("Disk image exists. Verification successful.");
        ExitCode::SUCCESS
    } else {
        eprintln!("Disk image does not exist or is not a file. Verification failed.");
        ExitCode::FAILURE
    }
}

pub fn install_disk_image(image: &str) -> ExitCode {
    // Placeholder for the actual installation logic
    println!("Installing disk image: {image}");

    ExitCode::SUCCESS
}

#[cfg(test)]
mod test {

    #[test]
    pub fn test_verify_disk_image() {
        assert_eq!(
            super::verify_disk_image("uvd.uvd"),
            std::process::ExitCode::SUCCESS
        );
        assert_eq!(
            super::verify_disk_image("non_existent_image.uvd"),
            std::process::ExitCode::FAILURE
        );
        assert_eq!(
            super::verify_disk_image("not_a_disk_image.txt"),
            std::process::ExitCode::FAILURE
        );
    }
}
