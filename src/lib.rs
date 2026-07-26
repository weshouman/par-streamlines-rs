//! Pure Rust port of Philip Rideout's `par_streamlines.h`.
//!
//! The crate converts wide polylines, chained quadratic or cubic Bezier curves,
//! and advected particles into CPU-side mesh buffers. It does not perform any
//! rendering and has no runtime dependencies.
//!
//! # Basic example
//!
//! ```
//! use par_streamlines::{Config, Point, Spine, Tessellator};
//!
//! let points = [
//!     Point::new(0.0, 0.0),
//!     Point::new(2.0, 1.0),
//!     Point::new(4.0, 0.0),
//! ];
//!
//! let tessellator = Tessellator::new(Config {
//!     thickness: 3.0,
//!     ..Config::default()
//! })?;
//! let mesh = tessellator.lines(&[Spine::open(&points)])?;
//!
//! assert_eq!(mesh.triangle_count(), 4);
//! # Ok::<(), par_streamlines::Error>(())
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod curves;
mod math;
mod poisson;
mod tessellator;

pub use math::{Point, Viewport};
pub use tessellator::{
    Annotation, Config, CurveSpine, Error, Mesh, PrimitiveTopology, Spine, Tessellator, UMode,
};
