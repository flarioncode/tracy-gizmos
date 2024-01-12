// @Cleanup Feature-gate this for the nightly enjoyers?
#![feature(const_type_name)] // :UnstableTypeName
#![warn(missing_docs)]

//! @Incomplete Document this or attach the readme.

use std::sync::atomic::{AtomicBool, Ordering};
use std::marker::PhantomData;

mod color;
mod plot;

pub use color::*;
pub use plot::*;

/// Sets the current thread's name.
///
/// It is recommended to *always* use it in every thread, which uses
/// Tracy capabilities.
///
/// If not set, Tracy will try to capture thread names through
/// operating system data if context switch capture is active.
/// However, this is only a fallback mechanism, and it shouldn't be
/// relied upon.
///
/// # Examples
///
/// Ypu can specify a compile-time constant name for a particular
/// thread, for example, a dedicated I/O thread:
///
/// ```no_run
/// # #![feature(const_type_name)] // :UnstableTypeName
/// # use tracy_gizmos::*;
/// # fn loop_and_do_io() {}
/// std::thread::spawn(|| {
///     set_thread_name!("I/O processor");
///     zone!("I/O requests"); // Tracy will show it on this thread track.
///     loop_and_do_io();
/// });
/// ```
///
/// You can make a name in runtime, for example, when you have an
/// arbitrary amount of worker threads:
///
/// ```no_run
/// # #![feature(const_type_name)] // :UnstableTypeName
/// # use tracy_gizmos::*;
/// # fn get_next_worker_id() -> u32 { 0 }
/// # fn loop_and_do_work() {}
/// let id = get_next_worker_id();
/// std::thread::spawn(move || {
///     set_thread_name!("worker-thread {}", id);
///     zone!("Working"); // Tracy will show it on this thread track.
///     loop_and_do_work();
/// });
/// ```
#[macro_export]
macro_rules! set_thread_name {
	($name: literal) => {
		// SAFETY: We null-terminate the string.
		unsafe {
			$crate::details::set_thread_name(concat!($name, '\0').as_ptr());
		}
	};

	($format: literal, $($args:expr),*) => {
		$(
			{
				let name = format!(concat!($format, '\0'), $args).into_bytes();
				// SAFETY: We null-terminated the string during formatting.
				unsafe {
					let name = std::ffi::CString::from_vec_with_nul_unchecked(name);
					$crate::details::set_thread_name(name.as_ptr().cast());
				}
			}
		)*
	};
}

// @Cleanup This should not exist when we are not enabled.
static STARTED: AtomicBool = AtomicBool::new(false);

/// Represents a Tracy client.
///
/// Obtaining a `TracyClient` is *required* to instrument the code.
/// Otherwise, the behaviour of instrumenting is undefined.
///
/// It is not allowed to have multiple copies of the `TracyClient`.
///
/// When it is dropped, the Tracy connection will be shutdown.
pub struct TracyClient(PhantomData<*mut ()>);

impl TracyClient {
	/// Initializes the Tracy profiler.
	///
	/// Must be called *before* any other Tracy usage.
	///
	/// # Panics
	///
	/// Only one alive client can exist. Hence any consecutive
	/// `start()` will panic, unless previously started client is
	/// dropped.
	///
	/// # Examples
	///
	/// ```no_run
	/// use tracy_gizmos::TracyClient;
	///
	/// fn main() {
	/// 	let tracy = TracyClient::start();
	/// }
	/// ```
	pub fn start() -> Self {
		if STARTED.swap(true, Ordering::Acquire) {
			panic!("Tracy client has been started already.");
		}
		// SAFETY: Check above ensures this happens once.
		unsafe {
			sys::___tracy_startup_profiler();
		}
		Self(PhantomData)
	}

	/// Returns `true` if a connection is currently established with
	/// the Tracy server.
	///
	/// # Examples
	///
	/// This method can be used to ensure the profiler connection
	/// _before_ doing any profiled work.
	///
	/// ```no_run
	/// use tracy_gizmos::TracyClient;
	///
	/// fn main() {
	/// 	let tracy = TracyClient::start();
	/// 	while !tracy.is_connected() {
	/// 		std::thread::yield_now();
	/// 	}
	/// 	// You can do the profiling here knowing it will reach
	/// 	// Tracy.
	/// }
	/// ```
	pub fn is_connected(&self) -> bool {
		// SAFETY: self could exist only if startup was issued and
		// succeeded.
		unsafe {
			sys::___tracy_connected() != 0
		}
	}
}

impl Drop for TracyClient {
	fn drop(&mut self) {
		// SAFETY: self could exist only if startup was issued and
		// succeeded.
		unsafe {
			sys::___tracy_shutdown_profiler();
		}
		STARTED.store(false, Ordering::Release);
	}
}

/// Instruments the current scope with a profiling zone.
///
/// A zone represents the lifetime of a special on-stack profiler
/// variable. Typically, it would exist for the duration of a whole
/// profiled function scope, but it is also possible to measure time
/// spent in nested loops or if branches.
///
/// A custom name is required for the zone, which allows to identify
/// it later in the Trace visualization.
///
/// This will automatically record source file name, and location.
///
/// # Examples
///
/// Just adding a profiling zone is as easy as:
///
/// ```no_run
/// # #![feature(const_type_name)] // :UnstableTypeName
/// # use tracy_gizmos::*;
/// fn do_stuff() {
///     zone!("stuff");
///     // now actually do stuff :-)
/// }
/// ```
///
/// Optionally, a custom [`Color`] could be assigned for the zone. Note,
/// that the color value will be constant in the recording.
///
/// ```no_run
/// # #![feature(const_type_name)] // :UnstableTypeName
/// # use tracy_gizmos::*;
/// fn do_stuff() {
///     zone!("stuff", Color::BISQUE);
///     // now actually do stuff :-)
/// }
/// ```
///
/// ## Nesting
///
/// Multiple active zones can exist and they will be nested in
/// parent-child relationship automatically by Tracy.
///
/// ```no_run
/// # #![feature(const_type_name)] // :UnstableTypeName
/// # use tracy_gizmos::*;
/// # fn work() {}
/// fn work_cycle(times: usize) {
///     zone!("doing work");
///     for _ in 0..times {
///         zone!("unit of work");
///         work();
///     }
/// }
/// ```
///
/// ## Filtering zones
///
/// Zone logging can be disabled on a per-zone basis:
///
/// ```no_run
/// # #![feature(const_type_name)] // :UnstableTypeName
/// # use tracy_gizmos::*;
/// const PROFILE_JOBS: bool = false;
/// zone!("Do Jobs", enabled: PROFILE_JOBS); // no runtime cost.
/// ```
///
/// Note that this parameter may be a run-time expression, which
/// should evaluate to `bool`, such as a user-controller switch to
/// enable the profiling of a specific part of the code only when
/// needed.
///
/// ```no_run
/// # #![feature(const_type_name)] // :UnstableTypeName
/// # use tracy_gizmos::*;
/// # fn toggled() -> bool { true }
/// # let mut do_profiles = false;
/// if toggled() {
///    do_profiles = !do_profiles;
/// }
///
/// zone!("Do Jobs", enabled: do_profiles); // runtime cost.
/// ```
///
/// ## Dynamic data
///
/// It is possible to have a set of dynamic data attached to the
/// particular zone. Refer to [`Zone`] for more details.
///
/// ```no_run
/// # #![feature(const_type_name)] // :UnstableTypeName
/// # use tracy_gizmos::*;
/// # let file_path = "./main.rs";
/// zone!(parsing, "Parsing");
/// parsing.text(file_path);
/// ```
#[macro_export]
macro_rules! zone {
	(            $name:literal)                               => { zone!(_z,   $name, Color::UNSPECIFIED, enabled:true) };
	($var:ident, $name:literal)                               => { zone!($var, $name, Color::UNSPECIFIED, enabled:true) };
	(            $name:literal, $color:expr)                  => { zone!(_z,   $name, $color,             enabled:true) };
	($var:ident, $name:literal, $color:expr)                  => { zone!($var, $name, $color,             enabled:true) };
	(            $name:literal,              enabled:$e:expr) => { zone!(_z,   $name, Color::UNSPECIFIED, enabled:$e)   };
	($var:ident, $name:literal,              enabled:$e:expr) => { zone!($var, $name, Color::UNSPECIFIED, enabled:$e)   };
	(            $name:literal, $color:expr, enabled:$e:expr) => { zone!(_z,   $name, $color,             enabled:$e)   };
	($var:ident, $name:literal, $color:expr, enabled:$e:expr) => {
		// SAFETY: This macro ensures that location & context data are correct.
		let $var = unsafe {
			details::zone(zone!(@loc $name, $color), if $e {1} else {0})
		};
	};

	(@loc $name:literal, $color: expr) => {{
		// It is an implementation detail and can be changed at any moment.

		struct X;
		// Tracking issue on the Rust side:
		// https://github.com/rust-lang/rust/issues/63084
		// :UnstableTypeName
		const TYPE_NAME: &'static str = std::any::type_name::<X>();
		// We skip 3 of the '::X' suffix and add 1 for the terminating zero.
		const FUNCTION_LEN: usize = TYPE_NAME.len() - 3 + 1;
		const FUNCTION: &'static [u8] = &details::as_array::<FUNCTION_LEN>(TYPE_NAME);

		// SAFETY: All passed data is created here and is correct.
		static LOC: $crate::ZoneLocation = unsafe {
			details::zone_location(
				concat!($name, '\0'),
				FUNCTION,
				concat!(file!(), '\0'),
				line!(),
				Color::as_u32(&$color),
			)
		};
		&LOC
	}};
}

/// Profiling zone.
///
/// It instruments the current scope. Hence, the profiling zone will
/// end when [`Zone`] is dropped.
pub struct Zone {
	ctx:     sys::___tracy_c_zone_context,
	_unsend: PhantomData<*mut ()>,
}

impl Drop for Zone {
	#[inline(always)]
	fn drop(&mut self) {
		// SAFETY: The only way to have Zone is to construct it via
		// zone! macro, which ensures that ctx value is correct.
		unsafe {
			sys::___tracy_emit_zone_end(self.ctx);
		}
	}
}

impl Zone {
	/// Allows to control the zone color dynamically.
	///
	/// This can be called multiple times, however only the latest
	/// call will have an effect.
	pub fn color(&self, color: Color) {
		// SAFETY: self always contains a valid `ctx`.
		unsafe {
			sys::___tracy_emit_zone_color(self.ctx, color.as_u32());
		}
	}

	/// Adds a custom numeric value that will be displayed along with
	/// the zone information. E.g. a loop iteration or size of the
	/// processed buffer.
	///
	/// This method can be called multiple times, all of the passed
	/// values will be attached to the zone matching the call order.
	pub fn number(&self, value: u64) {
		// SAFETY: self always contains a valid `ctx`.
		unsafe {
			sys::___tracy_emit_zone_value(self.ctx, value);
		}
	}

	/// Adds a custom text string that will be displayed along with
	/// the zone information. E.g. name of the file you are
	/// processing.
	///
	/// The profiler will allocate a temporary internal buffer and
	/// copy the passed value there. Hence, this operation involves a
	/// small run-time cost.
	///
	/// This method can be called multiple times, all of the passed
	/// values will be attached to the zone matching the call order.
	///
	/// Be aware that the passed text slice couldn't be larger than 64
	/// Kb.
	pub fn text(&self, s: &str) {
		debug_assert!(s.len() < u16::MAX as usize);
		// SAFETY: self always contains a valid `ctx`.
		unsafe {
			sys::___tracy_emit_zone_text(self.ctx, s.as_ptr().cast(), s.len())
		}
	}
}

/// A statically allocated location for a profiling zone.
///
/// It is an implementation detail and can be changed at any moment.
#[doc(hidden)]
#[repr(transparent)]
pub struct ZoneLocation(sys::___tracy_source_location_data);

// SAFETY: It is fully static and constant.
unsafe impl Send for ZoneLocation {}
unsafe impl Sync for ZoneLocation {}

/// Implementation details, do not relay on anything from this module!
///
/// It is public only due to the usage in public macro bodies.
#[doc(hidden)]
pub mod details {
	use super::*;

	#[inline(always)]
	pub const unsafe fn zone_location(
		name: &'static str,
		func: &'static [u8],
		file: &'static str,
		line: u32,
		color: u32,
	) -> ZoneLocation {
		ZoneLocation(
			sys::___tracy_source_location_data {
				name:     name.as_ptr().cast(),
				function: func.as_ptr().cast(),
				file:     file.as_ptr().cast(),
				line,
				color,
			}
		)
	}

	#[inline(always)]
	pub unsafe fn zone(location: &ZoneLocation, enabled: i32) -> Zone {
		let ctx = sys::___tracy_emit_zone_begin(&location.0, enabled);
		Zone { ctx, _unsend: PhantomData }
	}

	#[inline(always)]
	pub unsafe fn set_thread_name(name: *const u8) {
		sys::___tracy_set_thread_name(name.cast());
	}

	pub const fn as_array<const N: usize>(s: &'static str) -> [u8; N] {
		let bytes   = s.as_bytes();
		let mut buf = [0; N];
		let mut i   = 0;
		while i < N - 1 {
			buf[i] = bytes[i];
			i += 1;
		}
		buf
	}
}

#[cfg(test)]
mod tests {
    use super::*;

	#[test]
	fn double_lifecycle() {
		let tracy = TracyClient::start();
		drop(tracy);
		let _tracy = TracyClient::start();
	}

	#[test]
	#[should_panic]
	fn double_start_fails() {
		let _tracy1 = TracyClient::start();
		let _tracy2 = TracyClient::start();
	}

	#[test]
	fn playground() {
		let tracy = TracyClient::start();
		while !tracy.is_connected() {
			std::thread::yield_now();
		}

		set_thread_name!("main-thread");
		let t = std::thread::spawn(|| {
			set_thread_name!("worker-thread {}", 1);
			zone!("work");
			std::thread::sleep(std::time::Duration::from_secs(1));
			zone!("inside work");
			std::thread::sleep(std::time::Duration::from_secs(1));
		});
		let d = std::thread::spawn(|| {
			set_thread_name!("worker-thread {}", 2);
			zone!("work");
			std::thread::sleep(std::time::Duration::from_secs(1));
			zone!("inside work");
			std::thread::sleep(std::time::Duration::from_secs(1));
		});

		let p = std::thread::spawn(|| {
			set_thread_name!("plotter");
			zone!("plotting", Color::NAVY_BLUE);
			let pconfig = PlotConfig {
				format: PlotFormat::Number,
				style:  PlotStyle::Smooth,
				color:  Color::PEACH_PUFF1,
				filled: false,
			};
			let p = make_plot!("Number of keks", pconfig);
			for i in 0..100 {
				p.emit(100 - i);
				plot!("i", i);
				std::thread::sleep(std::time::Duration::from_millis(30));
			}
		});

		{
			zone!("kek");
			zone!("enabled",  enabled: true);
			zone!("disabled", enabled: false);
			zone!("r", Color::BISQUE1);
			zone!("g", Color::BISQUE2);
			zone!("b", Color::BISQUE3);
			std::thread::sleep(std::time::Duration::from_secs(2));
			zone!(zone, "valueable!");
			zone.text("i am text 1");
			zone.text("i am text 2");
			zone.text("i am text 3");
			zone.number(u64::MAX);
			zone.number(31337);
			zone.number(u64::MIN);
			std::thread::sleep(std::time::Duration::from_secs(1));
		}

		t.join().unwrap();
		d.join().unwrap();
		p.join().unwrap();
	}
}
