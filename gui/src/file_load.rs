use anyhow::anyhow;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileError {
    #[error("Invalid directory: {dir:?})")]
    InvalidDirectory { dir: String },
}

pub struct Fcs_Files {
    directory: PathBuf,
    file_list: Vec<String>,
}

impl Fcs_Files {
    pub fn create(self, path: &str) -> anyhow::Result<Self> {
        let buf = PathBuf::from(path);

        let all_files = fs::read_dir(&buf).map_err(|_| anyhow!("Invalid directory: {}", path))?;

        let files: Vec<String> = all_files
            .filter_map(|f| match f {
                Ok(v) => match v.file_name().to_str() {
                    Some(name) => {
                        if name.ends_with(".fcs") {
                            return Some(name.to_string());
                        } else {
                            return None;
                        }
                    }
                    None => return None,
                },
                Err(_) => return None,
            })
            .collect();

        return Ok(Fcs_Files {
            directory: buf,
            file_list: files,
        });
    }

    pub fn file_list(&self) -> &[String] {
        return &self.file_list;
    }

    pub fn directory_path(&self) -> &str {
        return self.directory.to_str().unwrap_or("");
    }
}
