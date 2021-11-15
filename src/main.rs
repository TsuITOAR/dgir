use std::time::Instant;

use dgir::{
    color::LayerData,
    draw::{curve::Sweep, CircularArc, Resolution},
    gds::DgirLibrary,
    units::Angle,
    zero, MICROMETER,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    for _ in 1..100 {
        let mut lib = DgirLibrary::new("libname");
        let cir = CircularArc::new(
            100. * MICROMETER,
            (zero(), zero()),
            (Angle::from_deg(0.), Angle::from_deg(360.)),
            Resolution::MinNumber(2000),
        )
        .sweep((-MICROMETER, MICROMETER));
        lib.push(cir.to_polygon(LayerData::new(1, 1)).to_cell("cell_name"));
        lib.save("test.gds").unwrap();
    }
    println!("time costed:{}ms", start.elapsed().as_millis());
    Ok(())
}
