use std::collections::VecDeque;

static BUFFER_SIZE: usize = 16;
static WHEEL_DELTA: usize = 120;

pub struct Mouse {
    x: isize,
    y: isize,
    left_is_pressed: bool,
    right_is_pressed: bool,
    is_in_window: bool,
    wheel_delta_carry: usize,
    buffer: VecDeque<Event>,
}

impl Mouse {
    pub fn new() -> Mouse {
        Mouse{
            x: 0,
            y: 0,
            left_is_pressed: false,
            right_is_pressed: false,
            is_in_window: false,
            wheel_delta_carry: 0,
            buffer: VecDeque::<Event>::with_capacity(BUFFER_SIZE as usize),
        }
    }

    pub fn get_pos(&self) -> (isize, isize) {
        (self.x, self.y)
    }

    pub fn get_pos_x(&self) -> isize {
        self.x
    }

    pub fn get_pos_y(&self) -> isize {
        self.y
    }

    pub fn left_is_pressed(&self) -> bool {
        self.left_is_pressed
    }

    pub fn right_is_pressed(&self) -> bool {
        self.right_is_pressed
    }

    pub fn is_in_window(&self) -> bool {
        self.is_in_window
    }

    pub fn read(&mut self) -> Option<Event> {
        self.buffer.pop_front()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn flush(&mut self) {
        self.buffer.clear();
    }

    pub fn on_mouse_move(&mut self, new_x: isize, new_y: isize) {
        self.x = new_x;
        self.y = new_y;

        self.buffer.push_back(Event::new(EventType::Move, &self));
        self.trim_buffer();
    }

    pub fn on_left_pressed(&mut self) {
        self.left_is_pressed = true;

        self.buffer.push_back(Event::new(EventType::LPress, &self));
        self.trim_buffer();

    }

    pub fn on_left_released(&mut self) {
        self.left_is_pressed = false;

        self.buffer.push_back(Event::new(EventType::LRelease, &self));
        self.trim_buffer();
    }

    pub fn on_right_pressed(&mut self) {
        self.right_is_pressed = true;

        self.buffer.push_back(Event::new(EventType::RPress, &self));
        self.trim_buffer();
    }

    pub fn on_right_released(&mut self) {
        self.right_is_pressed = false;

        self.buffer.push_back(Event::new(EventType::RRelease, &self));
        self.trim_buffer();
    }

    pub fn on_wheel_up(&mut self) {
        self.buffer.push_back(Event::new(EventType::WheelUp, &self));
        self.trim_buffer();
    }

    pub fn on_wheel_down(&mut self) {
        self.buffer.push_back(Event::new(EventType::WheelDown, &self));
        self.trim_buffer();
    }

    pub fn on_wheel_delta(&mut self, x: isize, y: isize, delta: usize) {
        self.wheel_delta_carry += delta;
        while self.wheel_delta_carry >= WHEEL_DELTA {
            self.wheel_delta_carry -= WHEEL_DELTA;
            self.on_wheel_up();
        }
        while self.wheel_delta_carry <= WHEEL_DELTA {
            self.wheel_delta_carry += WHEEL_DELTA;
            self.on_wheel_down();
        }
    }

    pub fn on_mouse_leave(&mut self) {
        self.is_in_window = false;
        self.buffer.push_back(Event::new(EventType::Leave, &self));
        self.trim_buffer();
    }

    pub fn on_mouse_enter(&mut self) {
        self.is_in_window = true;
        self.buffer.push_back(Event::new(EventType::Enter, &self));
        self.trim_buffer();
    }

    pub fn trim_buffer(&mut self) {
        self.buffer.truncate(BUFFER_SIZE as usize)
    }

}

pub struct Event {
    event_type: EventType,
    x: isize,
    y: isize,
    left_is_pressed: bool,
    right_is_pressed: bool,

}

impl Default for Event {
    fn default() -> Self {
        Self {
            event_type: EventType::Invalid,
            x: 0,
            y: 0,
            left_is_pressed: false,
            right_is_pressed: false,
        }
    }
}

impl Event {
    pub fn new(event_type: EventType, parent: &Mouse) -> Event {
        Event{
            event_type: event_type,
            x: parent.x,
            y: parent.y,
            left_is_pressed: parent.left_is_pressed,
            right_is_pressed: parent.right_is_pressed,

        }
    }

    pub fn get_type(&self) -> EventType {
        self.event_type
    }

    pub fn get_pos(&self) -> (isize, isize) {
        (self.x, self.y )
    }

    pub fn get_pos_x(&self) -> isize {
        self.x
    }

    pub fn get_pos_y(&self) -> isize {
        self.y
    }

    pub fn left_is_pressed(&self) -> bool {
        self.left_is_pressed
    }

    pub fn right_is_pressed(&self) -> bool {
        self.right_is_pressed
    }

}

#[derive(Clone, Copy, PartialEq)]
pub enum EventType {
    LPress,
    LRelease,
    RPress,
    RRelease,
    WheelUp,
    WheelDown,
    Move,
    Enter,
    Leave,
    Invalid,
}
