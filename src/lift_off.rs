use std::sync::{Arc, Mutex};

use krpc_client::services::space_center::{SpaceCenter, Control};
use crate::connection::Connection;


pub struct LiftOffBuilder {
    t_minus: u8,
    conn: Option<Connection>,
    stopper: Option<Arc<Mutex<bool>>>,
}
pub struct LiftOff {
    t_minus: u8,
    stopper: Option<Arc<Mutex<bool>>>,
    ship_control: Control,
}
impl LiftOff {
    pub fn builder() -> LiftOffBuilder {
        LiftOffBuilder::new()
    }

    pub fn start(self) {
        // ativar SAS
        self.ship_control.set_sas(true).unwrap();

        // ativar aceleração máxima
        self.ship_control.set_throttle(1.0).unwrap();

        // contagem regressiva t-10s
        for i in (0..self.t_minus).rev() {
            println!("T-{}s", i);
            std::thread::sleep(std::time::Duration::from_secs(1));
            match self.stopper {
                Some(ref stopper) => {
                    let stopper = stopper.lock().unwrap();
                    if *stopper {
                        println!("lift off aborted");
                        return;
                    }
                },
                _ => {}
            }
        }

        // ativar proximo estagio = Decolar
        self.ship_control.activate_next_stage().unwrap();
    }
}
impl LiftOffBuilder {
    fn new() -> Self {
        Self {
            t_minus: 5,
            conn: None,
            stopper: None,
        }
    }

    pub fn t_minus(mut self, t_minus: u8) -> Self {
        self.t_minus = t_minus;
        self
    }

    pub fn stopper(mut self, stopper: Arc<Mutex<bool>> ) -> Self {
        self.stopper = Some(stopper);
        self
    }

    pub fn connect(mut self, conn: Connection) -> Self {
        self.conn = Some(conn);
        self
    }

    pub fn build(self) -> LiftOff {
        let ship_control: Control = SpaceCenter::new(self.conn.unwrap().client.clone())
            .get_active_vessel()
            .unwrap()
            .get_control()
            .unwrap();
        LiftOff {
            t_minus: self.t_minus,
            ship_control,
            stopper: self.stopper,
        }
    }
}