use crate::*;
use embedded_hal::digital::v2::OutputPin;

type Segments = (
    Pin<bank0::Gpio0, Output<PushPull>>,
    Pin<bank0::Gpio1, Output<PushPull>>,
    Pin<bank0::Gpio2, Output<PushPull>>,
    Pin<bank0::Gpio3, Output<PushPull>>,
    Pin<bank0::Gpio4, Output<PushPull>>,
    Pin<bank0::Gpio5, Output<PushPull>>,
    Pin<bank0::Gpio6, Output<PushPull>>,
    Pin<bank0::Gpio7, Output<PushPull>>,
);

type Commons = (
    Pin<bank0::Gpio8, Output<PushPull>>,
    Pin<bank0::Gpio9, Output<PushPull>>,
);

pub struct Display {
    frame: usize,
    laps: [usize; 3],
    commons: Commons,
    segments: Segments,
}

impl Display {
    pub fn new(commons: Commons, segments: Segments) -> Self {
        let mut commons = commons;
        commons
            .0
            .set_drive_strength(OutputDriveStrength::TwelveMilliAmps);
        commons
            .1
            .set_drive_strength(OutputDriveStrength::TwelveMilliAmps);

        let mut segments = segments;
        segments
            .0
            .set_drive_strength(OutputDriveStrength::TwelveMilliAmps);
        segments
            .1
            .set_drive_strength(OutputDriveStrength::TwelveMilliAmps);
        segments
            .2
            .set_drive_strength(OutputDriveStrength::TwelveMilliAmps);
        segments
            .3
            .set_drive_strength(OutputDriveStrength::TwelveMilliAmps);
        segments
            .4
            .set_drive_strength(OutputDriveStrength::TwelveMilliAmps);
        segments
            .5
            .set_drive_strength(OutputDriveStrength::TwelveMilliAmps);
        segments
            .6
            .set_drive_strength(OutputDriveStrength::TwelveMilliAmps);
        segments
            .7
            .set_drive_strength(OutputDriveStrength::TwelveMilliAmps);

        Self {
            commons,
            segments,
            frame: 0,
            laps: [0; 3],
        }
    }

    pub fn reset(&mut self) {
        self.laps = [0; 3];
    }

    pub fn set_track_laps(&mut self, track: Track, laps: usize) {
        if let Some(val) = self.laps.get_mut(track) {
            *val = laps
        }
    }

    pub fn animate(&mut self) {
        const DIGITS: [u8; 10] = [
            0b0111111, 0b0000110, 0b1011011, 0b1001111, 0b1100110, 0b1101101, 0b1111101, 0b0000111,
            0b1111111, 0b1101111,
        ];

        let laps = self.laps[0];
        self.set_active_digit(Some(42));
        self.update_segments(DIGITS[laps % 10]);
        self.frame = self.frame.wrapping_add(1);
    }

    fn set_active_digit(&mut self, idx: Option<usize>) {
        self.commons.0.set_high().ok();
        self.commons.1.set_high().ok();
        match idx {
            Some(0) => self.commons.0.set_low().ok(),
            Some(1) => self.commons.1.set_low().ok(),
            _ => None,
        };
    }

    fn update_segments(&mut self, segments: u8) {
        let test_bit = |bit: usize| {
            if (segments >> bit) & 1 == 1 {
                PinState::Low
            } else {
                PinState::High
            }
        };
        self.segments.0.set_state(test_bit(0)).ok();
        self.segments.1.set_state(test_bit(1)).ok();
        self.segments.2.set_state(test_bit(2)).ok();
        self.segments.3.set_state(test_bit(3)).ok();
        self.segments.4.set_state(test_bit(4)).ok();
        self.segments.5.set_state(test_bit(5)).ok();
        self.segments.6.set_state(test_bit(6)).ok();
        self.segments.7.set_state(test_bit(7)).ok();
    }
}
