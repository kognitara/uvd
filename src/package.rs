use crate::{
    ko,
    manifest::{DEVELOPER_FILENAME, generate_developer_json},
    ok,
};
use serde::{Deserialize, Serialize};
use std::{fs::File, path::Path};
use tar::Builder;
use unic_langid::LanguageIdentifier;
use zstd::Encoder;

#[derive(Serialize, Clone, Default, Deserialize)]
pub struct Package {
    name: String,
    version: String,
    description: String,
    packager: String,
    license: String,
    src: Vec<String>,
    readme: String,
    man: Vec<String>,
}

impl Package {
    pub fn archive(&mut self, lang: &LanguageIdentifier) -> bool {
        let archive_name = format!("{}_{}.tar.gz", self.name, self.version);
        let tmp_path = format!("/tmp/{}", archive_name);

        let content = generate_developer_json(&self.src);
        std::fs::write(DEVELOPER_FILENAME, content.as_str()).expect(DEVELOPER_FILENAME);
        let status = std::process::Command::new("gpg")
            .args([
                "--batch",
                "--yes",
                "--detach-sign",
                "--armor",
                DEVELOPER_FILENAME,
            ])
            .status()
            .expect("failed to execute gpg");

        if !status.success() {
            ko(lang, "failed-to-sign-check-your-gpg-keys");
            std::fs::remove_file(DEVELOPER_FILENAME).ok();
            return false;
        }
        {
            let archive =
                File::create(Path::new(&tmp_path)).expect("failed to create temp archive");

            let zstd_encoder = Encoder::new(archive, 19).expect("failed to encode");
            let mut tar = Builder::new(zstd_encoder);
            let mut header = tar::Header::new_gnu();
            header.set_size(content.len() as u64);
            header.set_mode(0o644);
            tar.append_data(&mut header, DEVELOPER_FILENAME, content.as_bytes())
                .expect("failed to add manifest");
            let signature = std::fs::read(format!("{DEVELOPER_FILENAME}.asc"))
                .expect("failed to read signature");

            let mut sig_header = tar::Header::new_gnu();
            sig_header.set_size(signature.len() as u64);
            sig_header.set_mode(0o644);
            tar.append_data(
                &mut sig_header,
                format!("{DEVELOPER_FILENAME}.asc").as_str(),
                signature.as_slice(),
            )
            .expect("failed to add signature to archive");

            for item in &self.src {
                let path = Path::new(item);
                if path.exists() {
                    if path.is_dir() {
                        tar.append_dir_all(item, item).expect("failed to add dir");
                    } else {
                        tar.append_path(item).expect("failed to add file");
                    }
                }
            }
            tar.finish().expect("failed to finish tar");
            let encoder = tar.into_inner().expect("failed to get encoder");
            encoder.finish().expect("failed to finish zstd");
        }
        std::fs::remove_file(DEVELOPER_FILENAME).ok();
        std::fs::remove_file(format!("{DEVELOPER_FILENAME}.asc")).ok();
        std::fs::copy(tmp_path.as_str(), archive_name.as_str()).expect("failed to copy");
        std::fs::remove_file(&tmp_path).expect("failed to remove tempfile");
        ok(lang, "developer-generated-successfully");
        true
    }
}
