use std::error::Error;
use std::fmt::{self, Display};
use std::io;
use std::process;

use serde::de::{self, Visitor};
use serde::ser;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use csv::ReaderBuilder;

#[derive(Debug)]
pub enum Strand {
    Forward,
    Reverse,
    Unknown,
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
                '?' => Ok(Some(Strand::Unknown)),
                '.' => Ok(None),
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
            Some(Strand::Unknown) => serializer.serialize_char('.'),
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

mod serde_score {
    use super::*;

    struct ScoreVisitor;

    impl<'de> Visitor<'de> for ScoreVisitor {
        type Value = Option<f64>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a floating point score or a dot")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match value {
                "." => Ok(None),
                _ => value
                    .parse::<f64>()
                    .map(Some)
                    .map_err(|_| E::custom(format!("invalid character {:?} in the strand", value))),
            }
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(ScoreVisitor)
    }

    pub fn serialize<S>(strand: &Option<f64>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *strand {
            Some(v) => serializer.serialize_f64(v),
            None => serializer.serialize_char('.'),
        }
    }

    #[derive(Debug, Clone)]
    pub enum ScoreError {
        Message(String),
    }

    impl de::Error for ScoreError {
        fn custom<T: Display>(msg: T) -> Self {
            ScoreError::Message(msg.to_string())
        }
    }

    impl Display for ScoreError {
        fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            match self {
                ScoreError::Message(msg) => formatter.write_str(msg),
            }
        }
    }

    impl std::error::Error for ScoreError {}
}

mod serde_frame {
    use super::*;

    struct FrameVisitor;

    impl<'de> Visitor<'de> for FrameVisitor {
        type Value = Option<u64>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("0, 1, 2 or a dot")
        }

        fn visit_char<E>(self, value: char) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match value {
                '0' => Ok(Some(0)),
                '1' => Ok(Some(1)),
                '2' => Ok(Some(2)),
                '.' => Ok(None),
                _ => Err(E::custom(format!(
                    "invalid character {:?} in the frame",
                    value
                ))),
            }
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_char(FrameVisitor)
    }

    pub fn serialize<S>(strand: &Option<u64>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *strand {
            Some(v) => if 0 < v && v < 3 {
                serializer.serialize_u64(0)
            } else {
                Err(ser::Error::custom(format!("invalid frame {}", v)))
            },
            None => serializer.serialize_char('.'),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Record {
    seqname: String,
    source: String,
    feature: String,
    start: u64,
    end: u64,
    #[serde(with = "serde_score")]
    score: Option<f64>,
    #[serde(with = "serde_strand")]
    strand: Option<Strand>,
    #[serde(with = "serde_frame")]
    frame: Option<u64>,
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
        .comment(Some(b'#'))
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
            score: None,
            strand: Some(Strand::Forward),
            frame: None,
            attributes: "Note=Removed,Obsolete;ID=test".to_owned(),
        },
        Record {
            seqname: "P0A7B8".to_owned(),
            source: "UniProtKB".to_owned(),
            feature: "Chain".to_owned(),
            start: 2,
            end: 176,
            score: Some(50.0),
            strand: Some(Strand::Forward),
            frame: None,
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
