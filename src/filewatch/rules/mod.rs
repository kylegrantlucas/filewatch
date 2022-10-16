//! rules module
// global variables
use crate::DRY_RUN;
use crate::VERBOSE;

// serde compatibility for regex
mod regex;
use self::regex::Regex;

use anyhow::{anyhow, Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use log::{error, info};
use rayon::prelude::*;
use serde::Deserialize;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use walkdir::WalkDir;

/// ``RuleActionType`` is an enum that defines multiple actions that can be performed on a file
#[derive(Hash, PartialEq, Eq, Debug, Deserialize)]
pub enum RuleActionType {
    /// ``Move`` moves a file to a new location
    #[serde(alias = "move")]
    Move,
    /// ``Rename`` renames a file
    #[serde(alias = "rename")]
    Rename,
    /// ``Delete`` deletes a file
    #[serde(alias = "delete")]
    Delete,
    /// ``Copy`` copies a file to a new location
    #[serde(alias = "copy")]
    Copy,
    /// ``Link`` creates a symlink to a file
    #[serde(alias = "link")]
    Link,
    /// ``Chmod`` changes the permissions of a file
    #[serde(alias = "chmod")]
    Chmod,
}

/// ``RuleAction`` is a struct that defines actions that can be performed on a file
#[derive(Debug, Deserialize)]
pub struct RuleAction {
    /// ``action`` is the type of action to perform
    pub action: RuleActionType,
    /// ``watch_dir`` is the directory to watch for files
    watch_dir: String,
    /// ``match_regex`` is the regex to match files against
    match_regex: Option<Regex>,
    /// ``rename_pattern`` is the pattern to rename files with
    rename_pattern: Option<String>,
    /// ``destination_dir`` is the directory to move files to
    destination_dir: Option<String>,
    /// ``permissions`` is the permissions to set on a file
    permissions: Option<u32>,
}

impl RuleAction {
    /// ``execute`` is a function that executes a rule action
    pub fn execute(&self) -> Result<()> {
        // parse match_regex
        let match_regex = match self.match_regex {
            Some(ref regex) => regex,
            None => {
                return Err(anyhow!(
                    "match_regex is required for rule action {:?}",
                    self.action
                ))
            }
        };

        // match paths with regex pattern
        let paths = match_directory_listing(self.watch_dir.as_str(), match_regex)
            .with_context(|| format!("Failed to match directory listing for rule: {:?}", self))?;

        if *VERBOSE {
            info!("matched paths: {:?}", paths);
        }

        let mut progress_bar: Option<ProgressBar> = Option::None;

        if !*VERBOSE {
            let paths_count = paths.len();
            progress_bar = Some(ProgressBar::new(u64::try_from(paths_count)?));

            // parse progress bar style
            let mut progress_bar_style = ProgressStyle::default_bar().template(
                "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})",
            )?;

            // get progress bar and handles errors
            let progress_bar = progress_bar.as_mut().ok_or_else(|| {
                anyhow!(
                    "Failed to get progress bar for rule action: {:?}",
                    self.action
                )
            })?;

            // set progress chars
            progress_bar_style = progress_bar_style.progress_chars("=>-");
            progress_bar.set_style(progress_bar_style);
        }

        // iterate over matched paths
        paths.par_iter().for_each(|path| {
            let path_str = path.as_str();

            let res = match self.action {
                RuleActionType::Rename => self.rename(path_str),
                RuleActionType::Move => self.mv(path_str),
                RuleActionType::Delete => self.delete(path_str),
                RuleActionType::Copy => self.copy(path_str),
                RuleActionType::Link => self.link(path_str),
                RuleActionType::Chmod => self.chmod(path_str),
            };

            match res {
                Ok(_) => {
                    if !*VERBOSE {
                        // increment progress bar
                        if let Some(ref progress_bar) = progress_bar {
                            progress_bar.inc(1);
                        }
                    }
                }
                Err(e) => {
                    if *VERBOSE {
                        error!("error: {:?}", e);
                    } else {
                        // match progress bar
                        match progress_bar {
                            Some(ref progress_bar) => {
                                progress_bar.println(format!("error: {:?}", e));
                            }
                            None => {
                                error!("error: {:?}", e);
                            }
                        }
                    }
                }
            }
        });

        if !*VERBOSE {
            // match progress bar
            match &progress_bar {
                Some(progress_bar) => {
                    progress_bar.finish_and_clear();
                }
                None => (),
            }
        }

        Ok(())
    }

    /// rename action performs a file rename
    fn rename(&self, path: &str) -> Result<()> {
        // parse match_regex
        let match_regex = match self.match_regex {
            Some(ref regex) => regex,
            None => {
                return Err(anyhow!(
                    "match_regex is required for rule action: {:?}",
                    self
                ))
            }
        };

        // parse rename_pattern
        let rename_pattern = match self.rename_pattern {
            Some(ref pattern) => pattern,
            None => {
                return Err(anyhow!(
                    "rename_pattern is required for rule action: {:?}",
                    self
                ))
            }
        };

        // build new path
        let new_filename =
            generate_new_filename(self.watch_dir.as_str(), path, match_regex, rename_pattern)
                .with_context(|| format!("Failed to generate new filename for path: {}", path))?;

        // rename file
        if *VERBOSE || *DRY_RUN {
            info!("renaming file: {:?} -> {:?}", path, new_filename);
        }
        if !*DRY_RUN && path != new_filename {
            fs::rename(path, new_filename).with_context(|| {
                format!(
                    "Failed to rename file: {:?} -> {:?}",
                    path,
                    new_filename.clone()
                )
            })?;
        }

        Ok(())
    }

    /// mv action is a combination of copy and delete
    fn mv(&self, path: &str) -> Result<()> {
        // parse destination_dir
        let destination_dir = match self.destination_dir {
            Some(ref dest) => dest.clone(),
            None => {
                return Err(anyhow!(
                    "destination_dir is required for rule action: {:?}",
                    self
                ))
            }
        };

        // parse filename
        let filename = match path.split('/').last() {
            Some(filename) => filename,
            None => return Err(anyhow!("Failed to parse filename from path: {:?}", path)),
        };

        // build mv path
        let new_path = format!("{}/{}", destination_dir, filename,);

        // move file
        if *VERBOSE || *DRY_RUN {
            info!("moving file: {:?} -> {:?}", path, new_path);
        }
        if !*DRY_RUN && path != new_path {
            fs::copy(path, new_path.clone()).with_context(|| {
                format!("Failed to copy file from {} to {}", path, new_path.clone())
            })?;

            fs::remove_file(path).with_context(|| format!("Failed to remove file {}", path))?;
        }

        Ok(())
    }

    /// copy actions performs a simple copy
    fn copy(&self, path: &str) -> Result<()> {
        // parse destination_dir
        let destination_dir = match self.destination_dir {
            Some(ref dest) => dest.clone(),
            None => {
                return Err(anyhow!(
                    "destination_dir is required for rule action: {:?}",
                    self
                ))
            }
        };

        // parse filename
        let filename = match path.split('/').last() {
            Some(filename) => filename,
            None => return Err(anyhow!("Failed to parse filename from path: {:?}", path)),
        };

        // build copy path
        let new_path = format!("{}/{}", destination_dir, filename,);

        // copy file
        if *VERBOSE || *DRY_RUN {
            info!("copying file: {:?} -> {:?}", path, new_path);
        }
        if !*DRY_RUN {
            fs::copy(path, new_path.clone()).with_context(|| {
                format!("Failed to copy file from {} to {}", path, new_path.clone())
            })?;
        }

        Ok(())
    }

    /// delete action performs a simple delete
    fn delete(&self, path: &str) -> Result<()> {
        // delete file
        if *VERBOSE || *DRY_RUN {
            info!("deleting file: {:?}", path);
        }
        if !*DRY_RUN {
            fs::remove_file(path).with_context(|| format!("Failed to delete file {}", path))?;
        }

        Ok(())
    }

    /// link action performs a simple link
    fn link(&self, path: &str) -> Result<()> {
        // parse destination_dir
        let destination_dir = match self.destination_dir {
            Some(ref dest) => dest.clone(),
            None => {
                return Err(anyhow!(
                    "destination_dir is required for rule action: {:?}",
                    self
                ))
            }
        };

        // parse filename
        let filename = match path.split('/').last() {
            Some(filename) => filename,
            None => return Err(anyhow!("Failed to parse filename from path: {:?}", path)),
        };

        // build link path
        let new_path = format!("{}/{}", destination_dir, filename,);

        // link file
        if *VERBOSE || *DRY_RUN {
            info!("linking file: {:?} -> {:?}", path, new_path);
        }
        if !*DRY_RUN {
            fs::hard_link(path, new_path.clone()).with_context(|| {
                format!("Failed to link file from {} to {}", path, new_path.clone())
            })?;
        }

        Ok(())
    }

    /// chmod action performs a simple chmod
    fn chmod(&self, path: &str) -> Result<()> {
        // parse mode
        let mode = match self.permissions {
            Some(ref mode) => mode,
            None => return Err(anyhow!("mode is required for rule action: {:?}", self)),
        };

        // chmod file
        if *VERBOSE || *DRY_RUN {
            info!("chmoding file: {:?} -> {:?}", path, mode);
        }
        if !*DRY_RUN {
            fs::set_permissions(path, fs::Permissions::from_mode(*mode))
                .with_context(|| format!("Failed to set permissions for file: {}", path))?;
        }

        Ok(())
    }
}

/// Rule represents a single rule for processing files
#[derive(Debug, Deserialize)]
pub struct Rule {
    /// interval is the time in seconds to wait running the rule
    pub interval: String,
    /// actions is a list of actions to perform on the file
    pub actions: Vec<RuleAction>,
}

/// ``match_directory_listing`` matches a directory listing against a regex
pub fn match_directory_listing(
    path: &str,
    match_regex: &Regex,
) -> Result<Vec<String>, anyhow::Error> {
    let mut paths = Vec::new();
    for e in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        // parse metadata
        let metadata = e.metadata().with_context(|| {
            format!(
                "Failed to parse metadata for file: {:?}",
                e.path().to_string_lossy()
            )
        })?;

        // if path is a file, check if it matches the regex
        if metadata.is_file() {
            let filepath = e.path().to_str().with_context(|| {
                format!(
                    "Failed to parse filepath for file: {:?}",
                    e.path().to_string_lossy()
                )
            })?;

            let trunc_path = str::replace(filepath, path, "");
            if match_regex.is_match(trunc_path.as_str()) {
                paths.push(filepath.to_owned());
            }
        }
    }

    Ok(paths)
}

/// ``generate_new_filename`` generates a new filename based on the regex capture groups
pub fn generate_new_filename(
    path: &str,
    filepath: &str,
    match_regex: &Regex,
    new_format: &str,
) -> Result<String, anyhow::Error> {
    let mut replaced_name = "".to_owned();
    let trunc_path = str::replace(filepath, path, "");
    let caps = match_regex
        .captures(trunc_path.as_str())
        .with_context(|| format!("Failed to parse captures for file: {:?}", filepath))?;

    caps.expand(new_format, &mut replaced_name);

    let new_name = str::replace(filepath, trunc_path.as_str(), replaced_name.as_str());

    Ok(new_name)
}
