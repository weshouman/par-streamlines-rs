use std::fmt;

use crate::curves::{flatten_cubic, flatten_quadratic};
use crate::poisson::{generate as generate_poisson, random_for_index};
use crate::{Point, Viewport};

/// Controls the longitudinal coordinate stored in [`Annotation::u_along_curve`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum UMode {
    /// Distance divided by total spine length. This is the default.
    #[default]
    NormalizedDistance,
    /// Distance in the same units as input coordinates.
    Distance,
    /// Integer point index represented as `f32`.
    SegmentIndex,
    /// Point index divided by the number of source points.
    SegmentFraction,
}

/// Optional per-vertex data generated alongside positions.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Annotation {
    /// Longitudinal coordinate selected by [`UMode`].
    pub u_along_curve: f32,
    /// Positive one for the left edge and negative one for the right edge.
    pub v_across_curve: f32,
    /// Vector from the source spine point to this generated edge vertex.
    pub spine_to_edge: Point,
}

/// Index interpretation used by a generated [`Mesh`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrimitiveTopology {
    /// Every three indices form one triangle.
    TriangleList,
    /// Every four indices form a closed three-edge outline of one triangle.
    TriangleOutlines,
}

/// Owned CPU-side geometry produced by the tessellator.
#[derive(Clone, Debug, PartialEq)]
pub struct Mesh {
    /// Generated edge vertices.
    pub positions: Vec<Point>,
    /// Triangle indices or closed triangle-outline indices, depending on [`Mesh::topology`].
    pub indices: Vec<u32>,
    /// Optional per-vertex annotations.
    pub annotations: Option<Vec<Annotation>>,
    /// Optional total source-spine length repeated for every generated vertex.
    pub spine_lengths: Option<Vec<f32>>,
    /// Optional deterministic random value repeated for every vertex of a spine.
    pub random_offsets: Option<Vec<f32>>,
    /// How to interpret [`Mesh::indices`].
    pub topology: PrimitiveTopology,
    triangle_count: usize,
}

impl Mesh {
    /// Returns the number of logical triangles represented by this mesh.
    pub const fn triangle_count(&self) -> usize {
        self.triangle_count
    }

    /// Returns the number of generated vertices.
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    /// Returns the number of indices used for one logical triangle.
    pub const fn indices_per_triangle(&self) -> usize {
        match self.topology {
            PrimitiveTopology::TriangleList => 3,
            PrimitiveTopology::TriangleOutlines => 4,
        }
    }
}

/// Immutable tessellation settings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Config {
    /// Width of normal line and curve geometry.
    pub thickness: f32,
    /// Emits four indices per triangle to describe a closed outline.
    pub wireframe: bool,
    /// Generates [`Mesh::annotations`].
    pub generate_annotations: bool,
    /// Generates [`Mesh::spine_lengths`].
    pub generate_spine_lengths: bool,
    /// Generates [`Mesh::random_offsets`].
    pub generate_random_offsets: bool,
    /// Includes one-unit-wide control guides when tessellating curves.
    pub generate_curve_guides: bool,
    /// Selects annotation U-coordinate behavior.
    pub u_mode: UMode,
    /// Maximum curve flatness error. Zero selects the upstream default of one.
    pub curves_max_flatness: f32,
    /// Minimum distance between generated streamline seeds.
    pub streamlines_seed_spacing: f32,
    /// Region in which streamline seeds are generated.
    pub streamlines_seed_viewport: Viewport,
    /// Maximum miter extent. Zero selects twice [`Config::thickness`].
    pub miter_limit: f32,
    /// Seed used for deterministic seed placement and random offsets.
    pub random_seed: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            thickness: 1.0,
            wireframe: false,
            generate_annotations: false,
            generate_spine_lengths: false,
            generate_random_offsets: false,
            generate_curve_guides: false,
            u_mode: UMode::NormalizedDistance,
            curves_max_flatness: 1.0,
            streamlines_seed_spacing: 0.1,
            streamlines_seed_viewport: Viewport::default(),
            miter_limit: 0.0,
            random_seed: 42,
        }
    }
}

/// Borrowed polyline input.
#[derive(Clone, Copy, Debug)]
pub struct Spine<'a> {
    /// Source points in path order.
    pub points: &'a [Point],
    /// Whether the last point connects back to the first.
    pub closed: bool,
}

impl<'a> Spine<'a> {
    /// Creates an open spine.
    pub const fn open(points: &'a [Point]) -> Self {
        Self {
            points,
            closed: false,
        }
    }

    /// Creates a closed spine.
    pub const fn closed(points: &'a [Point]) -> Self {
        Self {
            points,
            closed: true,
        }
    }
}

/// Borrowed chained Bezier input.
#[derive(Clone, Copy, Debug)]
pub struct CurveSpine<'a> {
    /// Encoded control and endpoint sequence.
    pub points: &'a [Point],
}

impl<'a> CurveSpine<'a> {
    /// Creates a curve spine from its encoded points.
    pub const fn new(points: &'a [Point]) -> Self {
        Self { points }
    }
}

/// Error returned for malformed input or settings.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// Thickness must be finite and greater than zero.
    InvalidThickness,
    /// Miter limit must be finite and non-negative.
    InvalidMiterLimit,
    /// Curve flatness must be finite and non-negative.
    InvalidCurveFlatness,
    /// A polyline contains fewer than two points.
    SpineTooShort {
        /// Zero-based spine index.
        spine: usize,
        /// Number of supplied points.
        len: usize,
    },
    /// A closed polyline contains fewer than three points.
    ClosedSpineTooShort {
        /// Zero-based spine index.
        spine: usize,
        /// Number of supplied points.
        len: usize,
    },
    /// A cubic chain is not `P C C P [C P]*`.
    InvalidCubicSpine {
        /// Zero-based spine index.
        spine: usize,
        /// Number of supplied points.
        len: usize,
    },
    /// A quadratic chain is not `P C P [C P]*`.
    InvalidQuadraticSpine {
        /// Zero-based spine index.
        spine: usize,
        /// Number of supplied points.
        len: usize,
    },
    /// An input coordinate is NaN or infinite.
    NonFinitePoint {
        /// Zero-based spine index.
        spine: usize,
        /// Zero-based point index.
        point: usize,
    },
    /// The configured seed viewport is invalid.
    InvalidViewport,
    /// Seed spacing must be finite and greater than zero.
    InvalidSeedSpacing,
    /// At least two generated points are required per streamline.
    InvalidTickCount,
    /// An advection callback produced NaN or infinity.
    NonFiniteAdvection {
        /// Zero-based seed index.
        seed: usize,
        /// Zero-based integration tick after the skipped prefix.
        tick: usize,
    },
    /// The Poisson acceleration grid would be unreasonably large.
    SamplingGridTooLarge,
    /// The generated mesh cannot be indexed with `u32`.
    IndexOverflow,
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::InvalidThickness => write!(formatter, "thickness must be finite and positive"),
            Self::InvalidMiterLimit => {
                write!(formatter, "miter limit must be finite and non-negative")
            }
            Self::InvalidCurveFlatness => {
                write!(formatter, "curve flatness must be finite and non-negative")
            }
            Self::SpineTooShort { spine, len } => {
                write!(formatter, "spine {spine} has {len} points; at least 2 are required")
            }
            Self::ClosedSpineTooShort { spine, len } => write!(
                formatter,
                "closed spine {spine} has {len} points; at least 3 are required"
            ),
            Self::InvalidCubicSpine { spine, len } => write!(
                formatter,
                "cubic spine {spine} has invalid length {len}; expected 4 + 2n points"
            ),
            Self::InvalidQuadraticSpine { spine, len } => write!(
                formatter,
                "quadratic spine {spine} has invalid length {len}; expected 3 + 2n points"
            ),
            Self::NonFinitePoint { spine, point } => {
                write!(formatter, "point {point} in spine {spine} is not finite")
            }
            Self::InvalidViewport => write!(formatter, "streamline seed viewport is invalid"),
            Self::InvalidSeedSpacing => {
                write!(formatter, "streamline seed spacing must be finite and positive")
            }
            Self::InvalidTickCount => write!(formatter, "streamlines require at least two ticks"),
            Self::NonFiniteAdvection { seed, tick } => write!(
                formatter,
                "advection produced a non-finite point for seed {seed} at tick {tick}"
            ),
            Self::SamplingGridTooLarge => {
                write!(formatter, "streamline sampling grid is too large")
            }
            Self::IndexOverflow => write!(formatter, "generated mesh exceeds u32 index capacity"),
        }
    }
}

impl std::error::Error for Error {}

/// Reusable object holding immutable configuration.
#[derive(Clone, Debug, Default)]
pub struct Tessellator {
    config: Config,
}

impl Tessellator {
    /// Validates configuration and creates a tessellator.
    pub fn new(config: Config) -> Result<Self, Error> {
        if !config.thickness.is_finite() || config.thickness <= 0.0 {
            return Err(Error::InvalidThickness);
        }
        if !config.miter_limit.is_finite() || config.miter_limit < 0.0 {
            return Err(Error::InvalidMiterLimit);
        }
        if !config.curves_max_flatness.is_finite() || config.curves_max_flatness < 0.0 {
            return Err(Error::InvalidCurveFlatness);
        }
        Ok(Self { config })
    }

    /// Returns the immutable configuration.
    pub const fn config(&self) -> &Config {
        &self.config
    }

    /// Tessellates one or more open or closed polylines.
    pub fn lines(&self, spines: &[Spine<'_>]) -> Result<Mesh, Error> {
        let views: Vec<_> = spines
            .iter()
            .map(|spine| Polyline {
                points: spine.points,
                closed: spine.closed,
                thin: false,
            })
            .collect();
        self.tessellate_polylines(&views)
    }

    /// Tessellates chained cubic Bezier curves encoded as `P C C P [C P]*`.
    ///
    /// For each segment after the first, the first control point is reflected
    /// from the preceding segment, matching SVG's smooth cubic `S` command.
    pub fn cubic_curves(&self, spines: &[CurveSpine<'_>]) -> Result<Mesh, Error> {
        let mut main_spines = Vec::with_capacity(spines.len());
        let mut guide_spines = Vec::new();
        let flatness = self.effective_flatness();
        let flatness_squared = flatness * flatness;

        for (spine_index, spine) in spines.iter().enumerate() {
            validate_points(spine_index, spine.points)?;
            let len = spine.points.len();
            if len < 4 || len % 2 != 0 {
                return Err(Error::InvalidCubicSpine {
                    spine: spine_index,
                    len,
                });
            }

            let points = spine.points;
            let mut flattened = Vec::new();
            flattened.push(points[0]);
            flatten_cubic(
                &mut flattened,
                points[0],
                points[1],
                points[2],
                points[3],
                flatness_squared,
                0,
            );

            if self.config.generate_curve_guides {
                guide_spines.push(OwnedPolyline::thin(vec![points[0], points[1]]));
                guide_spines.push(OwnedPolyline::thin(vec![points[3], points[2]]));
            }

            let mut endpoint = points[3];
            let mut previous_second_control = points[2];
            let mut index = 4;
            while index < len {
                let first_control = endpoint + (endpoint - previous_second_control);
                let second_control = points[index];
                let next_endpoint = points[index + 1];
                flatten_cubic(
                    &mut flattened,
                    endpoint,
                    first_control,
                    second_control,
                    next_endpoint,
                    flatness_squared,
                    0,
                );

                if self.config.generate_curve_guides {
                    guide_spines.push(OwnedPolyline::thin(vec![endpoint, first_control]));
                    guide_spines.push(OwnedPolyline::thin(vec![next_endpoint, second_control]));
                }

                endpoint = next_endpoint;
                previous_second_control = second_control;
                index += 2;
            }

            main_spines.push(OwnedPolyline::normal(flattened));
        }

        main_spines.extend(guide_spines);
        self.tessellate_owned(&main_spines)
    }

    /// Tessellates chained quadratic Bezier curves encoded as `P C P [C P]*`.
    pub fn quadratic_curves(&self, spines: &[CurveSpine<'_>]) -> Result<Mesh, Error> {
        let mut main_spines = Vec::with_capacity(spines.len());
        let mut guide_spines = Vec::new();
        let flatness = self.effective_flatness();
        let flatness_squared = flatness * flatness;

        for (spine_index, spine) in spines.iter().enumerate() {
            validate_points(spine_index, spine.points)?;
            let len = spine.points.len();
            if len < 3 || len % 2 == 0 {
                return Err(Error::InvalidQuadraticSpine {
                    spine: spine_index,
                    len,
                });
            }

            let points = spine.points;
            let mut flattened = Vec::new();
            flattened.push(points[0]);
            flatten_quadratic(
                &mut flattened,
                points[0],
                points[1],
                points[2],
                flatness_squared,
                0,
            );

            let mut endpoint = points[2];
            let mut index = 3;
            while index < len {
                let control = points[index];
                let next_endpoint = points[index + 1];
                flatten_quadratic(
                    &mut flattened,
                    endpoint,
                    control,
                    next_endpoint,
                    flatness_squared,
                    0,
                );
                endpoint = next_endpoint;
                index += 2;
            }

            main_spines.push(OwnedPolyline::normal(flattened));
            if self.config.generate_curve_guides {
                guide_spines.push(OwnedPolyline::thin(points.to_vec()));
            }
        }

        main_spines.extend(guide_spines);
        self.tessellate_owned(&main_spines)
    }

    /// Generates Poisson-distributed seeds, advects each particle, and tessellates its path.
    ///
    /// The callback mutates one point by exactly one integration step. As in the
    /// C library, `first_tick` steps are discarded, then every retained point is
    /// captured after an additional callback invocation.
    pub fn streamlines<F>(
        &self,
        first_tick: usize,
        num_ticks: usize,
        mut advect: F,
    ) -> Result<Mesh, Error>
    where
        F: FnMut(&mut Point),
    {
        if num_ticks < 2 {
            return Err(Error::InvalidTickCount);
        }
        let viewport = self.config.streamlines_seed_viewport;
        if !viewport.is_finite() || viewport.width() <= 0.0 || viewport.height() <= 0.0 {
            return Err(Error::InvalidViewport);
        }
        let spacing = self.config.streamlines_seed_spacing;
        if !spacing.is_finite() || spacing <= 0.0 {
            return Err(Error::InvalidSeedSpacing);
        }

        let seeds = generate_poisson(viewport, spacing, self.config.random_seed)?;
        let mut owned = Vec::with_capacity(seeds.len());
        for (seed_index, seed) in seeds.into_iter().enumerate() {
            let mut point = seed;
            for _ in 0..first_tick {
                advect(&mut point);
                if !point.is_finite() {
                    return Err(Error::NonFiniteAdvection {
                        seed: seed_index,
                        tick: 0,
                    });
                }
            }

            let mut path = Vec::with_capacity(num_ticks);
            for tick in 0..num_ticks {
                advect(&mut point);
                if !point.is_finite() {
                    return Err(Error::NonFiniteAdvection {
                        seed: seed_index,
                        tick,
                    });
                }
                path.push(point);
            }
            owned.push(OwnedPolyline::normal(path));
        }

        self.tessellate_owned(&owned)
    }

    fn effective_flatness(&self) -> f32 {
        if self.config.curves_max_flatness == 0.0 {
            1.0
        } else {
            self.config.curves_max_flatness
        }
    }

    fn tessellate_owned(&self, spines: &[OwnedPolyline]) -> Result<Mesh, Error> {
        let views: Vec<_> = spines
            .iter()
            .map(|spine| Polyline {
                points: &spine.points,
                closed: spine.closed,
                thin: spine.thin,
            })
            .collect();
        self.tessellate_polylines(&views)
    }

    fn tessellate_polylines(&self, spines: &[Polyline<'_>]) -> Result<Mesh, Error> {
        let topology = if self.config.wireframe {
            PrimitiveTopology::TriangleOutlines
        } else {
            PrimitiveTopology::TriangleList
        };
        let mut mesh = Mesh {
            positions: Vec::new(),
            indices: Vec::new(),
            annotations: self.config.generate_annotations.then(Vec::new),
            spine_lengths: self.config.generate_spine_lengths.then(Vec::new),
            random_offsets: self.config.generate_random_offsets.then(Vec::new),
            topology,
            triangle_count: 0,
        };

        for (spine_index, spine) in spines.iter().enumerate() {
            validate_points(spine_index, spine.points)?;
            if spine.points.len() < 2 {
                return Err(Error::SpineTooShort {
                    spine: spine_index,
                    len: spine.points.len(),
                });
            }
            if spine.closed && spine.points.len() < 3 {
                return Err(Error::ClosedSpineTooShort {
                    spine: spine_index,
                    len: spine.points.len(),
                });
            }

            self.append_spine(&mut mesh, spine_index, *spine)?;
        }

        debug_assert_eq!(
            mesh.indices.len(),
            mesh.triangle_count * mesh.indices_per_triangle()
        );
        if let Some(annotations) = &mesh.annotations {
            debug_assert_eq!(annotations.len(), mesh.positions.len());
        }
        if let Some(lengths) = &mesh.spine_lengths {
            debug_assert_eq!(lengths.len(), mesh.positions.len());
        }
        if let Some(offsets) = &mesh.random_offsets {
            debug_assert_eq!(offsets.len(), mesh.positions.len());
        }

        Ok(mesh)
    }

    fn append_spine(
        &self,
        mesh: &mut Mesh,
        spine_index: usize,
        spine: Polyline<'_>,
    ) -> Result<(), Error> {
        let points = spine.points;
        let source_count = points.len();
        let emitted_count = source_count + if spine.closed { 1 } else { 0 };
        let thickness = if spine.thin { 1.0 } else { self.config.thickness };
        let half_thickness = thickness * 0.5;
        let miter_limit = if self.config.miter_limit == 0.0 {
            self.config.thickness * 2.0
        } else {
            self.config.miter_limit
        };

        let mut offsets = Vec::with_capacity(emitted_count);
        if spine.closed {
            for index in 0..source_count {
                let previous = points[(index + source_count - 1) % source_count];
                let current = points[index];
                let next = points[(index + 1) % source_count];
                let previous_normal = segment_normal(previous, current).0;
                let next_normal = segment_normal(current, next).0;
                offsets.push(miter_offset(
                    previous_normal,
                    next_normal,
                    half_thickness,
                    miter_limit,
                ));
            }
            offsets.push(offsets[0]);
        } else {
            let first_normal = segment_normal(points[0], points[1]).0;
            offsets.push(first_normal * half_thickness);
            for index in 1..source_count - 1 {
                let previous_normal = segment_normal(points[index - 1], points[index]).0;
                let next_normal = segment_normal(points[index], points[index + 1]).0;
                offsets.push(miter_offset(
                    previous_normal,
                    next_normal,
                    half_thickness,
                    miter_limit,
                ));
            }
            let last_normal = segment_normal(points[source_count - 2], points[source_count - 1]).0;
            offsets.push(last_normal * half_thickness);
        }

        let mut cumulative = Vec::with_capacity(emitted_count);
        cumulative.push(0.0);
        let mut total_length = 0.0;
        for index in 1..source_count {
            total_length += (points[index] - points[index - 1]).length();
            cumulative.push(total_length);
        }
        if spine.closed {
            total_length += (points[0] - points[source_count - 1]).length();
            cumulative.push(total_length);
        }

        let base = u32::try_from(mesh.positions.len()).map_err(|_| Error::IndexOverflow)?;
        let random_offset = random_for_index(self.config.random_seed, spine_index);
        for index in 0..emitted_count {
            let center = if index == source_count {
                points[0]
            } else {
                points[index]
            };
            let offset = offsets[index];
            mesh.positions.push(center + offset);
            mesh.positions.push(center - offset);

            if let Some(annotations) = &mut mesh.annotations {
                let u = annotation_u(
                    self.config.u_mode,
                    cumulative[index],
                    total_length,
                    index,
                    source_count,
                );
                annotations.push(Annotation {
                    u_along_curve: u,
                    v_across_curve: 1.0,
                    spine_to_edge: offset,
                });
                annotations.push(Annotation {
                    u_along_curve: u,
                    v_across_curve: -1.0,
                    spine_to_edge: -offset,
                });
            }
            if let Some(lengths) = &mut mesh.spine_lengths {
                lengths.push(total_length);
                lengths.push(total_length);
            }
            if let Some(random_offsets) = &mut mesh.random_offsets {
                random_offsets.push(random_offset);
                random_offsets.push(random_offset);
            }
        }

        let segment_count = emitted_count - 1;
        for segment in 0..segment_count {
            let relative = u32::try_from(segment.checked_mul(2).ok_or(Error::IndexOverflow)?)
                .map_err(|_| Error::IndexOverflow)?;
            let a = base.checked_add(relative).ok_or(Error::IndexOverflow)?;
            let b = a.checked_add(1).ok_or(Error::IndexOverflow)?;
            let c = a.checked_add(2).ok_or(Error::IndexOverflow)?;
            let d = a.checked_add(3).ok_or(Error::IndexOverflow)?;

            if self.config.wireframe {
                mesh.indices.extend_from_slice(&[a, b, c, a, c, b, d, c]);
            } else {
                mesh.indices.extend_from_slice(&[a, b, c, c, b, d]);
            }
            mesh.triangle_count += 2;
        }

        Ok(())
    }
}

#[derive(Clone, Copy)]
struct Polyline<'a> {
    points: &'a [Point],
    closed: bool,
    thin: bool,
}

struct OwnedPolyline {
    points: Vec<Point>,
    closed: bool,
    thin: bool,
}

impl OwnedPolyline {
    fn normal(points: Vec<Point>) -> Self {
        Self {
            points,
            closed: false,
            thin: false,
        }
    }

    fn thin(points: Vec<Point>) -> Self {
        Self {
            points,
            closed: false,
            thin: true,
        }
    }
}

fn validate_points(spine: usize, points: &[Point]) -> Result<(), Error> {
    for (point, value) in points.iter().enumerate() {
        if !value.is_finite() {
            return Err(Error::NonFinitePoint { spine, point });
        }
    }
    Ok(())
}

fn segment_normal(start: Point, end: Point) -> (Point, f32) {
    let delta = end - start;
    let length = delta.length();
    if length == 0.0 {
        (Point::default(), 0.0)
    } else {
        (Point::new(-delta.y / length, delta.x / length), length)
    }
}

fn miter_offset(
    previous_normal: Point,
    next_normal: Point,
    half_thickness: f32,
    miter_limit: f32,
) -> Point {
    let dot = previous_normal.dot(next_normal);
    let mut extent = half_thickness;
    if (-1.0..1.0).contains(&dot) {
        let phi = dot.acos() * 0.5;
        let theta = std::f32::consts::FRAC_PI_2 - phi;
        let sine = theta.sin();
        if sine != 0.0 {
            extent = (half_thickness / sine).clamp(-miter_limit, miter_limit);
        }
    }
    (previous_normal + next_normal).normalized_or_zero() * extent
}

fn annotation_u(
    mode: UMode,
    distance: f32,
    total_length: f32,
    index: usize,
    source_count: usize,
) -> f32 {
    match mode {
        UMode::Distance => distance,
        UMode::NormalizedDistance => {
            if total_length == 0.0 {
                0.0
            } else {
                distance / total_length
            }
        }
        UMode::SegmentIndex => index as f32,
        UMode::SegmentFraction => index as f32 / source_count as f32,
    }
}
