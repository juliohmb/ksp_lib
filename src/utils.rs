use crate::connection::Connection;
use krpc_client::services::drawing::Line;
use krpc_client::stream::Stream;
use krpc_client::services::space_center::{Vessel, CelestialBody, ReferenceFrame};


pub fn init_altitude_stream(ship: &Vessel) -> Stream<f64> {
    let current_celestial_body: CelestialBody = ship.get_orbit().unwrap().get_body().unwrap();
    let orbital_reference: ReferenceFrame = current_celestial_body.get_reference_frame().unwrap();
    let fligh_params = ship.flight(Option::from(&orbital_reference)).unwrap();
    fligh_params.get_mean_altitude_stream().unwrap()
}

pub fn init_apoastro_stream(ship: &Vessel) -> Stream<f64> {
    ship.get_orbit()
        .unwrap()
        .get_apoapsis_altitude_stream()
        .unwrap()
}

// fn init_pitch_stream(ship: &Vessel) -> Stream<f32> {
//     let current_celestial_body: CelestialBody = ship.get_orbit().unwrap().get_body().unwrap();
//     let orbital_reference: ReferenceFrame = current_celestial_body.get_reference_frame().unwrap();
//     let fligh_params = ship.flight(Option::from(&orbital_reference)).unwrap();
//     fligh_params.get_pitch_stream().unwrap()
// }

pub fn draw_axis(conn: Connection, ref_frame: &ReferenceFrame) -> (Line, Line, Line) {
    let z_line = krpc_client::services::drawing::Drawing::new(conn.client.clone())
        .add_line((0.0, 0.0, 0.0), (0.0, 0.0, 100.0), ref_frame, true)
        .unwrap();
    z_line.set_color((255.0, 0.0, 0.0)).unwrap();
    let y_line = krpc_client::services::drawing::Drawing::new(conn.client.clone())
        .add_line((0.0, 0.0, 0.0), (0.0, 100.0, 0.0), ref_frame, true)
        .unwrap();
    y_line.set_color((0.0, 255.0, 0.0)).unwrap();
    let x_line = krpc_client::services::drawing::Drawing::new(conn.client.clone())
        .add_line((0.0, 0.0, 0.0), (100.0, 0.0, 0.0), ref_frame, true)
        .unwrap();
    x_line.set_color((0.0, 0.0, 255.0)).unwrap();
    (x_line, y_line, z_line)
}