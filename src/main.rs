mod base;
use std::convert::TryInto;

use base::{Circle, LayerData, Polygon, Resolution::*};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let c = Circle::new((0., 0.), 50000., MinNumber(5001));
    let p: Polygon<f64> = (c, LayerData::new(1, 1)).into();
    use gds21::*;
    let mut lib = GdsLibrary::new("mylib");
    let mut newcell = GdsStruct::new("mycell");
    let b = p.try_into().unwrap();
    newcell.elems.push(GdsElement::GdsBoundary(b));
    lib.structs.push(newcell);
    lib.save("example.gds")?;
    Ok(())
}
