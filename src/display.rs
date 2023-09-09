use crate::*;

const DIGITS: [u8; 10] = [
    0b0111111, 0b0000110, 0b1011011, 0b1001111, 0b1100110, 0b1101101, 0b1111101, 0b0000111,
    0b1111111, 0b1101111,
];

pub type DisplaySegmentA = Pin<bank0::Gpio8, Output<PushPull>>;
pub type DisplaySegmentB = Pin<bank0::Gpio9, Output<PushPull>>;
pub type DisplaySegmentC = Pin<bank0::Gpio10, Output<PushPull>>;
pub type DisplaySegmentD = Pin<bank0::Gpio11, Output<PushPull>>;
pub type DisplaySegmentE = Pin<bank0::Gpio12, Output<PushPull>>;
pub type DisplaySegmentF = Pin<bank0::Gpio13, Output<PushPull>>;
pub type DisplaySegmentG = Pin<bank0::Gpio14, Output<PushPull>>;
pub type DisplaySegmentDP = Pin<bank0::Gpio15, Output<PushPull>>;

pub type DisplayCommon1 = Pin<bank0::Gpio20, Output<PushPull>>;
pub type DisplayCommon2 = Pin<bank0::Gpio21, Output<PushPull>>;
pub type DisplayCommon3 = Pin<bank0::Gpio18, Output<PushPull>>;
pub type DisplayCommon4 = Pin<bank0::Gpio19, Output<PushPull>>;
pub type DisplayCommon5 = Pin<bank0::Gpio16, Output<PushPull>>;
pub type DisplayCommon6 = Pin<bank0::Gpio17, Output<PushPull>>;

pub type Segments = (
    DisplaySegmentA,
    DisplaySegmentB,
    DisplaySegmentC,
    DisplaySegmentD,
    DisplaySegmentE,
    DisplaySegmentF,
    DisplaySegmentG,
    DisplaySegmentDP,
);

pub type Commons = (
    DisplayCommon1,
    DisplayCommon2,
    DisplayCommon3,
    DisplayCommon4,
    DisplayCommon5,
    DisplayCommon6,
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
        let mut segments = segments;

        commons
            .0
            .set_drive_strength(OutputDriveStrength::TwelveMilliAmps);
        commons
            .1
            .set_drive_strength(OutputDriveStrength::TwelveMilliAmps);
        commons
            .2
            .set_drive_strength(OutputDriveStrength::TwelveMilliAmps);
        commons
            .3
            .set_drive_strength(OutputDriveStrength::TwelveMilliAmps);
        commons
            .4
            .set_drive_strength(OutputDriveStrength::TwelveMilliAmps);
        commons
            .5
            .set_drive_strength(OutputDriveStrength::TwelveMilliAmps);

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
        self.update_segments(0);
        self.laps = [0; 3];
    }

    pub fn set_track_laps(&mut self, track: Track, laps: usize) {
        if let Some(val) = self.laps.get_mut(track.index()) {
            *val = laps
        }
    }

    pub fn animate(&mut self) {
        let track_idx = (self.frame / 2) % 3;
        let digit_idx = self.frame % 6;
        let laps = self.laps[track_idx];

        self.set_active_digit(None);
        if digit_idx % 2 == 0 {
            self.update_segments(DIGITS[laps % 10]);
        } else {
            self.update_segments(DIGITS[(laps / 10) % 10]);
        }
        self.set_active_digit(Some(digit_idx));
        self.frame = self.frame.wrapping_add(1);
    }

    fn set_active_digit(&mut self, idx: Option<usize>) {
        self.commons.0.set_high().ok();
        self.commons.1.set_high().ok();
        self.commons.2.set_high().ok();
        self.commons.3.set_high().ok();
        self.commons.4.set_high().ok();
        self.commons.5.set_high().ok();
        match idx {
            Some(0) => self.commons.0.set_low().ok(),
            Some(1) => self.commons.1.set_low().ok(),
            Some(2) => self.commons.2.set_low().ok(),
            Some(3) => self.commons.3.set_low().ok(),
            Some(4) => self.commons.4.set_low().ok(),
            Some(5) => self.commons.5.set_low().ok(),
            _ => None,
        };
    }

    fn update_segments(&mut self, data: u8) {
        let test_bit = |bit: usize| {
            if (data >> bit) & 1 == 1 {
                PinState::High
            } else {
                PinState::Low
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
