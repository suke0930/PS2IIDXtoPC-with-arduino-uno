import { ButtonEvent, OutputAdapter, X360Mapping, X360ButtonName } from '../types';

const ViGEmClient = require('vigemclient');

type X360OutputOptions = {
  offsetMs: number;
  debug: boolean;
};

type DpadState = {
  up: boolean;
  down: boolean;
  left: boolean;
  right: boolean;
};

const BUTTON_NAMES: X360ButtonName[] = [
  'START',
  'BACK',
  'LEFT_THUMB',
  'RIGHT_THUMB',
  'LEFT_SHOULDER',
  'RIGHT_SHOULDER',
  'GUIDE',
  'A',
  'B',
  'X',
  'Y',
];

function schedule(action: () => void, delayMs: number): void {
  if (delayMs > 0) {
    setTimeout(action, delayMs);
  } else {
    action();
  }
}

export function createX360Output(
  mapping: X360Mapping,
  options: X360OutputOptions
): OutputAdapter {
  const client = new ViGEmClient();
  let controller: any;

  try {
    client.connect();
    controller = client.createX360Controller();
    controller.connect();
  } catch (error) {
    console.error('Failed to initialize ViGEm:', error);
    process.exit(1);
  }

  const dpad: DpadState = {
    up: false,
    down: false,
    left: false,
    right: false,
  };

  function updateDpad(): void {
    const horz = dpad.left ? -1 : dpad.right ? 1 : 0;
    const vert = dpad.up ? 1 : dpad.down ? -1 : 0;
    controller.axis.dpadHorz.setValue(horz);
    controller.axis.dpadVert.setValue(vert);
    controller.update();
  }

  function resolveButton(name: X360ButtonName): any {
    const map: Record<X360ButtonName, any> = {
      START: controller.button.START,
      BACK: controller.button.BACK,
      LEFT_THUMB: controller.button.LEFT_THUMB,
      RIGHT_THUMB: controller.button.RIGHT_THUMB,
      LEFT_SHOULDER: controller.button.LEFT_SHOULDER,
      RIGHT_SHOULDER: controller.button.RIGHT_SHOULDER,
      GUIDE: controller.button.GUIDE,
      A: controller.button.A,
      B: controller.button.B,
      X: controller.button.X,
      Y: controller.button.Y,
    };
    return map[name];
  }

  function setButton(name: X360ButtonName, pressed: boolean): void {
    const button = resolveButton(name);
    if (!button || typeof button.setValue !== 'function') {
      return;
    }
    button.setValue(pressed);
    controller.update();
  }

  function setTrigger(trigger: 'left' | 'right', pressed: boolean): void {
    const value = pressed ? 255 : 0;
    if (trigger === 'left') {
      controller.axis.leftTrigger.setValue(value);
    } else {
      controller.axis.rightTrigger.setValue(value);
    }
    controller.update();
  }

  return {
    handleButton(event: ButtonEvent) {
      const entry = mapping.buttons[event.id.toString()];
      if (!entry) {
        return;
      }

      if (options.debug) {
        console.log(
          `[x360] ${event.pressed ? 'press' : 'release'} button ${event.id}`
        );
      }

      if (entry.type === 'trigger') {
        setTrigger(entry.trigger, event.pressed);
        return;
      }

      schedule(() => {
        if (entry.type === 'dpad') {
          dpad[entry.direction] = event.pressed;
          updateDpad();
          return;
        }

        if (entry.type === 'button') {
          if (!BUTTON_NAMES.includes(entry.name)) {
            return;
          }
          setButton(entry.name, event.pressed);
        }
      }, options.offsetMs);
    },
    shutdown() {
      try {
        for (const name of BUTTON_NAMES) {
          const button = resolveButton(name);
          if (button && typeof button.setValue === 'function') {
            button.setValue(false);
          }
        }
        controller.axis.dpadHorz.setValue(0);
        controller.axis.dpadVert.setValue(0);
        controller.axis.leftTrigger.setValue(0);
        controller.axis.rightTrigger.setValue(0);
        controller.update();
        controller.disconnect();
      } catch (error) {
        console.error('Error during x360 shutdown:', error);
      }
    },
  };
}
