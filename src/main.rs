use std::time::Instant;

use dgir::{
    album::{Painting, Polygon},
    draw::{self, elements::RulerFactory, Resolution},
    paint::LayerData,
    Cell, Lib, MICROMETER, NANOMETER,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    let mut lib = Lib::new("first_lib");
    let mut cell = Cell::new("first_alb");
    let layer = LayerData::new(1, 0);
    let circle = draw::elements::Circle::new(
        (MICROMETER * 0., MICROMETER * 0.),
        MICROMETER * 100.,
        Resolution::MinNumber(5000),
    );
    cell.push(Painting::Polygon(Polygon {
        polygon: layer.color(circle.produce().draw()),
    }));
    println!("time costed:{}ms", start.elapsed().as_millis());
    let mut top_cell = Cell::new("sec_alb");
    top_cell.insert(cell.as_ref());
    lib.push(top_cell);
    let user_unit = MICROMETER;
    let db_unit = NANOMETER;
    lib.to_gds(user_unit, db_unit).save("first_file.gds")?;
    println!("time costed:{}ms", start.elapsed().as_millis());
    Ok(())
}
