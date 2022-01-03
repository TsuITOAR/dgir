use dgir::{
    color::{Decorated, LayerData},
    cursor::{ArcCurve, Assembler, CellCursor},
    draw::{curve::groups::Group, CircularArc, Resolution},
    units::{AbsoluteLength, Angle},
    zero, MICROMETER,
};

mod common;

#[test]
fn pulley() {
    #[allow(non_snake_case)]
    let RING_WIDTH: [AbsoluteLength<f64>; 2] = [MICROMETER * 4., MICROMETER * 10.];
    #[allow(non_snake_case)]
    let BUS_WIDTH: [AbsoluteLength<f64>; 2] = [MICROMETER * 4., MICROMETER * 10.];
    #[allow(non_snake_case)]
    let COLOR: [LayerData; 2] = [LayerData::new(1, 0), LayerData::new(1, 1)];
    #[allow(non_snake_case)]
    let RESOLUTION: Resolution = Resolution::MinNumber(8001);
    #[allow(non_snake_case)]
    let GAP: AbsoluteLength<f64> = MICROMETER;
    #[allow(non_snake_case)]
    let RADIUS: AbsoluteLength<f64> = MICROMETER * 240.;
    #[allow(non_snake_case)]
    let PUL_ANG: Angle = Angle::from_deg(30.);
    #[allow(non_snake_case)]
    let PUL_RAD: AbsoluteLength<f64> = RADIUS + RING_WIDTH[0] / 2. + BUS_WIDTH[0] / 2. + GAP;

    let ring = ArcCurve::new(
        CircularArc::new_origin(
            RADIUS,
            (Angle::from_deg(0.), Angle::from_deg(360.)),
            RESOLUTION,
        ),
        RING_WIDTH,
    );
    let mut cursor = CellCursor::new("topcell", Group::from(COLOR));
    cursor
        .mut_cell()
        .push(ring.into_group().color(Group::from(COLOR)));
    cursor.cursor.pos = [zero(), PUL_RAD].into();
    let bus_curve: ArcCurve<[_; 2]> = ArcCurve::new(
        CircularArc::new_origin(PUL_RAD, (Angle::from_deg(0.), -PUL_ANG / 2.), RESOLUTION),
        BUS_WIDTH.into(),
    );
    cursor.assemble_in(bus_curve.into_group());
    cursor.assemble_in(bus_curve.rev().into_group());
    cursor.cursor.pos = [zero(), PUL_RAD].into();
    cursor.cursor.dir = Angle::from_deg(180.);
    cursor.assemble_in(bus_curve.rev().into_group());
    cursor.assemble_in(bus_curve.into_group());
    cursor
        .into_cell()
        .save_as_lib(common::get_file_path("pulley.gds"))
        .unwrap();
}

#[test]
fn assemble_pulley() {
    #[allow(non_snake_case)]
    let RING_WIDTH: [AbsoluteLength<f64>; 2] = [MICROMETER * 4., MICROMETER * 10.];
    #[allow(non_snake_case)]
    let BUS_WIDTH: [AbsoluteLength<f64>; 2] = [MICROMETER * 4., MICROMETER * 10.];
    #[allow(non_snake_case)]
    let COLOR: [LayerData; 2] = [LayerData::new(1, 0), LayerData::new(1, 1)];
    #[allow(non_snake_case)]
    let RESOLUTION: Resolution = Resolution::MinNumber(8001);
    #[allow(non_snake_case)]
    let GAP: AbsoluteLength<f64> = MICROMETER;
    #[allow(non_snake_case)]
    let RADIUS: AbsoluteLength<f64> = MICROMETER * 240.;
    #[allow(non_snake_case)]
    let PUL_ANG: Angle = Angle::from_deg(30.);
    #[allow(non_snake_case)]
    let PUL_RAD: AbsoluteLength<f64> = RADIUS + RING_WIDTH[0] / 2. + BUS_WIDTH[0] / 2. + GAP;

    let mut cursor: Assembler<_> = Assembler::new("topcell", COLOR, BUS_WIDTH, RESOLUTION);
    cursor.mut_cell().push(
        ArcCurve::new(
            CircularArc::new_origin(
                RADIUS,
                (Angle::from_deg(0.), Angle::from_deg(360.)),
                RESOLUTION,
            ),
            RING_WIDTH,
        )
        .into_group()
        .color(Group::from(COLOR)),
    );
    cursor
        .set_pos([zero(), PUL_RAD])
        .set_dir(Angle::from_deg(0.))
        .turn(PUL_RAD, -PUL_ANG / 2.)
        .turn(PUL_RAD, PUL_ANG / 2.)
        .extend(MICROMETER * 1000.)
        .taper(MICROMETER * 100., [MICROMETER * 2., MICROMETER * 8.])
        .extend(MICROMETER * 100.);
    cursor.width = BUS_WIDTH;
    cursor
        .set_pos([zero(), PUL_RAD])
        .set_dir(Angle::from_deg(180.))
        .turn(PUL_RAD, PUL_ANG / 2.)
        .turn(PUL_RAD, -PUL_ANG / 2.)
        .extend(MICROMETER * 1000.)
        .taper(MICROMETER * 100., [MICROMETER * 2., MICROMETER * 8.])
        .extend(MICROMETER * 100.);
    cursor
        .into_cell()
        .save_as_lib(common::get_file_path("assemble_pulley.gds"))
        .unwrap();
}
