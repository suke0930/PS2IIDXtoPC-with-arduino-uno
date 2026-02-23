export type OutputType = 'keyboard' | 'x360';

export type KeyboardSpecialConfig = {
  ignoreKey?: string;
  tapKeys?: string[];
  tapDurationMs?: number;
  releaseOnIgnore?: string[];
};

export type KeyboardButtonEntry = {
  key: string;
};

export type X360ButtonName =
  | 'START'
  | 'BACK'
  | 'LEFT_THUMB'
  | 'RIGHT_THUMB'
  | 'LEFT_SHOULDER'
  | 'RIGHT_SHOULDER'
  | 'GUIDE'
  | 'A'
  | 'B'
  | 'X'
  | 'Y';

export type DpadDirection = 'up' | 'down' | 'left' | 'right';
export type TriggerName = 'left' | 'right';

export type X360ButtonEntry =
  | { type: 'button'; name: X360ButtonName }
  | { type: 'dpad'; direction: DpadDirection }
  | { type: 'trigger'; trigger: TriggerName };

export type MappingConfigRaw = {
  name?: string;
  output: OutputType;
  buttons: Record<string, unknown>;
  special?: KeyboardSpecialConfig;
};

export type KeyboardMapping = {
  name?: string;
  output: 'keyboard';
  buttons: Record<string, KeyboardButtonEntry>;
  special?: KeyboardSpecialConfig;
};

export type X360Mapping = {
  name?: string;
  output: 'x360';
  buttons: Record<string, X360ButtonEntry>;
};

export type MappingConfig = KeyboardMapping | X360Mapping;

export type ButtonEvent = {
  id: number;
  pressed: boolean;
};

export type OutputAdapter = {
  handleButton(event: ButtonEvent): void;
  shutdown(): Promise<void> | void;
};
