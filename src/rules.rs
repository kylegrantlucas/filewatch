use crate::DRY_RUN;
use crate::VERBOSE;
use anyhow::{anyhow, Context, Result};
use console::{style, Emoji};
use indicatif::ProgressBar;
use lazy_static::lazy_static;
use log::info;
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs;
use walkdir::WalkDir;

// static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍  ", "");
// static TRUCK: Emoji<'_, '_> = Emoji("🚚  ", "");
// static CLIP: Emoji<'_, '_> = Emoji("🔗  ", "");
// static PAPER: Emoji<'_, '_> = Emoji("📃  ", "");
// static SPARKLE: Emoji<'_, '_> = Emoji("✨ ", ":-)");

#[derive(Hash, PartialEq, Eq, Debug, Deserialize)]
enum RuleActionType {
    #[serde(alias = "move")]
    Move,
    #[serde(alias = "rename")]
    Rename,
    #[serde(alias = "delete")]
    Delete,
    #[serde(alias = "copy")]
    Copy,
}

// RuleActionType to String HashMap
lazy_static! {
    static ref RULE_ACTION_TYPE_MAP: HashMap<RuleActionType, &'static str> = {
        let mut m = HashMap::new();
        m.insert(RuleActionType::Move, "move");
        m.insert(RuleActionType::Rename, "rename");
        m.insert(RuleActionType::Delete, "delete");
        m.insert(RuleActionType::Copy, "copy");
        m
    };
}

#[derive(Debug, Deserialize)]
pub struct RuleAction {
    action: RuleActionType,
    watch_dir: String,
    match_pattern: Option<Regex>,
    rename_pattern: Option<String>,
    destination: Option<String>,
}

impl RuleAction {
    pub fn execute(&self) -> Result<()> {
        let rule_action_type = RULE_ACTION_TYPE_MAP.get(&self.action).with_context(|| {
            format!(
                "RuleActionType {:?} not found in list of types",
                self.action
            )
        })?;

        if *VERBOSE {
            info!("executing {:?}: {:?}", rule_action_type, self);
        } else {
            println!(
                "{} {}",
                style("executing").bold().dim(),
                style(rule_action_type).bold()
            );
        }

        // parse match_regex TODO: is this right?
        let match_regex = match &self.match_pattern {
            Some(regex) => regex.clone(),
            None => {
                return Err(anyhow!(
                    "match_pattern is required for rule action: {:?}",
                    self
                ))
            }
        };

        // match paths with regex pattern
        let paths = match_directory_listing(self.watch_dir.clone(), match_regex)
            .with_context(|| format!("Failed to match directory listing for rule: {:?}", self))?;

        if *VERBOSE {
            info!("matched paths: {:?}", paths);
        }

        let paths_count = paths.len();
        let pb = ProgressBar::new(paths_count as u64);

        // iterate over matched paths
        paths.par_iter().for_each(|path| {
            match self.action {
                RuleActionType::Rename => self.rename(path),
                RuleActionType::Move => self.mv(path),
                RuleActionType::Delete => self.delete(path),
                RuleActionType::Copy => self.copy(path),
            };

            pb.inc(1);
        });

        pb.finish();

        Ok(())
    }

    // rename action performs a file rename
    fn rename(&self, path: &String) -> Result<()> {
        // parse match_regex
        let match_regex = match &self.match_pattern {
            Some(regex) => regex.clone(),
            None => {
                return Err(anyhow!(
                    "match_pattern is required for rule action: {:?}",
                    self
                ))
            }
        };

        // parse rename_pattern
        let rename_pattern = match &self.rename_pattern {
            Some(pattern) => pattern.clone(),
            None => {
                return Err(anyhow!(
                    "rename_pattern is required for rule action: {:?}",
                    self
                ))
            }
        };

        // build new path
        let new_filename = generate_new_filename(
            self.watch_dir.clone(),
            path.to_string(),
            match_regex,
            rename_pattern,
        )
        .with_context(|| format!("Failed to generate new filename for path: {}", path))?;

        // rename file
        if *VERBOSE || *DRY_RUN {
            info!("renaming file: {:?} -> {:?}", path, new_filename);
        }
        if !*DRY_RUN {
            fs::rename(path, new_filename.clone()).with_context(|| {
                format!(
                    "Failed to rename file: {:?} -> {:?}",
                    path,
                    new_filename.clone()
                )
            })?;
        }

        Ok(())
    }

    // mv action is a combination of copy and delete
    fn mv(&self, path: &String) -> Result<()> {
        // parse destination
        let destination = match &self.destination {
            Some(dest) => dest.clone(),
            None => {
                return Err(anyhow!(
                    "destination is required for rule action: {:?}",
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
        let new_path = format!("{}/{}", destination, filename,);

        // move file
        if *VERBOSE || *DRY_RUN {
            info!("\t|-> moving file: {:?} -> {:?}", path, new_path);
        }
        if !*DRY_RUN {
            fs::copy(path, new_path.clone()).with_context(|| {
                format!("Failed to copy file from {} to {}", path, new_path.clone())
            })?;

            fs::remove_file(path).with_context(|| format!("Failed to remove file {}", path))?;
        }

        Ok(())
    }

    // copy actions performs a simple copy
    fn copy(&self, path: &String) -> Result<()> {
        // parse destination
        let destination = match &self.destination {
            Some(dest) => dest.clone(),
            None => {
                return Err(anyhow!(
                    "destination is required for rule action: {:?}",
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
        let new_path = format!("{}/{}", destination, filename,);

        // copy file
        if *VERBOSE || *DRY_RUN {
            info!("copying file: {:?} -> {:?}", path, new_path);
        }
        if !*DRY_RUN {
            fs::copy(path, new_path.clone()).with_context(|| {
                format!(
                    "Failed to copy file from {} to {}",
                    path.to_string(),
                    new_path.clone().to_string()
                )
            })?;
        }

        Ok(())
    }

    // delete action performs a simple delete
    fn delete(&self, path: &String) -> Result<()> {
        // delete file
        if *VERBOSE || *DRY_RUN {
            info!("deleting file: {:?}", path);
        }
        if !*DRY_RUN {
            fs::remove_file(path).with_context(|| format!("Failed to delete file {}", path))?;
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct Rule {
    pub interval: String,
    pub actions: Vec<RuleAction>,
}

pub type Rules = BTreeMap<String, Rule>;

#[derive(Clone, Debug)]
pub struct Regex(regex::Regex);

impl std::ops::Deref for Regex {
    type Target = regex::Regex;
    fn deref(&self) -> &regex::Regex {
        &self.0
    }
}

impl<'de> serde::Deserialize<'de> for Regex {
    fn deserialize<D>(de: D) -> Result<Regex, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{Error, Visitor};

        struct RegexVisitor;

        impl<'de> Visitor<'de> for RegexVisitor {
            type Value = Regex;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a regular expression pattern")
            }

            fn visit_str<E: Error>(self, v: &str) -> Result<Regex, E> {
                regex::Regex::new(v)
                    .map(Regex)
                    .map_err(|err| E::custom(err.to_string()))
            }
        }

        de.deserialize_str(RegexVisitor)
    }
}

pub fn match_directory_listing(
    path: String,
    match_regex: Regex,
) -> Result<Vec<String>, anyhow::Error> {
    let mut paths = Vec::new();
    for e in WalkDir::new(path.clone())
        .into_iter()
        .filter_map(|e| e.ok())
    {
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

            let trunc_path = str::replace(filepath, path.as_str(), "");
            if match_regex.is_match(trunc_path.as_str()) {
                paths.push(filepath.to_string())
            }
        }
    }

    Ok(paths)
}

pub fn generate_new_filename(
    path: String,
    filepath: String,
    match_regex: Regex,
    new_format: String,
) -> Result<String, anyhow::Error> {
    let mut new_name = "".to_string();
    let trunc_path = str::replace(filepath.as_str(), path.as_str(), "");
    let caps = match_regex
        .captures(trunc_path.as_str())
        .with_context(|| format!("Failed to parse captures for file: {:?}", filepath))?;

    caps.expand(new_format.as_str(), &mut new_name);

    let new_name = str::replace(filepath.as_str(), trunc_path.as_str(), new_name.as_str());

    Ok(new_name)
}
