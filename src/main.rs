use std::error::Error;
use std::fmt::{self, Display};
use std::process;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer};

use csv::ReaderBuilder;

#[derive(Debug)]
pub enum Strand {
    Forward,
    Reverse,
    Unknown,
}

struct StrandVisitor;

impl<'de> Visitor<'de> for StrandVisitor {
    type Value = Strand;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a character")
    }

    fn visit_char<E>(self, value: char) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match value {
            '+' | 'f' | 'F' => Ok(Strand::Forward),
            '-' | 'r' | 'R' => Ok(Strand::Reverse),
            '.' | '?' => Ok(Strand::Unknown),
            _ => Err(E::custom(format!(
                "invalid character {:?} in the strand",
                value
            ))),
        }
    }
}

impl<'de> Deserialize<'de> for Strand {
    fn deserialize<D>(deserializer: D) -> Result<Strand, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_char(StrandVisitor)
    }
}

#[derive(Debug, Clone)]
pub enum StrandError {
    Message(String),
}

impl de::Error for StrandError {
    fn custom<T: Display>(msg: T) -> Self {
        StrandError::Message(msg.to_string())
    }
}

impl Display for StrandError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StrandError::Message(msg) => formatter.write_str(msg),
        }
    }
}

impl std::error::Error for StrandError {}

#[derive(Debug, Deserialize)]
struct Record {
    seqname: String,
    source: String,
    feature: String,
    start: u64,
    end: u64,
    score: String,
    strand: Option<Strand>,
    frame: String,
    attributes: String,
}

const GFF_FILE: &[u8] = b"P0A7B8\tUniProtKB\tInitiator methionine\t1\t1\t.\t.\t.\t\
Note=Removed,Obsolete;ID=test
P0A7B8\tUniProtKB\tChain\t2\t176\t50\t+\t.\tNote=ATP-dependent protease subunit HslV;\
ID=PRO_0000148105";

fn reader() -> Result<(), Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_reader(GFF_FILE);
    for result in rdr.deserialize() {
        let record: Record = result?;
        println!("{:?}", record);
    }
    Ok(())
}

fn main() {
    if let Err(err) = reader() {
        println!("error: {}", err);
        process::exit(1);
    }
}
