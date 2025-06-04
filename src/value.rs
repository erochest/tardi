use std::cell::RefCell;
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Write};
use std::ops::{Add, Div, Mul, Sub};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use lambda::Lambda;

use crate::error::{Error, Result, VMError};
use crate::shared::{shared, unshare_clone, Shared};

pub mod lambda;

// TODO: group Value and ValueData implementations better

/// Shared value type for all values
pub type SharedValue = Rc<RefCell<Value>>;

#[derive(Debug, Clone, Eq)]
pub struct Value {
    pub data: ValueData,

    /// The actual text of the token from source
    pub lexeme: Option<String>,

    pub pos: Option<Pos>,
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> Ordering {
        self.data.cmp(&other.data)
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

// -- they're more closely tied to `Environment` and they're part of what
// bridges across layers
// TODO: Have a Value member for doc comments so we can grab those in macros
// TODO: cache common values like small numbers, booleans, and empty collections.
/// Enum representing different types of values that can be stored on the stack
#[derive(Debug, Clone)]
pub enum ValueData {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Char(char),
    String(String),
    // TODO: rename to Vector
    List(Vec<SharedValue>),
    Function(Lambda),
    Address(usize),
    Word(String),
    Symbol { module: String, word: String },
    Macro,
    Literal(Box<Value>),
    Return(usize, bool),
    Writer(TardiWriter),
    EndOfInput,
}

#[derive(Debug, Clone)]
pub enum TardiWriter {
    // TODO: Stdout,
    // TODO: Stderr,
    File {
        name: String,
        writer: Shared<BufWriter<File>>,
    },
    // TODO: add for empty, network, and pipes
}

impl TardiWriter {
    pub fn from_path(path: &Path) -> Result<Self> {
        let name = path.to_string_lossy().to_string();
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        let writer = shared(BufWriter::new(file));
        Ok(TardiWriter::File { name, writer })
    }

    pub fn get_path(&self) -> Option<String> {
        let TardiWriter::File { name, .. } = self;
        Some(name.clone())
    }
}

impl fmt::Display for TardiWriter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let TardiWriter::File { name, .. } = self;
        write!(f, "<writer: {:?}>", name)
    }
}

impl Write for TardiWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let TardiWriter::File { ref mut writer, .. } = self;
        writer.borrow_mut().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let TardiWriter::File { ref mut writer, .. } = self;
        writer.borrow_mut().flush()
    }
}

impl ValueData {
    pub fn get_word(&self) -> Option<&str> {
        if let ValueData::Word(ref w) = self {
            Some(w)
        } else if let ValueData::Symbol { word: ref w, .. } = self {
            Some(w)
        } else {
            None
        }
    }

    pub fn as_writer(&self) -> Option<&TardiWriter> {
        if let Self::Writer(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_writer_mut(&mut self) -> Option<&mut TardiWriter> {
        if let Self::Writer(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct Pos {
    /// Line number in source (1-based)
    pub line: usize,

    /// Column number in source (1-based)
    pub column: usize,

    /// Offset from start of source (0-based)
    pub offset: usize,

    /// Length of the token in characters
    pub length: usize,
}

impl Value {
    pub fn new(data: ValueData) -> Self {
        Value {
            data,
            lexeme: None,
            pos: None,
        }
    }

    pub fn with_lexeme(data: ValueData, lexeme: &str) -> Self {
        Value {
            data,
            lexeme: Some(lexeme.to_string()),
            pos: None,
        }
    }

    pub fn with_pos(data: ValueData, lexeme: &str, pos: Pos) -> Self {
        Value {
            data,
            lexeme: Some(lexeme.to_string()),
            pos: Some(pos),
        }
    }

    pub fn from_parts(
        data: ValueData,
        lexeme: &str,
        line: usize,
        column: usize,
        offset: usize,
        length: usize,
    ) -> Self {
        let pos = Pos {
            line,
            column,
            offset,
            length,
        };
        Value::with_pos(data, lexeme, pos)
    }

    // TODO: change these into as_boolean
    pub fn as_boolean(&self) -> Option<bool> {
        if let ValueData::Boolean(b) = self.data {
            Some(b)
        } else {
            None
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        if let ValueData::Integer(i) = self.data {
            Some(i)
        } else {
            None
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        if let ValueData::String(ref s) = self.data {
            Some(s)
        } else {
            None
        }
    }

    pub fn as_char(&self) -> Option<char> {
        if let ValueData::Char(c) = self.data {
            Some(c)
        } else {
            None
        }
    }

    pub fn is_list(&self) -> bool {
        matches!(self.data, ValueData::List(_))
    }

    pub fn as_list(&self) -> Option<&Vec<SharedValue>> {
        if let ValueData::List(ref list) = self.data {
            Some(list)
        } else {
            None
        }
    }

    pub fn as_list_mut(&mut self) -> Option<&mut Vec<SharedValue>> {
        if let ValueData::List(ref mut list) = self.data {
            Some(list)
        } else {
            None
        }
    }

    pub fn as_function(&self) -> Option<&Lambda> {
        if let ValueData::Function(ref callable) = self.data {
            Some(callable)
        } else {
            None
        }
    }

    pub fn as_function_mut(&mut self) -> Option<&mut Lambda> {
        if let ValueData::Function(ref mut callable) = self.data {
            Some(callable)
        } else {
            None
        }
    }

    pub fn as_address(&self) -> Option<usize> {
        match self.data {
            ValueData::Address(address) => Some(address),
            ValueData::Return(address, _) => Some(address),
            _ => None,
        }
    }

    pub fn is_breakpoint(&self) -> bool {
        if let ValueData::Return(_, breakpoint) = self.data {
            breakpoint
        } else {
            false
        }
    }

    pub fn as_word(&self) -> Option<&str> {
        if let ValueData::Word(ref w) = self.data {
            Some(w)
        } else if let ValueData::Symbol { word: ref w, .. } = self.data {
            Some(w)
        } else {
            None
        }
    }

    pub fn as_symbol(&self) -> Option<(&str, &str)> {
        if let ValueData::Symbol {
            ref module,
            ref word,
        } = self.data
        {
            Some((module, word))
        } else {
            None
        }
    }
}

// From implementations for each of the contained values
impl From<i64> for ValueData {
    fn from(value: i64) -> Self {
        ValueData::Integer(value)
    }
}

impl From<f64> for ValueData {
    fn from(value: f64) -> Self {
        ValueData::Float(value)
    }
}

impl From<bool> for ValueData {
    fn from(value: bool) -> Self {
        ValueData::Boolean(value)
    }
}

impl From<char> for ValueData {
    fn from(value: char) -> Self {
        ValueData::Char(value)
    }
}

impl From<&str> for ValueData {
    fn from(value: &str) -> Self {
        ValueData::String(value.to_string())
    }
}

impl From<String> for ValueData {
    fn from(value: String) -> Self {
        ValueData::String(value)
    }
}

// From implementations for each of the contained values
impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::with_lexeme(ValueData::Integer(value), &value.to_string())
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::with_lexeme(ValueData::Float(value), &value.to_string())
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::with_lexeme(ValueData::Boolean(value), &value.to_string())
    }
}

impl From<char> for Value {
    fn from(value: char) -> Self {
        Value::with_lexeme(ValueData::Char(value), &value.to_string())
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        let value = ValueData::String(value);
        let lexeme = format!("{}", value);
        Value::with_lexeme(value, &lexeme)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        let value = value.to_string();
        let value = ValueData::String(value);
        let lexeme = format!("{}", value);
        Value::with_lexeme(value, &lexeme)
    }
}

impl From<u8> for Value {
    fn from(byte: u8) -> Self {
        Value::from(byte as i64)
    }
}

impl From<i32> for Value {
    fn from(byte: i32) -> Self {
        Value::from(byte as i64)
    }
}

impl From<Vec<SharedValue>> for Value {
    fn from(value: Vec<SharedValue>) -> Self {
        let repr = value
            .iter()
            .map(|v| format!("{}", &v.borrow()))
            .collect::<Vec<_>>();
        let repr = format!("[ {} ]", repr.join(" "));
        Value::with_lexeme(ValueData::List(value), &repr)
    }
}

impl From<ValueData> for Value {
    fn from(value: ValueData) -> Self {
        Value::new(value)
    }
}

impl<V> From<Vec<V>> for Value
where
    V: Into<Value>,
{
    fn from(vector: Vec<V>) -> Self
    where
        V: Into<Value>,
    {
        vector
            .into_iter()
            .map(|v| shared(v.into()))
            .collect::<Vec<_>>()
            .into()
    }
}

impl TryFrom<Value> for Vec<Value> {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match value.data {
            ValueData::List(list) => Ok(list.into_iter().map(unshare_clone).collect()),
            _ => {
                Err(VMError::TypeMismatch(format!("cannot convert to list {}", value.data)).into())
            }
        }
    }
}

pub struct ValueVec<'a>(pub &'a Vec<Value>);

impl fmt::Display for ValueVec<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ValueVec(values) = self;
        write!(
            f,
            "{{ {} }}",
            values
                .iter()
                .map(|v| format!("{}", v))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.data)
    }
}

impl fmt::Display for ValueData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueData::Integer(n) => write!(f, "{}", n),
            ValueData::Float(x) => {
                let s = format!("{}", x);
                if !s.contains('.') {
                    write!(f, "{}.0", s)
                } else {
                    write!(f, "{}", s)
                }
            }
            ValueData::Boolean(true) => write!(f, "#t"),
            ValueData::Boolean(false) => write!(f, "#f"),
            ValueData::Char(c) => match c {
                '\n' => write!(f, "'\\n'"),
                '\r' => write!(f, "'\\r'"),
                '\t' => write!(f, "'\\t'"),
                '\\' => write!(f, "'\\\\'"),
                '\'' => write!(f, "'\\''"),
                c => write!(f, "'{}'", c),
            },
            ValueData::List(list) => {
                write!(f, "{{")?;
                for item in list.iter() {
                    write!(f, " {}", item.borrow())?;
                }
                write!(f, " }}")
            }
            // TODO: don't quote this
            ValueData::String(s) => write!(f, "\"{}\"", s.replace('"', "\\\"")),
            ValueData::Function(lambda) => write!(f, "{}", lambda),
            ValueData::Address(addr) => write!(f, "<@{}>", addr),
            ValueData::Word(word) => write!(f, "{}", word),
            // TODO: escape word if it starts with punctuation (`module::\:` or something)
            ValueData::Symbol { module, word } => write!(f, "{}::{}", module, word),
            ValueData::Macro => write!(f, "MACRO:"),
            ValueData::Literal(value) => write!(f, "\\ {}", value),
            ValueData::Return(address, breakpoint) => write!(f, "<@{} - {}>", address, breakpoint),
            ValueData::Writer(writer) => write!(f, "{}", writer),
            ValueData::EndOfInput => write!(f, "<EOI>"),
        }
    }
}

impl Eq for ValueData {}

impl Ord for ValueData {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Less)
    }
}

// TODO: should symbols also match on strings?
impl PartialEq for ValueData {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ValueData::Integer(a), ValueData::Integer(b)) => a == b,
            (ValueData::Float(a), ValueData::Float(b)) => a == b,
            (ValueData::Integer(a), ValueData::Float(b)) => (*a as f64) == *b,
            (ValueData::Float(a), ValueData::Integer(b)) => *a == (*b as f64),
            (ValueData::Boolean(a), ValueData::Boolean(b)) => a == b,
            (ValueData::Char(a), ValueData::Char(b)) => a == b,
            (ValueData::String(a), ValueData::String(b)) => a == b,
            (ValueData::List(a), ValueData::List(b)) => {
                a.len() == b.len()
                    && a.iter()
                        .zip(b.iter())
                        .all(|(x, y)| *x.borrow() == *y.borrow())
            }
            (ValueData::Function(a), ValueData::Function(b)) => a == b,
            (ValueData::Address(a), ValueData::Address(b)) => a == b,
            (ValueData::Word(a), ValueData::Word(b)) => a == b,
            (ValueData::Word(a), ValueData::Symbol { word: b, .. }) => a == b,
            (ValueData::Symbol { word: a, .. }, ValueData::Word(b)) => a == b,
            (
                ValueData::Symbol {
                    module: m1,
                    word: a,
                },
                ValueData::Symbol {
                    module: m2,
                    word: b,
                },
            ) => m1 == m2 && a == b,
            (ValueData::Macro, ValueData::Macro) => true,
            (ValueData::Literal(a), ValueData::Literal(b)) => a == b,
            (ValueData::EndOfInput, ValueData::EndOfInput) => true,
            _ => false,
        }
    }
}

impl PartialOrd for ValueData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (ValueData::Integer(a), ValueData::Integer(b)) => a.partial_cmp(b),
            (ValueData::Float(a), ValueData::Float(b)) => a.partial_cmp(b),
            (ValueData::Integer(a), ValueData::Float(b)) => (*a as f64).partial_cmp(b),
            (ValueData::Float(a), ValueData::Integer(b)) => a.partial_cmp(&(*b as f64)),
            (ValueData::Char(a), ValueData::Char(b)) => a.partial_cmp(b),
            (ValueData::Boolean(a), ValueData::Boolean(b)) => a.partial_cmp(b),
            (ValueData::List(a), ValueData::List(b)) => {
                // First compare lengths
                match a.len().partial_cmp(&b.len()) {
                    Some(Ordering::Equal) => {
                        // If lengths are equal, compare elements
                        for (x, y) in a.iter().zip(b.iter()) {
                            let x_val = &*x.borrow();
                            let y_val = &*y.borrow();
                            match x_val.partial_cmp(y_val) {
                                Some(Ordering::Equal) => continue,
                                other => return other,
                            }
                        }
                        Some(Ordering::Equal)
                    }
                    other => other,
                }
            }
            (ValueData::String(a), ValueData::String(b)) => a.partial_cmp(b),
            (ValueData::Function(a), ValueData::Function(b)) => a.partial_cmp(b), // Functions cannot be ordered
            (ValueData::Word(a), ValueData::Word(b)) => a.partial_cmp(b),
            (
                ValueData::Symbol {
                    module: m1,
                    word: w1,
                },
                ValueData::Symbol {
                    module: m2,
                    word: w2,
                },
            ) => (m1, w1).partial_cmp(&(m2, w2)),

            _ => None,
        }
    }
}

impl Add for Value {
    type Output = Result<Self>;

    fn add(self, rhs: Self) -> Self::Output {
        Ok(Value::new((self.data + rhs.data)?))
    }
}

impl Add for ValueData {
    type Output = Result<Self>;

    fn add(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            (ValueData::Integer(a), ValueData::Integer(b)) => Ok(ValueData::Integer(a + b)),
            (ValueData::Float(a), ValueData::Float(b)) => Ok(ValueData::Float(a + b)),
            (ValueData::Integer(a), ValueData::Float(b)) => Ok(ValueData::Float(*a as f64 + b)),
            (ValueData::Float(a), ValueData::Integer(b)) => Ok(ValueData::Float(a + *b as f64)),
            _ => Err(VMError::TypeMismatch(format!("{:?} + {:?}", &self, &rhs)).into()),
        }
    }
}

impl Sub for Value {
    type Output = Result<Self>;

    fn sub(self, rhs: Self) -> Self::Output {
        Ok(Value::new((self.data - rhs.data)?))
    }
}

impl Sub for ValueData {
    type Output = Result<Self>;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (ValueData::Integer(a), ValueData::Integer(b)) => Ok(ValueData::Integer(a - b)),
            (ValueData::Float(a), ValueData::Float(b)) => Ok(ValueData::Float(a - b)),
            (ValueData::Integer(a), ValueData::Float(b)) => Ok(ValueData::Float(a as f64 - b)),
            (ValueData::Float(a), ValueData::Integer(b)) => Ok(ValueData::Float(a - b as f64)),
            _ => Err(VMError::TypeMismatch("subtraction".to_string()).into()),
        }
    }
}

impl Mul for Value {
    type Output = Result<Self>;

    fn mul(self, rhs: Self) -> Self::Output {
        Ok(Value::new(self.data.mul(rhs.data)?))
    }
}

impl Mul for ValueData {
    type Output = Result<Self>;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (ValueData::Integer(a), ValueData::Integer(b)) => Ok(ValueData::Integer(a * b)),
            (ValueData::Float(a), ValueData::Float(b)) => Ok(ValueData::Float(a * b)),
            (ValueData::Integer(a), ValueData::Float(b)) => Ok(ValueData::Float(a as f64 * b)),
            (ValueData::Float(a), ValueData::Integer(b)) => Ok(ValueData::Float(a * b as f64)),
            _ => Err(VMError::TypeMismatch("multiplication".to_string()).into()),
        }
    }
}

impl Div for Value {
    type Output = Result<Self>;

    fn div(self, rhs: Self) -> Self::Output {
        Ok(Value::new(self.data.div(rhs.data)?))
    }
}

impl Div for ValueData {
    type Output = Result<Self>;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (ValueData::Integer(a), ValueData::Integer(b)) => {
                if b == 0 {
                    Err(VMError::DivisionByZero.into())
                } else {
                    Ok(ValueData::Integer(a / b))
                }
            }
            (ValueData::Float(a), ValueData::Float(b)) => {
                if b == 0.0 {
                    Err(VMError::DivisionByZero.into())
                } else {
                    Ok(ValueData::Float(a / b))
                }
            }
            (ValueData::Integer(a), ValueData::Float(b)) => {
                if b == 0.0 {
                    Err(VMError::DivisionByZero.into())
                } else {
                    Ok(ValueData::Float(a as f64 / b))
                }
            }
            (ValueData::Float(a), ValueData::Integer(b)) => {
                if b == 0 {
                    Err(VMError::DivisionByZero.into())
                } else {
                    Ok(ValueData::Float(a / b as f64))
                }
            }
            _ => Err(VMError::TypeMismatch("division".to_string()).into()),
        }
    }
}
