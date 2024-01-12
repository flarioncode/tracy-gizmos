#![feature(const_type_name)] // :UnstableTypeName

use std::thread::{self, sleep};
use std::time::Duration;

use tracy_gizmos::{TracyClient, zone, set_thread_name, Color};

// @Incomplete Add plots, frames, etc.

fn main() {
	println!("Connecting to Tracy...");
	let tracy = TracyClient::start();
	while !tracy.is_connected() {
		std::thread::yield_now();
	}

	zone!("main");

	println!("Connected! Let's do some work...");

	let w1 = thread::spawn(|| {
		set_thread_name!("Worker 1");

		zone!("work1", Color::BISQUE1);
		heavy_work1();
	});

	let w2 = thread::spawn(|| {
		set_thread_name!("Worker 2");

		{
			zone!("delay", Color::ORANGE);
			sleep(Duration::from_millis(100));
		}

		zone!("work2", Color::BISQUE2);
		heavy_work2();
	});

	w1.join().unwrap();
	w2.join().unwrap();
}

fn heavy_work1() {
	let mut x: u64 = 1;
	for i in 0..900_000_000 {
		x = x.wrapping_add(i);
	}
	println!("work1 yielded {x}");
	some_sub_work();
}

fn heavy_work2() {
	let mut x: u64 = 1;
	for i in 1..700_000_000 {
		x = x.wrapping_add(x.wrapping_mul(i));
	}
	println!("work2 yielded {x}");
	some_sub_work();
}

fn some_sub_work() {
	zone!("sub work");

	let mut x: u64 = 1;
	for i in 0..200_000_000 {
		x = x.wrapping_sub(i);
	}
	println!("sub work yielded {x}");
}
