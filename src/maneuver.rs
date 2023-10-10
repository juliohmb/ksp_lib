use crate::connection::Connection;
use krpc_client::services::space_center::{Node, SpaceCenter};

/*
CelestialBody::equatorial_radius => raio equatorial
Orbital speed can be calculated by v = sqrt(GM/r), where G is the gravitational constant, M is the mass of the body being orbited, and r is the distance between the two bodies.
u ~= GM
M = CelestialBody::mass
CelestialBody::gravitational_parameter = GM
G = u/M
Can get the CelestialBody of the current orbit with Orbit::body
Can get the Orbit by Vessel::orbit
To-do => Maneuver with execute algorithm to the best maneuver
To-do => ManeuverBuilder has has circularize_in(apsis: Enum) -> Maneuver
*/

pub enum Apsis {
    Apoapsis,
    Periapsis,
}
pub struct Maneuver {
    node: Node,
    conn: Option<Connection>,
}
pub struct ManeuverBuilder {
    conn: Option<Connection>,
    rx_stopper: Option<std::sync::mpsc::Receiver<()>>,
    delta_v_prograde: f32,
    delta_v_normal: f32,
    delta_v_radial: f32,
    ut: f64,
}
impl Maneuver {
    pub fn builder() -> ManeuverBuilder {
        ManeuverBuilder::new()
    }

    pub fn execute(self) {
        //get ship
        let ship = SpaceCenter::new(self.conn.clone().unwrap().client)
            .get_active_vessel()
            .unwrap();
        // let ship_control = ship.get_control().unwrap();
        // engage auto pilot
        let auto_pilot = ship.get_auto_pilot().unwrap();
        auto_pilot.engage().unwrap();
        // get remaining burn vector stream with a surface reference frame
        let ref_frame = ship.get_surface_reference_frame().unwrap();
        let burn_vector_stream = self
            .node
            .remaining_burn_vector_stream(Some(&ref_frame))
            .unwrap();
        let mut burn_vector = burn_vector_stream.get().unwrap();
        // position ship to the maneuver node
        auto_pilot.set_target_direction(burn_vector).unwrap();
        // get maneuver delta_v stream
        let maneuver_delta_v_stream = self.node.get_remaining_delta_v_stream().unwrap();
        // set time to maneuver node stream let mut
        let time_to_maneuver_stream = self.node.get_time_to_stream().unwrap();
        // get extimated burn time
        let ship_mass = ship.get_mass().unwrap();
        let ship_force = ship.get_available_thrust().unwrap();
        let ship_aceleration = (ship_force / ship_mass) as f64;
        let maneuver_delta_v = maneuver_delta_v_stream.get().unwrap();
        let maneuver_burn_time = maneuver_delta_v / ship_aceleration;
        // set throttle to 0.0
        let mut throtle = 0.0;
        while maneuver_delta_v_stream.get().unwrap() > 0.1 {
            burn_vector = burn_vector_stream.get().unwrap();
            // set target in maneuver node
            auto_pilot.set_target_direction(burn_vector).unwrap();
            // check if time to maneuver is less than 0.1s
            if time_to_maneuver_stream.get().unwrap() - (maneuver_burn_time / 2.0) < 0.1 {
                // set throttle to 1.0
                throtle = 1.0;
                let maneuver_deltav = maneuver_delta_v_stream.get().unwrap();
                if maneuver_deltav < 100.0 {
                    throtle = maneuver_deltav / 100.0;
                }
            }
            ship.get_control()
                .unwrap()
                .set_throttle(throtle as f32)
                .unwrap();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        ship.get_control().unwrap().set_throttle(0.0).unwrap();
        auto_pilot.disengage().unwrap();
    }
}
impl ManeuverBuilder {
    fn new() -> Self {
        Self {
            conn: None,
            rx_stopper: None,
            delta_v_prograde: 0.0,
            delta_v_normal: 0.0,
            delta_v_radial: 0.0,
            ut: 0.0,
        }
    }

    pub fn connect(mut self, conn: Connection) -> Self {
        self.conn = Some(conn);
        self
    }

    pub fn rx_stopper(mut self, rx_stopper: std::sync::mpsc::Receiver<()>) -> Self {
        self.rx_stopper = Some(rx_stopper);
        self
    }

    pub fn set_delta_vs(
        &mut self,
        delta_v_prograde: f32,
        delta_v_normal: f32,
        delta_v_radial: f32,
    ) {
        self.delta_v_prograde = delta_v_prograde;
        self.delta_v_normal = delta_v_normal;
        self.delta_v_radial = delta_v_radial;
    }

    pub fn set_ut(&mut self, ut: f64) {
        self.ut = ut;
    }

    pub fn get_maneuver_by_index(mut self, maneuver_index: usize) -> Self {
        let ship_control = SpaceCenter::new(self.conn.clone().unwrap().client)
            .get_active_vessel()
            .unwrap()
            .get_control()
            .unwrap();
        let node = &ship_control.get_nodes().unwrap()[maneuver_index];
        let ut = node.get_ut().unwrap();
        self.set_ut(ut);
        self.set_delta_vs(
            node.get_prograde().unwrap() as f32,
            node.get_normal().unwrap() as f32,
            node.get_radial().unwrap() as f32,
        );
        self
    }

    pub fn circularize_in(mut self, apsis: Apsis) -> Self {
        let space_center = SpaceCenter::new(self.conn.clone().unwrap().client);
        let ship = space_center.get_active_vessel().unwrap();
        let current_orbit = ship.get_orbit().unwrap();
        let r: f32;
        // get the time to apsis
        let time_to_apsis: f64;
        match apsis {
            Apsis::Apoapsis => {
                // get current orbit apoapsis r
                r = current_orbit.get_apoapsis().unwrap() as f32;
                // get the time to apsis
                time_to_apsis = current_orbit.get_time_to_apoapsis().unwrap();
            }
            Apsis::Periapsis => {
                // get current orbit periapsis r
                r = current_orbit.get_periapsis().unwrap() as f32;
                // get the time to apsis
                time_to_apsis = current_orbit.get_time_to_periapsis().unwrap();
            }
        }
        // sum the time to apoapsis with the current time
        // keep printing the current time
        let ut = space_center.get_ut().unwrap();
        let maneuver_ut = ut + time_to_apsis;
        self.ut = maneuver_ut;
        // get the u of the current orbit with Orbit::body::gravitational_parameter = GM
        let u = current_orbit
            .get_body()
            .unwrap()
            .get_gravitational_parameter()
            .unwrap();
        // get the speed in the apoapsis with Orbit::orbital_speed_at(time) **
        // https://en.wikipedia.org/wiki/Elliptic_orbit => v = sqrt(u(2/r - 1/a))
        let a = current_orbit.get_semi_major_axis().unwrap() as f32;
        let apsis = (u * (2.0 / r - 1.0 / a)).sqrt();
        // Calculate the v to circularize => v = sqrt(u/r)
        let v_to_circularize = (u / r).sqrt();
        // Calculate the delta_v to circularize => delta_v = v - speed_in_apoapsis
        let prograde_delta_v = v_to_circularize - apsis;
        self.delta_v_prograde = prograde_delta_v as f32;
        self
    }

    pub fn build(self) -> Maneuver {
        let ship = SpaceCenter::new(self.conn.clone().unwrap().client)
            .get_active_vessel()
            .unwrap();
        let ship_control = ship.get_control().unwrap();
        let node = ship_control
            .add_node(
                self.ut,
                self.delta_v_prograde,
                self.delta_v_normal,
                self.delta_v_radial,
            )
            .unwrap();
        Maneuver {
            node,
            conn: self.conn,
        }
    }
}
