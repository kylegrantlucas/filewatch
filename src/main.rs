//!  filewatch = "0.1.0"
mod filewatch;
use anyhow::{Context, Result};
use clap::Parser;
use console::{style, Emoji};
use lazy_static::lazy_static;
use log::info;
use simple_logger::SimpleLogger;
use std::collections::HashMap;

/// ``Args`` is a struct that holds the command line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// file positional
    file: String,
    /// verbose flag
    #[arg(short, long)]
    verbose: bool,
    /// dry run flag
    #[arg(short, long)]
    dry_run: bool,
}

lazy_static! {
    // set cli
    static ref CLI: Args = Args::parse();
    // read verbose from cli args
    static ref VERBOSE: bool = CLI.verbose;
    // read dry_run from cli args
    static ref DRY_RUN: bool = CLI.dry_run;

    // RuleActionType to String HashMap
    static ref RULE_ACTION_TYPE_MAP: HashMap<filewatch::rules::RuleActionType, &'static str> = {
        let mut m = HashMap::new();
        m.insert(filewatch::rules::RuleActionType::Move, "move");
        m.insert(filewatch::rules::RuleActionType::Rename, "rename");
        m.insert(filewatch::rules::RuleActionType::Delete, "delete");
        m.insert(filewatch::rules::RuleActionType::Copy, "copy");
        m.insert(filewatch::rules::RuleActionType::Link, "link");
        m
    };
    static ref RULE_ACTION_TYPE_EMOJI_MAP: HashMap<filewatch::rules::RuleActionType, &'static str> = {
        let mut m = HashMap::new();
        m.insert(filewatch::rules::RuleActionType::Move, "\u{1f69a} ");
        m.insert(filewatch::rules::RuleActionType::Rename, "\u{1f4dd} ");
        m.insert(filewatch::rules::RuleActionType::Delete, "\u{1f5d1}\u{fe0f}  ");
        m.insert(filewatch::rules::RuleActionType::Copy, "\u{1f4cb} ");
        m.insert(filewatch::rules::RuleActionType::Link, "\u{1f517} ");
        m
    };
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if *VERBOSE {
        SimpleLogger::new()
            .init()
            .with_context(|| "Failed to initialize logger")?;
    }

    // only run if file is present
    if std::path::Path::new(&CLI.file).exists() {
        let f = std::fs::File::open(&CLI.file)?;
        let rules: filewatch::Rules = serde_yaml::from_reader(f)?;

        for (rule_name, rule) in &rules {
            if *VERBOSE || *DRY_RUN {
                info!("executing rule: {:?}", rule_name);
            } else {
                println!(
                    "{} {}",
                    style("executing rule").bold().dim(),
                    style(rule_name).bold()
                );
            }
            let _result = execute_rule(rule);
        }
    }

    Ok(())
}

/// ``execute_rule`` is a function that takes a ``Rule`` struct and executes it
fn execute_rule(rule: &filewatch::rules::Rule) -> Result<()> {
    for (i, action) in rule.actions.iter().enumerate() {
        let rule_action_type = RULE_ACTION_TYPE_MAP
            .get(&action.action)
            .with_context(|| format!("RuleActionType {:?} not found in list of types", action))?;

        if *VERBOSE {
            info!("executing {:?}: {:?}", rule_action_type, action);
        } else {
            // parse emoji
            let emoji = RULE_ACTION_TYPE_EMOJI_MAP
                .get(&action.action)
                .with_context(|| {
                    format!("RuleActionType {:?} not found in list of types", action)
                })?;

            println!(
                "[{}/{}] {} {} {}...",
                i + 1,
                rule.actions.len(),
                Emoji(emoji, ""),
                style("executing").bold().dim(),
                style(rule_action_type).bold()
            );
        }

        action
            .execute()
            .with_context(|| format!("Failed to execute action: {:?}", action))?;
    }

    Ok(())
}
