use std::collections::VecDeque;
static NKEYS: u16 = 255;
static BUFFER_SIZE: u8 = 16;

pub struct Keyboard {
    auto_repeat_enabled: bool,
    key_states: Vec<bool>,
    key_buffer: VecDeque<Event>,
    char_buffer: VecDeque<u16>,
}

impl Keyboard {
    pub fn new() -> Keyboard {
        Keyboard {
            auto_repeat_enabled: false,
            key_states: vec![false; NKEYS as usize],
            key_buffer: VecDeque::<Event>::with_capacity(BUFFER_SIZE as usize),
            char_buffer: VecDeque::<u16>::with_capacity(BUFFER_SIZE as usize),
        }
    }

    // Key Event Stuff
    pub fn key_is_pressed(&self, keycode: u16) -> bool {
        self.key_states[keycode as usize]
    }

    pub fn read_key(&mut self) -> Option<Event> {
        self.key_buffer.pop_front()
    }

    pub fn key_is_empty(&self) -> bool {
        self.key_buffer.is_empty()
    }

    // Char event stuff
    pub fn read_char(&mut self) -> Option<u16> {
        self.char_buffer.pop_front()
    }

    pub fn char_is_empty(&self) -> bool {
        self.char_buffer.is_empty()
    }

    pub fn flush_key(&mut self) {
        self.key_buffer.clear();
    }

    pub fn flush_char(&mut self) {
        self.char_buffer.clear();
    }

    pub fn flush(&mut self) {
        self.flush_key();
        self.flush_char();
    }

    // Autorepeat control

    pub fn enable_auto_repeat(&mut self) {
        self.auto_repeat_enabled = true;
    }

    pub fn disable_auto_repeat(&mut self) {
        self.auto_repeat_enabled = false;
    }

    pub fn auto_repeat_is_enabled(&self) -> bool {
        self.auto_repeat_enabled
    }

    pub fn on_key_pressed(&mut self, keycode: u16) {
        self.key_states[keycode as usize] = true;
        self.key_buffer.push_back(Event {
            event_type: EventType::Press,
            code: keycode,
        });
        Self::trim_buffer(&mut self.key_buffer);

    }

    pub fn on_key_released(&mut self, keycode: u16) {
        self.key_states[keycode as usize] = false;
        self.key_buffer.push_back(Event {
            event_type: EventType::Release,
            code: keycode,
        });
        Self::trim_buffer(&mut self.key_buffer);
    }

    pub fn on_char(&mut self, character: u16) {
        self.char_buffer.push_back(character);
        Self::trim_buffer(&mut self.char_buffer)
    }

    pub fn clear_state(&mut self) {
        self.key_states.fill(false);
    }

    // Trims the buffer back to BUFFER_SIZE
    fn trim_buffer<B>(buffer: &mut VecDeque<B>) {
        buffer.truncate(BUFFER_SIZE as usize);
    }
}

pub struct Event {
    event_type: EventType,
    code: u16,
}

impl Event {
    pub fn new(event_type: EventType, code: u16) -> Event {
        Event { event_type, code }
    }

    pub fn is_press(&self) -> bool {
        self.event_type == EventType::Press
    }

    pub fn is_release(&self) -> bool {
        self.event_type == EventType::Release
    }

    pub fn is_valid(&self) -> bool {
        self.event_type != EventType::Invalid
    }

    pub fn get_code(&self) -> u16 {
        return self.code;
    }
}

impl Default for Event {
    fn default() -> Self {
        Self {
            event_type: EventType::Invalid,
            code: 0,
        }
    }
}
#[derive(PartialEq)]
pub enum EventType {
    Press,
    Release,
    Invalid,
}
