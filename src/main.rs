use std::time::Instant;

use dgir::{
    color::{Color, LayerData, Shader},
    draw::{
        elements::{CircularArc, Rectangle, RulerFactory},
        Resolution,
    },
    units::{Angle, Deg, Rad},
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
        (Rad::from_deg(0.), Rad::from_deg(180.)),
        Resolution::MinNumber(300),
    )
    .produce()
    .draw();
    cell.push(layer.to_brush(MICROMETER).color(arc));
    let mut top_cell = Cell::new("sec_alb");
    top_cell.insert(cell.as_ref());
    let mut rec = Rectangle::<_, Deg<f64>>::from_lens(MICROMETER * 50., MICROMETER * 50.);
    rec.rotate(Deg::<f64>::from_deg(19.));
    top_cell.push(layer.to_filler().color(rec.produce().draw()));
    lib.push(top_cell);
    let user_unit = MICROMETER;
    let db_unit = NANOMETER;
    lib.to_gds(user_unit, db_unit).save("first_file.gds")?;
    println!("time costed:{}ms", start.elapsed().as_millis());
    Ok(())
}
