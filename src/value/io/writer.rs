use std::fs::{File, OpenOptions};
use std::hash::Hash;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::{fmt, io};

use crate::error::Result;
use crate::shared::{shared, Shared};

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
