use std::env;
use std::process;
use std::time::Duration;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

use enigo::{Enigo, Key, KeyboardControllable};

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
        .timeout(Duration::from_millis(1000))
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

    let mut line = String::new();
    println!("Listening for input...");

    loop {
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
                                    std::thread::sleep(Duration::from_millis(100));
                                    enigo.key_up(key);
                                }
                            } else {
                                // Release event
                                if key == Key::Layout('a') {
                                    ignore = false;
                                }
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
