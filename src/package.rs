use serde::{Deserialize, Serialize};
use std::{fs::File, path::Path};
use tar::Builder;
use zstd::Encoder;

use crate::manifest::generate_manifest;

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
    pub fn archive(&mut self) -> bool {
        let archive_name = format!("{}_{}.tar.gz", self.name, self.version);
        let tmp_path = format!("/tmp/{}", archive_name);

        {
            // 1. Création du fichier
            let archive =
                File::create(Path::new(&tmp_path)).expect("failed to create temp archive");

            // 2. Création de l'encodeur Zstd
            let zstd_encoder = Encoder::new(archive, 19).expect("failed to encode");

            // 3. Création du Builder
            let mut tar = Builder::new(zstd_encoder);
            let manifest_path = "manifest.json";
            let content = generate_manifest(&self.src);
            std::fs::write(manifest_path, content.as_str()).expect("failed to write manifest.json");
            // Exécuter GPG pour générer la signature détachée
            std::process::Command::new("gpg")
                .args([
                    "--batch",
                    "--yes",
                    "--detach-sign",
                    "--armor",
                    "manifest.json",
                ])
                .status()
                .expect("failed to sign manifest");
            let mut header = tar::Header::new_gnu();
            header.set_size(content.len() as u64);
            header.set_mode(0o644);
            tar.append_data(&mut header, manifest_path, content.as_bytes())
                .expect("failed to add manifest");
            let signature = std::fs::read("manifest.json.asc").expect("failed to read signature");

            let mut sig_header = tar::Header::new_gnu();
            sig_header.set_size(signature.len() as u64);
            sig_header.set_mode(0o644);
            tar.append_data(&mut sig_header, "manifest.json.asc", signature.as_slice())
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

            // 4. TRÈS IMPORTANT : Finaliser le tar manuellement
            tar.finish().expect("failed to finish tar");

            // 5. Récupérer l'encodeur pour le fermer proprement
            let encoder = tar.into_inner().expect("failed to get encoder");
            encoder.finish().expect("failed to finish zstd");
        } // <-- Ici, tout est fermé et flushé sur le disque
        std::fs::remove_file("manifest.json").ok();
        std::fs::remove_file("manifest.json.asc").ok();
        // Maintenant, le fichier est complet, on peut copier
        std::fs::copy(&tmp_path, &archive_name).expect("failed to copy");
        std::fs::remove_file(&tmp_path).expect("failed to remove temp");
        true
    }
}
