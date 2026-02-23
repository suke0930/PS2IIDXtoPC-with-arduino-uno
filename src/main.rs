use std::env;
use std::process;
use std::time::Duration;
use std::time::Instant;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

use enigo::{Enigo, Key, KeyboardControllable};

#[derive(Default)]
struct ScratchTapReleases {
    down_deadline: Option<Instant>,
    up_deadline: Option<Instant>,
}

impl ScratchTapReleases {
    fn schedule(&mut self, key: Key, now: Instant, duration: Duration) {
        match key {
            Key::Layout('f') => self.down_deadline = Some(now + duration),
            Key::Layout('r') => self.up_deadline = Some(now + duration),
            _ => {}
        }
    }

    fn clear(&mut self, key: Key) {
        match key {
            Key::Layout('f') => self.down_deadline = None,
            Key::Layout('r') => self.up_deadline = None,
            _ => {}
        }
    }

    fn take_due_keys(&mut self, now: Instant) -> Vec<Key> {
        let mut due_keys = Vec::with_capacity(2);

        if self.down_deadline.is_some_and(|deadline| deadline <= now) {
            self.down_deadline = None;
            due_keys.push(Key::Layout('f'));
        }

        if self.up_deadline.is_some_and(|deadline| deadline <= now) {
            self.up_deadline = None;
            due_keys.push(Key::Layout('r'));
        }

        due_keys
    }

    fn clear_all(&mut self) {
        self.down_deadline = None;
        self.up_deadline = None;
    }
}

fn release_due_scratch_keys(enigo: &mut Enigo, releases: &mut ScratchTapReleases) {
    let now = Instant::now();
    for key in releases.take_due_keys(now) {
        enigo.key_up(key);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut port_name = String::new();
    let mut baud_rate: u32 = 115200;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-p" | "--port" => {
                if i + 1 < args.len() {
                    port_name = args[i + 1].clone();
                    i += 1;
                }
            }
            "-b" | "--baud" => {
                if i + 1 < args.len() {
                    if let Ok(b) = args[i + 1].parse() {
                        baud_rate = b;
                    }
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    if port_name.is_empty() {
        eprintln!("Error: COM port must be specified (-p <COM_PORT>)");
        process::exit(1);
    }

    // Button mapping
    let mut button_mapping: HashMap<u8, Key> = HashMap::new();
    button_mapping.insert(0, Key::Layout('a'));  // SELECT
    button_mapping.insert(1, Key::Layout('s'));  // L3
    button_mapping.insert(2, Key::Layout('d'));  // R3
    button_mapping.insert(3, Key::Layout('w'));  // START
    button_mapping.insert(4, Key::Layout('r'));  // UP
    button_mapping.insert(5, Key::Layout('t'));  // RIGHT
    button_mapping.insert(6, Key::Layout('f'));  // DOWN
    button_mapping.insert(7, Key::Layout('g'));  // LEFT
    button_mapping.insert(8, Key::Layout('b'));  // L2
    button_mapping.insert(9, Key::Layout('n'));  // R2
    button_mapping.insert(10, Key::Layout('m')); // L1
    button_mapping.insert(11, Key::Layout('j')); // R1
    button_mapping.insert(12, Key::Layout('u')); // TRIANGLE
    button_mapping.insert(13, Key::Layout('k')); // CIRCLE
    button_mapping.insert(14, Key::Layout('p')); // CROSS
    button_mapping.insert(15, Key::Layout('l')); // SQUARE

    println!("Opening serial port {} at {} baud...", port_name, baud_rate);
    let port_result = serialport::new(port_name.clone(), baud_rate)
        .timeout(Duration::from_millis(10))
        .open();

    let mut port = match port_result {
        Ok(port) => {
            println!("Port {} opened successfully", port_name);
            port
        }
        Err(e) => {
            eprintln!("Error opening port: {:?}", e);
            process::exit(1);
        }
    };

    let mut reader = BufReader::new(port.as_mut());
    let mut enigo = Enigo::new();
    let mut ignore = false;
    let mut scratch_tap_releases = ScratchTapReleases::default();
    let scratch_tap_duration = Duration::from_millis(100);

    let mut line = String::new();
    println!("Listening for input...");

    loop {
        release_due_scratch_keys(&mut enigo, &mut scratch_tap_releases);
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => {
                // EOF? Shouldn't happen on a serial port usually unless closed
            }
            Ok(_) => {
                let s = line.trim();
                let parts: Vec<&str> = s.split(':').collect();

                if parts.len() == 3 && parts[0] == "b" {
                    if let (Ok(button_id), Ok(state)) = (parts[1].parse::<u8>(), parts[2].parse::<u8>()) {
                        let is_press = state == 1;

                        if let Some(&key) = button_mapping.get(&button_id) {
                            if is_press {
                                if ignore {
                                    enigo.key_up(Key::Layout('r'));
                                    enigo.key_up(Key::Layout('f'));
                                    scratch_tap_releases.clear_all();
                                }

                                if key == Key::Layout('a') {
                                    ignore = true;
                                }

                                if (key == Key::Layout('f') || key == Key::Layout('r')) && !ignore {
                                    println!("A");
                                    println!("{}", ignore);
                                    // print key is tricky in Rust without debug on Enigo Key layout, skipping for now
                                    enigo.key_down(key);
                                } else if key != Key::Layout('f') && key != Key::Layout('r') {
                                    enigo.key_down(key);
                                } else if (key == Key::Layout('f') || key == Key::Layout('r')) && ignore {
                                    enigo.key_down(key);
                                    scratch_tap_releases.schedule(key, Instant::now(), scratch_tap_duration);
                                }
                            } else {
                                // Release event
                                if key == Key::Layout('a') {
                                    ignore = false;
                                }
                                scratch_tap_releases.clear(key);
                                enigo.key_up(key);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::TimedOut {
                    continue;
                }
                eprintln!("Error reading from serial port: {:?}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ScratchTapReleases;
    use enigo::Key;
    use std::time::Duration;
    use std::time::Instant;

    #[test]
    fn scratch_tap_releases_after_deadline() {
        let mut releases = ScratchTapReleases::default();
        let start = Instant::now();

        releases.schedule(Key::Layout('f'), start, Duration::from_millis(100));
        assert!(releases.take_due_keys(start + Duration::from_millis(99)).is_empty());

        let due = releases.take_due_keys(start + Duration::from_millis(100));
        assert_eq!(due, vec![Key::Layout('f')]);
    }

    #[test]
    fn clear_prevents_late_release() {
        let mut releases = ScratchTapReleases::default();
        let start = Instant::now();

        releases.schedule(Key::Layout('r'), start, Duration::from_millis(100));
        releases.clear(Key::Layout('r'));
        assert!(releases.take_due_keys(start + Duration::from_millis(100)).is_empty());
    }
}
