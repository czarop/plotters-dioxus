use anyhow::anyhow;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileError {
    #[error("Invalid directory: {dir:?})")]
    InvalidDirectory { dir: String },
}

#[derive(PartialEq, Clone)]
pub struct FcsSampleStub {
    pub name: String,
    pub full_path: PathBuf,
}

#[derive(PartialEq, Clone)]
pub struct FcsFiles {
    directory: PathBuf,
    file_list: Vec<FcsSampleStub>,
}

impl FcsFiles {
    pub fn create(path: &str) -> anyhow::Result<Self> {
        let buf = PathBuf::from(path);

        let all_files = fs::read_dir(&buf).map_err(|_| anyhow!("Invalid directory: {}", path))?;

        let files: Vec<FcsSampleStub> = all_files
            .map(|entry| {
                let entry = entry?;
                let name = entry.file_name();
                let name_str = name
                    .to_str()
                    .ok_or_else(|| anyhow!("Invalid UTF-8 in filename"))?;
                if name_str.ends_with(".fcs") {
                    let full_path = buf.join(&name_str);
                    Ok(Some(FcsSampleStub{
                        name: name_str.to_string(),
                        full_path: full_path
                    }))
                } else {
                    Ok(None)
                }
            })
            .collect::<anyhow::Result<Vec<Option<FcsSampleStub>>>>()?
            .into_iter()
            .flatten() // Removes the Nones, leaving just the Strings
            .collect();

        return Ok(FcsFiles {
            directory: buf,
            file_list: files,
        });
    }

    pub fn file_list(&self) -> &[FcsSampleStub] {
        return &self.file_list;
    }

    pub fn directory_path(&self) -> &str {
        return self.directory.to_str().unwrap_or("");
    }

    pub fn sample_count(&self) -> usize {
        self.file_list.len()
    }
}
