use crate::*;

pub type ButtonPin = Pin<DynPinId, FunctionSio<SioInput>, PullUp>;
pub type SensorPin = Pin<DynPinId, FunctionSio<SioInput>, PullNone>;

#[derive(Clone, Copy, Debug)]
pub enum Track {
    A,
    B,
    C,
}

impl Track {
    pub fn index(&self) -> usize {
        match self {
            Track::A => 0,
            Track::B => 1,
            Track::C => 2,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Button {
    A,
    B,
    C,
}

impl Button {
    pub fn index(&self) -> usize {
        match self {
            Button::A => 0,
            Button::B => 1,
            Button::C => 2,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum IoEvent {
    ButtonPressed(Button),
    CarDetected(Track, Instant),
}

pub struct Buttons {
    buttons: [ButtonPin; 3],
}

impl Buttons {
    pub fn new(buttons: [ButtonPin; 3]) -> Self {
        let mut buttons = buttons;
        for button in &mut buttons {
            button.set_interrupt_enabled(Interrupt::EdgeLow, true);
        }
        Self { buttons }
    }

    pub fn is_pressed(&mut self, btn: Button) -> bool {
        let btn = &mut self.buttons[btn.index()];
        if btn.interrupt_status(Interrupt::EdgeLow) {
            btn.clear_interrupt(Interrupt::EdgeLow);
            return true;
        }
        false
    }
}

pub struct Sensors {
    tracks: [SensorPin; 3],
}

impl Sensors {
    pub fn new(tracks: [SensorPin; 3]) -> Self {
        let mut tracks = tracks;
        for track in &mut tracks {
            track.set_interrupt_enabled(Interrupt::EdgeLow, true);
        }
        Self { tracks }
    }

    pub fn is_car_detected(&mut self, track: Track) -> bool {
        let track = &mut self.tracks[track.index()];
        if track.interrupt_status(Interrupt::EdgeLow) {
            track.clear_interrupt(Interrupt::EdgeLow);
            return true;
        }
        false
    }
}
