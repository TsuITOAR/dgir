use std::time::Instant;

use dgir::{
    album::{Painting, Polygon},
    draw::{
        self,
        elements::{Rectangle, RulerFactory},
        Resolution,
    },
    paint::LayerData,
    units::{Angle, Deg},
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
        polygon: circle.produce().draw(),
        color: layer,
    }));
    let mut top_cell = Cell::new("sec_alb");
    top_cell.insert(cell.as_ref());
    let mut rec = Rectangle::<_, Deg<f64>>::from_lens(MICROMETER * 50., MICROMETER * 50.);
    rec.rotate(Deg::<f64>::from_deg(19.));
    top_cell.push(Painting::Polygon(Polygon {
        polygon: rec.produce().draw(),
        color: layer,
    }));
    lib.push(top_cell);
    let user_unit = MICROMETER;
    let db_unit = NANOMETER;
    lib.to_gds(user_unit, db_unit).save("first_file.gds")?;
    println!("time costed:{}ms", start.elapsed().as_millis());
    Ok(())
}
