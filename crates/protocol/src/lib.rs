use std::{
    borrow::Cow,
    io::{self, Write},
};

const SPACER: &str = "\r\n";

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
pub struct HelloCmd<'a> {
    pub version: Option<ProcVersion>,
    pub auth: Option<(Cow<'a, str>, Cow<'a, str>)>,
    pub client_name: Option<Cow<'a, str>>,
}

#[derive(Debug)]
pub enum Cmd<'a> {
    Ping(&'a str),
    Hello(HelloCmd<'a>),
}

#[derive(Debug)]
pub enum Token<'a> {
    SimpleString(&'a str),
    SimpleError(&'a str),
    Integer(i64),
    // this can be pretty huge (max 512 MB)
    BulkString(Option<String>),
    Array(Option<&'a [Token<'a>]>),
    Null,
    Bool(bool),
    Double(f64),
    BigNumber,
    BulkError,
    VerbatimString,
    Map,
    Set,
    Push,
}

impl<'a> Token<'a> {
    pub fn write_resp_str<T: Write>(&self, out_buffer: &mut T) -> io::Result<()> {
        match self {
            Token::SimpleString(s) => write!(out_buffer, "+{s}{SPACER}"),
            Token::SimpleError(s) => write!(out_buffer, "-{s}{SPACER}"),
            Token::Integer(i) => write!(out_buffer, ":{i}{SPACER}"),
            Token::BulkString(s) => match s {
                Some(s) => write!(out_buffer, "${}{SPACER}{s}{SPACER}", s.len()),
                None => write!(out_buffer, "$-1{SPACER}"),
            },
            Token::Array(arr) => match arr {
                Some(arr) => {
                    write!(out_buffer, "*{}{SPACER}", arr.len())?;
                    arr.iter()
                        .map(|t| t.write_resp_str(out_buffer))
                        .find(|x| x.is_err())
                        .unwrap_or(Ok(()))
                }
                None => {
                    write!(out_buffer, "*-1{SPACER}")
                }
            },
            Token::Null => write!(out_buffer, "_{SPACER}"),
            Token::Bool(b) => match b {
                true => write!(out_buffer, "#t{SPACER}"),
                false => write!(out_buffer, "#f{SPACER}"),
            },
            Token::Double(d) => write!(out_buffer, ",{d}{SPACER}"),
            Token::BigNumber => todo!(),
            Token::BulkError => todo!(),
            Token::VerbatimString => todo!(),
            Token::Map => todo!(),
            Token::Set => todo!(),
            Token::Push => todo!(),
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
    pub fn lex(&mut self) -> Result<Token<'o>, RedirsError> {
        Err(RedirsError::ParsingError)
    }
}
