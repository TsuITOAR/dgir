use std::time::Instant;

use dgir::{
    color::{Color, LayerData, Shader},
    draw::{
        elements::{CircularArc, Offset, Rectangle, RulerFactory},
        Resolution,
    },
    units::Angle,
    Cell, Lib, MICROMETER, NANOMETER,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    let mut lib = Lib::new("first_lib");
    let mut cell = Cell::new("first_alb");
    let layer = LayerData::new(1, 0);
    let arc = CircularArc::new(
        (MICROMETER * 0., MICROMETER * 0.),
        MICROMETER * 100.,
        (Angle::from_deg(0.), Angle::from_deg(360.)),
        Resolution::MinNumber(2001),
    )
    .into_compound((MICROMETER, -MICROMETER))
    .produce()
    .draw();
    cell.push(layer.to_filler().color(arc));
    let mut top_cell = Cell::new("sec_alb");
    top_cell.insert(cell.as_ref());
    let rec = Rectangle::new(MICROMETER * 50., MICROMETER * 50.)
        .produce()
        .rotate(Angle::from_deg(30.))
        .move_evenly(MICROMETER * 10., MICROMETER * 30.);
    top_cell.push(layer.to_filler().color(rec.draw()));
    lib.push(top_cell);
    let user_unit = MICROMETER;
    let db_unit = NANOMETER;
    lib.to_gds(user_unit, db_unit).save("first_file.gds")?;
    println!("time costed:{}ms", start.elapsed().as_millis());
    Ok(())
}
