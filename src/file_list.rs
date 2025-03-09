use std::path::{Path, PathBuf};

/// Information to track/represent files
#[derive(Clone, Debug, PartialEq)]
pub struct FileEntry {
    pub path: std::path::PathBuf,
    pub last_mod: std::time::SystemTime,
    pub is_dir: bool,
}

impl std::fmt::Display for FileEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            self.path
                .as_os_str()
                .to_str()
                .expect("Error converting path to &str"),
            if self.is_dir { "/" } else { "" }
        )
    }
}

type FileEntries = Vec<FileEntry>;

/// Contains recursive list of files under root path
pub struct FileList {
    root: std::path::PathBuf,
    entries: FileEntries,
}

impl FileList {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            entries: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.entries.clear()
    }

    pub async fn get_inbox_entries(&mut self, sort: bool) -> std::io::Result<FileEntries> {
        self.get_file_entries(&self.root.clone(), sort).await
    }

    async fn get_file_entries(
        &mut self,
        curr_path: &PathBuf,
        sort: bool,
    ) -> std::io::Result<FileEntries> {
        let mut read_dir = tokio::fs::read_dir(curr_path).await?;
        while let Some(entry) = read_dir.next_entry().await? {
            let path = entry.path();

            let last_mod = path
                .metadata()
                .expect("Failed to get metadata")
                .modified()
                .expect("Failed to get file modification time");

            self.entries.push(FileEntry {
                path: path
                    .strip_prefix(&self.root)
                    .expect("Failed to generate relative path")
                    .to_path_buf(),
                last_mod,
                is_dir: path.is_dir(),
            });

            if path.is_dir() {
                Box::pin(self.get_file_entries(&path, false)).await?;
            }
        }

        // TODO: optimize to remove 'clone() - this could get expensive
        if sort {
            self.entries.sort_by_key(|f| f.path.clone());
        }

        Ok(self.entries.clone())
    }
}
