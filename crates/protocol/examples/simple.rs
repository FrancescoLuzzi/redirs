use std::io;

use protocol::{RedirsOutput, RedirsValue};

fn main() -> io::Result<()> {
    RedirsValue::Array(Some(vec![
        RedirsValue::SimpleString("ollare".into()),
        RedirsValue::Integer(12),
    ]))
    .write_resp_str(&mut std::io::stdout())
}
