use crate::*;

pub type DisplayPin = Pin<DynPinId, FunctionSio<SioOutput>, PullDown>;

const DIGITS: [u8; 10] = [
    0b0111111, 0b0000110, 0b1011011, 0b1001111, 0b1100110, 0b1101101, 0b1111101, 0b0000111,
    0b1111111, 0b1101111,
];

pub struct Display {
    frame: usize,
    laps: [usize; 3],
    commons: [DisplayPin; 6],
    segments: [DisplayPin; 8],
}

impl Display {
    pub fn new(commons: [DisplayPin; 6], segments: [DisplayPin; 8]) -> Self {
        let mut commons = commons;
        let mut segments = segments;

        for common in &mut commons {
            common.set_drive_strength(OutputDriveStrength::TwelveMilliAmps);
        }

        for segment in &mut segments {
            segment.set_drive_strength(OutputDriveStrength::TwelveMilliAmps);
        }

        Self {
            commons,
            segments,
            frame: 0,
            laps: [0; 3],
        }
    }

    pub fn reset(&mut self) {
        self.set_active_segments(0);
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
            self.set_active_segments(DIGITS[laps % 10]);
        } else {
            self.set_active_segments(DIGITS[(laps / 10) % 10]);
        }
        self.set_active_digit(Some(digit_idx));
        self.frame = self.frame.wrapping_add(1);
    }

    fn set_active_digit(&mut self, digit_idx: Option<usize>) {
        for (idx, common) in self.commons.iter_mut().enumerate() {
            match digit_idx {
                Some(digit_idx) if digit_idx == idx => common.set_low(),
                _ => common.set_high(),
            }
            .ok();
        }
    }

    fn set_active_segments(&mut self, data: u8) {
        let mut data = data;
        for segment in &mut self.segments {
            let segment_on = data & 1 == 1;
            data >>= 1;
            segment.set_state(segment_on.into()).ok();
        }
    }
}
