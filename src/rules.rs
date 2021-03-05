use serde::Deserialize;
use std::collections::BTreeMap;
use walkdir::WalkDir;

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct RuleAction {
    action: RuleActionType,
    watch_dir: String,
    match_pattern: Option<Regex>,
    rename_pattern: Option<String>,
    destination: Option<String>,
}

impl RuleAction {
    pub fn execute(&self) {
        match self.action {
            RuleActionType::Rename => self.rename(),
            RuleActionType::Move => self.mv(),
            RuleActionType::Delete => self.delete(),
            RuleActionType::Copy => self.copy(),
        }
    }
    fn rename(&self) {
        println!("executing rename: {:?}", self);
        let paths =
            match_directory_listing(self.watch_dir.clone(), self.match_pattern.clone().unwrap())
                .unwrap();

        for path in paths.iter() {
            generate_new_filename(
                self.watch_dir.clone(),
                path.to_string(),
                self.match_pattern.clone().unwrap(),
                self.rename_pattern.clone().unwrap(),
            )
        }
        println!("Paths: {:?}", paths);
    }
    fn mv(&self) {
        println!("executing move: {:?}", self);
        let paths =
            match_directory_listing(self.watch_dir.clone(), self.match_pattern.clone().unwrap())
                .unwrap();
        println!("Paths: {:?}", paths);
    }
    fn copy(&self) {
        println!("executing copy: {:?}", self);
        let paths =
            match_directory_listing(self.watch_dir.clone(), self.match_pattern.clone().unwrap())
                .unwrap();
        println!("Paths: {:?}", paths);
    }
    fn delete(&self) {
        println!("executing delete: {:?}", self);
        let paths =
            match_directory_listing(self.watch_dir.clone(), self.match_pattern.clone().unwrap())
                .unwrap();
        println!("Paths: {:?}", paths);
    }
}

#[derive(Debug, Deserialize)]
pub struct Rule {
    pub interval: String,
    pub actions: Vec<RuleAction>
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
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut paths = Vec::new();
    for e in WalkDir::new(path.clone())
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if e.metadata().unwrap().is_file() {
            let filepath = e.path().to_str().unwrap();
            let trunc_path = str::replace(filepath, path.as_str(), "");
            if match_regex.is_match(trunc_path.as_str()) {
                paths.push(filepath.to_string())
            }
        }
    }

    Ok(paths)
}

pub fn generate_new_filename(path: String, filepath: String, match_regex: Regex, new_format: String) {
    let mut new_name = "".to_string();
    let trunc_path = str::replace(filepath.as_str(), path.as_str(), "");
    let caps = match_regex.captures(trunc_path.as_str()).unwrap();

    caps.expand(new_format.as_str(), &mut new_name);
    let result = str::replace(filepath.as_str(), trunc_path.as_str(), new_name.as_str());
    println!("Old Name: {}", filepath);
    println!("New Name: {}", result);
}
