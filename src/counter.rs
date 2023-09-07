use crate::*;
use heapless::Vec;

pub type LapsHistory = Vec<Instant, 128>;

#[derive(Default)]
pub struct LapCounter {
    history: [LapsHistory; 3],
}

impl LapCounter {
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
