use std::io::Write;
use std::net::{TcpListener, TcpStream};

//use webrtc::packet::{RtpPacket};

fn main() {
    use std::mem;

    /*
    let p = RtpPacket::new();
    println!("{:?}",p);
    let temp : u32 = 100;
    let temp_ss = unsafe {mem::transmute::<u32,[u8;4]>(temp) };
    println!("{:?}",temp_ss);
    */
}

/*
fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let response = b"Hello World";
                println!("message {:?}",stream.read);
                stream.write(response).expect("Response failed");
            }
            Err(e) => {
                println!("Unable to connect: {}", e);
            }
        }
    }
}
*/
