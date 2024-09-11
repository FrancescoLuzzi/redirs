use std::io;

use protocol::Token;

fn main() -> io::Result<()> {
    Token::Array(Some(&[Token::SimpleString("ollare"), Token::Integer(12)]))
        .write_resp_str(&mut std::io::stdout())
}
