use std::fmt;
use std::hash::Hash;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::{fs::File, io};

use crate::error::Result;
use crate::shared::{shared, Shared};
use crate::value::io::error::TardiIoError;

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
