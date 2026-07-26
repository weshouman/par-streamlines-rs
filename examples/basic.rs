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
        generate_spine_lengths: true,
        ..Config::default()
    })?;
    let mesh = tessellator.lines(&[Spine::open(&points)])?;

    println!("vertices: {}", mesh.vertex_count());
    println!("triangles: {}", mesh.triangle_count());
    println!("indices: {:?}", mesh.indices);
    Ok(())
}
