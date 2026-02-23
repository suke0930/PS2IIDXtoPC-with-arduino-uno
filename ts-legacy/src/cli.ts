import path from 'path';
import { Command } from 'commander';
import { DEFAULT_MAPS, loadMapping } from './mapping';
import { createKeyboardOutput } from './outputs/keyboard';
import { createX360Output } from './outputs/x360';
import { startSerial } from './serial';
import { runLauncher } from './launcher';
import { MappingConfig } from './types';
import { loadEnvFile } from './env';

type CliOverrides = {
  defaultMode?: string;
};

function parseNumber(value: string | undefined, fallback: number, label: string): number {
  if (value === undefined || value === '') {
    return fallback;
  }
  const parsed = Number.parseInt(value, 10);
  if (Number.isNaN(parsed)) {
    throw new Error(`Invalid ${label}: ${value}`);
  }
  return parsed;
}

function resolveMapPath(
  mode: string,
  mapPath: string | undefined
): string {
  if (mapPath) {
    return path.resolve(mapPath);
  }
  const defaultMap = DEFAULT_MAPS[mode];
  if (!defaultMap) {
    throw new Error(`Unknown mode "${mode}". Available: ${Object.keys(DEFAULT_MAPS).join(', ')}`);
  }
  return path.resolve(defaultMap);
}

function createOutput(mapping: MappingConfig, offsetMs: number, debug: boolean) {
  if (mapping.output === 'keyboard') {
    return createKeyboardOutput(mapping, { offsetMs, debug });
  }
  return createX360Output(mapping, { offsetMs, debug });
}

export async function runCli(argv: string[], overrides: CliOverrides = {}): Promise<void> {
  loadEnvFile();

  const program = new Command();
  program
    .version('1.0.0')
    .option('-p, --port <port>', 'specify COM port')
    .option('-b, --baud <rate>', 'specify baud rate')
    .option('-o, --offset <time>', 'offset in milliseconds')
    .option('-m, --mode <mode>', 'mapping mode (iidx, popn, x360)')
    .option('--map <path>', 'mapping JSON path')
    .option('--launcher', 'interactive launcher')
    .option('-d, --debug', 'enable debug mode')
    .parse(argv);

  const options = program.opts();
  const env = process.env;

  const defaultMode = options.mode || overrides.defaultMode || env.DEFAULT_MODE || 'iidx';
  const defaultPort = options.port || env.DEFAULT_PORT;
  const defaultBaud = parseNumber(options.baud, parseNumber(env.DEFAULT_BAUD, 115200, 'baud rate'), 'baud rate');
  const defaultOffset = parseNumber(options.offset, parseNumber(env.DEFAULT_OFFSET, 0, 'offset'), 'offset');
  const debug = Boolean(options.debug || env.DEFAULT_DEBUG === '1');

  let port = defaultPort;
  let baudRate = defaultBaud;
  let mode = defaultMode;
  let mapPath = options.map || env.DEFAULT_MAP;

  const shouldLaunch = Boolean(options.launcher || !port);

  if (shouldLaunch) {
    const launchResult = await runLauncher({
      port,
      baudRate,
      mode,
    });
    port = launchResult.port;
    baudRate = launchResult.baudRate;
    mode = launchResult.mode;
    if (launchResult.mapPath) {
      mapPath = launchResult.mapPath;
    }
  }

  if (!port) {
    console.error('Error: COM port must be specified (or use --launcher / DEFAULT_PORT).');
    process.exit(1);
  }

  const resolvedMapPath = resolveMapPath(mode, mapPath);
  const mapping = loadMapping(resolvedMapPath);

  const output = createOutput(mapping, defaultOffset, debug);
  const serial = startSerial({
    path: port,
    baudRate,
    debug,
    onButton: (event) => output.handleButton(event),
  });

  console.log(`Mapping: ${mapping.name ?? resolvedMapPath}`);
  console.log(`Output: ${mapping.output}`);
  console.log(`Baud rate: ${baudRate}`);

  process.on('SIGINT', () => {
    console.log('Closing serial port...');
    try {
      output.shutdown();
    } catch (error) {
      console.error('Error during shutdown:', error);
    }
    serial.close(() => {
      process.exit();
    });
  });
}
