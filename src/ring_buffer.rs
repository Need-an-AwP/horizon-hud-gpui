use crate::config::HISTORY_LEN;

pub(crate) struct RingBuffer {
    samples: [f32; HISTORY_LEN],
    write: usize,
    filled: usize,
}

impl RingBuffer {
    pub(crate) fn new() -> Self {
        Self {
            samples: [0.0; HISTORY_LEN],
            write: 0,
            filled: 0,
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.filled
    }

    pub(crate) fn clear(&mut self) {
        *self = Self::new();
    }

    pub(crate) fn push(&mut self, value: f32) {
        self.samples[self.write] = value;
        self.write = (self.write + 1) % HISTORY_LEN;
        if self.filled < HISTORY_LEN {
            self.filled += 1;
        }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = f32> + '_ {
        let start = if self.filled < HISTORY_LEN {
            0
        } else {
            self.write
        };
        (0..self.filled).map(move |i| self.samples[(start + i) % HISTORY_LEN])
    }

    pub(crate) fn last_n(&self, n: usize) -> impl Iterator<Item = f32> + '_ {
        let n = n.min(self.filled);
        let start = (self.write + HISTORY_LEN - n) % HISTORY_LEN;
        (0..n).map(move |i| self.samples[(start + i) % HISTORY_LEN])
    }

    pub(crate) fn samples(&self) -> Vec<f32> {
        self.iter().collect()
    }
}
