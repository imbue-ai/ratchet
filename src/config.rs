//! Configuration file parsing and validation

pub mod ratchet_toml;

pub use ratchet_toml::{
    ColorOption, Config, OutputConfig, OutputFormat, RuleSettings, RulesConfig,
};
