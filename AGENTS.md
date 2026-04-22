# AGENTS.md - CREBAIN Development Guide

## Build Commands

```bash
# Development
cargo run                        # Run the Bevy app
cargo run --release              # Release build and run

# Type checking
cargo check --workspace          # Type check all crates
cargo check -p crebain-core      # Type check core only
cargo check -p crebain-app       # Type check app only

# Linting
cargo clippy --workspace         # Lint all crates
cargo clippy -- -D warnings      # Lint with warnings as errors

# Testing
cargo test --workspace           # Run all tests
cargo test -p crebain-core       # Run core tests only
cargo test -- --nocapture         # Run tests with stdout

# Build
cargo build --workspace          # Debug build
cargo build --release --workspace # Release build
```

## Code Style

### Rust
- Run `cargo clippy` before committing
- Use `log::info/warn/error` instead of `println!`
- Validate all external inputs (paths, user data)
- Use `spawn_blocking` for CPU-intensive operations in async contexts
- Use functional components with Bevy ECS (systems, resources, events)
- Use `ResMut` only when mutation is needed; prefer `Res` for read-only
- Derive `Resource` for app state, `Component` for entity data

## Architecture Notes

### Core (`crates/crebain-core/`)
- `common/` - Detection types, NMS, YOLO helpers, COCO labels, error types, path validation
- `inference/` - ML abstraction layer (CoreML, ONNX, CUDA, TensorRT, MLX)
- `sensor_fusion.rs` - Kalman/EKF/UKF/Particle/IMM filters
- `transport/` - Zenoh low-latency transport + broadcast channels

### App (`crates/crebain-app/`)
- `app_state/` - CrebainConfig, AppState, RenderQuality
- `camera/` - Tactical camera (WASD+QE controls, zoom)
- `detection/` - DetectionPlugin, DetectionState, detection loop
- `transport/` - TransportPlugin bridging Zenoh → Bevy events
- `ui/hud/` - Status bar, performance panel, sensor fusion panel
- `ui/top_menu/` - Menu bar (File/View/Detection/Help)
- `viewer/` - Tactical grid, terrain, drones, detection overlay

### Native (`native/`)
- `coreml-ffi/` - Swift/CoreML FFI bridge (macOS)

## Performance Guidelines

- Use `CircularBuffer` for high-frequency position data (crebain-app/src/circular_buffer.rs)
- Prefer squared distance comparisons (avoid `sqrt()`)
- Memoize derived state to prevent unnecessary recomputations
- Keep camera feed updates at ~12 FPS (83ms interval) when processing
- Use Bevy change detection (`is_changed()`) to avoid redundant work

## Testing

Tests use Rust's built-in test framework. Place tests in `#[cfg(test)]` modules.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_example() { ... }
}
```