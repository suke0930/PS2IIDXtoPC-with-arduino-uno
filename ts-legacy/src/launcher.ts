import readline from 'readline';
import { SerialPort } from 'serialport';
import { DEFAULT_MAPS } from './mapping';

export type LauncherDefaults = {
  port?: string;
  baudRate: number;
  mode: string;
};

export type LauncherResult = {
  port: string;
  baudRate: number;
  mode: string;
  mapPath?: string;
};

function askQuestion(rl: readline.Interface, prompt: string): Promise<string> {
  return new Promise((resolve) => {
    rl.question(prompt, (answer) => resolve(answer.trim()));
  });
}

function selectPortInteractive(
  ports: string[],
  defaultIndex: number
): Promise<string | undefined> {
  return new Promise((resolve) => {
    const stdin = process.stdin;
    const stdout = process.stdout;

    if (!stdin.isTTY || typeof stdin.setRawMode !== 'function') {
      resolve(undefined);
      return;
    }

    readline.emitKeypressEvents(stdin);
    const previousRawMode = Boolean(stdin.isRaw);
    let selectedIndex = defaultIndex >= 0 ? defaultIndex : 0;

    const render = () => {
      process.stdout.write('\x1B[2J\x1B[H');
      readline.cursorTo(stdout, 0);
      readline.clearScreenDown(stdout);

      stdout.write('Select serial port (Up/Down, Enter to select, Esc to cancel)\n');
      ports.forEach((port, index) => {
        const marker = index === selectedIndex ? '> ' : '  ';
        const isDefault = index === defaultIndex ? ' [default]' : '';
        stdout.write(`${marker}${port}${isDefault}\n`);
      });
    };

    const cleanup = () => {
      stdin.setRawMode(previousRawMode);
      stdin.removeListener('keypress', onKeypress);
      stdout.write('\n');
    };

    const onKeypress = (_: string, key: readline.Key) => {
      if (key.ctrl && key.name === 'c') {
        cleanup();
        process.kill(process.pid, 'SIGINT');
        return;
      }

      if (key.name === 'up') {
        selectedIndex = (selectedIndex - 1 + ports.length) % ports.length;
        render();
        return;
      }

      if (key.name === 'down') {
        selectedIndex = (selectedIndex + 1) % ports.length;
        render();
        return;
      }

      if (key.name === 'return') {
        const selected = ports[selectedIndex];
        cleanup();
        resolve(selected);
        return;
      }

      if (key.name === 'escape' || key.name === 'q') {
        cleanup();
        resolve(undefined);
      }
    };

    stdin.on('keypress', onKeypress);
    stdin.setRawMode(true);
    render();
  });
}

export async function runLauncher(defaults: LauncherDefaults): Promise<LauncherResult> {
  const ports = await SerialPort.list();
  const portPaths = ports.map((port) => port.path).filter(Boolean);
  let selectedPort: string | undefined;

  if (portPaths.length > 0) {
    if (process.stdin.isTTY) {
      const defaultIndex = defaults.port
        ? portPaths.findIndex((port) => port === defaults.port)
        : -1;
      selectedPort = await selectPortInteractive(portPaths, defaultIndex);
    } else {
      console.log('Available ports:');
      portPaths.forEach((port, index) => {
        console.log(`  [${index + 1}] ${port}`);
      });
    }
  } else {
    console.log('No serial ports detected. Enter a port manually.');
  }

  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
  });

  try {
    const portInput = selectedPort
      ? ''
      : await askQuestion(
        rl,
        `Port${defaults.port ? ` [${defaults.port}]` : ''}: `
      );
    const port =
      selectedPort ||
      portInput ||
      defaults.port ||
      (portPaths.length > 0 ? portPaths[0] : '');
    if (!port) {
      throw new Error('Port is required to continue.');
    }

    const baudInput = await askQuestion(
      rl,
      `Baud rate [${defaults.baudRate}]: `
    );
    const baudRate = Number.parseInt(baudInput || `${defaults.baudRate}`, 10);
    if (Number.isNaN(baudRate)) {
      throw new Error('Invalid baud rate.');
    }

    const modes = Object.keys(DEFAULT_MAPS);
    console.log(`Available modes: ${modes.join(', ')} (or "custom")`);
    const modeInput = await askQuestion(
      rl,
      `Mode [${defaults.mode}]: `
    );
    const selectedMode = modeInput || defaults.mode;

    if (selectedMode === 'custom') {
      const mapPath = await askQuestion(rl, 'Mapping JSON path: ');
      if (!mapPath) {
        throw new Error('Mapping path is required for custom mode.');
      }
      return { port, baudRate, mode: selectedMode, mapPath };
    }

    return { port, baudRate, mode: selectedMode };
  } finally {
    rl.close();
  }
}
