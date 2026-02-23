use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use clap::Parser;

use crate::env::load_env_file;
use crate::launcher::{run_launcher, LauncherDefaults};
use crate::mapping::{default_maps, load_mapping};
use crate::outputs::keyboard::KeyboardOutput;
use crate::serial::{run_serial_loop, SerialEvent, SerialOptions};
use crate::types::{MappingConfig, OutputAdapter};

#[cfg(target_os = "windows")]
use crate::outputs::x360::X360Output;

#[derive(Parser, Debug)]
#[command(name = "ps2iidx_controller", version = "1.0.0")]
#[command(about = "PS2 IIDX Controller to PC input converter")]
pub struct CliArgs {
    /// Specify COM port (e.g., COM10, /dev/ttyACM0)
    #[arg(short = 'p', long = "port")]
    port: Option<String>,

    /// Specify baud rate
    #[arg(short = 'b', long = "baud")]
    baud: Option<u32>,

    /// Offset in milliseconds
    #[arg(short = 'o', long = "offset")]
    offset: Option<u64>,

    /// Mapping mode (iidx, popn, x360)
    #[arg(short = 'm', long = "mode")]
    mode: Option<String>,

    /// Custom mapping JSON path
    #[arg(long = "map")]
    map: Option<String>,

    /// Launch interactive port/mode selector
    #[arg(long = "launcher")]
    launcher: bool,

    /// Enable debug mode
    #[arg(short = 'd', long = "debug")]
    debug: bool,
}

fn resolve_map_path(mode: &str, map_path: Option<&str>) -> Result<PathBuf, String> {
    if let Some(mp) = map_path {
        return Ok(PathBuf::from(mp)
            .canonicalize()
            .unwrap_or_else(|_| PathBuf::from(mp)));
    }
    let maps = default_maps();
    let default_map = maps.get(mode).ok_or_else(|| {
        let available: Vec<&str> = maps.keys().copied().collect();
        format!(
            "Unknown mode \"{}\". Available: {}",
            mode,
            available.join(", ")
        )
    })?;
    Ok(PathBuf::from(default_map))
}

fn env_var_or(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_string())
}

fn env_var_opt(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|s| !s.is_empty())
}

pub fn run_cli() -> Result<(), String> {
    load_env_file(None);

    let args = CliArgs::parse();

    let default_mode = args
        .mode
        .clone()
        .or_else(|| env_var_opt("DEFAULT_MODE"))
        .unwrap_or_else(|| "iidx".to_string());

    let default_port = args.port.clone().or_else(|| env_var_opt("DEFAULT_PORT"));

    let default_baud = args.baud.unwrap_or_else(|| {
        env_var_or("DEFAULT_BAUD", "115200")
            .parse()
            .unwrap_or(115200)
    });

    let default_offset = args.offset.unwrap_or_else(|| {
        env_var_or("DEFAULT_OFFSET", "0").parse().unwrap_or(0)
    });

    let debug = args.debug || env_var_or("DEFAULT_DEBUG", "0") == "1";

    let mut port = default_port;
    let mut baud_rate = default_baud;
    let mut mode = default_mode;
    let mut map_path = args.map.clone().or_else(|| env_var_opt("DEFAULT_MAP"));

    let should_launch = args.launcher || port.is_none();

    if should_launch {
        let result = run_launcher(LauncherDefaults {
            port: port.clone(),
            baud_rate,
            mode: mode.clone(),
        })?;
        port = Some(result.port);
        baud_rate = result.baud_rate;
        mode = result.mode;
        if result.map_path.is_some() {
            map_path = result.map_path;
        }
    }

    let port =
        port.ok_or("Error: COM port must be specified (or use --launcher / DEFAULT_PORT).")?;

    let resolved_map_path = resolve_map_path(&mode, map_path.as_deref())?;
    let mapping = load_mapping(resolved_map_path.to_str().unwrap_or(""))?;

    println!(
        "Mapping: {}",
        mapping
            .name()
            .unwrap_or(resolved_map_path.to_str().unwrap_or(""))
    );
    println!("Output: {:?}", mapping.output_type());
    println!("Baud rate: {}", baud_rate);

    // Setup Ctrl+C handler
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        println!("\nClosing...");
        r.store(false, Ordering::SeqCst);
    })
    .map_err(|e| format!("Failed to set Ctrl+C handler: {}", e))?;

    let serial_opts = SerialOptions {
        path: port,
        baud_rate,
        debug,
    };

    match mapping {
        MappingConfig::Keyboard(km) => {
            let mut kb = KeyboardOutput::new(km, default_offset, debug);
            let running_ref = running.clone();
            run_serial_loop(&serial_opts, |event| match event {
                SerialEvent::Button(btn) => {
                    kb.handle_button(&btn);
                }
                SerialEvent::Tick => {
                    kb.process_tap_releases();
                    if !running_ref.load(Ordering::SeqCst) {
                        kb.shutdown();
                        std::process::exit(0);
                    }
                }
            })?;
        }
        MappingConfig::X360(_xm) => {
            #[cfg(target_os = "windows")]
            {
                let mut x360 = X360Output::new(_xm, default_offset, debug)
                    .map_err(|e| format!("Failed to create X360 output: {}", e))?;
                let running_ref = running.clone();
                run_serial_loop(&serial_opts, |event| match event {
                    SerialEvent::Button(btn) => {
                        x360.handle_button(&btn);
                    }
                    SerialEvent::Tick => {
                        if !running_ref.load(Ordering::SeqCst) {
                            x360.shutdown();
                            std::process::exit(0);
                        }
                    }
                })?;
            }
            #[cfg(not(target_os = "windows"))]
            {
                return Err(
                    "X360 output is only supported on Windows (requires ViGEmBus).".to_string(),
                );
            }
        }
    }

    Ok(())
}
