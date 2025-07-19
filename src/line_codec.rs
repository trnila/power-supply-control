use std::{fmt::Write, io};

use bytes::BytesMut;
use log::trace;
use tokio_util::codec::{Decoder, Encoder};

pub struct LineCodec;

impl Decoder for LineCodec {
    type Item = String;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let newline = src.as_ref().iter().position(|b| *b == b'\n');
        if let Some(n) = newline {
            let line = src.split_to(n + 1);
            return match std::str::from_utf8(line.as_ref()) {
                Ok(s) => {
                    let received = s.trim().to_string();
                    trace!("Received {received}");
                    Ok(Some(received))
                }
                Err(_) => Err(io::Error::other("Invalid String")),
            };
        }
        Ok(None)
    }
}

impl Encoder<String> for LineCodec {
    type Error = io::Error;

    fn encode(&mut self, _item: String, _dst: &mut BytesMut) -> Result<(), Self::Error> {
        _dst.write_str(&_item).unwrap();
        _dst.write_char('\n').unwrap();
        trace!("Sending {_item}");
        Ok(())
    }
}
