use std::sync::{Arc, Mutex};

use crate::connection::Connection;
use krpc_client::services::space_center::{AutoPilot, Control, SpaceCenter, Vessel};

use crate::utils::{init_altitude_stream, init_apoastro_stream};
use krpc_client::stream::Stream;

// todo: implementar maneira de conseguir as streams de altitude e apoastro
pub struct GravitationalCurve {
    ship: Vessel,
    ship_control: Control,
    altitude: Stream<f64>,
    apoastro: Stream<f64>,
    direction: f32,
    grav_curve_initial_altitude: f64,
    final_apoastro: f64,
    stopper: Option<Arc<Mutex<bool>>>,
}
pub struct GravitationalCurveBuilder {
    conn: Option<Connection>,
    direction: f32,
    grav_curve_initial_altitude: f64,
    final_apoastro: f64,
    stopper: Option<Arc<Mutex<bool>>>,
}
impl GravitationalCurve {
    pub fn builder() -> GravitationalCurveBuilder {
        GravitationalCurveBuilder::new()
    }
    pub fn start(self) {
        // ativar piloto automatico
        let auto_pilot: AutoPilot = self.ship.get_auto_pilot().unwrap();
        auto_pilot.engage().unwrap();

        // set pitch(inclinação) and heading(direção)
        auto_pilot
            .target_pitch_and_heading(90.0, self.direction)
            .unwrap();
        auto_pilot.set_target_roll(90.0).unwrap();

        // curva gravitacional feita por Peste Renan
        let mut pitch: f64 = 90.0;
        while pitch > 1.0 {
            if self.altitude.get().unwrap() > self.grav_curve_initial_altitude
                && self.altitude.get().unwrap() < self.final_apoastro
            {
                // curva inclinacao por raiz quadrada da altitude
                let progress: f64 = (self.altitude.get().unwrap()
                    - self.grav_curve_initial_altitude)
                    / (self.final_apoastro - self.grav_curve_initial_altitude);
                let circular_increment: f64 = (1.0 - (progress - 1.0).powi(2)).sqrt();
                pitch = 90.0 - (90.0 * circular_increment);

                auto_pilot.set_target_pitch(pitch as f32).unwrap();
                // auto_pilot.set_target_roll(0.0).unwrap();
                println!(
                    "Altitude: {}m, autopilot target pitch: {}",
                    self.altitude.get().unwrap(),
                    auto_pilot.get_target_pitch().unwrap()
                );
                std::thread::sleep(std::time::Duration::from_millis(200));
                if self.apoastro.get().unwrap() > (self.final_apoastro * 0.75) {
                    self.ship_control.set_throttle(0.5).unwrap();
                    if self.apoastro.get().unwrap() > self.final_apoastro {
                        self.ship_control.set_throttle(0.0).unwrap();
                    }
                }
            }
            match self.stopper {
                Some(ref stopper) => {
                    let stopper = stopper.lock().unwrap();
                    if *stopper {
                        auto_pilot.disengage().unwrap();
                        println!("lift off aborted");
                        return;
                    }
                }
                _ => {}
            }
        }
        // destaivar piloto automatico
        auto_pilot.disengage().unwrap();
    }
}
impl GravitationalCurveBuilder {
    pub fn new() -> Self {
        Self {
            conn: None,
            direction: 90.0,
            grav_curve_initial_altitude: 400.0,
            final_apoastro: 80000.0,
            stopper: None,
        }
    }

    pub fn connect(mut self, conn: Connection) -> Self {
        self.conn = Some(conn);
        self
    }

    pub fn direction(mut self, direction: f32) -> Self {
        self.direction = direction;
        self
    }

    pub fn grav_curve_initial_altitude(mut self, grav_curve_initial_altitude: f64) -> Self {
        self.grav_curve_initial_altitude = grav_curve_initial_altitude;
        self
    }

    pub fn final_apoastro(mut self, final_apoastro: f64) -> Self {
        self.final_apoastro = final_apoastro;
        self
    }

    pub fn stopper(mut self, stopper: Arc<Mutex<bool>>) -> Self {
        self.stopper = Some(stopper);
        self
    }

    pub fn build(self) -> GravitationalCurve {
        let ship = SpaceCenter::new(self.conn.unwrap().client)
            .get_active_vessel()
            .unwrap();
        let ship_control = ship.get_control().unwrap();
        let altitude = init_altitude_stream(&ship);
        let apoastro = init_apoastro_stream(&ship);
        GravitationalCurve {
            ship,
            ship_control,
            altitude,
            apoastro,
            direction: self.direction,
            grav_curve_initial_altitude: self.grav_curve_initial_altitude,
            final_apoastro: self.final_apoastro,
            stopper: self.stopper,
        }
    }
}
