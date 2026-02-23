# PS2IIDXtoPC-with-arduino-uno

## Project Overview

This project converts PS2 IIDX (beatmania IIDX) dedicated controller input to PC keyboard or Xbox 360 gamepad input. It uses an Arduino Uno/Atmega328p to read PS2 controller signals via serial communication and converts them to keyboard or virtual gamepad input on the PC side.

**Tech Stack:**
- **Hardware:** Arduino Uno/Atmega328p with PsxControllerBitBang library
- **PC Side:** Node.js + TypeScript
- **Dependencies:**
  - `serialport`: Arduino serial communication
  - `@serialport/parser-readline`: Serial data parsing
  - `@nut-tree-fork/nut-js`: Keyboard input emulation
  - `vigemclient`: Virtual Xbox 360 controller (requires ViGEmBus on Windows)
  - `commander`: CLI argument parsing
  - `chalk`: Terminal output formatting

**Platform:** Primarily Windows (ViGEm dependency), but keyboard mode works cross-platform.

## Architecture

The project follows a modular architecture with clear separation of concerns:

### File Structure

```
├── arduino/
│   └── sketch_dec16a/
│       └── sketch_dec16a.ino   # Arduino firmware for PS2 controller reading
├── src/
│   ├── cli.ts                  # Main CLI entry point and command parsing
│   ├── serial.ts               # Serial communication handler
│   ├── launcher.ts             # Interactive launcher for port/mode selection
│   ├── mapping.ts              # JSON mapping loader and validator
│   ├── types.ts                # TypeScript type definitions
│   ├── env.ts                  # Environment variable loader
│   └── outputs/
│       ├── keyboard.ts         # Keyboard output adapter (@nut-tree-fork/nut-js)
│       └── x360.ts             # Xbox 360 gamepad output adapter (vigemclient)
├── mapping/
│   ├── iidx.keyboard.json      # IIDX keyboard mapping configuration
│   ├── popn.keyboard.json      # Pop'n Music keyboard mapping
│   └── x360.pad.json           # Xbox 360 gamepad mapping
├── index.ts                    # Entry point (delegates to src/cli.ts)
├── .env.example                # Environment variable template
└── package.json                # Dependencies and build scripts
```

### Legacy Files (Deprecated)

The following files are from the old implementation and should not be modified:
- `indexV2.ts`, `popen.ts`, `pad.js`, `pad.ts` - Old implementations before refactoring
- These are kept for reference but are superseded by the `src/` directory structure

### Data Flow

1. **Arduino → Serial:** PS2 controller button events sent as `b:<id>:<state>` (e.g., `b:14:1`)
2. **Serial → Parser:** `src/serial.ts` parses incoming data into `ButtonEvent` objects
3. **Parser → Output Adapter:** Events dispatched to keyboard or x360 output adapter
4. **Output → OS:** Keyboard/gamepad input injected into operating system

### Key Components

#### 1. Serial Communication (`src/serial.ts`)
- Opens serial port connection to Arduino
- Parses `b:<id>:<state>` protocol
- Emits `ButtonEvent` objects: `{ id: number, pressed: boolean }`
- Ignores malformed or non-button messages (turntable `t:` messages currently unused)

#### 2. Output Adapters (`src/outputs/`)
- **Keyboard Adapter:** Uses @nut-tree-fork/nut-js to press/release keyboard keys
  - Supports special features: tap keys, ignore keys, delayed release
  - Configurable via `special` section in JSON mappings
- **X360 Adapter:** Creates virtual Xbox 360 controller via ViGEmClient
  - Handles buttons, D-pad (as axes), and triggers separately
  - Requires ViGEmBus driver installed on Windows

#### 3. Mapping System (`src/mapping.ts`)
- Loads JSON mapping files from `mapping/` directory
- Validates button IDs and output types
- Supports two output types: `keyboard` and `x360`
- Default mappings: `iidx`, `popn`, `x360`

#### 4. Interactive Launcher (`src/launcher.ts`)
- Auto-detects available serial ports
- Interactive arrow-key selection menu
- Prompts for baud rate and mode selection
- Supports custom mapping file paths

#### 5. Environment Configuration (`src/env.ts`)
- Loads `.env` file for default values
- Supported variables:
  - `DEFAULT_PORT`: Default COM port (e.g., `COM10`)
  - `DEFAULT_BAUD`: Default baud rate (default: `115200`)
  - `DEFAULT_MODE`: Default mapping mode (default: `iidx`)
  - `DEFAULT_OFFSET`: Default input delay in milliseconds (default: `0`)
  - `DEFAULT_DEBUG`: Enable debug logging (`0` or `1`)
  - `DEFAULT_MAP`: Custom mapping file path

## Development Commands

### Setup
```bash
npm install              # Install dependencies
cp .env.example .env     # Create environment configuration
```

### Running
```bash
# Interactive launcher (recommended for first-time use)
npm start -- --launcher

# Direct execution with options
npm start -- -p COM10 -b 115200 -m iidx
npm start -- -p COM10 -m popn --offset 10
npm start -- -p COM10 --map ./custom-mapping.json

# Using environment variables
# Edit .env file first, then:
npm start
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
npm run build           # Compile TypeScript to JavaScript
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
4. **Dependency Changes:** Upgrading major dependencies (serialport, vigemclient, nut-js)
5. **Output Behavior:** Changing tap key behavior, offset defaults, or special key handling
6. **Build/Distribution:** Creating executables with nexe or similar bundlers

## Common Patterns

### Adding a New Mapping Mode

1. Create JSON file in `mapping/` directory (e.g., `mapping/custom.keyboard.json`)
2. Add entry to `DEFAULT_MAPS` in `src/mapping.ts`:
   ```typescript
   export const DEFAULT_MAPS: Record<string, string> = {
     iidx: path.join('mapping', 'iidx.keyboard.json'),
     popn: path.join('mapping', 'popn.keyboard.json'),
     x360: path.join('mapping', 'x360.pad.json'),
     custom: path.join('mapping', 'custom.keyboard.json'), // Add this
   };
   ```
3. Test with: `npm start -- -p COM10 -m custom`

### Debugging Serial Communication

Enable debug mode to see raw serial events:
```bash
npm start -- -p COM10 -d
```

Output will show:
- `[serial] ignored: <message>` - Non-button messages
- `[serial] invalid: <message>` - Malformed messages
- `[keyboard] press/release <key>` - Key events
- `[x360] press/release button <id>` - Gamepad events

## Known Issues

1. **Character Encoding in README.md:** Some Japanese characters may appear garbled
2. **Turntable Messages:** Arduino sends `t:` messages but PC side doesn't process them
3. **Legacy Files:** Old implementations (`indexV2.ts`, `popen.ts`, etc.) should be cleaned up in future releases

## Future Improvements

See `IMPROVEMENT_PLAN.md` for detailed roadmap. Key items:
- Schema validation for mapping JSON files
- Executable packaging for distribution
- Enhanced debug output with input/output logging
- Dependency separation for smaller builds
