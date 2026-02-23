# PS2IIDXtoPC-with-arduino-uno

## Project Overview

This project converts PS2 IIDX (beatmania IIDX) dedicated controller input to PC keyboard or Xbox 360 gamepad input. It uses an Arduino Uno/Atmega328p to read PS2 controller signals via serial communication and converts them to keyboard or virtual gamepad input on the PC side.

**Tech Stack:**
- **Hardware:** Arduino Uno/Atmega328p with PsxControllerBitBang library
- **PC Side:** Rust (stable edition 2021) with Cargo build system
- **Key Dependencies:**
  - `serialport`: Arduino serial communication
  - `enigo`: Keyboard input emulation (cross-platform)
  - `clap`: CLI argument parsing with derive macros
  - `serde`/`serde_json`: JSON mapping configuration
  - `dotenvy`: Environment variable loading (.env support)
  - `crossterm`: Terminal UI (interactive launcher)
  - `vigem-client`: Virtual Xbox 360 controller (Windows-only, feature-gated)
  - `ctrlc`: Graceful shutdown handling

**Platform:** Primarily Windows (ViGEm dependency), but keyboard mode works cross-platform.

## Architecture

The project follows a modular architecture with clear separation of concerns:

### File Structure

```
├── arduino/
│   └── sketch_dec16a/
│       └── sketch_dec16a.ino   # Arduino firmware for PS2 controller reading
├── src/
│   ├── main.rs                 # Entry point, delegates to cli::run_cli()
│   ├── cli.rs                  # Main CLI logic and output adapter creation
│   ├── serial.rs               # Serial communication handler
│   ├── launcher.rs             # Interactive launcher for port/mode selection
│   ├── mapping.rs              # JSON mapping loader and validator
│   ├── types.rs                # Rust type definitions
│   ├── env.rs                  # Environment variable loader
│   └── outputs/
│       ├── mod.rs              # Output adapter trait and factory
│       ├── keyboard.rs         # Keyboard output adapter (enigo)
│       └── x360.rs             # Xbox 360 gamepad output adapter (vigem-client)
├── mapping/
│   ├── iidx.keyboard.json      # IIDX keyboard mapping configuration
│   ├── popn.keyboard.json      # Pop'n Music keyboard mapping
│   └── x360.pad.json           # Xbox 360 gamepad mapping
├── Cargo.toml                  # Rust package manifest and dependencies
├── Cargo.lock                  # Locked dependency versions
├── .env.example                # Environment variable template
└── target/                     # Build output directory
```

### Legacy Files (Archived)

The TypeScript/Node.js implementation has been archived:
- `ts-legacy/` - Original TypeScript/Node.js implementation (kept for reference, do not use)
- Old implementations are superseded by the current Rust implementation

### Data Flow

1. **Arduino → Serial:** PS2 controller button events sent as `b:<id>:<state>` (e.g., `b:14:1`)
2. **Serial → Parser:** `src/serial.rs` parses incoming data into `ButtonEvent` objects
3. **Parser → Output Adapter:** Events dispatched to keyboard or x360 output adapter
4. **Output → OS:** Keyboard/gamepad input injected into operating system

### Key Components

#### 1. Serial Communication (`src/serial.rs`)
- Opens serial port connection to Arduino
- Parses `b:<id>:<state>` protocol
- Emits `ButtonEvent` objects: `{ id: u8, pressed: bool }`
- Ignores malformed or non-button messages (turntable `t:` messages currently unused)

#### 2. Output Adapters (`src/outputs/`)
- **Keyboard Adapter:** Uses `enigo` crate to press/release keyboard keys
  - Supports special features: tap keys, ignore keys, delayed release
  - Configurable via `special` section in JSON mappings
- **X360 Adapter:** Creates virtual Xbox 360 controller via `vigem-client`
  - Handles buttons, D-pad (as axes), and triggers separately
  - Requires ViGEmBus driver installed on Windows
  - Feature-gated with conditional compilation for Windows

#### 3. Mapping System (`src/mapping.rs`)
- Loads JSON mapping files from `mapping/` directory
- Validates button IDs and output types using serde
- Supports two output types: `keyboard` and `x360`
- Default mappings: `iidx`, `popn`, `x360`

#### 4. Interactive Launcher (`src/launcher.rs`)
- Auto-detects available serial ports
- Interactive menu UI using crossterm
- Prompts for baud rate and mode selection
- Supports custom mapping file paths

#### 5. Environment Configuration (`src/env.rs`)
- Loads `.env` file for default values via dotenvy
- Supported variables:
  - `DEFAULT_PORT`: Default COM port (e.g., `COM10`)
  - `DEFAULT_BAUD`: Default baud rate (default: `115200`)
  - `DEFAULT_MODE`: Default mapping mode (default: `iidx`)
  - `DEFAULT_OFFSET`: Default input delay in milliseconds (default: `0`)
  - `DEFAULT_DEBUG`: Enable debug logging (`0` or `1`)

## Development Commands

### Setup
```bash
cargo build --release   # Build optimized binary
cp .env.example .env    # Create environment configuration (optional)
```

### Running
```bash
# Interactive launcher (recommended for first-time use)
cargo run -- --launcher

# Direct execution with options
cargo run -- -p COM10 -b 115200 -m iidx
cargo run -- -p COM10 -m popn --offset 10
cargo run -- -p COM10 --map ./custom-mapping.json

# Using environment variables
# Edit .env file first, then:
cargo run

# Release binary
./target/release/ps2iidx_controller -p COM10 -m iidx
```

### CLI Options
- `-p, --port <port>`: Specify COM port (e.g., `COM10`)
- `-b, --baud <rate>`: Specify baud rate (default: `115200`)
- `-m, --mode <mode>`: Mapping mode (`iidx`, `popn`, `x360`)
- `--map <path>`: Custom mapping JSON path
- `-o, --offset <time>`: Input delay in milliseconds
- `-d, --debug`: Enable debug logging
- `--launcher`: Launch interactive port/mode selector

### Building
```bash
cargo build             # Compile debug binary
cargo build --release   # Compile optimized binary for distribution
```

### Arduino Setup
1. Install `PsxControllerBitBang` library in Arduino IDE
2. Wire PS2 controller connector to Arduino:
   - `PIN_PS2_ATT = 9`
   - `PIN_PS2_CMD = 6`
   - `PIN_PS2_DAT = 5`
   - `PIN_PS2_CLK = 8`
   - Connect GND and 3.3V as well
3. Upload `arduino/sketch_dec16a/sketch_dec16a.ino` to Arduino

## Mapping Configuration

### Keyboard Mapping Format

```json
{
  "name": "iidx.keyboard",
  "output": "keyboard",
  "buttons": {
    "0": { "key": "F21" },
    "1": { "key": "RightShift" },
    ...
  },
  "special": {
    "ignoreKey": "F14",
    "tapKeys": ["F13", "F15"],
    "tapDurationMs": 13,
    "releaseOnIgnore": ["F13", "F15"]
  }
}
```

**Special Features:**
- `ignoreKey`: When this key is held, tap keys behave differently
- `tapKeys`: Keys that use tap behavior (auto-release after duration)
- `tapDurationMs`: Duration for tap keys (default: 13ms)
- `releaseOnIgnore`: Keys to release when ignore key is pressed

### Xbox 360 Mapping Format

```json
{
  "name": "x360.pad",
  "output": "x360",
  "buttons": {
    "0": { "type": "button", "name": "A" },
    "1": { "type": "dpad", "direction": "up" },
    "8": { "type": "trigger", "trigger": "left" }
  }
}
```

**Entry Types:**
- `button`: Standard Xbox 360 buttons (`A`, `B`, `X`, `Y`, `START`, `BACK`, `LEFT_SHOULDER`, `RIGHT_SHOULDER`, `GUIDE`, etc.)
- `dpad`: D-pad directions (`up`, `down`, `left`, `right`)
- `trigger`: Analog triggers (`left`, `right`)

## Important Notes

### Turntable Support
- Arduino sends turntable data as `t:<position>` messages
- Currently **not processed** on PC side
- Turntable is handled via button events (`b:3` / `b:6`) mapped to `F18` / `F15` in IIDX mode
- This behavior is intentional and should not be changed without discussion

### Platform Requirements
- **Keyboard Mode:** Works on Windows, Linux, macOS
- **X360 Mode:** Windows only (requires ViGEmBus driver)

### Performance
- Default offset: `0ms` (adjust via `-o` flag if input lag is detected)
- Keyboard auto-delay: `0ms` (configured in `src/outputs/keyboard.ts`)
- Tap duration: `13ms` (configurable per mapping)

## When to Use AskUserQuestion

Before making significant changes, ask the user for clarification in these cases:

1. **Changing Protocol:** Implementing turntable `t:` message processing (currently intentionally unused)
2. **Adding New Modes:** Creating new mapping files beyond IIDX/Pop'n/X360
3. **Modifying Arduino Code:** Changes to pin assignments or communication protocol
4. **Dependency Changes:** Major Cargo dependency updates (serialport, vigem-client, enigo, clap)
5. **Output Behavior:** Changing tap key behavior, offset defaults, or special key handling
6. **Build/Distribution:** Creating standalone executables or Windows installers

## Common Patterns

### Adding a New Mapping Mode

1. Create JSON file in `mapping/` directory (e.g., `mapping/custom.keyboard.json`)
2. Add entry to `DEFAULT_MAPS` in `src/mapping.rs`:
   ```rust
   "custom" => Some("mapping/custom.keyboard.json".to_string()),
   ```
3. Test with: `cargo run -- -p COM10 -m custom`

### Debugging Serial Communication

Enable debug mode to see raw serial events:
```bash
cargo run -- -p COM10 -d
```

Output will show:
- `[serial] ignored: <message>` - Non-button messages
- `[serial] invalid: <message>` - Malformed messages
- `[keyboard] press: <key> / release: <key>` - Key events
- `[x360] press button <id> / release button <id>` - Gamepad events

## Known Issues

1. **README.md:** Japanese documentation may need updating for Rust implementation
2. **Turntable Messages:** Arduino sends `t:` messages but PC side doesn't process them (intentional)
3. **Legacy TypeScript:** Superseded by Rust implementation in `ts-legacy/` directory

## Future Improvements

- Schema validation for mapping JSON files
- Enhanced debug output with input/output logging
- Windows installer or standalone executable packaging
- Performance optimization for real-time input handling
- Additional mapping presets for other rhythm games
