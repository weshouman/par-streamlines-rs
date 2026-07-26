# par-streamlines

A safe, dependency-free Rust port of Philip Rideout's
[`par_streamlines.h`](https://github.com/prideout/par/blob/30f2c72cbf4568401e3175b0755af4cdaa66f692/par_streamlines.h).
It tessellates wide polylines, chained quadratic and cubic Bezier curves, and
vector-field streamlines into owned CPU-side mesh buffers.

The renderer is deliberately outside the crate. The returned positions,
indices, and optional annotations can be uploaded to wgpu, OpenGL, Vulkan,
Bevy, macroquad, or any other graphics API.

## Features

- Open and closed thick polylines with clamped miter joins
- Chained quadratic curves encoded as `P C P [C P]*`
- Smooth chained cubic curves encoded as `P C C P [C P]*`
- Adaptive curve flattening
- Deterministic Poisson-disk seed placement and streamline advection
- Normalized-distance, distance, point-index, and point-fraction U coordinates
- Optional spine lengths, random offsets, and curve control guides
- Triangle-list and triangle-outline index buffers
- Owned results, no unsafe code, and no runtime dependencies

## Basic usage

```rust
use par_streamlines::{Config, Point, Spine, Tessellator};

fn main() -> Result<(), par_streamlines::Error> {
    let points = [
        Point::new(0.0, 0.0),
        Point::new(2.0, 1.0),
        Point::new(4.0, 0.0),
    ];

    let tessellator = Tessellator::new(Config {
        thickness: 3.0,
        generate_annotations: true,
        ..Config::default()
    })?;

    let mesh = tessellator.lines(&[Spine::open(&points)])?;
    println!("{} vertices", mesh.vertex_count());
    println!("{} triangles", mesh.triangle_count());
    Ok(())
}
```

## Cubic curves

The first segment uses four points. Each later segment adds a second control
point and endpoint. Its first control point is reflected from the preceding
segment, matching an SVG `M C S S ...` path.

```rust
use par_streamlines::{Config, CurveSpine, Point, Tessellator};

let points = [
    Point::new(0.0, 0.0),
    Point::new(0.0, 2.0),
    Point::new(2.0, 2.0),
    Point::new(2.0, 0.0),
    Point::new(4.0, -2.0),
    Point::new(4.0, 0.0),
];

let tessellator = Tessellator::new(Config {
    thickness: 0.15,
    curves_max_flatness: 0.02,
    ..Config::default()
})?;
let mesh = tessellator.cubic_curves(&[CurveSpine::new(&points)])?;
# Ok::<(), par_streamlines::Error>(())
```

## Streamlines

```rust
use par_streamlines::{Config, Point, Tessellator, Viewport};

let tessellator = Tessellator::new(Config {
    thickness: 0.015,
    streamlines_seed_spacing: 0.18,
    streamlines_seed_viewport: Viewport::new(-1.0, -1.0, 1.0, 1.0),
    ..Config::default()
})?;

let mesh = tessellator.streamlines(0, 32, |point: &mut Point| {
    let x = point.x;
    point.x += -point.y * 0.025;
    point.y += x * 0.025;
})?;
# Ok::<(), par_streamlines::Error>(())
```

## Development

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo run --example basic
```

The minimum supported Rust version is 1.70.

## Releasing

1. Confirm that the `par-streamlines` crate name is available to the intended
   crates.io account, or rename the package before the first publication.
2. Update `version` in `Cargo.toml` and add a `CHANGELOG.md` entry.
3. Add a repository secret named `CARGO_REGISTRY_TOKEN` when crates.io
   publishing is desired. Without it, the workflow still creates a GitHub
   release and uploads the packaged crate.
4. Commit the release and push a matching tag:

```sh
git tag v0.1.0
git push origin main v0.1.0
```

`.github/workflows/release.yml` verifies that the tag and package version are
identical, runs formatting, Clippy, tests, and `cargo package`, optionally
publishes to crates.io, and creates a GitHub release with the `.crate` archive
and SHA-256 checksum.

## Compatibility notes

The port follows the source project's line layout, miter calculation, curve
encodings, guide geometry, U modes, and Poisson-disk streamline sampling
strategy. Rust-facing differences are intentional:

- Each returned mesh owns its buffers instead of borrowing an arena.
- Closure is selected per spine instead of once for an entire spine list.
- Random offsets are deterministic and correctly contain one value per vertex.
- Invalid data returns `Error` instead of relying on C assertions.
- The recursion limit always emits the final endpoint, keeping the mesh connected.

## License and attribution

MIT licensed. The implementation is derived from `par_streamlines.h`,
Copyright (c) 2019 Philip Rideout. See `NOTICE` and `LICENSE`.
