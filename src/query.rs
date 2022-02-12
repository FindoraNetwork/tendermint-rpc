//! Structured querying for the Tendermint RPC event subscription system.
//!
//! See [`Query`] for details as to how to construct queries.
//!
//! [`Query`]: struct.Query.html

// TODO(thane): These warnings are generated by the PEG for some reason. Try to fix and remove.
#![allow(clippy::redundant_closure_call, clippy::unit_arg)]

use crate::{Error, Result};
use chrono::{Date, DateTime, FixedOffset, Utc};
use std::fmt;
use std::str::FromStr;

/// A structured query for use in interacting with the Tendermint RPC event
/// subscription system.
///
/// Allows for compile-time validation of queries.
///
/// See the [subscribe endpoint documentation] for more details.
///
/// ## Examples
///
/// ### Direct construction of queries
///
/// ```rust
/// use tendermint_rpc::query::{Query, EventType};
///
/// let query = Query::from(EventType::NewBlock);
/// assert_eq!("tm.event = 'NewBlock'", query.to_string());
///
/// let query = Query::from(EventType::Tx).and_eq("tx.hash", "XYZ");
/// assert_eq!("tm.event = 'Tx' AND tx.hash = 'XYZ'", query.to_string());
///
/// let query = Query::from(EventType::Tx).and_gte("tx.height", 100_u64);
/// assert_eq!("tm.event = 'Tx' AND tx.height >= 100", query.to_string());
/// ```
///
/// [subscribe endpoint documentation]: https://docs.tendermint.com/master/rpc/#/Websocket/subscribe
#[derive(Debug, Clone, PartialEq)]
pub struct Query {
    // We can only have at most one event type at present in a query.
    event_type: Option<EventType>,
    // We can have zero or more additional conditions associated with a query.
    // Conditions are currently exclusively joined by logical ANDs.
    conditions: Vec<Condition>,
}

impl Query {
    /// Query constructor testing whether `<key> = <value>`
    pub fn eq(key: impl ToString, value: impl Into<Operand>) -> Self {
        Self {
            event_type: None,
            conditions: vec![Condition::Eq(key.to_string(), value.into())],
        }
    }

    /// Query constructor testing whether `<key> < <value>`
    pub fn lt(key: impl ToString, value: impl Into<Operand>) -> Self {
        Self {
            event_type: None,
            conditions: vec![Condition::Lt(key.to_string(), value.into())],
        }
    }

    /// Query constructor testing whether `<key> <= <value>`
    pub fn lte(key: impl ToString, value: impl Into<Operand>) -> Self {
        Self {
            event_type: None,
            conditions: vec![Condition::Lte(key.to_string(), value.into())],
        }
    }

    /// Query constructor testing whether `<key> > <value>`
    pub fn gt(key: impl ToString, value: impl Into<Operand>) -> Self {
        Self {
            event_type: None,
            conditions: vec![Condition::Gt(key.to_string(), value.into())],
        }
    }

    /// Query constructor testing whether `<key> >= <value>`
    pub fn gte(key: impl ToString, value: impl Into<Operand>) -> Self {
        Self {
            event_type: None,
            conditions: vec![Condition::Gte(key.to_string(), value.into())],
        }
    }

    /// Query constructor testing whether `<key> CONTAINS <value>` (assuming
    /// `key` contains a string, this tests whether `value` is a sub-string
    /// within it).
    pub fn contains(key: impl ToString, value: impl ToString) -> Self {
        Self {
            event_type: None,
            conditions: vec![Condition::Contains(key.to_string(), value.to_string())],
        }
    }

    /// Query constructor testing whether `<key> EXISTS`.
    pub fn exists(key: impl ToString) -> Self {
        Self {
            event_type: None,
            conditions: vec![Condition::Exists(key.to_string())],
        }
    }

    /// Add the condition `<key> = <value>` to the query.
    pub fn and_eq(mut self, key: impl ToString, value: impl Into<Operand>) -> Self {
        self.conditions
            .push(Condition::Eq(key.to_string(), value.into()));
        self
    }

    /// Add the condition `<key> < <value>` to the query.
    pub fn and_lt(mut self, key: impl ToString, value: impl Into<Operand>) -> Self {
        self.conditions
            .push(Condition::Lt(key.to_string(), value.into()));
        self
    }

    /// Add the condition `<key> <= <value>` to the query.
    pub fn and_lte(mut self, key: impl ToString, value: impl Into<Operand>) -> Self {
        self.conditions
            .push(Condition::Lte(key.to_string(), value.into()));
        self
    }

    /// Add the condition `<key> > <value>` to the query.
    pub fn and_gt(mut self, key: impl ToString, value: impl Into<Operand>) -> Self {
        self.conditions
            .push(Condition::Gt(key.to_string(), value.into()));
        self
    }

    /// Add the condition `<key> >= <value>` to the query.
    pub fn and_gte(mut self, key: impl ToString, value: impl Into<Operand>) -> Self {
        self.conditions
            .push(Condition::Gte(key.to_string(), value.into()));
        self
    }

    /// Add the condition `<key> CONTAINS <value>` to the query.
    pub fn and_contains(mut self, key: impl ToString, value: impl ToString) -> Self {
        self.conditions
            .push(Condition::Contains(key.to_string(), value.to_string()));
        self
    }

    /// Add the condition `<key> EXISTS` to the query.
    pub fn and_exists(mut self, key: impl ToString) -> Self {
        self.conditions.push(Condition::Exists(key.to_string()));
        self
    }
}

impl Default for Query {
    /// An empty query matches any set of events. See [these docs].
    ///
    /// [these docs]: https://godoc.org/github.com/tendermint/tendermint/libs/pubsub/query#Empty
    fn default() -> Self {
        Self {
            event_type: None,
            conditions: Vec::new(),
        }
    }
}

impl From<EventType> for Query {
    fn from(t: EventType) -> Self {
        Self {
            event_type: Some(t),
            conditions: Vec::new(),
        }
    }
}

impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(t) = &self.event_type {
            write!(f, "tm.event = '{}'", t)?;

            if !self.conditions.is_empty() {
                write!(f, " AND ")?;
            }
        }

        join(f, " AND ", &self.conditions)?;

        Ok(())
    }
}

fn join<S, I>(f: &mut fmt::Formatter<'_>, separator: S, iterable: I) -> fmt::Result
where
    S: fmt::Display,
    I: IntoIterator,
    I::Item: fmt::Display,
{
    let mut iter = iterable.into_iter();
    if let Some(first) = iter.next() {
        write!(f, "{}", first)?;
    }

    for item in iter {
        write!(f, "{}{}", separator, item)?;
    }

    Ok(())
}

/// The types of Tendermint events for which we can query at present.
#[derive(Debug, Clone, PartialEq)]
pub enum EventType {
    NewBlock,
    Tx,
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventType::NewBlock => write!(f, "NewBlock"),
            EventType::Tx => write!(f, "Tx"),
        }
    }
}

impl FromStr for EventType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "NewBlock" => Ok(Self::NewBlock),
            "Tx" => Ok(Self::Tx),
            invalid => Err(Error::invalid_params(&format!(
                "unrecognized event type: {}",
                invalid
            ))),
        }
    }
}

/// The different types of conditions supported by a [`Query`].
///
/// [`Query`]: struct.Query.html
#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    /// Equals
    Eq(String, Operand),
    /// Less than
    Lt(String, Operand),
    /// Less than or equal to
    Lte(String, Operand),
    /// Greater than
    Gt(String, Operand),
    /// Greater than or equal to
    Gte(String, Operand),
    /// Contains (to check if a key contains a certain sub-string)
    Contains(String, String),
    /// Exists (to check if a key exists)
    Exists(String),
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Condition::Eq(key, op) => write!(f, "{} = {}", key, op),
            Condition::Lt(key, op) => write!(f, "{} < {}", key, op),
            Condition::Lte(key, op) => write!(f, "{} <= {}", key, op),
            Condition::Gt(key, op) => write!(f, "{} > {}", key, op),
            Condition::Gte(key, op) => write!(f, "{} >= {}", key, op),
            Condition::Contains(key, op) => write!(f, "{} CONTAINS {}", key, escape(op)),
            Condition::Exists(key) => write!(f, "{} EXISTS", key),
        }
    }
}

/// A typed operand for use in an [`Condition`].
///
/// According to the [Tendermint RPC subscribe docs][tm-subscribe],
/// an operand can be a string, number, date or time. We differentiate here
/// between integer and floating point numbers.
///
/// [`Condition`]: enum.Condition.html
/// [tm-subscribe]: https://docs.tendermint.com/master/rpc/#/Websocket/subscribe
#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    String(String),
    Signed(i64),
    Unsigned(u64),
    Float(f64),
    Date(Date<Utc>),
    DateTime(DateTime<Utc>),
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::String(s) => write!(f, "{}", escape(s)),
            Operand::Signed(i) => write!(f, "{}", i),
            Operand::Unsigned(u) => write!(f, "{}", u),
            Operand::Float(h) => write!(f, "{}", h),
            Operand::Date(d) => write!(f, "DATE {}", d.format("%Y-%m-%d").to_string()),
            Operand::DateTime(dt) => write!(f, "TIME {}", dt.to_rfc3339()),
        }
    }
}

impl From<String> for Operand {
    fn from(source: String) -> Self {
        Operand::String(source)
    }
}

impl From<char> for Operand {
    fn from(source: char) -> Self {
        Operand::String(source.to_string())
    }
}

impl From<&str> for Operand {
    fn from(source: &str) -> Self {
        Operand::String(source.to_string())
    }
}

impl From<i64> for Operand {
    fn from(source: i64) -> Self {
        Operand::Signed(source)
    }
}

impl From<i32> for Operand {
    fn from(source: i32) -> Self {
        Operand::Signed(source as i64)
    }
}

impl From<i16> for Operand {
    fn from(source: i16) -> Self {
        Operand::Signed(source as i64)
    }
}

impl From<i8> for Operand {
    fn from(source: i8) -> Self {
        Operand::Signed(source as i64)
    }
}

impl From<u64> for Operand {
    fn from(source: u64) -> Self {
        Operand::Unsigned(source)
    }
}

impl From<u32> for Operand {
    fn from(source: u32) -> Self {
        Operand::Unsigned(source as u64)
    }
}

impl From<u16> for Operand {
    fn from(source: u16) -> Self {
        Operand::Unsigned(source as u64)
    }
}

impl From<u8> for Operand {
    fn from(source: u8) -> Self {
        Operand::Unsigned(source as u64)
    }
}

impl From<usize> for Operand {
    fn from(source: usize) -> Self {
        Operand::Unsigned(source as u64)
    }
}

impl From<f64> for Operand {
    fn from(source: f64) -> Self {
        Operand::Float(source)
    }
}

impl From<f32> for Operand {
    fn from(source: f32) -> Self {
        Operand::Float(source as f64)
    }
}

impl From<Date<Utc>> for Operand {
    fn from(source: Date<Utc>) -> Self {
        Operand::Date(source)
    }
}

impl From<DateTime<Utc>> for Operand {
    fn from(source: DateTime<Utc>) -> Self {
        Operand::DateTime(source)
    }
}

impl From<DateTime<FixedOffset>> for Operand {
    fn from(source: DateTime<FixedOffset>) -> Self {
        Operand::DateTime(source.into())
    }
}

/// Escape backslashes and single quotes within the given string with a backslash.
fn escape(s: &str) -> String {
    let mut result = String::new();
    for ch in s.chars() {
        if ch == '\\' || ch == '\'' {
            result.push('\\');
        }
        result.push(ch);
    }
    format!("'{}'", result)
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn empty_query() {
        let query = Query::default();
        assert_eq!("", query.to_string());
    }

    #[test]
    fn simple_event_type() {
        let query = Query::from(EventType::NewBlock);
        assert_eq!("tm.event = 'NewBlock'", query.to_string());

        let query = Query::from(EventType::Tx);
        assert_eq!("tm.event = 'Tx'", query.to_string());
    }

    #[test]
    fn simple_condition() {
        let query = Query::eq("key", "value");
        assert_eq!("key = 'value'", query.to_string());

        let query = Query::eq("key", 'v');
        assert_eq!("key = 'v'", query.to_string());

        let query = Query::eq("key", "'value'");
        assert_eq!("key = '\\'value\\''", query.to_string());

        let query = Query::eq("key", "\\'value'");
        assert_eq!("key = '\\\\\\'value\\''", query.to_string());

        let query = Query::lt("key", 42_i64);
        assert_eq!("key < 42", query.to_string());

        let query = Query::lt("key", 42_u64);
        assert_eq!("key < 42", query.to_string());

        let query = Query::lte("key", 42_i64);
        assert_eq!("key <= 42", query.to_string());

        let query = Query::gt("key", 42_i64);
        assert_eq!("key > 42", query.to_string());

        let query = Query::gte("key", 42_i64);
        assert_eq!("key >= 42", query.to_string());

        let query = Query::eq("key", 42_u8);
        assert_eq!("key = 42", query.to_string());

        let query = Query::contains("key", "some-substring");
        assert_eq!("key CONTAINS 'some-substring'", query.to_string());

        let query = Query::exists("key");
        assert_eq!("key EXISTS", query.to_string());
    }

    #[test]
    fn date_condition() {
        let query = Query::eq(
            "some_date",
            Date::from_utc(NaiveDate::from_ymd(2020, 9, 24), Utc),
        );
        assert_eq!("some_date = DATE 2020-09-24", query.to_string());
    }

    #[test]
    fn date_time_condition() {
        let query = Query::eq(
            "some_date_time",
            DateTime::parse_from_rfc3339("2020-09-24T10:17:23-04:00").unwrap(),
        );
        assert_eq!(
            "some_date_time = TIME 2020-09-24T14:17:23+00:00",
            query.to_string()
        );
    }

    #[test]
    fn complex_query() {
        let query = Query::from(EventType::Tx).and_eq("tx.height", 3_i64);
        assert_eq!("tm.event = 'Tx' AND tx.height = 3", query.to_string());

        let query = Query::from(EventType::Tx)
            .and_lte("tx.height", 100_i64)
            .and_eq("transfer.sender", "AddrA");
        assert_eq!(
            "tm.event = 'Tx' AND tx.height <= 100 AND transfer.sender = 'AddrA'",
            query.to_string()
        );

        let query = Query::from(EventType::Tx)
            .and_lte("tx.height", 100_i64)
            .and_contains("meta.attr", "some-substring");
        assert_eq!(
            "tm.event = 'Tx' AND tx.height <= 100 AND meta.attr CONTAINS 'some-substring'",
            query.to_string()
        );
    }
}