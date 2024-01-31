use std::thread::sleep;
use std::time::{SystemTime, UNIX_EPOCH, Duration};

use tracy_gizmos::{
	Color,
	PlotConfig,
	PlotFormat,
	PlotStyle,
	make_plot,
	plot,
	zone,
};

const POINTS: usize = 128;

fn main() {
	println!("Connecting to Tracy...");
	let tracy = tracy_gizmos::start_capture();

	// This could be removed if `no-exit` feature is enabled.
	while !tracy.is_connected() {
		std::thread::yield_now();
	}

	let mut seed: u64 = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.unwrap_or_default()
		.as_secs();

	zone!("Plotting");

	make_plot!(percents, "Load percentage", PlotConfig {
		format: PlotFormat::Percentage,
		style:  PlotStyle::Smooth,
		color:  Color::PAPAYA_WHIP,
		filled: true,
	});

	make_plot!(highmark, "High memory mark", PlotConfig {
		format: PlotFormat::Memory,
		style:  PlotStyle::Staircase,
		color:  Color::ROSY_BROWN,
		filled: false,
	});

	for i in 0..POINTS {
		let r = lcg(&mut seed) as i64;

		let r = plot!("random", r % 1000);
		plot!("i", i as i64);
		plot!(percents, r % 100);
		plot!(highmark, r);

		sleep(Duration::from_millis(10));
	}
}

// Numerical Recipes, Chapter 7.1, An Even Quicker Generator,
// Eq. 7.1.6 parameters from Knuth and H. W. Lewis>
const A: u64 = 1664525;
const C: u64 = 1013904223;
const M: u64 = 1 << 32;

fn lcg(seed: &mut u64) -> u64 {
	zone!("lcg");
	*seed = (seed.wrapping_mul(A).wrapping_add(C)) % M;
	*seed
}
