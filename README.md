# Audio Mixer

**A Rust library for multi-track digital audio mixing** — combines multiple audio tracks with independent gain, pan, and channel configuration into a single output buffer, with fade and normalize utilities.

## Why It Matters

Digital audio mixing is the foundation of DAWs (Digital Audio Workstations), game engines, podcast tools, and live sound software. The core operation — summing sample buffers while applying per-track gain and stereo panning — appears deceptively simple but requires careful handling of channel layouts (mono, stereo), pan laws (constant power vs. linear), and sample-accurate envelope processing.

This crate handles the three critical concerns:

1. **Channel mapping** — correctly upmixing mono tracks to stereo or downmixing stereo to mono
2. **Pan/gain application** — linear pan law with equal-power gains for left/right channels
3. **Envelope processing** — fade-in/fade-out and peak normalization

## How It Works

**Mixing** (`mix_tracks`): Each track contributes samples frame-by-frame. For stereo output, tracks are panned using a linear pan law: a pan value of -1.0 (full left) produces `(1.0, 0.0)` gains, 0.0 (center) produces `(1.0, 1.0)`, and +1.0 (full right) produces `(0.0, 1.0)`. Samples are multiplied by both the track's gain and the pan gain, then summed into the output buffer. Mono tracks are duplicated to both channels; stereo tracks use per-channel data directly.

**Fades**: Linear amplitude ramps. `fade_in` multiplies the first N frames by `i/N` (0→1), `fade_out` multiplies the last N frames by `1-i/N` (1→0).

**Normalization**: Finds the peak absolute amplitude, then scales all samples so the peak equals the target (typically 1.0 = 0 dBFS). O(n) single-pass.

## Quick Start

```rust
use audio_mixer::{AudioTrack, mix_tracks, normalize};

// Create two stereo tracks
let track_a = AudioTrack::new(vec![0.5, 0.5, 0.3, 0.3], 44100, 2);
let mut track_b = AudioTrack::new(vec![0.4, 0.4, 0.6, 0.6], 44100, 2);
track_b.gain = 0.8;
track_b.pan = -0.5; // pan left

// Mix down to stereo
let mut output = mix_tracks(&[track_a, track_b], 2);

// Normalize to peak amplitude 1.0
normalize(&mut output, 1.0);
```

## API

- **`AudioTrack`** — Samples, sample rate, channel count, gain, pan (-1.0 to 1.0)
- **`mix_tracks(tracks, output_channels)`** → `Vec<f32>` — Sum all tracks into one buffer
- **`fade_in(samples, frames)`** / **`fade_out(samples, frames)`** — Apply linear fades
- **`normalize(samples, target_peak)`** — Scale to target peak amplitude

## Architecture Notes

Provides the audio processing primitive for SuperInstance media tooling. The mixer is designed to be embedded in real-time audio pipelines where latency matters — all operations are in-place or pre-allocated. See the [architecture overview](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

## License

MIT
