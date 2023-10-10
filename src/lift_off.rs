use krpc_client::services::space_center::{SpaceCenter, Control};
use crate::connection::Connection;


pub struct LiftOffBuilder {
    t_minus: u8,
    conn: Option<Connection>,
    rx_stopper: Option<std::sync::mpsc::Receiver<()>>,
}
pub struct LiftOff {
    t_minus: u8,
    rx_stopper: Option<std::sync::mpsc::Receiver<()>>,
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
            if self.rx_stopper.is_some() {
                let rx = self.rx_stopper.as_ref().unwrap();
                if rx.try_recv().is_ok() {
                    println!("lift off aborted");
                    return;
                }
            }
        }

        // ativar proximo estagio = Decolar
        // self.ship_control.activate_next_stage().unwrap();
    }
}
impl LiftOffBuilder {
    fn new() -> Self {
        Self {
            t_minus: 5,
            conn: None,
            rx_stopper: None,
        }
    }

    pub fn t_minus(mut self, t_minus: u8) -> Self {
        self.t_minus = t_minus;
        self
    }

    pub fn rx_stopper(mut self, rx_stopper: std::sync::mpsc::Receiver<()>) -> Self {
        self.rx_stopper = Some(rx_stopper);
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
            rx_stopper: self.rx_stopper,
        }
    }
}