use super::Error;
use std::{collections::HashMap, str::FromStr};

#[derive(Debug, PartialEq)]
pub struct Variable {
    key: String,
    val: String,
}

impl Variable {
    pub fn format(raw: &str) -> &str {
        // remove surrounding whitespace
        raw.trim()
    }
}

impl FromStr for Variable {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once('=') {
            Some((k, v)) => Ok(Self {
                key: k.to_string(),
                val: v.to_string(),
            }),
            None => Err(Error::VariableParseError),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct VarMap {
    inner: HashMap<String, String>,
}

impl VarMap {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: &str, val: &str) -> Option<String> {
        // format the key
        let key = format!("koopa.{}", Variable::format(key));
        self.inner.insert(key, val.to_string())
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        let key = Variable::format(key);
        self.inner.get(key)
    }
}

impl From<&Vec<Variable>> for VarMap {
    fn from(value: &Vec<Variable>) -> Self {
        let mut vars = VarMap::new();
        value.into_iter().for_each(|var| {
            vars.insert(&var.key, &var.val);
        });
        vars
    }
}
