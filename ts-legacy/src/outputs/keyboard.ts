import { Key, keyboard } from '@nut-tree-fork/nut-js';
import { ButtonEvent, KeyboardMapping, OutputAdapter } from '../types';

keyboard.config.autoDelayMs = 0;

type KeyboardOutputOptions = {
  offsetMs: number;
  debug: boolean;
};

function resolveKey(name: string): Key | undefined {
  return (Key as unknown as Record<string, Key>)[name.toUpperCase()];
}

function schedule(action: () => void, delayMs: number): void {
  if (delayMs > 0) {
    setTimeout(action, delayMs);
  } else {
    action();
  }
}

export function createKeyboardOutput(
  mapping: KeyboardMapping,
  options: KeyboardOutputOptions
): OutputAdapter {
  const tapKeys = new Set(mapping.special?.tapKeys ?? []);
  const releaseOnIgnore = new Set(mapping.special?.releaseOnIgnore ?? []);
  const ignoreKey = mapping.special?.ignoreKey;
  const tapDurationMs = mapping.special?.tapDurationMs ?? 13;
  let ignore = false;

  function releaseKeyByName(keyName: string): void {
    const key = resolveKey(keyName);
    if (!key) {
      return;
    }
    keyboard.releaseKey(key);
  }

  return {
    handleButton(event: ButtonEvent) {
      const entry = mapping.buttons[event.id.toString()];
      if (!entry) {
        return;
      }

      const keyName = entry.key;
      const key = resolveKey(keyName);
      if (!key) {
        if (options.debug) {
          console.warn(`[keyboard] Unknown key "${keyName}" for button ${event.id}`);
        }
        return;
      }

      if (options.debug) {
        console.log(
          `[keyboard] ${event.pressed ? 'press' : 'release'} ${keyName} (id ${event.id})`
        );
      }

      if (event.pressed) {
        if (ignore && releaseOnIgnore.size > 0) {
          for (const releaseKey of releaseOnIgnore) {
            releaseKeyByName(releaseKey);
          }
        }

        if (ignoreKey && keyName === ignoreKey) {
          ignore = true;
        }

        if (tapKeys.has(keyName) && !ignore) {
          keyboard.pressKey(key);
          return;
        }

        if (!tapKeys.has(keyName)) {
          schedule(() => keyboard.pressKey(key), options.offsetMs);
          return;
        }

        if (tapKeys.has(keyName) && ignore) {
          schedule(() => keyboard.pressKey(key), options.offsetMs);
          schedule(() => keyboard.releaseKey(key), options.offsetMs + tapDurationMs);
        }
      } else {
        if (ignoreKey && keyName === ignoreKey) {
          ignore = false;
        }
        schedule(() => keyboard.releaseKey(key), options.offsetMs);
      }
    },
    shutdown() {
      // No special shutdown needed for keyboard output.
    },
  };
}
