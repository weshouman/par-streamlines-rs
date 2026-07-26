use par_streamlines::{Config, CurveSpine, Point, Tessellator};

fn main() -> Result<(), par_streamlines::Error> {
    let cubic = [
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
        generate_curve_guides: true,
        ..Config::default()
    })?;
    let mesh = tessellator.cubic_curves(&[CurveSpine::new(&cubic)])?;

    println!("vertices including guides: {}", mesh.vertex_count());
    println!("triangles including guides: {}", mesh.triangle_count());
    Ok(())
}
