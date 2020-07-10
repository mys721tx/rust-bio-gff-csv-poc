use std::error::Error;
use std::fmt::{self, Display};
use std::io;
use std::process;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use csv::ReaderBuilder;

#[derive(Debug)]
pub enum Strand {
    Forward,
    Reverse,
}

mod serde_strand {
    use super::*;

    struct StrandVisitor;

    impl<'de> Visitor<'de> for StrandVisitor {
        type Value = Option<Strand>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a character")
        }

        fn visit_char<E>(self, value: char) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match value {
                '+' | 'f' | 'F' => Ok(Some(Strand::Forward)),
                '-' | 'r' | 'R' => Ok(Some(Strand::Reverse)),
                '.' | '?' => Ok(None),
                _ => Err(E::custom(format!(
                    "invalid character {:?} in the strand",
                    value
                ))),
            }
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Strand>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_char(StrandVisitor)
    }

    pub fn serialize<S>(strand: &Option<Strand>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match strand {
            Some(Strand::Forward) => serializer.serialize_char('+'),
            Some(Strand::Reverse) => serializer.serialize_char('-'),
            None => serializer.serialize_char('.'),
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
}


#[derive(Debug, Deserialize, Serialize)]
struct Record {
    seqname: String,
    source: String,
    feature: String,
    start: u64,
    end: u64,
    score: String,
    #[serde(with = "serde_strand")]
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

fn writer() -> Result<(), Box<dyn Error>> {
    let records = vec![
        Record {
            seqname: "P0A7B8".to_owned(),
            source: "UniProtKB".to_owned(),
            feature: "Initiator methionine".to_owned(),
            start: 1,
            end: 1,
            score: ".".to_owned(),
            strand: Some(Strand::Forward),
            frame: ".".to_owned(),
            attributes: "Note=Removed,Obsolete;ID=test".to_owned(),
        },
        Record {
            seqname: "P0A7B8".to_owned(),
            source: "UniProtKB".to_owned(),
            feature: "Chain".to_owned(),
            start: 2,
            end: 176,
            score: "50".to_owned(),
            strand: Some(Strand::Forward),
            frame: ".".to_owned(),
            attributes: "Note=ATP-dependent protease subunit HslV;ID=PRO_0000148105".to_owned(),
        },
    ];

    let mut wtr = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .quote_style(csv::QuoteStyle::Necessary)
        .from_writer(io::stdout());

    for record in records {
        wtr.serialize(&record)?;
    }

    wtr.flush()?;
    Ok(())
}

fn main() {
    if let Err(err) = reader() {
        println!("error: {}", err);
        process::exit(1);
    }
    if let Err(err) = writer() {
        println!("error: {}", err);
        process::exit(1);
    }
}
