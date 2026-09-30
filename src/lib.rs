//! Multi-track audio mixer

/// Audio track for mixing
#[derive(Debug, Clone)]
pub struct AudioTrack {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
    pub gain: f32,
    pub pan: f32, // -1.0 (left) to 1.0 (right)
}

impl AudioTrack {
    pub fn new(samples: Vec<f32>, sample_rate: u32, channels: u16) -> Self {
        Self { samples, sample_rate, channels, gain: 1.0, pan: 0.0 }
    }

    pub fn duration_secs(&self) -> f64 {
        if self.sample_rate == 0 || self.channels == 0 { return 0.0; }
        (self.samples.len() as f64 / self.channels as f64) / self.sample_rate as f64
    }

    pub fn num_frames(&self) -> usize {
        self.samples.len() / self.channels as usize
    }

    fn pan_gains(&self) -> (f32, f32) {
        let pan = self.pan.clamp(-1.0, 1.0);
        let left = (1.0 - pan.max(0.0)).min(1.0);
        let right = (1.0 + pan.min(0.0)).min(1.0);
        (left, right)
    }
}

/// Mix multiple stereo tracks into a single output
pub fn mix_tracks(tracks: &[AudioTrack], output_channels: u16) -> Vec<f32> {
    if tracks.is_empty() { return Vec::new(); }
    let max_frames = tracks.iter().map(|t| t.num_frames()).max().unwrap_or(0);
    let mut output = vec![0.0f32; max_frames * output_channels as usize];

    for track in tracks {
        let (pan_l, pan_r) = track.pan_gains();
        let gain = track.gain;
        let tc = track.channels as usize;
        let oc = output_channels as usize;

        for frame in 0..max_frames {
            let ti = frame * tc;
            let oi = frame * oc;

            let (l, r) = if tc >= 2 && ti + 1 < track.samples.len() {
                (track.samples[ti], track.samples[ti + 1])
            } else if tc == 1 && ti < track.samples.len() {
                (track.samples[ti], track.samples[ti])
            } else {
                (0.0, 0.0)
            };

            let scaled_l = l * gain * pan_l;
            let scaled_r = r * gain * pan_r;

            if oc >= 2 {
                output[oi] += scaled_l;
                output[oi + 1] += scaled_r;
            } else {
                output[oi] += (scaled_l + scaled_r) * 0.5;
            }
        }
    }
    output
}

/// Apply a simple fade-in envelope
pub fn fade_in(samples: &mut [f32], duration_frames: usize) {
    if duration_frames == 0 { return; }
    for (i, s) in samples.iter_mut().enumerate().take(duration_frames) {
        *s *= i as f32 / duration_frames as f32;
    }
}

/// Apply a simple fade-out envelope
pub fn fade_out(samples: &mut [f32], duration_frames: usize) {
    if duration_frames == 0 { return; }
    let len = samples.len();
    let start = len.saturating_sub(duration_frames);
    for (i, s) in samples[start..].iter_mut().enumerate() {
        *s *= 1.0 - (i as f32 / duration_frames as f32);
    }
}

/// Normalize samples to peak amplitude
pub fn normalize(samples: &mut [f32], target_peak: f32) {
    let peak = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    if peak == 0.0 { return; }
    let scale = target_peak / peak;
    for s in samples.iter_mut() {
        *s *= scale;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mix_empty() {
        let result = mix_tracks(&[], 2);
        assert!(result.is_empty());
    }

    #[test]
    fn test_fade_in() {
        let mut buf = vec![1.0f32; 100];
        fade_in(&mut buf, 10);
        assert!(buf[0] < buf[9]);
    }

    #[test]
    fn test_normalize() {
        let mut buf = vec![0.5f32, -0.5f32];
        normalize(&mut buf, 1.0);
        assert!((buf[0] - 1.0).abs() < 0.001);
    }
}

/// FNV-1a 64 — the digest every substrate in the SuperInstance fleet agrees on.
pub const FNV_OFFSET: u64 = 0xcbf29ce484222325;
pub const FNV_PRIME: u64 = 0x100000001b3;

#[inline]
pub fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h = FNV_OFFSET;
    for &b in bytes {
        h = (h ^ b as u64).wrapping_mul(FNV_PRIME);
    }
    h
}

/// True if this crate's FNV-1a still agrees with the rest of the fleet.
pub fn canary_holds() -> bool {
    fnv1a64("café Δ 日本語".as_bytes()) == 0x024a555471370b18d
}
