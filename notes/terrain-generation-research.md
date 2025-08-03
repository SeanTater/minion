# Terrain Generation Library Research

## Executive Summary
For Stage 6 procedural terrain generation in the mapgen binary, I recommend **noise-functions** as the primary library with **noise-rs** as a fallback option. The choice prioritizes performance, simplicity, and alignment with the project's minimalist approach.

## Library Analysis

### Recommended: noise-functions (v0.4.x)
**Decision Criteria:** Performance-first, lightweight, simple API
- **Maintenance:** Active (14 releases, recent activity)
- **Performance:** Static permutation tables, f32-based, no runtime allocation
- **API Design:** Functional approach, easy composition
- **Features:** Perlin, OpenSimplex2, Value, Cell noise + fractals
- **Dependencies:** Minimal
- **Downloads:** 248/month (growing, niche but solid)

**Pros:**
- Zero-allocation approach aligns with minimalist principles
- Static permutation tables = better performance for mapgen tool
- f32 precision matches existing TerrainData structure
- Simple function calls vs complex builder patterns
- Supports domain warping and tileable noise

**Cons:**
- Smaller ecosystem (5 crates vs 5.9k for noise-rs)
- Less battle-tested than noise-rs
- Fewer noise types than noise-rs

### Fallback: noise-rs (v0.9.x)
**Decision Criteria:** Proven reliability, extensive features
- **Maintenance:** Very active (997 stars, 126 forks, 47 contributors)
- **Performance:** f64-based, runtime permutation tables
- **API Design:** Trait-based NoiseFn composition
- **Features:** Extensive noise types, fractal support, complex compositions
- **Dependencies:** Well-established
- **Downloads:** High adoption (5.9k projects)

**Pros:**
- Battle-tested in thousands of projects
- Comprehensive feature set
- Strong community support
- Excellent documentation and examples

**Cons:**
- f64 precision requires conversion to f32
- Runtime allocation for permutation tables
- More complex API than needed for simple heightmaps
- Heavier dependency footprint

### Rejected: bracket-noise
**Reasoning:** Game-specific ecosystem lock-in, FastNoise C++ port overhead

## Performance Considerations

### Static vs Runtime Permutation Tables
- **noise-functions:** Static tables = better cache locality, no initialization cost
- **noise-rs:** Runtime tables = memory overhead, initialization required

### Precision Tradeoffs
- **f32 (noise-functions):** Direct compatibility with TerrainData, sufficient for terrain heights
- **f64 (noise-rs):** Higher precision, but requires conversion, memory overhead

### Memory Usage
- **noise-functions:** Minimal allocation, stateless functions
- **noise-rs:** Noise generators require storage, reusable but heavier

## Implementation Strategy

### Phase 1: Core Noise Generation
Use noise-functions for initial implementation:
```rust
use noise_functions::*;

// Simple height generation
let height = fbm_2d(x, y, seed, 4);  // 4 octaves

// Multi-layered terrain
let base = perlin_2d(x, y, seed);
let detail = fbm_2d(x * 4.0, y * 4.0, seed + 1, 3);
let final_height = base + detail * 0.3;
```

### Phase 2: Fallback Integration
If noise-functions proves insufficient, implement noise-rs wrapper:
```rust
use noise::{NoiseFn, Perlin, Fbm};

let perlin = Perlin::new(seed);
let fbm = Fbm::<Perlin>::new(seed).set_octaves(4);
let height = fbm.get([x as f64, y as f64]) as f32;
```

## Risk Assessment

### Low Risk (noise-functions)
- **Technical:** Simple API, proven algorithms
- **Maintenance:** Active development, stable API
- **Performance:** Predictable, optimized for common use cases

### Medium Risk (noise-rs)
- **Complexity:** More complex than needed, but well-documented
- **Performance:** Good but with overhead we don't need
- **Ecosystem:** Very stable, unlikely to break

## Next Steps

1. **Prototype with noise-functions** - Implement basic height generation
2. **Performance validation** - Benchmark against target map sizes (64x64 to 256x256)
3. **Feature gap analysis** - Ensure sufficient noise types for terrain variety
4. **Fallback preparation** - Keep noise-rs integration path open

## Decision Rationale

This recommendation aligns with project principles:
- **Minimalist:** noise-functions avoids unnecessary complexity
- **Performance-first:** Static tables and f32 precision optimize for use case
- **Pragmatic:** Clear fallback path if limitations discovered
- **Code reuse:** Functional approach enables easy composition and testing