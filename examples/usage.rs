#![feature(const_type_name)] // :UnstableTypeName

use tracy_gizmos::{TracyClient, zone};

fn main() {
	let tracy = TracyClient::start();
	while !tracy.is_connected() {
		std::thread::yield_now();
	}

	zone!("main");

	println!("Hello, sailor!");
}
