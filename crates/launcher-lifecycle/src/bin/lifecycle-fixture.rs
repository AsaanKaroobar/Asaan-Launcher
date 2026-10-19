//! Test-only process fixture. Never packaged with Asaaan Launcher.
use asaaan_lifecycle::{Packet, Request, wire};
use std::{io::BufReader, time::Duration};
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let mode = args.get(1).map(String::as_str).unwrap_or("echo");
    let mut attempt = 1;
    if let Some(file) = args.get(2) {
        attempt = std::fs::read_to_string(file)
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0)
            + 1;
        std::fs::write(file, attempt.to_string()).unwrap();
    }
    if mode == "crash" || (mode == "fail-first" && attempt == 1) {
        std::process::exit(1);
    }
    let mut input = BufReader::new(std::io::stdin());
    let mut output = std::io::stdout();
    if mode == "hang" || (mode == "hang-first" && attempt == 1) {
        loop {
            std::thread::sleep(Duration::from_secs(1));
        }
    }
    wire::write(&mut output, &Packet::Ready).unwrap();
    while let Ok(Some(packet)) = wire::read(&mut input) {
        match packet {
            Packet::Shutdown => break,
            Packet::Request { request, sequence } => {
                wire::write(
                    &mut output,
                    &Packet::Request {
                        request: request.clone(),
                        sequence,
                    },
                )
                .unwrap();
                if request == Request::Hide {
                    wire::write(&mut output, &Packet::Idle { sequence }).unwrap();
                }
            }
            _ => {}
        }
    }
}
