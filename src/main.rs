use std::error::Error;
use std::ffi::OsStr;
use std::ops::RangeInclusive;
use std::path::Path;

use chrono::{DateTime, Utc};
use clap::Parser;
use dirs;
use rusqlite::{Connection, Result};

static DEFAULT_DB_NAME: &str = "mood.db";
static MOOD_RANGE: RangeInclusive<f32> = 0.0..=10.0;

#[derive(Parser, Debug)]
struct ArgumentParser {
    #[clap(value_parser = value_in_range)]
    value: f32,

    #[clap(short, long, value_parser)]
    message: Option<String>,

    #[clap(long, value_parser)]
    datetime: Option<String>,

    #[clap(short, long, value_parser = validate_dbpath)]
    dbpath: Option<String>,
}

#[derive(Debug)]
struct Mood {
    timestamp: i64,
    value: f32,
    message: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = ArgumentParser::parse();

    let dbpath: String = args.dbpath.unwrap_or_else(|| match dirs::data_dir() {
        Some(ref p) => String::from(p.join(DEFAULT_DB_NAME).to_str().unwrap()),
        None => panic!("cant't resolve dbpath. specify --dbpath option"),
    });

    let timestamp: i64 = match args.datetime {
        Some(ref dt) => DateTime::parse_from_rfc3339(dt)?.timestamp(),
        None => Utc::now().timestamp(),
    };

    let conn = Connection::open(&dbpath)?;

    let mood = Mood {
        timestamp,
        value: args.value,
        message: args.message,
    };

    match insert(&conn, &mood) {
        Ok(n_inserts) => {
            let mut msg = format!("Inserted {} rows. ", n_inserts);
            if args.value < 5.0 {
                msg.push_str("Cheer up Negative Nancy");
            } else if args.value > 5.0 {
                msg.push_str("Chill out Positive Pete");
            } else {
                msg.push_str("To feel nothing so as to feel anything. Such a shame")
            }
            println!("{}", msg)
        }
        Err(err) => {
            println!("Rusqlite insertion error: {}", err)
        }        
    };

    Ok(())
}

fn insert(conn: &Connection, mood: &Mood) -> Result<usize, rusqlite::Error> {
    conn.execute(
        "CREATE TABLE if NOT EXISTS mood (
            id         INTEGER PRIMARY KEY,
            timestamp  INTEGER NOT NULL,
            value      REAL NOT NULL,
            message    TEXT
        )",
        (),
    )?;
    conn.execute(
        "INSERT INTO mood (timestamp, value, message) VALUES (?1, ?2, ?3)",
        (&mood.timestamp, &mood.value, &mood.message),
    )
}

fn value_in_range(s: &str) -> Result<f32, String> {
    let value: f32 = s.parse().map_err(|_| format!("`{}` is non-numeric", s))?;

    if MOOD_RANGE.contains(&value) {
        Ok(value)
    } else {
        Err(format!(
            "invalid value. value not in range {} -{}",
            MOOD_RANGE.start(),
            MOOD_RANGE.end()
        ))
    }
}

fn validate_dbpath(path: &str) -> Result<String, String> {
    let path = Path::new(path);

    match path.extension() {
        Some(ext) => {
            if ext == OsStr::new("db") {
                Ok(String::from(path.to_str().unwrap()))
            } else {
                Err(format!(
                    "invalid dbpath. invalid extension `.{}` use .db",
                    ext.to_str().unwrap()
                ))
            }
        }
        None => Err(format!("invalid dbpath. `.db` extension not found")),
    }
}

