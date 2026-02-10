//! Bencode parser and encoder
//!
//! Bencode is the encoding used by BitTorrent for storing and transmitting
//! loosely structured data. It supports four types:
//! - Integers: i<number>e (e.g., i42e)
//! - Strings: <length>:<string> (e.g., 4:spam)
//! - Lists: l<contents>e (e.g., l4:spam4:eggse)
//! - Dictionaries: d<key><value>...e (e.g., d3:cow3:moo4:spam4:eggse)

use crate::error::{Error, Result};
use std::collections::HashMap;

/// Bencode value types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BencodeValue {
    /// Integer value
    Integer(i64),

    /// Byte string (not necessarily UTF-8)
    ByteString(Vec<u8>),

    /// List of bencode values
    List(Vec<BencodeValue>),

    /// Dictionary (ordered map)
    Dictionary(HashMap<Vec<u8>, BencodeValue>),
}

impl BencodeValue {
    /// Parse bencode data from bytes
    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut parser = Parser::new(data);
        parser.parse_value()
    }

    /// Get as integer
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Self::Integer(n) => Some(*n),
            _ => None,
        }
    }

    /// Get as byte string
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::ByteString(bytes) => Some(bytes),
            _ => None,
        }
    }

    /// Get as UTF-8 string
    pub fn as_str(&self) -> Option<&str> {
        self.as_bytes()
            .and_then(|bytes| std::str::from_utf8(bytes).ok())
    }

    /// Get as list
    pub fn as_list(&self) -> Option<&[BencodeValue]> {
        match self {
            Self::List(list) => Some(list),
            _ => None,
        }
    }

    /// Get as dictionary
    pub fn as_dict(&self) -> Option<&HashMap<Vec<u8>, BencodeValue>> {
        match self {
            Self::Dictionary(dict) => Some(dict),
            _ => None,
        }
    }

    /// Get dictionary value by key
    pub fn dict_get(&self, key: &[u8]) -> Option<&BencodeValue> {
        self.as_dict().and_then(|dict| dict.get(key))
    }

    /// Get dictionary string value by key
    pub fn dict_get_str(&self, key: &[u8]) -> Option<&str> {
        self.dict_get(key).and_then(|v| v.as_str())
    }

    /// Get dictionary integer value by key
    pub fn dict_get_int(&self, key: &[u8]) -> Option<i64> {
        self.dict_get(key).and_then(|v| v.as_integer())
    }
}

/// Bencode parser
struct Parser<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn parse_value(&mut self) -> Result<BencodeValue> {
        if self.pos >= self.data.len() {
            return Err(Error::BencodeError("unexpected end of data".to_string()));
        }

        match self.data[self.pos] {
            b'i' => self.parse_integer(),
            b'l' => self.parse_list(),
            b'd' => self.parse_dictionary(),
            b'0'..=b'9' => self.parse_byte_string(),
            c => Err(Error::BencodeError(format!(
                "unexpected character: {}",
                c as char
            ))),
        }
    }

    fn parse_integer(&mut self) -> Result<BencodeValue> {
        // Format: i<number>e
        self.expect(b'i')?;

        let start = self.pos;
        let mut found_end = false;

        while self.pos < self.data.len() {
            if self.data[self.pos] == b'e' {
                found_end = true;
                break;
            }
            self.pos += 1;
        }

        if !found_end {
            return Err(Error::BencodeError("unterminated integer".to_string()));
        }

        let num_str = std::str::from_utf8(&self.data[start..self.pos])
            .map_err(|_| Error::BencodeError("invalid integer encoding".to_string()))?;

        let num = num_str
            .parse::<i64>()
            .map_err(|_| Error::BencodeError(format!("invalid integer: {num_str}")))?;

        self.expect(b'e')?;

        Ok(BencodeValue::Integer(num))
    }

    fn parse_byte_string(&mut self) -> Result<BencodeValue> {
        // Format: <length>:<bytes>
        let start = self.pos;
        let mut found_colon = false;

        while self.pos < self.data.len() {
            if self.data[self.pos] == b':' {
                found_colon = true;
                break;
            }
            self.pos += 1;
        }

        if !found_colon {
            return Err(Error::BencodeError(
                "missing colon in byte string".to_string(),
            ));
        }

        let len_str = std::str::from_utf8(&self.data[start..self.pos])
            .map_err(|_| Error::BencodeError("invalid length encoding".to_string()))?;

        let len = len_str
            .parse::<usize>()
            .map_err(|_| Error::BencodeError(format!("invalid length: {len_str}")))?;

        self.expect(b':')?;

        if self.pos + len > self.data.len() {
            return Err(Error::BencodeError(
                "string length exceeds data".to_string(),
            ));
        }

        let bytes = self.data[self.pos..self.pos + len].to_vec();
        self.pos += len;

        Ok(BencodeValue::ByteString(bytes))
    }

    fn parse_list(&mut self) -> Result<BencodeValue> {
        // Format: l<value>...e
        self.expect(b'l')?;

        let mut list = Vec::new();

        while self.pos < self.data.len() && self.data[self.pos] != b'e' {
            list.push(self.parse_value()?);
        }

        self.expect(b'e')?;

        Ok(BencodeValue::List(list))
    }

    fn parse_dictionary(&mut self) -> Result<BencodeValue> {
        // Format: d<key><value>...e
        self.expect(b'd')?;

        let mut dict = HashMap::new();

        while self.pos < self.data.len() && self.data[self.pos] != b'e' {
            // Keys must be byte strings
            let key = match self.parse_value()? {
                BencodeValue::ByteString(bytes) => bytes,
                _ => {
                    return Err(Error::BencodeError(
                        "dictionary key must be a string".to_string(),
                    ))
                }
            };

            let value = self.parse_value()?;
            dict.insert(key, value);
        }

        self.expect(b'e')?;

        Ok(BencodeValue::Dictionary(dict))
    }

    fn expect(&mut self, expected: u8) -> Result<()> {
        if self.pos >= self.data.len() {
            return Err(Error::BencodeError(format!(
                "expected '{}' but got end of data",
                expected as char
            )));
        }

        if self.data[self.pos] != expected {
            return Err(Error::BencodeError(format!(
                "expected '{}' but got '{}'",
                expected as char, self.data[self.pos] as char
            )));
        }

        self.pos += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_integer() {
        let value = BencodeValue::parse(b"i42e").unwrap();
        assert_eq!(value.as_integer(), Some(42));

        let value = BencodeValue::parse(b"i-42e").unwrap();
        assert_eq!(value.as_integer(), Some(-42));
    }

    #[test]
    fn test_parse_string() {
        let value = BencodeValue::parse(b"4:spam").unwrap();
        assert_eq!(value.as_str(), Some("spam"));
    }

    #[test]
    fn test_parse_list() {
        let value = BencodeValue::parse(b"l4:spam4:eggse").unwrap();
        let list = value.as_list().unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].as_str(), Some("spam"));
        assert_eq!(list[1].as_str(), Some("eggs"));
    }

    #[test]
    fn test_parse_dict() {
        let value = BencodeValue::parse(b"d3:cow3:moo4:spam4:eggse").unwrap();
        assert_eq!(value.dict_get_str(b"cow"), Some("moo"));
        assert_eq!(value.dict_get_str(b"spam"), Some("eggs"));
    }

    #[test]
    fn test_nested_structure() {
        // d4:listl1:a1:be6:numberi42ee
        // {"list": ["a", "b"], "number": 42}
        let data = b"d4:listl1:a1:be6:numberi42ee";
        let value = BencodeValue::parse(data).unwrap();

        let list = value.dict_get(b"list").and_then(|v| v.as_list()).unwrap();
        assert_eq!(list.len(), 2);

        assert_eq!(value.dict_get_int(b"number"), Some(42));
    }
}
