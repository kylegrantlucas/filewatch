mod rules;
use anyhow::{Context, Result};
use clap::Parser;
use console::style;
use lazy_static::lazy_static;
use log::info;
use simple_logger::SimpleLogger;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // file positional
    file: String,
    // Name of the person to greet
    #[arg(short, long)]
    verbose: bool,

    // Name of the person to greet
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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new()
        .init()
        .with_context(|| "Failed to initialize logger")?;

    // only run if file is present
    if std::path::Path::new(&CLI.file).exists() {
        let f = std::fs::File::open(&CLI.file)?;
        let rules: rules::Rules = serde_yaml::from_reader(f)?;

        for (rule_name, rule) in rules.iter() {
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

fn execute_rule(rule: &rules::Rule) -> Result<()> {
    for action in rule.actions.iter() {
        action
            .execute()
            .with_context(|| format!("Failed to execute action: {:?}", action))?;
    }

    Ok(())
}
