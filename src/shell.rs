use super::Error;
use std::fmt::Debug;
use std::hash::Hash;
use std::{collections::HashMap, str::FromStr};

const KEY_PREFIX: &str = "koopa.";

#[derive(Eq, Clone)]
pub struct Key(String);

impl Key {
    pub fn new() -> Self {
        Self(String::new())
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn push(&mut self, ch: char) {
        self.0.push(ch)
    }

    pub fn pop(&mut self) -> Option<char> {
        self.0.pop()
    }

    fn as_internal_repr(&self) -> &str {
        self.0.trim()
    }

    /// Determines if the given key is indeed a key recognized by koopa.
    pub fn is_koopa_key(&self) -> bool {
        self.as_internal_repr().starts_with(KEY_PREFIX)
    }
}

impl Hash for Key {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write(self.as_internal_repr().as_bytes());
        state.finish();
    }
}

impl Debug for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{{{{}}}}}", self.0)
    }
}

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.as_internal_repr().eq(other.as_internal_repr())
    }
}

impl From<&str> for Key {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for Key {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Value(String);

impl Value {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Shell {
    key: Key,
    value: Value,
}

impl Shell {
    pub fn with(key: String, value: String) -> Self {
        Self {
            key: Key(key),
            value: Value(value),
        }
    }

    pub fn key(&self) -> &Key {
        &self.key
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

    /// Splits the struct into its underlying components: a key and a value.
    pub fn split(self) -> (Key, Value) {
        (self.key, self.value)
    }
}

impl FromStr for Shell {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once('=') {
            Some((k, v)) => Ok(Self {
                key: Key::from(format!("{}{}", KEY_PREFIX, k)),
                value: Value::from(v),
            }),
            None => Err(Error::ShellParseMissingEq),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ShellMap {
    inner: HashMap<Key, Value>,
}

impl ShellMap {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn insert(&mut self, shell: Shell) -> Option<Value> {
        let (key, value) = shell.split();
        self.inner.insert(key, value)
    }

    pub fn get(&self, key: &Key) -> Option<&Value> {
        self.inner.get(key)
    }
}

impl From<&Vec<Shell>> for ShellMap {
    fn from(value: &Vec<Shell>) -> Self {
        let mut shell_map = ShellMap::new();
        value.into_iter().for_each(|shell| {
            shell_map.insert(shell.clone());
        });
        shell_map
    }
}
