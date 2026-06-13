# audio-mixer

**Multi-track digital audio mixing in Rust — gain, pan, fade, and normalization for PCM sample buffers.**

Digital audio mixing is the process of combining multiple audio tracks (each a sequence of floating-point samples representing sound pressure waves) into a single output stream. At each sample frame, the mixer sums contributions from all tracks, applying per-track gain and stereo panning. `audio-mixer` implements this core DSP operation with envelope shaping (fade in/out) and peak normalization.

## Why It Matters

Audio mixing is the final stage of every digital audio pipeline — music production, game audio, podcast editing, live sound, and voice assistants all depend on it. The mathematical operations are simple (multiply and add), but the engineering constraints are subtle:

- **Sample-accurate timing**: A 44.1 kHz stereo signal processes 88,200 samples/second. At 32-bit float, that's 352 KB/s per track.
- **Click-free fades**: Abrupt amplitude changes produce audible clicks (broadband spectral artifacts). Linear ramps with duration ≥ 10ms are the minimum acceptable fade.
- **Pan law**: Equal-power panning requires √2 scaling to maintain constant perceived loudness as sound moves between speakers.
- **Clipping prevention**: Summing multiple tracks can exceed [-1.0, 1.0]. Normalization rescales after mixing.

## How It Works

### Signal Model

An `AudioTrack` is a buffer of interleaved f32 samples:

> samples[n] ∈ [-1.0, 1.0], n = 0, 1, ..., (frames × channels) - 1

For stereo (channels = 2): `samples[2*i]` = left, `samples[2*i + 1]` = right.

### Gain and Pan

Each track has:
- **gain** g ∈ [0, ∞) — linear amplitude multiplier (0 = silent, 1 = unity, 2 = +6 dB)
- **pan** p ∈ [-1, 1] — stereo position (-1 = full left, 0 = center, +1 = full right)

The pan gains use **equal-power panning** approximated by linear interpolation:

> left_gain = 1 - max(p, 0)  
> right_gain = 1 + min(p, 0)  

At center (p=0): both = 1.0 (mono compatibility). At full left (p=-1): left = 1, right = 0.

### Mixing Equation

For output channel c at frame f:

> output[f, c] = Σ_tracks (sample[f, L] · g · left + sample[f, R] · g · right) / |tracks|

where L, R depend on the track's channel count:
- **Stereo track** → L = sample[2f], R = sample[2f+1]
- **Mono track** → L = R = sample[f]
- **Downmix to mono** → output[f] = (scaled_L + scaled_R) / 2

### Fade Envelopes

**Linear fade-in** over D frames:

> envelope(i) = i / D,  for i ∈ [0, D)

**Linear fade-out** over D frames at the end of a buffer of length N:

> envelope(i) = 1 - (i - (N - D)) / D,  for i ∈ [N-D, N)

Linear fades are O(D) in time. While exponential or S-curve fades produce smoother results, linear is the standard baseline.

### Normalization

Peak normalization rescales so the maximum |sample| equals `target_peak`:

> scale = target_peak / max(|samples[i]|)  
> samples[i] *= scale

This preserves the spectral shape while fixing the dynamic range. Time: O(n).

### Complexity

| Operation | Time | Space |
|-----------|------|-------|
| `mix_tracks(tracks, out_ch)` | O(T · F · C) | O(F · C_out) |
| `fade_in(buf, D)` | O(D) | O(1) in-place |
| `fade_out(buf, D)` | O(D) | O(1) in-place |
| `normalize(buf, target)` | O(n) | O(1) in-place |
| `duration_secs()` | O(1) | — |

Where T = tracks, F = max frames, C = channels, n = total samples.

## Quick Start

```rust
use audio_mixer::{AudioTrack, mix_tracks, fade_in, fade_out, normalize};

// Two stereo tracks: a sine-ish wave and a noise floor
let track_a = AudioTrack {
    samples: vec![0.5; 44100 * 2], // 1 second of 0.5 amplitude
    sample_rate: 44100,
    channels: 2,
    gain: 0.8,
    pan: -0.3, // slightly left
};
let track_b = AudioTrack {
    samples: vec![0.2; 44100 * 2],
    sample_rate: 44100,
    channels: 2,
    gain: 1.0,
    pan: 0.5,  // slightly right
};

// Mix to stereo
let mut output = mix_tracks(&[track_a, track_b], 2);

// Apply fades
let fade_samples = 4410; // 100ms at 44.1kHz
fade_in(&mut output, fade_samples);
fade_out(&mut output, fade_samples);

// Normalize to -3dB (0.707 peak)
normalize(&mut output, 0.707);
```

## API

- **`AudioTrack`** — { samples: Vec\<f32\>, sample_rate, channels, gain, pan }
  - `new(samples, sample_rate, channels)` — unity gain, center pan
  - `duration_secs()` → f64
  - `num_frames()` → usize
- **`mix_tracks(tracks, output_channels)`** → Vec\<f32\> — Sum-mix with gain/pan
- **`fade_in(samples, duration_frames)`** — Linear ramp from 0 to 1
- **`fade_out(samples, duration_frames)`** — Linear ramp from 1 to 0
- **`normalize(samples, target_peak)`** — Peak rescale to target amplitude

## Architecture Notes

The mixer sits at the γ+η=C boundary: **γ (generative capacity)** is the number and diversity of input tracks (sources), while **η (evaluative depth)** is the quality of the mixing decisions (gain staging, pan positions, fade curves). The output complexity C = f(γ, η) — more tracks (γ) with better-tuned parameters (η) produces a richer mix. Clipping and phasing artifacts occur when γ·η exceeds the dynamic range budget.

## References

1. Pirkle, W. (2012). *Designing Audio Effect Plugins in C++*. Focal Press. — DSP mixing fundamentals.
2. Zölzer, U. (2008). *Digital Audio Signal Processing* (2nd ed.). Wiley. — Amplitude panning and normalization theory.
3. Roads, C. (1996). *The Computer Music Tutorial*. MIT Press. — Envelope generation and click-free editing.
4. AES (2002). "AES Recommended Practice for Digital Audio Engineering." *AES-2id-2002*. — Sample rate and level standards.

## License

MIT
