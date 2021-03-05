use serde_yaml;
mod rules;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let f = std::fs::File::open("fixtures/example.yml")?;
    let rules: rules::Rules = serde_yaml::from_reader(f)?;

    for (rule_name, rule) in rules.iter() {
        println!("executing rule: {:?}", rule_name);
        let _result = execute_rule(rule);
    }

    Ok(())
}

fn execute_rule(rule: &rules::Rule) -> Result<(), Box<dyn std::error::Error>> {
    for action in rule.actions.iter() {
        action.execute()
    }
    Ok(())
}
