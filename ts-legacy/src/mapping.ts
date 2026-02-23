import fs from 'fs';
import path from 'path';
import {
  MappingConfig,
  MappingConfigRaw,
  KeyboardMapping,
  KeyboardButtonEntry,
  X360Mapping,
  X360ButtonEntry,
  X360ButtonName,
  DpadDirection,
  TriggerName,
} from './types';

const X360_BUTTONS: X360ButtonName[] = [
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

const DPAD_DIRECTIONS: DpadDirection[] = ['up', 'down', 'left', 'right'];
const TRIGGERS: TriggerName[] = ['left', 'right'];

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function normalizeKeyboardEntry(value: unknown, keyId: string): KeyboardButtonEntry {
  if (typeof value === 'string') {
    return { key: value };
  }
  if (isRecord(value) && typeof value.key === 'string') {
    return { key: value.key };
  }
  throw new Error(`Invalid keyboard mapping entry for button ${keyId}`);
}

function normalizeX360Entry(value: unknown, keyId: string): X360ButtonEntry {
  if (!isRecord(value) || typeof value.type !== 'string') {
    throw new Error(`Invalid x360 mapping entry for button ${keyId}`);
  }

  if (value.type === 'button' && typeof value.name === 'string') {
    if (!X360_BUTTONS.includes(value.name as X360ButtonName)) {
      throw new Error(`Invalid x360 button name '${value.name}' for button ${keyId}`);
    }
    return { type: 'button', name: value.name as X360ButtonName };
  }

  if (value.type === 'dpad' && typeof value.direction === 'string') {
    if (!DPAD_DIRECTIONS.includes(value.direction as DpadDirection)) {
      throw new Error(
        `Invalid dpad direction '${value.direction}' for button ${keyId}`
      );
    }
    return { type: 'dpad', direction: value.direction as DpadDirection };
  }

  if (value.type === 'trigger' && typeof value.trigger === 'string') {
    if (!TRIGGERS.includes(value.trigger as TriggerName)) {
      throw new Error(
        `Invalid trigger '${value.trigger}' for button ${keyId}`
      );
    }
    return { type: 'trigger', trigger: value.trigger as TriggerName };
  }

  throw new Error(`Invalid x360 mapping entry for button ${keyId}`);
}

export function loadMapping(filePath: string): MappingConfig {
  const resolved = path.resolve(filePath);
  const rawText = fs.readFileSync(resolved, 'utf8');
  let parsed: unknown;
  try {
    parsed = JSON.parse(rawText);
  } catch (error) {
    throw new Error(`Failed to parse mapping JSON at ${resolved}`);
  }

  if (!isRecord(parsed)) {
    throw new Error(`Mapping JSON must be an object (${resolved})`);
  }

  const raw = parsed as MappingConfigRaw;
  if (raw.output !== 'keyboard' && raw.output !== 'x360') {
    throw new Error(`Mapping output must be "keyboard" or "x360" (${resolved})`);
  }

  if (!isRecord(raw.buttons)) {
    throw new Error(`Mapping "buttons" must be an object (${resolved})`);
  }

  if (raw.output === 'keyboard') {
    const buttons: Record<string, KeyboardButtonEntry> = {};
    for (const [keyId, value] of Object.entries(raw.buttons)) {
      buttons[keyId] = normalizeKeyboardEntry(value, keyId);
    }
    const mapping: KeyboardMapping = {
      name: raw.name,
      output: 'keyboard',
      buttons,
      special: raw.special,
    };
    return mapping;
  }

  const buttons: Record<string, X360ButtonEntry> = {};
  for (const [keyId, value] of Object.entries(raw.buttons)) {
    buttons[keyId] = normalizeX360Entry(value, keyId);
  }
  const mapping: X360Mapping = {
    name: raw.name,
    output: 'x360',
    buttons,
  };
  return mapping;
}

export const DEFAULT_MAPS: Record<string, string> = {
  iidx: path.join('mapping', 'iidx.keyboard.json'),
  popn: path.join('mapping', 'popn.keyboard.json'),
  x360: path.join('mapping', 'x360.pad.json'),
};
