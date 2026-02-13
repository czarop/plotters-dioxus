use anyhow::anyhow;
use flow_fcs::keyword::StringableKeyword;
use flow_fcs::parameter::ParameterBuilder;
use flow_fcs::{Header, Metadata, Parameter, ParameterMap, TransformType};
use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

use anyhow::Result;

#[derive(Error, Debug)]
pub enum FileError {
    #[error("Invalid directory: {dir:?})")]
    InvalidDirectory { dir: String },
}

// #[derive(PartialEq, Clone)]
// pub struct FcsSampleStub {
//     pub name: String,
//     pub full_path: PathBuf,
// }

// impl std::fmt::Display for FcsSampleStub {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self.name)
//     }
// }

#[derive(PartialEq, Clone)]
pub struct FcsFiles {
    directory: PathBuf,
    file_list: Vec<FcsSampleStub>,
}

impl FcsFiles {
    pub fn create(path: &str) -> Result<Self> {
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
                    match FcsSampleStub::open(full_path.to_str().unwrap_or_default()) {
                        Ok(s) => Ok(Some(s)),
                        Err(e) => {
                            println!("error in file {e}");
                            Ok(None)
                        }
                    }
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

    pub fn get_file_names(&self) -> Vec<String> {
        return self
            .file_list
            .iter()
            .map(|f| match f.get_fil_keyword() {
                Ok(n) => n.to_string(),
                Err(_) => f.get_filepath().to_string_lossy().to_string(),
            })
            .collect();
    }

    pub fn directory_path(&self) -> &str {
        return self.directory.to_str().unwrap_or("");
    }

    pub fn sample_count(&self) -> usize {
        self.file_list.len()
    }
}

#[derive(Debug, Clone)]
pub struct FcsSampleStub {
    /// The header segment of the fcs file, including the version, and byte offsets to the text, data, and analysis segments
    pub header: Header,
    /// The metadata segment of the fcs file, including the delimiter, and a hashmap of keyword/value pairs
    pub metadata: Metadata,
    /// A hashmap of the parameter names and their associated metadata
    pub parameters: ParameterMap,

    pub filepath: PathBuf,
}

impl PartialEq for FcsSampleStub {
    fn eq(&self, other: &Self) -> bool {
        self.get_guid().expect("should be a guid") == other.get_guid().expect("should be a guid")
    }
}

impl FcsSampleStub {
    pub fn new() -> Result<Self> {
        Ok(Self {
            header: Header::new(),
            metadata: Metadata::new(),
            parameters: ParameterMap::default(),
            filepath: PathBuf::new(),
        })
    }

    pub fn open(path: &str) -> Result<Self> {
        // Attempt to open the file path
        let file_access = flow_fcs::file::AccessWrapper::new(path)
            .expect("Should be able make new access wrapper");

        // Validate the file extension
        Self::validate_fcs_extension(&file_access.path)
            .expect("Should have a valid file extension");

        // Create header and metadata structs from a memory map of the file
        let header = Header::from_mmap(&file_access.mmap)
            .expect("Should be able to create header from mmap");
        let mut metadata = Metadata::from_mmap(&file_access.mmap, &header);

        metadata
            .validate_text_segment_keywords(&header)
            .expect("Should have valid text segment keywords");
        metadata.validate_guid();

        let fcs = Self {
            parameters: Self::generate_parameter_map(&metadata)
                .expect("Should be able to generate parameter map"),
            header,
            metadata,
            filepath: PathBuf::from(path),
        };

        Ok(fcs)
    }

    /// Validates that the file extension is `.fcs`
    /// # Errors
    /// Will return `Err` if the file extension is not `.fcs`
    fn validate_fcs_extension(path: &Path) -> Result<()> {
        let extension = path
            .extension()
            .ok_or_else(|| anyhow!("File has no extension"))?
            .to_str()
            .ok_or_else(|| anyhow!("File extension is not valid UTF-8"))?;

        if extension.to_lowercase() != "fcs" {
            return Err(anyhow!("Invalid file extension: {}", extension));
        }

        Ok(())
    }

    pub fn get_filepath(&self) -> &Path {
        &self.filepath
    }

    // pub fn get_sample_name(&self) -> Result<&str> {
    //     if let Ok(flow_fcs::keyword::StringKeyword::FIL(d)) = self.metadata.get_string_keyword("$FIL"){
    //         return Ok(d)
    //     } else {
    //         return Err(anyhow!("could not find sample name in metadata"))
    //     }
    // }

    pub fn find_parameter(&self, parameter_name: &str) -> Result<&Parameter> {
        // Try exact match first (fast path)
        if let Some(param) = self.parameters.get(parameter_name) {
            return Ok(param);
        }

        // Case-insensitive fallback: search through parameter map
        for (key, param) in self.parameters.iter() {
            if key.eq_ignore_ascii_case(parameter_name) {
                return Ok(param);
            }
        }

        Err(anyhow!("Parameter not found: {parameter_name}"))
    }

    /// Looks for the parameter name as a key in the `parameters` hashmap and returns a mutable reference to it
    /// Performs case-insensitive lookup for parameter names
    /// # Errors
    /// Will return `Err` if the parameter name is not found in the `parameters` hashmap
    pub fn find_mutable_parameter(&mut self, parameter_name: &str) -> Result<&mut Parameter> {
        // Try exact match first (fast path)
        // Note: We need to check if the key exists as Arc<str>, so we iterate to find exact match
        let exact_key = self
            .parameters
            .keys()
            .find(|k| k.as_ref() == parameter_name)
            .map(|k| k.clone());

        if let Some(key) = exact_key {
            return self
                .parameters
                .get_mut(&key)
                .ok_or_else(|| anyhow!("Parameter not found: {parameter_name}"));
        }

        // Case-insensitive fallback: find the key first (clone Arc to avoid borrow issues)
        let matching_key = self
            .parameters
            .keys()
            .find(|key| key.eq_ignore_ascii_case(parameter_name))
            .map(|k| k.clone());

        if let Some(key) = matching_key {
            return self
                .parameters
                .get_mut(&key)
                .ok_or_else(|| anyhow!("Parameter not found: {parameter_name}"));
        }

        Err(anyhow!("Parameter not found: {parameter_name}"))
    }

    /// Creates a new `HashMap` of `Parameter`s
    /// using the `Fcs` file's metadata to find the channel and label names from the `PnN` and `PnS` keywords.
    /// Does NOT store events on the parameter.
    /// # Errors
    /// Will return `Err` if:
    /// - the number of parameters cannot be found in the metadata,
    /// - the parameter name cannot be found in the metadata,
    /// - the parameter cannot be built (using the Builder pattern)
    pub fn generate_parameter_map(metadata: &Metadata) -> Result<ParameterMap> {
        let mut map = ParameterMap::default();
        let number_of_parameters = metadata.get_number_of_parameters()?;
        for parameter_number in 1..=*number_of_parameters {
            let channel_name = metadata.get_parameter_channel_name(parameter_number)?;

            // Use label name or fallback to the parameter name
            let label_name = match metadata.get_parameter_label(parameter_number) {
                Ok(label) => label,
                Err(_) => channel_name,
            };

            let transform = if channel_name.contains("FSC")
                || channel_name.contains("SSC")
                || channel_name.contains("Time")
            {
                TransformType::Linear
            } else {
                TransformType::default()
            };

            // Get excitation wavelength from metadata if available
            let excitation_wavelength = metadata
                .get_parameter_excitation_wavelength(parameter_number)
                .ok()
                .flatten();

            let parameter = ParameterBuilder::default()
                // For the ParameterBuilder, ensure we're using the proper methods
                // that may be defined by the Builder derive macro
                .parameter_number(parameter_number)
                .channel_name(channel_name)
                .label_name(label_name)
                .transform(transform)
                .excitation_wavelength(excitation_wavelength)
                .build()?;

            // Add the parameter events to the hashmap keyed by the parameter name
            map.insert(channel_name.to_string().into(), parameter);
        }

        Ok(map)
    }

    /// Looks for a keyword among the metadata and returns its value as a `&str`
    /// # Errors
    /// Will return `Err` if the `Keyword` is not found in the `metadata` or if the `Keyword` cannot be converted to a `&str`
    pub fn get_keyword_string_value(&self, keyword: &str) -> Result<Cow<'_, str>> {
        // TODO: This should be a match statement
        if let Ok(keyword) = self.metadata.get_string_keyword(keyword) {
            Ok(keyword.get_str())
        } else if let Ok(keyword) = self.metadata.get_integer_keyword(keyword) {
            Ok(keyword.get_str())
        } else if let Ok(keyword) = self.metadata.get_float_keyword(keyword) {
            Ok(keyword.get_str())
        } else if let Ok(keyword) = self.metadata.get_byte_keyword(keyword) {
            Ok(keyword.get_str())
        } else if let Ok(keyword) = self.metadata.get_mixed_keyword(keyword) {
            Ok(keyword.get_str())
        } else {
            Err(anyhow!("Keyword not found: {}", keyword))
        }
    }
    /// A convenience function to return the `GUID` keyword from the `metadata` as a `&str`
    /// # Errors
    /// Will return `Err` if the `GUID` keyword is not found in the `metadata` or if the `GUID` keyword cannot be converted to a `&str`
    pub fn get_guid(&self) -> Result<Cow<'_, str>> {
        Ok(self.metadata.get_string_keyword("$GUID")?.get_str())
    }

    /// Set or update the GUID keyword in the file's metadata
    pub fn set_guid(&mut self, guid: String) {
        self.metadata
            .insert_string_keyword("$GUID".to_string(), guid);
    }

    /// A convenience function to return the `$FIL` keyword from the `metadata` as a `&str`
    /// # Errors
    /// Will return `Err` if the `$FIL` keyword is not found in the `metadata` or if the `$FIL` keyword cannot be converted to a `&str`
    pub fn get_fil_keyword(&self) -> Result<Cow<'_, str>> {
        Ok(self.metadata.get_string_keyword("$FIL")?.get_str())
    }

    /// A convenience function to return the `$TOT` keyword from the `metadata` as a `usize`
    /// # Errors
    /// Will return `Err` if the `$TOT` keyword is not found in the `metadata` or if the `$TOT` keyword cannot be converted to a `usize`
    pub fn get_number_of_events(&self) -> Result<&usize> {
        self.metadata.get_number_of_events()
    }

    /// A convenience function to return the `$PAR` keyword from the `metadata` as a `usize`
    /// # Errors
    /// Will return `Err` if the `$PAR` keyword is not found in the `metadata` or if the `$PAR` keyword cannot be converted to a `usize`
    pub fn get_number_of_parameters(&self) -> Result<&usize> {
        self.metadata.get_number_of_parameters()
    }
}
