use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::{File, OpenOptions};
use std::hash::Hash;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::ops::{Add, Div, Mul, Sub};
use std::path::Path;
use std::rc::Rc;
use std::{error, fmt, io};

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

impl Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.data.hash(state);
        // We don't include lexeme and pos in the hash calculation
        // because they don't affect equality in PartialEq
    }
}

// -- they're more closely tied to `Environment` and they're part of what
// bridges across layers
// TODO: Have a Value member for doc comments so we can grab those in macros
// TODO: cache common values like small numbers, booleans, and empty collections.
// XXX: need some way to freeze these values for hashmap keys. one option might
// be to have frozen versions of members that have interior mutability. will
// need to make sure they have Eq, PartialEq, and Hash the same though
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
    HashMap(HashMap<ValueData, SharedValue>),
    Function(Lambda),
    Address(usize),
    Word(String),
    Symbol { module: String, word: String },
    Macro,
    Literal(Box<Value>),
    Return(usize, bool),
    Writer(TardiWriter),
    Reader(TardiReader),
    EndOfInput,
}

// TODO: pull this into its own module
#[derive(Debug)]
pub enum TardiIoError {
    ResourceClosed(String),
}

impl fmt::Display for TardiIoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TardiIoError::ResourceClosed(name) => write!(f, "resource closed: {}", name),
        }
    }
}

impl error::Error for TardiIoError {}

impl From<TardiIoError> for Error {
    fn from(value: TardiIoError) -> Self {
        Error::TardiError(Box::new(value))
    }
}

#[derive(Debug, Clone, Default)]
pub enum TardiWriter {
    #[default]
    Stdout,
    Stderr,
    File {
        name: String,
        // TODO: make this an Option<BufWriter<File>>> and if it's consumed.
        // TODO: if it's None, return an error `#f`
        writer: Shared<BufWriter<File>>,
    },
    // TODO: add for empty, network, and pipes
}

impl Hash for TardiWriter {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            TardiWriter::Stdout => {
                "stdout".hash(state);
            }
            TardiWriter::Stderr => {
                "stderr".hash(state);
            }
            TardiWriter::File { name, .. } => {
                "file".hash(state);
                name.hash(state);
            }
        }
    }
}

impl Eq for TardiWriter {}

impl PartialEq for TardiWriter {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TardiWriter::Stdout, TardiWriter::Stdout) => true,
            (TardiWriter::Stderr, TardiWriter::Stderr) => true,
            (TardiWriter::File { name: name1, .. }, TardiWriter::File { name: name2, .. }) => {
                name1 == name2
            }
            _ => false,
        }
    }
}

impl TardiWriter {
    pub fn from_path(path: &Path) -> Result<Self> {
        let name = path.to_string_lossy().to_string();
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        let writer = shared(BufWriter::new(file));
        Ok(TardiWriter::File { name, writer })
    }

    pub fn get_path(&self) -> Option<String> {
        let name = match self {
            TardiWriter::Stdout => "<stdout>".to_string(),
            TardiWriter::Stderr => "<stderr>".to_string(),
            TardiWriter::File { name, .. } => name.clone(),
        };
        Some(name)
    }
}

impl fmt::Display for TardiWriter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.get_path().unwrap_or_else(|| "<unknown>".to_string());
        write!(f, "<writer: {:?}>", name)
    }
}

impl Write for TardiWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            TardiWriter::Stdout => {
                let stdout = io::stdout();
                let mut stdout = stdout.lock();
                stdout.write(buf)
            }
            TardiWriter::Stderr => {
                let stderr = io::stderr();
                let mut stderr = stderr.lock();
                stderr.write(buf)
            }
            TardiWriter::File { ref mut writer, .. } => writer.borrow_mut().write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            TardiWriter::Stdout => {
                let stdout = io::stdout();
                let mut stdout = stdout.lock();
                stdout.flush()
            }
            TardiWriter::Stderr => {
                let stderr = io::stderr();
                let mut stderr = stderr.lock();
                stderr.flush()
            }
            TardiWriter::File { ref mut writer, .. } => writer.borrow_mut().flush(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum TardiReader {
    #[default]
    Stdin,
    File {
        name: String,
        reader: Shared<Option<BufReader<File>>>,
    },
    // TODO: add for empty, network, and pipes
}

impl Hash for TardiReader {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            TardiReader::Stdin => {
                "stdin".hash(state);
            }
            TardiReader::File { name, .. } => {
                "file".hash(state);
                name.hash(state);
            }
        }
    }
}

impl Eq for TardiReader {}

impl PartialEq for TardiReader {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TardiReader::Stdin, TardiReader::Stdin) => true,
            (TardiReader::File { name: name1, .. }, TardiReader::File { name: name2, .. }) => {
                name1 == name2
            }
            _ => false,
        }
    }
}

impl TardiReader {
    pub fn from_path(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(TardiReader::File {
            name: path.to_string_lossy().to_string(),
            reader: shared(Some(reader)),
        })
    }

    pub fn get_path(&self) -> Option<String> {
        match self {
            TardiReader::File { name, .. } => Some(name.clone()),
            TardiReader::Stdin => Some("<stdin>".to_string()),
        }
    }

    pub fn is_consumed(&self) -> bool {
        match self {
            TardiReader::File { reader, .. } => reader.borrow().is_none(),
            TardiReader::Stdin => {
                false
                // TODO: this is unstable. use this later. in the meantime, we'll assume it's always good
                // let stdin = io::stdin();
                // let mut stdin = stdin.lock();
                // !stdin.has_data_left().unwrap_or(false)
            }
        }
    }

    // fn with_reader<'a, F>(&'a mut self, action: &mut F) -> Result<()>
    //     where F: FnMut(Box<dyn BufRead + 'a>) -> Result<()> {
    //     match self {
    //         TardiReader::Stdin => {
    //             let stdin = io::stdin();
    //             let stdin = stdin.lock();
    //             action(Box::new(stdin))
    //         }
    //         TardiReader::File { reader, name, .. } => {
    //             let mut reader = reader.borrow_mut();
    //             let reader = reader.as_mut();
    //             if let Some(reader) = reader {
    //                 action(Box::new(reader))
    //             } else {
    //                 Err(TardiIoError::ResourceClosed(name.clone()).into())
    //             }
    //         }
    //                 }
    // }

    pub fn read_line(&mut self) -> Result<String> {
        let mut buffer = String::new();

        match self {
            TardiReader::Stdin => {
                let stdin = io::stdin();
                let mut stdin = stdin.lock();
                stdin.read_line(&mut buffer)?;
            }
            TardiReader::File { reader, name, .. } => {
                if let Some(ref mut reader) = reader.borrow_mut().as_mut() {
                    reader.read_line(&mut buffer)?;
                } else {
                    return Err(TardiIoError::ResourceClosed(name.clone()).into());
                }
            }
        }

        Ok(buffer)
    }

    pub fn read_lines(&mut self) -> Result<Vec<String>> {
        let lines = match self {
            TardiReader::Stdin => {
                let stdin = io::stdin();
                let stdin = stdin.lock();
                stdin.lines().collect::<io::Result<Vec<_>>>()?
            }
            TardiReader::File { name, reader } => {
                if let Some(ref mut reader) = reader.borrow_mut().as_mut() {
                    reader.lines().collect::<io::Result<Vec<_>>>()?
                } else {
                    return Err(TardiIoError::ResourceClosed(name.clone()).into());
                }
            }
        };

        Ok(lines)
    }
}

impl fmt::Display for TardiReader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            TardiReader::Stdin => "<stdin>".to_string(),
            TardiReader::File { name, .. } => name.clone(),
        };
        write!(f, "<reader: {:?}>", name)
    }
}

impl Read for TardiReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // TODO: be more defensive
        match self {
            TardiReader::Stdin => {
                let stdin = io::stdin();
                let mut stdin = stdin.lock();
                stdin.read(buf)
            }
            TardiReader::File { reader, .. } => {
                reader.borrow_mut().as_mut().map(|r| r.read(buf)).unwrap()
            }
        }
    }
}

impl ValueData {
    pub fn to_repr(&self) -> String {
        if let ValueData::String(ref s) = self {
            format!("\"{}\"", s.replace("\\", "\\\\").replace("\"", "\\\""))
        } else {
            self.to_string()
        }
    }

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

    pub fn as_reader(&self) -> Option<&TardiReader> {
        if let Self::Reader(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_reader_mut(&mut self) -> Option<&mut TardiReader> {
        if let Self::Reader(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the value data is [`HashMap`].
    ///
    /// [`HashMap`]: ValueData::HashMap
    pub fn is_hash_map(&self) -> bool {
        matches!(self, Self::HashMap(..))
    }

    pub fn as_hash_map(&self) -> Option<&HashMap<ValueData, SharedValue>> {
        if let Self::HashMap(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_hash_map_mut(&mut self) -> Option<&mut HashMap<ValueData, SharedValue>> {
        if let Self::HashMap(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
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

    pub fn to_repr(&self) -> String {
        if let ValueData::String(ref s) = self.data {
            format!("\"{}\"", s.replace("\\", "\\\\").replace("\"", "\\\""))
        } else {
            self.data.to_string()
        }
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
                .map(|v| v.to_repr())
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
                    write!(f, " {}", item.borrow().to_repr())?;
                }
                write!(f, " }}")
            }
            ValueData::HashMap(hash_map) => {
                write!(f, "H{{")?;
                let mut pairs = hash_map.iter().collect::<Vec<_>>();
                pairs.sort();
                for (k, v) in pairs.iter() {
                    write!(f, " {{ {} {} }}", k.to_repr(), v.borrow().to_repr())?;
                }
                write!(f, " }}")
            }
            ValueData::String(s) => write!(f, "{}", s),
            ValueData::Function(lambda) => write!(f, "{}", lambda),
            ValueData::Address(addr) => write!(f, "<@{}>", addr),
            ValueData::Word(word) => write!(f, "{}", word),
            // TODO: escape word if it starts with punctuation (`module::\:` or something)
            ValueData::Symbol { module, word } => write!(f, "{}::{}", module, word),
            ValueData::Macro => write!(f, "MACRO:"),
            ValueData::Literal(value) => write!(f, "\\ {}", value),
            ValueData::Return(address, breakpoint) => write!(f, "<@{} - {}>", address, breakpoint),
            ValueData::Writer(writer) => write!(f, "{}", writer),
            ValueData::Reader(reader) => write!(f, "{}", reader),
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
            (ValueData::HashMap(hm1), ValueData::HashMap(hm2)) => {
                hm1.len() == hm2.len()
                    && hm1
                        .iter()
                        .all(|(k, v)| hm2.get(k).is_some_and(|v2| v.borrow().eq(&v2.borrow())))
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

#[allow(clippy::non_canonical_partial_ord_impl)]
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
            // TODO: compare hashmaps
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

impl Hash for ValueData {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            ValueData::Integer(n) => n.hash(state),
            ValueData::Float(f) => f.to_bits().hash(state),
            ValueData::Boolean(b) => b.hash(state),
            ValueData::Char(c) => c.hash(state),
            ValueData::String(s) => s.hash(state),
            ValueData::List(list) => {
                for item in list {
                    item.borrow().hash(state);
                }
            }
            ValueData::HashMap(map) => {
                let mut map = map.iter().collect::<Vec<_>>();
                map.sort_by_key(|p| p.0);
                for (k, v) in map {
                    k.hash(state);
                    v.borrow().hash(state);
                }
            }
            ValueData::Function(lambda) => lambda.hash(state),
            ValueData::Address(addr) => addr.hash(state),
            ValueData::Word(word) => word.hash(state),
            ValueData::Symbol { module, word } => {
                module.hash(state);
                word.hash(state);
            }
            ValueData::Macro => "MACRO".hash(state),
            ValueData::Literal(value) => value.hash(state),
            ValueData::Return(address, breakpoint) => {
                address.hash(state);
                breakpoint.hash(state);
            }
            ValueData::Writer(writer) => writer.hash(state),
            ValueData::Reader(reader) => reader.hash(state),
            ValueData::EndOfInput => "EOI".hash(state),
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
