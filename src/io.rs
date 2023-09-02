use crate::*;

pub type Track = usize;
pub type Button = usize;

pub enum IoEvent {
    ButtonPressed(Button),
    CarDetected(Track, Instant),
}

pub struct Buttons {
    btn_a: Pin<bank0::Gpio11, Input<PullUp>>,
    btn_b: Pin<bank0::Gpio12, Input<PullUp>>,
    btn_c: Pin<bank0::Gpio13, Input<PullUp>>,
}

impl Buttons {
    pub fn new(
        btn_a: Pin<bank0::Gpio11, Input<PullUp>>,
        btn_b: Pin<bank0::Gpio12, Input<PullUp>>,
        btn_c: Pin<bank0::Gpio13, Input<PullUp>>,
    ) -> Self {
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
            0 if self.btn_a.interrupt_status(Interrupt::EdgeLow) => {
                self.btn_a.clear_interrupt(Interrupt::EdgeLow);
                true
            }
            1 if self.btn_b.interrupt_status(Interrupt::EdgeLow) => {
                self.btn_b.clear_interrupt(Interrupt::EdgeLow);
                true
            }
            2 if self.btn_c.interrupt_status(Interrupt::EdgeLow) => {
                self.btn_c.clear_interrupt(Interrupt::EdgeLow);
                true
            }
            _ => false,
        }
    }
}

pub struct Sensors {
    track_a: Pin<bank0::Gpio20, Input<Floating>>,
    track_b: Pin<bank0::Gpio21, Input<Floating>>,
    track_c: Pin<bank0::Gpio22, Input<Floating>>,
}

impl Sensors {
    pub fn new(
        track_a: Pin<bank0::Gpio20, Input<Floating>>,
        track_b: Pin<bank0::Gpio21, Input<Floating>>,
        track_c: Pin<bank0::Gpio22, Input<Floating>>,
    ) -> Self {
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
            0 if self.track_a.interrupt_status(Interrupt::EdgeLow) => {
                self.track_a.clear_interrupt(Interrupt::EdgeLow);
                true
            }
            1 if self.track_b.interrupt_status(Interrupt::EdgeLow) => {
                self.track_b.clear_interrupt(Interrupt::EdgeLow);
                true
            }
            2 if self.track_c.interrupt_status(Interrupt::EdgeLow) => {
                self.track_c.clear_interrupt(Interrupt::EdgeLow);
                true
            }
            _ => false,
        }
    }
}
