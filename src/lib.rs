// @Cleanup Feature-gate this for the nightly enjoyers?
#![feature(const_type_name)] // :UnstableTypeName
#![warn(missing_docs)]

//! @Incomplete Document this or attach the readme.

use std::sync::atomic::{AtomicBool, Ordering};
use std::marker::PhantomData;

mod color;

pub use color::*;

/// Sets the current thread's name.
///
/// It is recommended to *always* use it in every thread, which uses
/// Tracy capabilities.
///
/// If not set, Tracy will try to capture thread names through
/// operating system data if context switch capture is active.
/// However, this is only a fallback mechanism, and it shouldn't be
/// relied upon.
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
	/// Returns `true` if the profiling is enabled.
	///
	/// Result of this is determined during compile-time and will
	/// never change after that.
	pub const fn is_enabled() -> bool {
		cfg!(feature = "enabled")
	}

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

// auto-function proc-macro attributes:
// #[zone]
// #[zone(name)]
// #[zone(color)]
// #[zone(name, color)]
// + callstacks?! + enabled
// fn foo() {}

/// @Incomplete Document this.
#[macro_export]
macro_rules! zone {
	($name:literal)                                           => { zone!(_z,   $name, 0,      enabled:true) };
	($name:literal, $color:expr)                              => { zone!(_z,   $name, $color, enabled:true) };
	($name:literal,              enabled:$e:expr)             => { zone!(_z,   $name, 0,      enabled:$e)   };
	($name:literal, $color:expr, enabled:$e:expr)             => { zone!(_z,   $name, $color, enabled:$e)   };
	($var:ident, $name:literal)                               => { zone!($var, $name, 0,      enabled:true) };
	($var:ident, $name:literal, $color:expr)                  => { zone!($var, $name, $color, enabled:true) };
	($var:ident, $name:literal,              enabled:$e:expr) => { zone!($var, $name, 0,      enabled:$e)   };
	($var:ident, $name:literal, $color:expr, enabled:$e:expr) => {
		let _loc  = zone_location!($name, $color);
		let _ctx  = unsafe { sys::___tracy_emit_zone_begin(&_loc._data, if $e {1} else {0}) };
		let $var = Zone { ctx: _ctx, _unsend: PhantomData };
	};
}

/// Profiling zone.
///
/// The profiled zone will end when it is dropped.
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
	pub fn color(&self, _value: u32) {
		todo!()
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

// A statically allocated location for a profiling zone.
// It is an implementation detail and can be changed at any moment.
#[doc(hidden)]
#[repr(transparent)]
pub struct ZoneLocation {
	_data: sys::___tracy_source_location_data,
}

// SAFETY: It is fully static and constant.
unsafe impl Send for ZoneLocation {}
unsafe impl Sync for ZoneLocation {}

// @Cleanup Make this a part of the zone macro?
// It is an implementation detail and can be changed at any moment.
#[doc(hidden)]
#[macro_export]
macro_rules! zone_location {
	($name:literal, $color: expr) => {{
		struct X;
		// Tracking issue on the Rust side:
		// https://github.com/rust-lang/rust/issues/63084
		// :UnstableTypeName
		const TYPE_NAME: &'static str = std::any::type_name::<X>();
		// We skip 3 of the '::X' suffix and add 1 for the terminating zero.
		const FUNCTION_LEN: usize = TYPE_NAME.len() - 3 + 1;
		const FUNCTION: &'static [u8] = &details::as_array::<FUNCTION_LEN>(TYPE_NAME);

		static LOC: $crate::ZoneLocation = ZoneLocation {
			_data: sys::___tracy_source_location_data {
				name:     concat!($name, '\0').as_ptr().cast(),
				function: FUNCTION.as_ptr().cast(),
				file:     concat!(file!(), '\0').as_ptr().cast(),
				line:     line!(),
				color:    $color,
			}
		};
		&LOC
	}};
}

/// Implementation details, do not relay on anything from this module!
///
/// It is public only due to the usage in public macro bodies.
#[doc(hidden)]
pub mod details {
	#[inline(always)]
	pub unsafe fn set_thread_name(name: *const u8) { // @Cleanup Could be a &'static CStr instead?
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

// predefined colours ( https://en.wikipedia.org/wiki/X11_color_names)
// 0 is not black, it is no-color. 1 is close enough.

// mark_frame at the end vs frame_scope?
// ^ it is optional though

// discontinuous frames aka frame start/end pair with same name pointer

// what's up with locks & C API?
// what's up with alloc & free? named overloads?
// what's up with gfx stuff?

// plot number
// plot memory sizes
// plot percentages
// plot has name, color
// plot is step or linear
// default plot setup

// TracyMessageL gets static
// TracyMessage(text, size) no terminating zero and can't be larger than 64 Kb. will be copied.

// tracy app info (text, size)

// callstacks! depth is at most 62 could be disabled with TRACY_NO_CALLSTACK, TRACY_NO_CALLSTACK_INLINES

// dbghelp thread-safety

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
		assert_eq!(TracyClient::is_enabled(), cfg!(feature = "enabled"));

		let tracy = TracyClient::start();
		while !tracy.is_connected() {
			std::thread::yield_now();
		}

		set_thread_name!("main-thread");
		// @Bug This does not work now.
		// let x = 42;
		// set_thread_name!("Worker {}", x);
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

		{
			zone!("kek");
			zone!("enabled",  enabled: true);
			zone!("disabled", enabled: false);
			zone!("r", 0xFF0000);
			zone!("g", 0x00FF00);
			zone!("b", 0x0000FF);
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
	}
}
