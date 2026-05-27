use crate::db::{Db, ExtractionRecord};
use anyhow::Result;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

const ARCHIVE_EXTENSIONS: &[&str] = &[
    ".zip", ".rar", ".7z", ".tar.gz", ".tar.bz2", ".tar.xz", ".tgz",
];

const RAR_NUMERIC_EXTENSIONS: &[&str] = &[
    ".r00", ".r01", ".r02", ".r03", ".r04", ".r05", ".r06", ".r07", ".r08", ".r09",
    ".r10", ".r11", ".r12", ".r13", ".r14", ".r15", ".r16", ".r17", ".r18", ".r19",
    ".r20", ".r21", ".r22", ".r23", ".r24", ".r25", ".r26", ".r27", ".r28", ".r29",
    ".r30", ".r31", ".r32", ".r33", ".r34", ".r35", ".r36", ".r37", ".r38", ".r39",
    ".r40", ".r41", ".r42", ".r43", ".r44", ".r45", ".r46", ".r47", ".r48", ".r49",
    ".r50", ".r51", ".r52", ".r53", ".r54", ".r55", ".r56", ".r57", ".r58", ".r59",
    ".r60", ".r61", ".r62", ".r63", ".r64", ".r65", ".r66", ".r67", ".r68", ".r69",
    ".r70", ".r71", ".r72", ".r73", ".r74", ".r75", ".r76", ".r77", ".r78", ".r79",
    ".r80", ".r81", ".r82", ".r83", ".r84", ".r85", ".r86", ".r87", ".r88", ".r89",
    ".r90", ".r91", ".r92", ".r93", ".r94", ".r95", ".r96", ".r97", ".r98", ".r99",
];

const SAFETY_BLOCKLIST: &[&str] = &[
    ".mp4", ".mkv", ".avi", ".mov", ".wmv", ".flv", ".webm",
    ".mp3", ".flac", ".aac", ".wav", ".ogg",
    ".jpg", ".jpeg", ".png", ".gif", ".webp", ".heic",
    ".pdf", ".epub",
];

pub struct ExtractionManager {
    db: Db,
    has_7z: bool,
    has_unrar: bool,
}

impl ExtractionManager {
    pub fn new(db: Db) -> Result<Self> {
        let has_7z = which_exists("7z") || which_exists("7za");
        let has_unrar = which_exists("unrar");
        Ok(Self { db, has_7z, has_unrar })
    }

    pub async fn get_queue(&self) -> Result<Vec<ExtractionRecord>> {
        self.db.get_extractions()
    }

    pub async fn submit_password(&self, id: &str, password: &str) -> Result<()> {
        self.db.update_extraction_password(id, password)?;
        Ok(())
    }

    pub fn is_archive(path: &str) -> bool {
        let lower = path.to_lowercase();
        for ext in ARCHIVE_EXTENSIONS {
            if lower.ends_with(ext) {
                return !Self::is_blocked(path);
            }
        }
        for ext in RAR_NUMERIC_EXTENSIONS {
            if lower.ends_with(ext) {
                return !Self::is_blocked(path);
            }
        }
        false
    }

    fn is_blocked(path: &str) -> bool {
        let lower = path.to_lowercase();
        for ext in SAFETY_BLOCKLIST {
            if lower.ends_with(ext) {
                return true;
            }
        }
        false
    }

    pub async fn scan_and_queue(&self, torrent_id: &str, download_dir: &str) -> Result<Vec<String>> {
        let mut archives = Vec::new();
        let dir = Path::new(download_dir);

        if !dir.exists() {
            return Ok(archives);
        }

        let entries = std::fs::read_dir(dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            let path_str = path.to_string_lossy().to_string();

            if Self::is_archive(&path_str) {
                let id = uuid::Uuid::new_v4().to_string();
                let now = chrono::Utc::now().to_rfc3339();
                let output_dir = path
                    .parent()
                    .unwrap_or(dir)
                    .join(path.file_stem().unwrap_or_default())
                    .to_string_lossy()
                    .to_string();

                let record = ExtractionRecord {
                    id: id.clone(),
                    torrent_id: torrent_id.to_string(),
                    archive_path: path_str.clone(),
                    output_dir,
                    status: "queued".to_string(),
                    progress: 0.0,
                    password: None,
                    error_message: None,
                    started_at: now,
                    completed_at: None,
                };

                self.db.insert_extraction(&record)?;
                archives.push(id);
            }
        }

        Ok(archives)
    }

    pub async fn process_queue(&self) -> Result<()> {
        let queue = self.db.get_extractions()?;
        for record in queue {
            if record.status == "queued" || (record.status == "needs_password" && record.password.is_some()) {
                self.extract_one(&record).await?;
            }
        }
        Ok(())
    }

    async fn extract_one(&self, record: &ExtractionRecord) -> Result<()> {
        self.db.update_extraction_status(&record.id, "processing", 0.0)?;

        let output_dir = if record.output_dir.is_empty() {
            Path::new(&record.archive_path)
                .parent()
                .unwrap_or(Path::new("."))
                .to_string_lossy()
                .to_string()
        } else {
            record.output_dir.clone()
        };

        std::fs::create_dir_all(&output_dir).ok();

        let lower = record.archive_path.to_lowercase();
        let result = if (lower.ends_with(".zip") || lower.ends_with(".7z") || lower.ends_with(".rar")
            || lower.ends_with(".tar.gz") || lower.ends_with(".tar.bz2") || lower.ends_with(".tar.xz")
            || lower.ends_with(".tgz"))
            && self.has_7z
        {
            self.extract_with_7z(&record.archive_path, &output_dir, record.password.as_deref()).await
        } else if (lower.ends_with(".rar") || RAR_NUMERIC_EXTENSIONS.iter().any(|e| lower.ends_with(e)))
            && self.has_unrar
        {
            self.extract_with_unrar(&record.archive_path, &output_dir, record.password.as_deref()).await
        } else if lower.ends_with(".zip") {
            self.extract_zip_rust(&record.archive_path, &output_dir).await
        } else if lower.ends_with(".tar.gz") || lower.ends_with(".tgz") {
            self.extract_tar_gz_rust(&record.archive_path, &output_dir).await
        } else if lower.ends_with(".tar.bz2") {
            self.extract_tar_bz2_rust(&record.archive_path, &output_dir).await
        } else if lower.ends_with(".tar.xz") {
            self.extract_tar_xz_rust(&record.archive_path, &output_dir).await
        } else {
            Err(anyhow::anyhow!("no suitable extraction tool found for: {}", record.archive_path))
        };

        match result {
            Ok(_) => {
                self.db.update_extraction_complete(&record.id)?;
            }
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("password") || msg.contains("encrypted") || msg.contains("Wrong password") {
                    self.db.update_extraction_status(&record.id, "needs_password", 0.0)?;
                } else {
                    self.db.update_extraction_error(&record.id, &msg)?;
                }
            }
        }

        Ok(())
    }

    async fn extract_with_7z(&self, archive: &str, output: &str, password: Option<&str>) -> Result<()> {
        let mut cmd = Command::new("7z");
        cmd.arg("x").arg("-y").arg(format!("-o{}", output));
        if let Some(pw) = password {
            cmd.arg(format!("-p{}", pw));
        }
        cmd.arg(archive);
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        let output_result = cmd.output().await?;
        if !output_result.status.success() {
            let stderr = String::from_utf8_lossy(&output_result.stderr);
            return Err(anyhow::anyhow!("7z failed: {}", stderr));
        }
        Ok(())
    }

    async fn extract_with_unrar(&self, archive: &str, output: &str, password: Option<&str>) -> Result<()> {
        let mut cmd = Command::new("unrar");
        cmd.arg("x").arg("-o+");
        if let Some(pw) = password {
            cmd.arg(format!("-p{}", pw));
        } else {
            cmd.arg("-p-");
        }
        cmd.arg(archive).arg(output);
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        let output_result = cmd.output().await?;
        if !output_result.status.success() {
            let stderr = String::from_utf8_lossy(&output_result.stderr);
            return Err(anyhow::anyhow!("unrar failed: {}", stderr));
        }
        Ok(())
    }

    async fn extract_zip_rust(&self, archive: &str, output: &str) -> Result<()> {
        let archive = archive.to_string();
        let output = output.to_string();
        tokio::task::spawn_blocking(move || -> Result<()> {
            let file = std::fs::File::open(&archive)?;
            let mut zip = zip::ZipArchive::new(file)?;
            for i in 0..zip.len() {
                let mut entry = zip.by_index(i)?;
                let outpath = Path::new(&output).join(entry.mangled_name());
                if entry.is_dir() {
                    std::fs::create_dir_all(&outpath)?;
                } else {
                    if let Some(parent) = outpath.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    let mut outfile = std::fs::File::create(&outpath)?;
                    std::io::copy(&mut entry, &mut outfile)?;
                }
            }
            Ok(())
        })
        .await??;
        Ok(())
    }

    async fn extract_tar_gz_rust(&self, archive: &str, output: &str) -> Result<()> {
        let archive = archive.to_string();
        let output = output.to_string();
        tokio::task::spawn_blocking(move || -> Result<()> {
            let file = std::fs::File::open(&archive)?;
            let decoder = flate2::read::GzDecoder::new(file);
            let mut archive = tar::Archive::new(decoder);
            archive.unpack(&output)?;
            Ok(())
        })
        .await??;
        Ok(())
    }

    async fn extract_tar_bz2_rust(&self, archive: &str, output: &str) -> Result<()> {
        let archive = archive.to_string();
        let output = output.to_string();
        tokio::task::spawn_blocking(move || -> Result<()> {
            let file = std::fs::File::open(&archive)?;
            let decoder = bzip2::read::BzDecoder::new(file);
            let mut archive = tar::Archive::new(decoder);
            archive.unpack(&output)?;
            Ok(())
        })
        .await??;
        Ok(())
    }

    async fn extract_tar_xz_rust(&self, archive: &str, output: &str) -> Result<()> {
        let archive = archive.to_string();
        let output = output.to_string();
        tokio::task::spawn_blocking(move || -> Result<()> {
            let file = std::fs::File::open(&archive)?;
            let decoder = xz2::read::XzDecoder::new(file);
            let mut archive = tar::Archive::new(decoder);
            archive.unpack(&output)?;
            Ok(())
        })
        .await??;
        Ok(())
    }
}

fn which_exists(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_archive() {
        assert!(ExtractionManager::is_archive("file.zip"));
        assert!(ExtractionManager::is_archive("file.rar"));
        assert!(ExtractionManager::is_archive("file.7z"));
        assert!(ExtractionManager::is_archive("file.tar.gz"));
        assert!(ExtractionManager::is_archive("file.r00"));
        assert!(ExtractionManager::is_archive("file.r99"));
        assert!(!ExtractionManager::is_archive("file.mp4"));
        assert!(!ExtractionManager::is_archive("file.txt"));
        assert!(!ExtractionManager::is_archive("file.jpg"));
    }

    #[test]
    fn test_safety_blocklist() {
        assert!(!ExtractionManager::is_archive("movie.mp4"));
        assert!(!ExtractionManager::is_archive("song.mp3"));
        assert!(!ExtractionManager::is_archive("image.jpg"));
        assert!(!ExtractionManager::is_archive("document.pdf"));
    }
}
