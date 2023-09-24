use crate::*;
use fugit::MicrosDurationU64;
use heapless::Vec;

pub type Duration = MicrosDurationU64;
pub type LapsHistory = Vec<Instant, 256>;
pub type LapsStats = Vec<Duration, 256>;

pub struct TrackStats {
    stats: LapsStats,
}

impl TrackStats {
    pub fn laps(&self) -> usize {
        self.stats.len()
    }

    pub fn last(&self) -> Option<&Duration> {
        self.stats.iter().last()
    }

    pub fn best(&self) -> Option<&Duration> {
        self.stats.iter().min()
    }

    pub fn history(&self) -> &[Duration] {
        &self.stats
    }
}

#[derive(Default)]
pub struct LapCounter {
    history: [LapsHistory; 3],
}

impl LapCounter {
    pub fn reset(&mut self) {
        let ts = app::monotonics::now();
        for history in self.history.iter_mut() {
            history.clear();
            history.push(ts).ok();
        }
    }

    pub fn record_lap(&mut self, track: Track, ts: Instant) -> Option<TrackStats> {
        self.history
            .get_mut(track)
            .and_then(|history| history.push(ts).ok())
            .and(self.stats(track))
    }

    pub fn stats(&self, track: Track) -> Option<TrackStats> {
        self.history.get(track).map(|history| {
            let stats = history
                .windows(2)
                .filter_map(|win| win[1].checked_duration_since(win[0]))
                .collect();
            TrackStats { stats }
        })
    }
}
