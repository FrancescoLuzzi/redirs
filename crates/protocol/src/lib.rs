use std::{
    borrow::Cow,
    collections::{BTreeMap, HashSet},
    fmt::Display,
    io::{self, Write},
};

const SPACER: &str = "\r\n";

pub trait RedirsOutput {
    fn write_resp_str<T: Write>(&self, out: &mut T) -> io::Result<()>;
}

pub enum RedirsError {
    WhitespaceError,
    StringError,
    ParsingError,
}

#[derive(Debug)]
pub enum ProcVersion {
    V2,
    V3,
}

#[derive(Debug)]
pub enum Sign {
    Positive,
    Negative,
}

impl Display for Sign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Sign::Positive => "+",
            Sign::Negative => "-",
        })
    }
}

#[derive(Debug)]
pub enum VerbatimEncoding {
    Txt,
    Mrk,
}
impl Display for VerbatimEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            VerbatimEncoding::Txt => "txt",
            VerbatimEncoding::Mrk => "mrk",
        })
    }
}

#[derive(Debug)]
pub struct HelloCmd<'a> {
    pub version: Option<ProcVersion>,
    pub auth: Option<(Cow<'a, str>, Cow<'a, str>)>,
    pub client_name: Option<Cow<'a, str>>,
}

#[derive(Debug)]
pub enum Cmd<'a> {
    System(System<'a>),
    Action(Action<'a>),
}

#[derive(Debug)]
pub enum Action<'a> {
    GET(&'a str),
    SET((String, RedirsValue)),
    DEL(&'a str),
}

#[derive(Debug)]
pub enum System<'a> {
    PING(&'a str),
    HELLO(HelloCmd<'a>),
    ECHO(&'a str),
}

#[derive(Debug)]
pub enum RedirsValue {
    SimpleString(String),
    SimpleError(String),
    Integer(i64),
    // this can be pretty huge (max 512 MB)
    BulkString(Option<String>),
    Array(Option<Vec<RedirsValue>>),
    Null,
    Bool(bool),
    Double(f64),
    BigNumber(Sign, String),
    BulkError(String),
    VerbatimString(VerbatimEncoding, String),
    Map(BTreeMap<RedirsValue, RedirsValue>),
    Set(HashSet<RedirsValue>),
    Push(Vec<RedirsValue>),
}

impl RedirsOutput for RedirsValue {
    fn write_resp_str<T: Write>(&self, out: &mut T) -> io::Result<()> {
        match self {
            RedirsValue::SimpleString(s) => write!(out, "+{s}{SPACER}"),
            RedirsValue::SimpleError(s) => write!(out, "-{s}{SPACER}"),
            RedirsValue::Integer(i) => write!(out, ":{i}{SPACER}"),
            RedirsValue::BulkString(s) => match s {
                Some(s) => write!(out, "${}{SPACER}{s}{SPACER}", s.len()),
                None => write!(out, "$-1{SPACER}"),
            },
            RedirsValue::Array(arr) => match arr {
                Some(arr) => {
                    write!(out, "*{}{SPACER}", arr.len())?;
                    arr.iter()
                        .map(|t| t.write_resp_str(out))
                        .find(|x| x.is_err())
                        .unwrap_or(Ok(()))
                }
                None => {
                    write!(out, "*-1{SPACER}")
                }
            },
            RedirsValue::Null => write!(out, "_{SPACER}"),
            RedirsValue::Bool(b) => match b {
                true => write!(out, "#t{SPACER}"),
                false => write!(out, "#f{SPACER}"),
            },
            RedirsValue::Double(d) => write!(out, ",{d}{SPACER}"),
            RedirsValue::BigNumber(sign, value) => write!(out, "({sign}{value}{SPACER}"),
            RedirsValue::BulkError(err) => write!(out, "!{}{SPACER}{err}{SPACER}", err.len()),
            RedirsValue::VerbatimString(enc, s) => {
                write!(out, "={}{SPACER}{enc}:{s}{SPACER}", s.len() + 4)
            }
            RedirsValue::Map(map) => {
                write!(out, "%{}{SPACER}", map.len())?;
                map.iter()
                    .map(|(k, v)| {
                        k.write_resp_str(out)?;
                        v.write_resp_str(out)
                    })
                    .find(|x| x.is_err())
                    .unwrap_or(Ok(()))
            }
            RedirsValue::Set(set) => {
                write!(out, "~{}{SPACER}", set.len())?;
                set.iter()
                    .map(|v| v.write_resp_str(out))
                    .find(|x| x.is_err())
                    .unwrap_or(Ok(()))
            }
            RedirsValue::Push(vals) => {
                write!(out, ">{}{SPACER}", vals.len())?;
                vals.iter()
                    .map(|v| v.write_resp_str(out))
                    .find(|x| x.is_err())
                    .unwrap_or(Ok(()))
            }
        }
    }
}

pub struct Lexer<'o> {
    buffer: &'o str,
    curr_pos: usize,
}

impl<'o> Lexer<'o> {
    pub fn pop(&mut self) -> &'o str {
        self.curr_pos += 1;
        &self.buffer[self.curr_pos..self.curr_pos]
    }
    pub fn peek(&mut self) -> &'o str {
        &self.buffer[self.curr_pos..self.curr_pos]
    }
    pub fn read_spacer(&mut self) -> Result<(), RedirsError> {
        match &self.buffer[self.curr_pos..self.curr_pos + 2] {
            SPACER => {
                self.curr_pos += 2;
                Ok(())
            }
            _ => Err(RedirsError::WhitespaceError),
        }
    }
    pub fn read_str(&mut self) -> Result<&'o str, RedirsError> {
        let splits = &mut self.buffer[self.curr_pos..].splitn(2, SPACER);
        let out = splits.next().ok_or(RedirsError::StringError)?;
        self.buffer = splits.next().ok_or(RedirsError::StringError)?;
        Ok(out)
    }
    pub fn lex(&mut self) -> Result<RedirsValue, RedirsError> {
        Err(RedirsError::ParsingError)
    }
}
