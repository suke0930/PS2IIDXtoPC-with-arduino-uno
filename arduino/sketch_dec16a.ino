#include <DigitalIO.h>
#include <PsxControllerBitBang.h>

// Pin definitions
const byte PIN_PS2_ATT = 9;
const byte PIN_PS2_CMD = 6;
const byte PIN_PS2_DAT = 5;
const byte PIN_PS2_CLK = 8;

// Controller instance
PsxControllerBitBang<PIN_PS2_ATT, PIN_PS2_CMD, PIN_PS2_DAT, PIN_PS2_CLK> psx;

// Turntable settings
const int SCR_SENSITIVITY = 1000;
int scr_pos = 0;
int last_scr_pos = 0;

// Button state tracking
uint16_t prev_buttons = 0;
uint16_t curr_buttons = 0;

void setup() {
    Serial.begin(115200);
}

void send_button_event(uint8_t button_id, bool is_press) {
    // Format: "b:ID:STATE\n" (例: "b:14:1\n")
    Serial.print("b:");
    Serial.print(button_id);
    Serial.print(":");
    Serial.println(is_press ? "1" : "0");
}

void send_turntable_position(int position) {
    Serial.print("t:");
    Serial.println(position);
}

void check_button_changes() {
    uint16_t changed_buttons = prev_buttons ^ curr_buttons;
    uint16_t mask = 1;
    
    for (uint8_t i = 0; i < 16; i++) {
        if (changed_buttons & mask) {
            bool is_pressed = curr_buttons & mask;
            send_button_event(i, is_pressed);
        }
        mask <<= 1;
    }
}

void check_turntable_changes() {
    if (scr_pos != last_scr_pos) {
        send_turntable_position(scr_pos);
        last_scr_pos = scr_pos;
    }
}

void loop() {
    psx.begin();
    psx.read();
    
    // Update button states
    prev_buttons = curr_buttons;
    curr_buttons = psx.getButtonWord();
    
    // Check for button state changes
    check_button_changes();
    
    // Update and check turntable position
    check_turntable_changes();
    
    //delay(1);
}
