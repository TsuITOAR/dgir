use std::time::Instant;

use dgir::{
    color::Decorated,
    color::LayerData,
    draw::{
        curve::{groups::Compound, Sweep},
        CircularArc, Line, Resolution,
    },
    gds::{DgirCell, DgirLibrary},
    units::Angle,
    zero, MICROMETER,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    let start = Instant::now();

    let mut lib = DgirLibrary::new("libname");
    let cir = CircularArc::new(
        100. * MICROMETER,
        (zero(), zero()),
        (Angle::from_deg(0.), Angle::from_deg(360.)),
        Resolution::MinNumber(8009),
    );
    let c: Compound<_, _> = (
        cir.sweep((-MICROMETER, MICROMETER)),
        cir.sweep((-MICROMETER * 2., MICROMETER * 2.)),
    )
        .into();
    let rec = Line::new((zero(), zero()), (MICROMETER * 100., MICROMETER * 100.))
        .sweep((zero(), MICROMETER * 5.));

    let mut topcell = DgirCell::new("top_cell");
    topcell
        .push(c.color(Compound::from((LayerData::new(1, 1), LayerData::new(1, 0)))))
        .push(rec.to_polygon(LayerData::new(1, 2)));

    lib.push(topcell);
    lib.save("test.gds").unwrap();

    println!("time costed:{}ms", start.elapsed().as_millis());
    Ok(())
}
