use dgir::{
    color::Decorated,
    color::LayerData,
    cursor::{ArcCurve, Assembler, CellCursor, Cursor, Rect},
    draw::{
        curve::{
            groups::{Compound, Group},
            Sweep,
        },
        CircularArc, Line, Resolution,
    },
    gds::{DgirCell, DgirLibrary},
    units::{AbsoluteLength, Angle},
    zero, MICROMETER, NANOMETER,
};

mod common;

#[test]
fn compound() {
    let mut lib = DgirLibrary::new("libname");
    let cir = CircularArc::new(
        100. * MICROMETER,
        (zero(), zero()),
        (Angle::from_deg(0.), Angle::from_deg(360.)),
        Resolution::MinNumber(10001),
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
    lib.save(common::get_file_path("compound.gds")).unwrap();
}

#[test]
fn cursor() {
    #[allow(non_snake_case)]
    let WIDTH: [AbsoluteLength<f64>; 2] = [MICROMETER * 4., MICROMETER * 10.];

    #[allow(non_snake_case)]
    let COLOR: [LayerData; 2] = [LayerData::new(1, 0), LayerData::new(1, 1)];

    #[allow(non_snake_case)]
    let RESOLUTION: Resolution = Resolution::MinDistance(20. * NANOMETER);
    let mut cur: Cursor = Cursor::default();
    let mut rec: Rect<[_; 2]> = Rect::new(
        Line::new((zero(), zero()), (zero(), MICROMETER * 120.)),
        WIDTH.into(),
    );
    let mut cir: ArcCurve<[_; 2]> = ArcCurve::new(
        CircularArc::new(
            MICROMETER * 240.,
            [zero(), zero()],
            (Angle::from_deg(0.), Angle::from_deg(120.)),
            RESOLUTION,
        ),
        WIDTH.into(),
    );
    let mut topcell: DgirCell = DgirCell::new("top_cell");
    let t: Group<_> = cur.assemble(rec.clone().into_group());
    topcell.push(t.color(Group::from(COLOR)));
    topcell.push(
        cur.assemble(cir.clone().into_group())
            .color(Group::from(COLOR)),
    );
    topcell.push(
        cur.assemble(rec.clone().into_group())
            .color(Group::from(COLOR)),
    );
    cir.arc_mut()
        .set_radius(20. * MICROMETER)
        .set_ang((Angle::from_deg(180.), Angle::from_deg(120.)));
    topcell.push(
        cur.assemble(cir.clone().into_group())
            .color(Group::from(COLOR)),
    );
    rec.line_mut().end = [-10. * MICROMETER, zero()].into();
    topcell.push(
        cur.assemble(rec.clone().into_group())
            .color(Group::from(COLOR)),
    );
    cir.arc_mut()
        .set_radius(10. * MICROMETER)
        .set_ang((Angle::from_deg(0.), Angle::from_deg(120.)));
    topcell.push(
        cur.assemble(cir.clone().into_group())
            .color(Group::from(COLOR)),
    );
    topcell.push(
        cur.assemble(rec.clone().into_group())
            .color(Group::from(COLOR)),
    );
    topcell
        .save_as_lib(common::get_file_path("cursor.gds"))
        .unwrap();
}

#[test]
fn cell_cursor() {
    #[allow(non_snake_case)]
    let WIDTH: [AbsoluteLength<f64>; 2] = [MICROMETER * 4., MICROMETER * 10.];

    #[allow(non_snake_case)]
    let COLOR: [LayerData; 2] = [LayerData::new(1, 0), LayerData::new(1, 1)];

    #[allow(non_snake_case)]
    let RESOLUTION: Resolution = Resolution::MinDistance(20. * NANOMETER);

    let mut cur: CellCursor<_, _> = CellCursor::new("topcell", Group::from(COLOR));
    let mut cir: ArcCurve<[_; 2]> = ArcCurve::new(
        CircularArc::new(
            MICROMETER * 240.,
            [zero(), zero()],
            (Angle::from_deg(0.), Angle::from_deg(120.)),
            RESOLUTION,
        ),
        WIDTH.into(),
    );
    let mut rec: Rect<[_; 2]> = Rect::new(
        Line::new((zero(), zero()), (zero(), MICROMETER * 120.)),
        WIDTH.into(),
    );
    cur.assemble_in(cir.into_group());
    cur.assemble_in(rec.into_group());
    cir.arc_mut().set_radius(MICROMETER * 200.);
    rec.line_mut().end = [MICROMETER * 20., zero()].into();
    cur.assemble_in(cir.into_group());
    cur.assemble_in(rec.into_group());
    cur.into_cell()
        .save_as_lib(common::get_file_path("cell_cursor.gds"))
        .unwrap();
}

#[test]
fn assembler() {
    #[allow(non_snake_case)]
    let WIDTH: [AbsoluteLength<f64>; 2] = [MICROMETER * 4., MICROMETER * 10.];

    #[allow(non_snake_case)]
    let COLOR: [LayerData; 2] = [LayerData::new(1, 0), LayerData::new(1, 1)];

    #[allow(non_snake_case)]
    let RESOLUTION: Resolution = Resolution::MinDistance(20. * NANOMETER);

    let mut cur: Assembler<_, _> = Assembler::new("topcell", Group::from(COLOR), WIDTH, RESOLUTION);
    cur.turn(MICROMETER * 240., Angle::from_deg(120.));
    cur.extend(MICROMETER * 120.);
    cur.turn(MICROMETER * 200., Angle::from_deg(120.));
    cur.extend(MICROMETER * 20.);
    cur.taper(MICROMETER * 500., [MICROMETER * 2., MICROMETER * 8.]);
    cur.turn(MICROMETER * 200., Angle::from_deg(120.));
    cur.into_cell()
        .save_as_lib(common::get_file_path("assembler.gds"))
        .unwrap();
}
