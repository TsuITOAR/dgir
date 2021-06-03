use std::time::Instant;

use dgir::{
    album::{Painting, Polygon},
    draw::{self, elements::RulerFactory, Resolution},
    paint::LayerData,
    units, Cell, Lib, MICROMETER, NANOMETER,
};
use units::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    let mut lib = Lib::new("first_lib");
    let mut cell = Cell::new("first_alb");
    let micro = Length::new_absolute::<Micrometer>(1.);
    let layer = LayerData::new(1, 0);
    let circle = draw::elements::Circle::new(
        (micro * 0., micro * 0.),
        micro * 100.,
        Resolution::MinNumber(5000000),
    );
    cell.push(Painting::Polygon(Polygon {
        polygon: layer.color(circle.produce().draw()),
    }));
    println!("time costed:{}ms", start.elapsed().as_millis());
    lib.push(cell);
    let user_unit = MICROMETER;
    let db_unit = NANOMETER;
    lib.to_gds(user_unit, db_unit).save("first_file.gds")?;
    println!("time costed:{}ms", start.elapsed().as_millis());
    Ok(())
}
