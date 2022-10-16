pub mod rules;
extern crate alloc;
use alloc::collections::BTreeMap;

/// Rules represents a map of rules
pub type Rules = BTreeMap<String, rules::Rule>;
