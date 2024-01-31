use std::{
	alloc::{alloc, dealloc, Layout},
	thread::sleep,
	time::Duration,
};

use tracy_gizmos::{
	emit_alloc,
	emit_free,
};

fn main() {
	println!("Connecting to Tracy...");
	let tracy = tracy_gizmos::start_capture();

	// This could be removed if `no-exit` feature is enabled.
	while !tracy.is_connected() {
		std::thread::yield_now();
	}

	for i in 0..10 {
		let layout = Layout::array::<u64>(i).unwrap();
		let ptr    = unsafe { alloc(layout) };
		if !ptr.is_null() {
			emit_alloc!("global", ptr, layout.size());
			sleep(Duration::from_millis(10));
			emit_free!("global", ptr);
			unsafe { dealloc(ptr, layout) };
		}
	}
}
