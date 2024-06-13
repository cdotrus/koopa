use super::Error;
use serde::Deserialize;
use std::fmt::Display;
use std::hash::Hash;
use std::{collections::HashMap, str::FromStr};

pub const KEY_PREFIX: &str = "koopa.";

#[derive(Debug, Eq, Clone, PartialOrd, Ord)]
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

    /// Accesses the name of the key without the koopa prefix.
    pub fn get_name(&self) -> &str {
        &self
            .0
            .trim()
            .get(
                match self.0.trim().find('.') {
                    Some(i) => i + 1,
                    None => 0,
                }..,
            )
            .unwrap()
    }

    /// Transforms the given key into a koopa key, if not already.
    pub fn into_koopa_key(self) -> Self {
        match self.is_koopa_key() {
            true => self,
            false => Self(format!("{}{}", KEY_PREFIX, self.0)),
        }
    }

    /// Verifies the given key is a valid representation by using the
    /// [FromStr] trait.
    pub fn validate(&self) -> Option<Error> {
        Self::from_str(&self.0).err()
    }
}

impl FromStr for Key {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let word_count = s.split_whitespace().into_iter().count();
        if word_count > 1 {
            return Err(Error::KeyContainsWhitespace(s.to_string()));
        }
        if s.contains('\n') == true {
            return Err(Error::KeyContainsNewline(s.to_string()));
        }
        let dot_count = s.chars().filter(|c| c == &'.').count();
        if s.trim().starts_with(KEY_PREFIX) == true && dot_count > 1 {
            return Err(Error::KeyContainsMoreDots(s.to_string()));
        } else if s.trim().starts_with(KEY_PREFIX) == false && dot_count > 0 {
            return Err(Error::KeyContainsOneDot(s.to_string()));
        }
        Ok(Self(s.to_string()))
    }
}

impl Hash for Key {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write(self.as_internal_repr().as_bytes());
        state.finish();
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{{{{}}}}}", self.0)
    }
}

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.as_internal_repr().eq(other.as_internal_repr())
    }
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
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

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
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

impl From<(Key, Value)> for Shell {
    fn from(value: (Key, Value)) -> Self {
        Self {
            key: value.0,
            value: value.1,
        }
    }
}

impl FromStr for Shell {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once('=') {
            Some((k, v)) => Ok(Self {
                key: Key::from_str(k)?.into_koopa_key(),
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

    /// Inserts existing shell entries into the current map, overwriting entries
    /// if they already existed.
    pub fn merge(&mut self, shells: ShellMap) {
        shells.inner.into_iter().for_each(|(key, value)| {
            self.insert(Shell {
                key: key,
                value: value,
            });
        });
    }

    pub fn inner(&self) -> &HashMap<Key, Value> {
        &self.inner
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

use serde::de;
use std::fmt;

impl<'de> Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> Result<Key, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct LayerVisitor;

        impl<'de> de::Visitor<'de> for LayerVisitor {
            type Value = Key;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a shell key")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match Key::from_str(v) {
                    Ok(v) => Ok(v),
                    Err(e) => Err(de::Error::custom(e)),
                }
            }
        }

        deserializer.deserialize_map(LayerVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ut_parse_key_ok() {
        let s = "helloworld";
        assert_eq!(Key::from_str(s), Ok(Key(s.to_string())));

        let s = "   mykey     ";
        assert_eq!(Key::from_str(s), Ok(Key(s.to_string())));
        let s = "koopa.name";
        assert_eq!(Key::from_str(s), Ok(Key(s.to_string())));
    }

    #[test]
    fn ut_parse_key_err() {
        let s = "hello world";
        assert_eq!(
            Key::from_str(s),
            Err(Error::KeyContainsWhitespace(s.to_string()))
        );
        let s = "mykey\n";
        assert_eq!(
            Key::from_str(s),
            Err(Error::KeyContainsNewline(s.to_string()))
        );
        let s = "koopa.nested.key";
        assert_eq!(
            Key::from_str(s),
            Err(Error::KeyContainsMoreDots(s.to_string()))
        );
    }
}
