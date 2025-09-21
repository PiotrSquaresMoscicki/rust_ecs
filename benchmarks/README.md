# Game Performance Benchmarks

This directory contains benchmarks comparing the performance of the ECS game simulation with and without replay recording enabled.

## Overview

The benchmark simulates a realistic ECS game scenario with:
- 3 actors moving between home and work positions
- Movement, wait, and render systems
- Component change tracking and replay logging

## Running the Benchmarks

```bash
cd benchmarks
cargo run
```

## Benchmark Results

### Performance Impact Summary

| Metric | Without Replay | With Replay | Overhead |
|--------|----------------|-------------|----------|
| Average Time (500 iterations) | 33.431µs | 1.380287ms | **4028.76%** |
| Slowdown Factor | 1x | **41.29x** | - |
| Per-frame Overhead | - | 2.693µs | - |

### Analysis

The benchmark reveals that replay recording introduces significant performance overhead:

1. **High Overhead**: ~40x slowdown when replay recording is enabled
2. **Per-frame Cost**: Each frame incurs approximately 2.7µs of additional overhead
3. **Memory Usage**: Additional allocations for logging and serialization
4. **I/O Impact**: Periodic flushing to disk adds latency

### Recommendations

Based on these results:

- ✅ **Development/Testing**: Replay recording is valuable for debugging and should be used during development
- ⚠️ **Production**: Consider disabling replay recording in production or implementing an optimized version
- 🔧 **Optimization**: Potential improvements could include:
  - Batch logging to reduce I/O operations
  - Compression of replay data
  - Asynchronous logging to background threads
  - Delta compression for component changes

## Implementation Details

The benchmark simulates:
- Component changes (Position, Target, WaitTimer)
- System execution logging
- Replay log serialization and flushing
- Realistic game update patterns

This provides an accurate representation of the performance impact of the replay recording system in the actual ECS framework.