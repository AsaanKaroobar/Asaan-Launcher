use crate::Packet;
use std::io::{self, BufRead, Write};
pub const MAX_PACKET: usize = 4 * 1024 * 1024;
pub fn read(input: &mut impl BufRead) -> io::Result<Option<Packet>> {
    let mut bytes = Vec::new();
    loop {
        let chunk = input.fill_buf()?;
        if chunk.is_empty() {
            return if bytes.is_empty() {
                Ok(None)
            } else {
                Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "truncated packet",
                ))
            };
        }
        let end = chunk.iter().position(|b| *b == b'\n');
        let count = end.map_or(chunk.len(), |i| i + 1);
        if bytes.len() + count > MAX_PACKET {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "oversized packet",
            ));
        }
        bytes.extend_from_slice(&chunk[..count]);
        input.consume(count);
        if end.is_some() {
            return serde_json::from_slice(&bytes)
                .map(Some)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e));
        }
    }
}
pub fn write(output: &mut impl Write, packet: &Packet) -> io::Result<()> {
    let bytes = serde_json::to_vec(packet)?;
    if bytes.len() + 1 > MAX_PACKET {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "oversized packet",
        ));
    }
    output.write_all(&bytes)?;
    output.write_all(b"\n")?;
    output.flush()
}
