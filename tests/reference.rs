use par_streamlines::{
    Config, CurveSpine, Error, Point, PrimitiveTopology, Spine, Tessellator, UMode, Viewport,
};

fn assert_point(actual: Point, expected: Point) {
    let epsilon = 1.0e-5;
    assert!((actual.x - expected.x).abs() <= epsilon, "x: {actual:?}");
    assert!((actual.y - expected.y).abs() <= epsilon, "y: {actual:?}");
}

#[test]
fn straight_segment_matches_reference_layout() {
    let points = [Point::new(0.0, 0.0), Point::new(2.0, 0.0)];
    let tessellator = Tessellator::new(Config {
        thickness: 2.0,
        ..Config::default()
    })
    .unwrap();
    let mesh = tessellator.lines(&[Spine::open(&points)]).unwrap();

    assert_eq!(mesh.topology, PrimitiveTopology::TriangleList);
    assert_eq!(mesh.triangle_count(), 2);
    assert_eq!(mesh.indices, [0, 1, 2, 2, 1, 3]);
    assert_point(mesh.positions[0], Point::new(0.0, 1.0));
    assert_point(mesh.positions[1], Point::new(0.0, -1.0));
    assert_point(mesh.positions[2], Point::new(2.0, 1.0));
    assert_point(mesh.positions[3], Point::new(2.0, -1.0));
}

#[test]
fn normalized_annotations_run_from_zero_to_one() {
    let points = [
        Point::new(0.0, 0.0),
        Point::new(3.0, 0.0),
        Point::new(3.0, 4.0),
    ];
    let tessellator = Tessellator::new(Config {
        generate_annotations: true,
        generate_spine_lengths: true,
        u_mode: UMode::NormalizedDistance,
        ..Config::default()
    })
    .unwrap();
    let mesh = tessellator.lines(&[Spine::open(&points)]).unwrap();
    let annotations = mesh.annotations.unwrap();
    let lengths = mesh.spine_lengths.unwrap();

    assert!((annotations[0].u_along_curve - 0.0).abs() < 1.0e-6);
    assert!((annotations[2].u_along_curve - 3.0 / 7.0).abs() < 1.0e-6);
    assert!((annotations[4].u_along_curve - 1.0).abs() < 1.0e-6);
    assert!(lengths.iter().all(|length| (*length - 7.0).abs() < 1.0e-6));
}

#[test]
fn closed_triangle_adds_closing_pair_and_segment() {
    let points = [
        Point::new(0.0, 0.0),
        Point::new(1.0, 0.0),
        Point::new(0.0, 1.0),
    ];
    let mesh = Tessellator::default()
        .lines(&[Spine::closed(&points)])
        .unwrap();

    assert_eq!(mesh.vertex_count(), 8);
    assert_eq!(mesh.triangle_count(), 6);
    assert_point(mesh.positions[0], mesh.positions[6]);
    assert_point(mesh.positions[1], mesh.positions[7]);
}

#[test]
fn wireframe_emits_four_indices_per_triangle() {
    let points = [Point::new(0.0, 0.0), Point::new(1.0, 0.0)];
    let tessellator = Tessellator::new(Config {
        wireframe: true,
        ..Config::default()
    })
    .unwrap();
    let mesh = tessellator.lines(&[Spine::open(&points)]).unwrap();

    assert_eq!(mesh.topology, PrimitiveTopology::TriangleOutlines);
    assert_eq!(mesh.indices, [0, 1, 2, 0, 2, 1, 3, 2]);
}

#[test]
fn quadratic_and_cubic_end_at_last_endpoint() {
    let quadratic = [
        Point::new(0.0, 0.0),
        Point::new(1.0, 2.0),
        Point::new(2.0, 0.0),
    ];
    let cubic = [
        Point::new(0.0, 0.0),
        Point::new(0.0, 2.0),
        Point::new(2.0, 2.0),
        Point::new(2.0, 0.0),
    ];
    let tessellator = Tessellator::new(Config {
        thickness: 0.1,
        curves_max_flatness: 0.01,
        ..Config::default()
    })
    .unwrap();

    let qmesh = tessellator
        .quadratic_curves(&[CurveSpine::new(&quadratic)])
        .unwrap();
    let cmesh = tessellator
        .cubic_curves(&[CurveSpine::new(&cubic)])
        .unwrap();

    assert!(qmesh.vertex_count() > 4);
    assert!(cmesh.vertex_count() > 4);
    let qlast = qmesh.positions[qmesh.positions.len() - 2];
    let clast = cmesh.positions[cmesh.positions.len() - 2];
    assert!((qlast.x - 2.0).abs() < 0.2);
    assert!((clast.x - 2.0).abs() < 0.2);
}

#[test]
fn curve_validation_is_explicit() {
    let invalid = [Point::new(0.0, 0.0); 5];
    let error = Tessellator::default()
        .cubic_curves(&[CurveSpine::new(&invalid)])
        .unwrap_err();
    assert_eq!(
        error,
        Error::InvalidCubicSpine {
            spine: 0,
            len: 5
        }
    );
}

#[test]
fn streamline_generation_is_deterministic() {
    let config = Config {
        thickness: 0.01,
        streamlines_seed_spacing: 0.3,
        streamlines_seed_viewport: Viewport::new(-1.0, -1.0, 1.0, 1.0),
        random_seed: 42,
        ..Config::default()
    };
    let tessellator = Tessellator::new(config).unwrap();
    let generate = || {
        tessellator
            .streamlines(0, 4, |point| {
                point.x += 0.01;
                point.y += 0.02;
            })
            .unwrap()
    };

    let first = generate();
    let second = generate();
    assert_eq!(first, second);
    assert!(!first.positions.is_empty());
}

#[test]
fn random_offsets_have_one_value_per_vertex() {
    let first = [Point::new(0.0, 0.0), Point::new(1.0, 0.0)];
    let second = [Point::new(0.0, 1.0), Point::new(1.0, 1.0)];
    let tessellator = Tessellator::new(Config {
        generate_random_offsets: true,
        random_seed: 7,
        ..Config::default()
    })
    .unwrap();
    let mesh = tessellator
        .lines(&[Spine::open(&first), Spine::open(&second)])
        .unwrap();
    let offsets = mesh.random_offsets.unwrap();

    assert_eq!(offsets.len(), mesh.positions.len());
    assert!(offsets[..4].iter().all(|value| *value == offsets[0]));
    assert!(offsets[4..].iter().all(|value| *value == offsets[4]));
    assert_ne!(offsets[0], offsets[4]);
}

#[test]
fn every_u_mode_matches_its_definition() {
    let points = [
        Point::new(0.0, 0.0),
        Point::new(3.0, 0.0),
        Point::new(3.0, 4.0),
    ];

    let values = |mode| {
        Tessellator::new(Config {
            generate_annotations: true,
            u_mode: mode,
            ..Config::default()
        })
        .unwrap()
        .lines(&[Spine::open(&points)])
        .unwrap()
        .annotations
        .unwrap()
        .into_iter()
        .step_by(2)
        .map(|annotation| annotation.u_along_curve)
        .collect::<Vec<_>>()
    };

    assert_eq!(values(UMode::Distance), [0.0, 3.0, 7.0]);
    assert_eq!(values(UMode::SegmentIndex), [0.0, 1.0, 2.0]);
    assert_eq!(
        values(UMode::SegmentFraction),
        [0.0, 1.0 / 3.0, 2.0 / 3.0]
    );
}

#[test]
fn curve_guides_add_thin_geometry() {
    let cubic = [
        Point::new(0.0, 0.0),
        Point::new(0.0, 2.0),
        Point::new(2.0, 2.0),
        Point::new(2.0, 0.0),
    ];
    let without_guides = Tessellator::new(Config {
        thickness: 0.1,
        curves_max_flatness: 0.1,
        ..Config::default()
    })
    .unwrap()
    .cubic_curves(&[CurveSpine::new(&cubic)])
    .unwrap();
    let with_guides = Tessellator::new(Config {
        thickness: 0.1,
        curves_max_flatness: 0.1,
        generate_curve_guides: true,
        ..Config::default()
    })
    .unwrap()
    .cubic_curves(&[CurveSpine::new(&cubic)])
    .unwrap();

    assert_eq!(with_guides.vertex_count(), without_guides.vertex_count() + 8);
    assert_eq!(
        with_guides.triangle_count(),
        without_guides.triangle_count() + 4
    );
}

#[test]
fn invalid_advection_is_reported() {
    let tessellator = Tessellator::new(Config {
        streamlines_seed_spacing: 2.0,
        ..Config::default()
    })
    .unwrap();
    let error = tessellator
        .streamlines(0, 2, |point| point.x = f32::NAN)
        .unwrap_err();

    assert_eq!(Error::NonFiniteAdvection { seed: 0, tick: 0 }, error);
}
