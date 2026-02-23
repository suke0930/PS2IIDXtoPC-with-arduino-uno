use std::io::{self, IsTerminal, Write};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{self, ClearType},
};

use crate::mapping::default_maps;

pub struct LauncherDefaults {
    pub port: Option<String>,
    pub baud_rate: u32,
    pub mode: String,
}

pub struct LauncherResult {
    pub port: String,
    pub baud_rate: u32,
    pub mode: String,
    pub map_path: Option<String>,
}

/// List available serial ports.
fn list_serial_ports() -> Vec<String> {
    match serialport::available_ports() {
        Ok(ports) => ports.into_iter().map(|p| p.port_name).collect(),
        Err(_) => Vec::new(),
    }
}

/// Interactive port selector using crossterm raw mode.
fn select_port_interactive(
    ports: &[String],
    default_index: Option<usize>,
) -> io::Result<Option<String>> {
    let mut selected = default_index.unwrap_or(0);
    let mut stdout = io::stdout();

    terminal::enable_raw_mode()?;

    let result = (|| -> io::Result<Option<String>> {
        loop {
            // Render
            execute!(
                stdout,
                terminal::Clear(ClearType::All),
                cursor::MoveTo(0, 0)
            )?;
            write!(
                stdout,
                "Select serial port (Up/Down, Enter to select, Esc to cancel)\r\n"
            )?;
            for (i, port) in ports.iter().enumerate() {
                let marker = if i == selected { "> " } else { "  " };
                let default_label = if Some(i) == default_index {
                    " [default]"
                } else {
                    ""
                };
                write!(stdout, "{}{}{}\r\n", marker, port, default_label)?;
            }
            stdout.flush()?;

            // Read key event
            if let Event::Key(key_event) = event::read()? {
                if key_event.modifiers.contains(KeyModifiers::CONTROL)
                    && key_event.code == KeyCode::Char('c')
                {
                    return Ok(None);
                }
                match key_event.code {
                    KeyCode::Up => {
                        selected = if selected == 0 {
                            ports.len() - 1
                        } else {
                            selected - 1
                        };
                    }
                    KeyCode::Down => {
                        selected = (selected + 1) % ports.len();
                    }
                    KeyCode::Enter => {
                        return Ok(Some(ports[selected].clone()));
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        return Ok(None);
                    }
                    _ => {}
                }
            }
        }
    })();

    terminal::disable_raw_mode()?;
    println!(); // newline after raw mode
    result
}

/// Read a line from stdin with a prompt.
fn ask_question(prompt: &str) -> io::Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

/// Run the interactive launcher.
pub fn run_launcher(defaults: LauncherDefaults) -> Result<LauncherResult, String> {
    let ports = list_serial_ports();
    let mut selected_port: Option<String> = None;

    if !ports.is_empty() {
        if io::stdin().is_terminal() {
            let default_index = defaults
                .port
                .as_ref()
                .and_then(|dp| ports.iter().position(|p| p == dp));
            selected_port = select_port_interactive(&ports, default_index)
                .map_err(|e| format!("Launcher error: {}", e))?;
        } else {
            println!("Available ports:");
            for (i, port) in ports.iter().enumerate() {
                println!("  [{}] {}", i + 1, port);
            }
        }
    } else {
        println!("No serial ports detected. Enter a port manually.");
    }

    let port = if let Some(sp) = selected_port {
        sp
    } else {
        let default_label = defaults
            .port
            .as_ref()
            .map(|p| format!(" [{}]", p))
            .unwrap_or_default();
        let input = ask_question(&format!("Port{}: ", default_label))
            .map_err(|e| format!("Input error: {}", e))?;
        if input.is_empty() {
            defaults
                .port
                .clone()
                .or_else(|| ports.first().cloned())
                .ok_or_else(|| "Port is required to continue.".to_string())?
        } else {
            input
        }
    };

    let baud_input = ask_question(&format!("Baud rate [{}]: ", defaults.baud_rate))
        .map_err(|e| format!("Input error: {}", e))?;
    let baud_rate = if baud_input.is_empty() {
        defaults.baud_rate
    } else {
        baud_input
            .parse::<u32>()
            .map_err(|_| "Invalid baud rate.".to_string())?
    };

    let maps = default_maps();
    let mode_names: Vec<&str> = maps.keys().copied().collect();
    println!(
        "Available modes: {} (or \"custom\")",
        mode_names.join(", ")
    );
    let mode_input = ask_question(&format!("Mode [{}]: ", defaults.mode))
        .map_err(|e| format!("Input error: {}", e))?;
    let selected_mode = if mode_input.is_empty() {
        defaults.mode.clone()
    } else {
        mode_input
    };

    if selected_mode == "custom" {
        let map_path =
            ask_question("Mapping JSON path: ").map_err(|e| format!("Input error: {}", e))?;
        if map_path.is_empty() {
            return Err("Mapping path is required for custom mode.".to_string());
        }
        return Ok(LauncherResult {
            port,
            baud_rate,
            mode: selected_mode,
            map_path: Some(map_path),
        });
    }

    Ok(LauncherResult {
        port,
        baud_rate,
        mode: selected_mode,
        map_path: None,
    })
}
