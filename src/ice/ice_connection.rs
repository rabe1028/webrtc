use std::net::SocketAddr;
use std::iter;
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;


enum TransportType {
    UDP,
    TCP,
}

struct TurnOption {
    addr: SocketAddr,
    user: String,
    password: String,
    is_ssl: bool,
    transport: TransportType,
}

struct IceConnection {
    ice_controlling: bool,
    local_username: String,
    local_password: String,
    remote_username: Option<String>,
    remote_password: Option<String>,
    components: usize,
    stun_server: Option<SocketAddr>,
    turn_server: Option<TurnOption>,
    use_ipv4: bool,
    use_ipv6: bool,

}

impl IceConnection {
    pub fn new(
        ice_controlling: bool,
        components: usize,
        stun_server: Option<SocketAddr>,
        turn_server: Option<TurnOption>,
        use_ipv4: bool,
        use_ipv6: bool,
    ) -> IceConnection {
        let mut rng = thread_rng();
        let local_username : String = iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .take(4)
            .collect();

        let local_password : String = iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .take(22)
            .collect();

        let remote_username = None;
        let remote_password = None;


        IceConnection {
            ice_controlling,
            local_username,
            local_password,
            remote_username,
            remote_password,
            components,
            stun_server,
            turn_server,
            use_ipv4,
            use_ipv6,
        }
    }
}