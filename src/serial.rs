use std::io::{BufRead, BufReader};
use std::time::Duration;

use crate::types::ButtonEvent;

pub struct SerialOptions {
    pub path: String,
    pub baud_rate: u32,
    pub debug: bool,
}

/// Event passed to the handler on each loop iteration.
pub enum SerialEvent {
    /// A valid button event was received.
    Button(ButtonEvent),
    /// Called every loop iteration (before reading) for housekeeping (e.g. tap releases).
    Tick,
}

/// Open a serial port and run the event loop.
/// The single `handler` callback receives both Tick and Button events,
/// avoiding borrow conflicts from having two separate closures.
pub fn run_serial_loop<F>(options: &SerialOptions, mut handler: F) -> Result<(), String>
where
    F: FnMut(SerialEvent),
{
    println!(
        "Opening serial port {} at {} baud...",
        options.path, options.baud_rate
    );

    let port = serialport::new(&options.path, options.baud_rate)
        .timeout(Duration::from_millis(10))
        .open()
        .map_err(|e| format!("Error opening port {}: {:?}", options.path, e))?;

    println!("Port {} opened successfully", options.path);
    println!("Listening for input...");

    let mut reader = BufReader::new(port);
    let mut line = String::new();

    loop {
        handler(SerialEvent::Tick);
        line.clear();

        match reader.read_line(&mut line) {
            Ok(0) => {
                // EOF - serial port closed
                break;
            }
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                let parts: Vec<&str> = trimmed.split(':').collect();
                if parts.len() != 3 || parts[0] != "b" {
                    if options.debug {
                        println!("[serial] ignored: {}", trimmed);
                    }
                    continue;
                }

                let button_id = match parts[1].parse::<u8>() {
                    Ok(id) => id,
                    Err(_) => {
                        if options.debug {
                            println!("[serial] invalid: {}", trimmed);
                        }
                        continue;
                    }
                };

                let state = parts[2];
                if state != "0" && state != "1" {
                    if options.debug {
                        println!("[serial] invalid: {}", trimmed);
                    }
                    continue;
                }

                handler(SerialEvent::Button(ButtonEvent {
                    id: button_id,
                    pressed: state == "1",
                }));
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::TimedOut {
                    continue;
                }
                eprintln!("Error reading from serial port: {:?}", e);
            }
        }
    }

    Ok(())
}
