use std::time::Instant;

use dgir::{
    color::LayerData,
    draw::{curve::IntoCurve, CircularArc, Resolution},
    gds::DgirLibrary,
    units::Angle,
    zero, MICROMETER,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    for _ in 1..100 {
        let mut lib = DgirLibrary::new("test");
        let cir = CircularArc::new(
            100. * MICROMETER,
            (zero(), zero()),
            (Angle::from_deg(0.), Angle::from_deg(360.)),
            Resolution::MinNumber(5000),
        );
        lib.push(
            cir.into_curve()
                .to_path(LayerData::new(1, 1))
                .to_cell("cell_name"),
        );
        lib.save("test.gds").unwrap();
    }
    println!("time costed:{}ms", start.elapsed().as_millis());
    Ok(())
}
