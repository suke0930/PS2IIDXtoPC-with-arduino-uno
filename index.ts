import { SerialPort } from 'serialport';
import { ReadlineParser } from '@serialport/parser-readline';
import { Command } from 'commander';
import { Key, keyboard } from '@nut-tree-fork/nut-js';

keyboard.config.autoDelayMs = 0;

// Command line interface setup
const program = new Command();
program
  .version('1.0.0')
  .option('-p, --port <port>', 'specify COM port')
  .option('-b, --baud <rate>', 'specify baud rate', '115200')
  .parse(process.argv);

const options = program.opts();

if (!options.port) {
  console.error('Error: COM port must be specified');
  process.exit(1);
}

// Button mapping to keyboard keys
const buttonMapping: { [key: number]: string } = {
  0: 'A',  // SELECT
  1: 'S',  // L3
  2: 'D',  // R3
  3: 'W',  // START
  4: 'R',  // UP
  5: 'T',  // RIGHT
  6: 'F',  // DOWN
  7: 'G',  // LEFT
  8: 'B',  // L2
  9: 'N',  // R2
  10: 'M', // L1
  11: 'J', // R1
  12: 'U', // TRIANGLE
  13: 'K', // CIRCLE
  14: 'P', // CROSS
  15: 'L'  // SQUARE
};

// Initialize serial port
const port = new SerialPort({
  path: options.port,
  baudRate: parseInt(options.baud),
  autoOpen: false,
});

// Create parser for the new line-based format
const parser = port.pipe(new ReadlineParser({ delimiter: '\n' }));

// State tracking
let ignore = false;

// Handle incoming data
parser.on('data', (line: string) => {
  const parts = line.trim().split(':');

  if (parts.length !== 3 && parts[0] === 'b') {
    return; // Invalid format for button command
  }

  if (parts[0] === 'b') {
    // Button event
    const buttonId = parseInt(parts[1]);
    const isPress = parts[2] === '1';
    const keyName = buttonMapping[buttonId];

    if (keyName !== undefined) {
      const key = (Key as any)[keyName.toUpperCase()];

      if (isPress) {
        if (ignore) {
          keyboard.releaseKey(Key.R);
          keyboard.releaseKey(Key.F);
        }

        if (key === Key.A) {
          ignore = true;
        }

        if ((key === Key.F || key === Key.R) && ignore === false) {
          console.log("A");
          console.log(ignore);
          console.log(key);
          keyboard.pressKey(key);
        } else if (key !== Key.F && key !== Key.R) {
          keyboard.pressKey(key);
        } else if ((key === Key.F || key === Key.R) && ignore === true) {
          keyboard.pressKey(key);
          setTimeout(() => {
            keyboard.releaseKey(key);
          }, 100);
        }
      } else {
        // Release event
        if (key === Key.A) {
          ignore = false;
        }
        keyboard.releaseKey(key);
      }
    }
  }
  // Note: turntable events ('t:position') are currently not handled
});

// Error handling
port.on('error', (err) => {
  console.error('Serial Port Error:', err);
});

// Open the port
port.open((err) => {
  if (err) {
    console.error('Error opening port:', err);
    process.exit(1);
  }
  console.log(`Port ${options.port} opened successfully`);
});

// Setup cleanup
process.on('SIGINT', () => {
  console.log('Closing serial port...');
  port.close(() => {
    console.log('Serial port closed');
    process.exit();
  });
});