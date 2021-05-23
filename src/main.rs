mod base;
mod units;
use std::convert::TryInto;

use base::{Circle, LayerData, Polygon, Resolution::*};
use units::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let c = Circle::new(
        (Length::<Micrometer>::new(0.), Length::<Micrometer>::new(0.)),
        Length::<Micrometer>::new(50.),
        MinNumber(5000),
    );
    let p: Polygon<Length<Micrometer>> = (c, LayerData::new(1, 1)).into();
    use gds21::*;
    let mut lib = GdsLibrary::new("mylib");
    let mut newcell = GdsStruct::new("mycell");
    let b = p.try_into().unwrap();
    newcell.elems.push(GdsElement::GdsBoundary(b));
    lib.structs.push(newcell);
    lib.save("example.gds")?;
    Ok(())
}
