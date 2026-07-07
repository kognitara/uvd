use serde::{Deserialize, Serialize};
use std::{fs::File, path::Path};
use tar::Builder;
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

        // Maintenant, le fichier est complet, on peut copier
        std::fs::copy(&tmp_path, &archive_name).expect("failed to copy");
        std::fs::remove_file(&tmp_path).expect("failed to remove temp");
        true
    }
}
