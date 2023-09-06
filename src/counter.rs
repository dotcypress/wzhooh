use crate::*;
use heapless::Vec;

pub type LapsHistory = Vec<Instant, 128>;

pub struct LapCounter {
    history: [LapsHistory; 3],
}

impl LapCounter {
    pub fn new() -> Self {
        Self {
            history: [
                LapsHistory::default(),
                LapsHistory::default(),
                LapsHistory::default(),
            ],
        }
    }

    pub fn reset(&mut self) {
        for laps in self.history.iter_mut() {
            laps.clear();
        }
    }

    pub fn record_lap(&mut self, track: Track, ts: Instant) -> usize {
        match self.history.get_mut(track.index()) {
            Some(laps) => {
                laps.push(ts).ok();
                laps.len()
            }
            _ => 0,
        }
    }
}

impl Default for LapCounter {
    fn default() -> Self {
        Self::new()
    }
}
