use std::time::Instant;

use dgir::{
    album::{Album, Painting, Polygon},
    draw::{self, elements::RulerFactory, Resolution},
    paint::LayerData,
    units, Alb, Lib,
};
use units::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    let mut lib = Lib::new("first_lib");
    let mut album = Alb::new("first_alb");
    let micro = Length::<Micrometer>::new(1.);
    let layer = LayerData::new(1, 0);
    let circle = draw::elements::Circle::new(
        (micro * 0., micro * 0.),
        micro * 100.,
        Resolution::MinNumber(5000000),
    );
    album.push(Painting::Polygon(Polygon {
        polygon: layer.color(circle.produce().draw()),
    }));
    println!("time costed:{}ms", start.elapsed().as_millis());
    lib.push(album);
    lib.to_gds(1e-6, 1e-9).save("first_file.gds")?;
    println!("time costed:{}ms", start.elapsed().as_millis());
    Ok(())
}
