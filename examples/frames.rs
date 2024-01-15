#![feature(const_type_name)] // :UnstableTypeName

use std::thread::{self, sleep};
use std::time::Duration;

use tracy_gizmos::{
	TracyClient,
	Color,
	set_thread_name,
	zone,
	frame,
	message,
};

const FRAMES: usize = 16;

fn main() {
	println!("Connecting to Tracy...");
	let tracy = TracyClient::start();
	while !tracy.is_connected() {
		std::thread::yield_now();
	}

	// Async AI updates, 30 Hz.
	let ai = thread::spawn(|| {
		set_thread_name!("AI");
		for _ in 0..FRAMES {
			zone!("AI brains", Color::OLIVE);
			sleep(Duration::from_millis(33));
			// Frame is done!
			frame!("AI");
		}
	});

	let io = thread::spawn(|| {
		set_thread_name!("I/O");
		// Just a delay, imagine we wait for a request here, instead.
		sleep(Duration::from_millis(50));

		// When io is dropped at the end of this scope - this
		// discontinuous frame is marked as finished.
		message!(Color::ORANGE, "io frame start");
		frame!(io, "IO");

		// Do the I/O work:
		zone!(streaming, "Streaming");
		streaming.text("asset1");
		sleep(Duration::from_millis(100));
		message!(Color::ORANGE, "io frame end");
	});

	// Main rendering loop, 60 Hz.
	for _ in 0..FRAMES*2 {
		{
			zone!("Update", Color::YELLOW);
			sleep(Duration::from_millis(8));
		}
		{
			zone!("Render", Color::BLUE);
			sleep(Duration::from_millis(8));
		}
		// Frame is done!
		frame!();
	}

	ai.join().unwrap();
	io.join().unwrap();
}
