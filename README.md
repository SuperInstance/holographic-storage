# Holographic Storage

[![crates.io](https://img.shields.io/crates/v/holographic-storage.svg)](https://crates.io/crates/holographic-storage)
[![docs.rs](https://docs.rs/holographic-storage/badge.svg)](https://docs.rs/holographic-storage)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

> **Distributed holographic encoding — store data as wave interference patterns, reconstruct from partial fragments.**

---

## The Problem

Traditional storage is all-or-nothing: lose a sector, lose the data. Biological holographic memory is distributed — every piece of the hologram contains information about the whole. If you could store data the same way, partial damage wouldn't destroy information.

## Why This Exists

Holographic Storage implements holographic encoding principles for data:
- **Wave patterns**: Data encoded as amplitude + phase waves
- **Interference**: Multiple waves superpose to create holograms
- **Multiplexing**: Multiple data sources encoded into one pattern
- **Partial reconstruction**: Data recoverable from fragments
- **Reference beams**: Keys for retrieving specific data
- **Fragment distribution**: Data spread across multiple locations

## Architecture

```
  Data ──→ Wave Pattern ──→ Interference ──→ Hologram
  [u8]     (amp, phase)    (superposition)   (distributed)

  ┌─────────┐    ┌─────────┐    ┌──────────────────────┐
  │ Source A │───→│ Wave A  │    │                      │
  └─────────┘    └────┬────┘    │  Multiplexed Hologram│
                      │         │  (A + B + C combined)│
  ┌─────────┐    ┌────▼────┐   │                      │
  │ Source B │───→│ Wave B  │──→│  Reconstruct with    │
  └─────────┘    └────┬────┘   │  reference beam       │
                      │         └──────────────────────┘
  ┌─────────┐    ┌────▼────┐
  │ Source C │───→│ Wave C  │
  └─────────┘    └─────────┘
```

## Installation

```toml
[dependencies]
holographic-storage = "0.1"
```

## API Reference

### `WavePattern`

Data encoded as wave amplitudes and phases:

```rust
use holographic_storage::interference_pattern::WavePattern;

let data = b"hello";
let pattern = WavePattern::from_data(data);

// Sample the wave at time t
let value = pattern.sample(0.5);

// Energy of the pattern
let energy = pattern.energy();
```

### Interference & Superposition

```rust
use holographic_storage::interference_pattern::*;

let a = WavePattern::from_data(b"hello");
let b = WavePattern::from_data(b"world");

// Superpose two waves
let combined = a.superpose(&b);

// Interference with mixing factor
let mixed = a.interference_with(&b, 0.7); // 70% a, 30% b
```

### Multiplexing

```rust
use holographic_storage::interference_pattern::*;

let sources: Vec<&[u8]> = vec![b"data_a", b"data_b", b"data_c"];
let hologram = multiplex_encode(&sources, 1.0);

// Extract specific data with demultiplexing
let extracted = demultiplex(&hologram, 0, 3, 6); // extract source 0
```

### `ReferenceBeam`

Keys for retrieving data:

```rust
use holographic_storage::reference_beam::ReferenceBeam;

let beam = ReferenceBeam::new(1.0, 0.0, 0.0); // frequency, angle, phase
let key = beam.generate_key(256);
let matches = beam.matches(&other_beam, 0.01);
```

### Fragment Distribution

```rust
use holographic_storage::fragment_distribution::*;

let data = b"important data";
let fragments = distribute(data, 5, 3); // 5 fragments, need 3 to reconstruct
```

## Usage Examples

### Example 1: Encode and Reconstruct

```rust
use holographic_storage::interference_pattern::*;

let data = b"secret message";
let pattern = WavePattern::from_data(data);

// Reconstruct (convert back)
let bytes = pattern.to_bytes();
```

### Example 2: Multiplex Multiple Sources

```rust
use holographic_storage::interference_pattern::*;

let sources = vec![b"alpha", b"beta", b"gamma"];
let hologram = multiplex_encode(&sources.iter().map(|s| s.as_slice()).collect::<Vec<_>>(), 1.0);

for i in 0..3 {
    let extracted = demultiplex(&hologram, i, 3, 5);
    println!("Source {}: {:?}", i, extracted);
}
```

### Example 3: Noise Tolerance

```rust
use holographic_storage::interference_pattern::*;

let mut pattern = WavePattern::from_data(b"robust data");
pattern.add_noise(0.1); // add 10% noise

// Data is still partially recoverable due to distributed encoding
let energy_before = WavePattern::from_data(b"robust data").energy();
let energy_after = pattern.energy();
```

## Mathematical Background

**Wave Superposition**: Two waves combine by adding their amplitudes and averaging phases:

```
A_combined[i] = A₁[i] + A₂[i]
φ_combined[i] = (φ₁[i] + φ₂[i]) / 2
```

**Multiplexing**: Multiple sources are encoded with unique angular offsets:

```
φ_k[i] += k × π / N  (for source k of N total)
```

**Sampling**: The wave value at time t:

```
W(t) = Σᵢ Aᵢ × cos(2πft + φᵢ)
```

## Performance

| Operation | Complexity |
|-----------|-----------|
| Encode data → wave | O(n) |
| Superpose two waves | O(n) |
| Multiplex K sources | O(K × n) |
| Demultiplex | O(n) |
| Generate reference key | O(n) |

## License

Licensed under the [MIT License](LICENSE).

## Contributing

1. Fork the repository
2. Create a feature branch
3. Write tests
4. Push and open a Pull Request
