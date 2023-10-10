use krpc_client::error::RpcError;
use krpc_client::Client;
use std::sync::Arc;

#[derive(Clone)]
pub struct Connection {
    pub client: Arc<Client>,
}
pub struct ConnectionBuilder {
    conn_name: String,
    ip_addr: String,
    rpc_port: u16,
    stream_port: u16,
}

impl Connection {
    pub fn builder() -> ConnectionBuilder {
        ConnectionBuilder::new()
    }
}
impl ConnectionBuilder {
    fn new() -> Self {
        Self {
            conn_name: "".to_string(),
            ip_addr: "127.0.0.1".to_string(),
            rpc_port: 50000,
            stream_port: 50001,
        }
    }

    pub fn conn_name(mut self, conn_name: String) -> Self {
        self.conn_name = conn_name;
        self
    }

    pub fn ip_addr(mut self, ip_addr: String) -> Self {
        self.ip_addr = ip_addr;
        self
    }

    pub fn build(self) -> Result<Connection, RpcError> {
        let client = Client::new(
            &self.conn_name,
            &self.ip_addr,
            self.rpc_port,
            self.stream_port,
        );
        match client {
            Ok(client) => {
                println!("Conectado ao servidor: {}", self.conn_name);
                Ok(Connection { client })
            }
            Err(e) => {
                println!("Erro ao conectar ao servidor: {}", e);
                Err(e)
            }
        }
    }
}