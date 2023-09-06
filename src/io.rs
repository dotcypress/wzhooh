use crate::*;

pub type ButtonA = Pin<bank0::Gpio7, Input<PullUp>>;
pub type ButtonB = Pin<bank0::Gpio6, Input<PullUp>>;
pub type ButtonC = Pin<bank0::Gpio5, Input<PullUp>>;

pub type SensorA = Pin<bank0::Gpio28, Input<Floating>>;
pub type SensorB = Pin<bank0::Gpio27, Input<Floating>>;
pub type SensorC = Pin<bank0::Gpio26, Input<Floating>>;

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

#[derive(Clone, Copy, Debug)]
pub enum IoEvent {
    ButtonPressed(Button),
    CarDetected(Track, Instant),
}

pub struct Buttons {
    btn_a: ButtonA,
    btn_b: ButtonB,
    btn_c: ButtonC,
}

impl Buttons {
    pub fn new(btn_a: ButtonA, btn_b: ButtonB, btn_c: ButtonC) -> Self {
        btn_a.set_interrupt_enabled(Interrupt::EdgeLow, true);
        btn_b.set_interrupt_enabled(Interrupt::EdgeLow, true);
        btn_c.set_interrupt_enabled(Interrupt::EdgeLow, true);
        Self {
            btn_a,
            btn_b,
            btn_c,
        }
    }

    pub fn is_pressed(&mut self, btn: Button) -> bool {
        match btn {
            Button::A if self.btn_a.interrupt_status(Interrupt::EdgeLow) => {
                self.btn_a.clear_interrupt(Interrupt::EdgeLow);
                true
            }
            Button::B if self.btn_b.interrupt_status(Interrupt::EdgeLow) => {
                self.btn_b.clear_interrupt(Interrupt::EdgeLow);
                true
            }
            Button::C if self.btn_c.interrupt_status(Interrupt::EdgeLow) => {
                self.btn_c.clear_interrupt(Interrupt::EdgeLow);
                true
            }
            _ => false,
        }
    }
}

pub struct Sensors {
    track_a: SensorA,
    track_b: SensorB,
    track_c: SensorC,
}

impl Sensors {
    pub fn new(track_a: SensorA, track_b: SensorB, track_c: SensorC) -> Self {
        track_a.set_interrupt_enabled(Interrupt::EdgeLow, true);
        track_b.set_interrupt_enabled(Interrupt::EdgeLow, true);
        track_c.set_interrupt_enabled(Interrupt::EdgeLow, true);
        Self {
            track_a,
            track_b,
            track_c,
        }
    }

    pub fn is_car_detected(&mut self, track: Track) -> bool {
        match track {
            Track::A if self.track_a.interrupt_status(Interrupt::EdgeLow) => {
                self.track_a.clear_interrupt(Interrupt::EdgeLow);
                true
            }
            Track::B if self.track_b.interrupt_status(Interrupt::EdgeLow) => {
                self.track_b.clear_interrupt(Interrupt::EdgeLow);
                true
            }
            Track::C if self.track_c.interrupt_status(Interrupt::EdgeLow) => {
                self.track_c.clear_interrupt(Interrupt::EdgeLow);
                true
            }
            _ => false,
        }
    }
}
