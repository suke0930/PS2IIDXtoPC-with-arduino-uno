use crate::types::{
    ButtonEvent, DpadDirection, OutputAdapter, TriggerName, X360ButtonEntry, X360ButtonName,
    X360Mapping,
};

use vigem_client::{Client, TargetId, XButtons, XGamepad, Xbox360Wired};

pub struct X360Output {
    target: Xbox360Wired<Client>,
    gamepad: XGamepad,
    mapping: X360Mapping,
    dpad_up: bool,
    dpad_down: bool,
    dpad_left: bool,
    dpad_right: bool,
    offset_ms: u64,
    debug: bool,
}

impl X360Output {
    pub fn new(mapping: X360Mapping, offset_ms: u64, debug: bool) -> Result<Self, String> {
        let client =
            Client::connect().map_err(|e| format!("Failed to connect to ViGEmBus: {:?}", e))?;
        let id = TargetId::XBOX360_WIRED;
        let mut target = Xbox360Wired::new(client, id);
        target
            .plugin()
            .map_err(|e| format!("Failed to plug in virtual controller: {:?}", e))?;
        target
            .wait_ready()
            .map_err(|e| format!("Failed to wait for controller ready: {:?}", e))?;

        Ok(Self {
            target,
            gamepad: XGamepad::default(),
            mapping,
            dpad_up: false,
            dpad_down: false,
            dpad_left: false,
            dpad_right: false,
            offset_ms,
            debug,
        })
    }

    fn resolve_button_flag(name: &X360ButtonName) -> u16 {
        match name {
            X360ButtonName::Start => XButtons::START,
            X360ButtonName::Back => XButtons::BACK,
            X360ButtonName::LeftThumb => XButtons::LTHUMB,
            X360ButtonName::RightThumb => XButtons::RTHUMB,
            X360ButtonName::LeftShoulder => XButtons::LB,
            X360ButtonName::RightShoulder => XButtons::RB,
            X360ButtonName::Guide => XButtons::GUIDE,
            X360ButtonName::A => XButtons::A,
            X360ButtonName::B => XButtons::B,
            X360ButtonName::X => XButtons::X,
            X360ButtonName::Y => XButtons::Y,
        }
    }

    fn set_button(&mut self, name: &X360ButtonName, pressed: bool) {
        let flag = Self::resolve_button_flag(name);
        if pressed {
            self.gamepad.buttons.raw |= flag;
        } else {
            self.gamepad.buttons.raw &= !flag;
        }
        let _ = self.target.update(&self.gamepad);
    }

    fn update_dpad(&mut self) {
        // Horizontal: left = -32768, right = 32767, center = 0
        let horz: i16 = if self.dpad_left {
            -32768
        } else if self.dpad_right {
            32767
        } else {
            0
        };
        // Vertical: up = 32767, down = -32768, center = 0
        let vert: i16 = if self.dpad_up {
            32767
        } else if self.dpad_down {
            -32768
        } else {
            0
        };

        self.gamepad.thumb_lx = horz;
        self.gamepad.thumb_ly = vert;
        let _ = self.target.update(&self.gamepad);
    }

    fn set_trigger(&mut self, trigger: &TriggerName, pressed: bool) {
        let value = if pressed { 255 } else { 0 };
        match trigger {
            TriggerName::Left => self.gamepad.left_trigger = value,
            TriggerName::Right => self.gamepad.right_trigger = value,
        }
        let _ = self.target.update(&self.gamepad);
    }
}

impl OutputAdapter for X360Output {
    fn handle_button(&mut self, event: &ButtonEvent) {
        let entry = match self.mapping.buttons.get(&event.id.to_string()) {
            Some(e) => e.clone(),
            None => return,
        };

        if self.debug {
            let action = if event.pressed { "press" } else { "release" };
            println!("[x360] {} button {} ", action, event.id);
        }

        if self.offset_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(self.offset_ms));
        }

        match &entry {
            X360ButtonEntry::Trigger { trigger } => {
                self.set_trigger(trigger, event.pressed);
            }
            X360ButtonEntry::Dpad { direction } => {
                match direction {
                    DpadDirection::Up => self.dpad_up = event.pressed,
                    DpadDirection::Down => self.dpad_down = event.pressed,
                    DpadDirection::Left => self.dpad_left = event.pressed,
                    DpadDirection::Right => self.dpad_right = event.pressed,
                }
                self.update_dpad();
            }
            X360ButtonEntry::Button { name } => {
                self.set_button(name, event.pressed);
            }
        }
    }

    fn shutdown(&mut self) {
        // Reset all state
        self.gamepad = XGamepad::default();
        let _ = self.target.update(&self.gamepad);
        let _ = self.target.unplug();
    }
}
