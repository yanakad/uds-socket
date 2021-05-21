use nix::{
    sys::wait::{waitpid, WaitStatus},
    unistd::{fork, ForkResult},
};
use socket2::{Socket, Domain, Type};
use std::os::unix::io::{AsRawFd, FromRawFd};

fn main() {
    println!("Hello, world!");

    let (mut master, minion) = Socket::pair(Domain::unix(), Type::stream(), None).unwrap();

    match unsafe { fork().unwrap() } {
        ForkResult::Parent { child } => match waitpid(Some(child), None).unwrap() {
            WaitStatus::Exited(_pid, code) => {
                println!("Happy exit");
                let mut result = vec![0; 24];
                let msg = master.recv(&mut result).unwrap();
                println!("Got {}", msg);
                let message = std::str::from_utf8(&result).unwrap();
                println!("Read {}", message);
            }
            status => panic!("Unexpected WaitStatus: {:?}", status),
        },
        ForkResult::Child => {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            let socket = minion.as_raw_fd();
            runtime.block_on(async { panic_func(socket) });
        }
    }
}

fn panic_func(sender: i32) {
    let socket = unsafe { Socket::from_raw_fd(sender) };
    std::panic::set_hook({
        Box::new(move |panic_info| {
            println!("YANA::Yes, we paniced");
            socket.send("TestMessage".as_bytes());
        })
    });
    panic!("Lets panic");
}
