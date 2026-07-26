use std::fmt;

#[derive(Debug)]
pub enum MusicXmlError {
    Io(std::io::Error),
    Xml(roxmltree::Error),
    Unsupported(String),
}

impl fmt::Display for MusicXmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MusicXmlError::Io(e) => write!(f, "I/O error: {e}"),
            MusicXmlError::Xml(e) => write!(f, "XML parse error: {e}"),
            MusicXmlError::Unsupported(msg) => write!(f, "Unsupported: {msg}"),
        }
    }
}

impl From<std::io::Error> for MusicXmlError {
    fn from(e: std::io::Error) -> Self {
        MusicXmlError::Io(e)
    }
}

impl From<roxmltree::Error> for MusicXmlError {
    fn from(e: roxmltree::Error) -> Self {
        MusicXmlError::Xml(e)
    }
}
