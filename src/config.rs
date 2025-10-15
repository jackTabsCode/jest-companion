use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub projects: HashMap<String, PathBuf>,
}
