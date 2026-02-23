import { SerialPort } from 'serialport';
import { ReadlineParser } from '@serialport/parser-readline';
import { ButtonEvent } from './types';

type SerialOptions = {
  path: string;
  baudRate: number;
  debug: boolean;
  onButton: (event: ButtonEvent) => void;
};

export function startSerial(
  options: SerialOptions
): { close: (onClose?: () => void) => void } {
  const port = new SerialPort({
    path: options.path,
    baudRate: options.baudRate,
    autoOpen: false,
  });

  const parser = port.pipe(new ReadlineParser({ delimiter: '\n' }));

  parser.on('data', (line: string) => {
    const trimmed = line.trim();
    if (!trimmed) {
      return;
    }

    const parts = trimmed.split(':');
    if (parts.length !== 3 || parts[0] !== 'b') {
      if (options.debug) {
        console.log(`[serial] ignored: ${trimmed}`);
      }
      return;
    }

    const buttonId = Number.parseInt(parts[1], 10);
    const state = parts[2];
    if (Number.isNaN(buttonId) || (state !== '0' && state !== '1')) {
      if (options.debug) {
        console.log(`[serial] invalid: ${trimmed}`);
      }
      return;
    }

    options.onButton({ id: buttonId, pressed: state === '1' });
  });

  port.on('error', (err) => {
    console.error('Serial Port Error:', err);
  });

  port.open((err) => {
    if (err) {
      console.error('Error opening port:', err);
      console.log(options);
      process.exit(1);
    }
    console.log(`Port ${options.path} opened successfully`);
  });

  return {
    close: (onClose?: () => void) => {
      port.close(() => {
        console.log('Serial port closed');
        if (onClose) {
          onClose();
        }
      });
    },
  };
}
