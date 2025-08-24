use std::time::{Duration, Instant};

fn smootherstep01(t: f32) -> f32 {
    // C2 smooth, zero slope at ends
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

// A single ramp centered on a "target time" with a chosen duration.
pub struct VolumeRamp {
    from: u8,
    to: u8,
    start: Instant,
    end: Instant,
}

impl VolumeRamp {
    // Build a ramp: e.g., 3-minute ramp ending at 08:00
    pub fn new(
        current_volume_now: u8,
        target: u8,
        target_time: Instant,
        duration: Duration,
    ) -> VolumeRamp {
        VolumeRamp {
            from: current_volume_now,
            to: target,
            start: target_time
                .checked_sub(duration)
                .expect("This failing is pathological"),
            end: target_time,
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    pub fn value_at(&self, now: Instant) -> u8 {
        if now <= self.start {
            return self.from;
        }
        if now >= self.end {
            return self.to;
        }

        let total = (self.end - self.start).as_secs_f32();
        let t = (now - self.start).as_secs_f32() / total; // 0..1
        let t = smootherstep01(t); // eased 0..1
        let v = f32::from(self.from) + (f32::from(self.to) - f32::from(self.from)) * t;

        v.round().clamp(0.0, 100.0) as u8
    }
}
