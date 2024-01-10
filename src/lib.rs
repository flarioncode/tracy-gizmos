// @Cleanup Feature-gate this for the nightly enjoyers?
#![feature(const_type_name)] // :UnstableTypeName

use std::marker::PhantomData;

// @Incomplete

// @Incomplete Add examples.
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
				let name = format!(concat!($format, '\0'), $args);
				// @Incomplete Explain this.
				let static_name: &'static str = name.leak();
				// SAFETY: We null-terminated the string during formatting.
				unsafe {
					$crate::details::set_thread_name(static_name.as_ptr());
				}
			}
		)*
	};
}

// @Safety We can have an atomic usize with init thread id to prevent
// multiple inits and also ensure the same thread does the shutdown. :Lifetime
/// Initializes the Tracy profiler.
///
/// Must be called *before* any other Trace usage.
pub unsafe fn startup() {
	sys::___tracy_startup_profiler();
}

// @Safety :Lifetime
/// Shutdowns the Tracy profiler.
///
/// Any Tracy usage after this is prohibited.
pub unsafe fn shutdown() {
	sys::___tracy_shutdown_profiler();
}

/// Determines if a connection is currently established with the Tracy
/// server.
pub fn is_connected() -> bool {
	unsafe {
		sys::___tracy_connected() != 0
	}
}

// auto-function proc-macro attributes:
// #[zone]
// #[zone(name)]
// #[zone(color)]
// #[zone(name, color)]
// + callstacks?!
// fn foo() {}

// macro_rules in the scopes:
// - n/a function, name and no color
// - n/a function, name and color
#[macro_export]
macro_rules! zone {
	($name: literal) => {
		let _loc  = zone_location!(0);
		let _ctx  = unsafe { sys::___tracy_emit_zone_begin(&_loc.data, 1) };
		let _zone = Zone { ctx: _ctx, _unsend: PhantomData };
	};

	($name: literal, $color: expr) => {
		let _loc  = zone_location!($color);
		let _ctx  = unsafe { sys::___tracy_emit_zone_begin(&_loc.data, 1) };
		let _zone = Zone { ctx: _ctx, _unsend: PhantomData };
	};

	(c) => {{
		let _loc  = zone_location!(0);
		let _ctx  = unsafe { sys::___tracy_emit_zone_begin(&_loc.data, 1) };
		Zone { ctx: _ctx, _unsend: PhantomData }
	}};
}

/// Profiling zone.
///
/// The profiled zone will end when it is dropped.
// scoped_zone
// scoped_zone with const color
pub struct Zone {
	ctx: sys::___tracy_c_zone_context,

	_unsend: PhantomData<*mut ()>,
}

// @Incomplete
// - dynamic name?
// - dynamic color       - last one wins
// - dynamic text        - can be multiples (u16::MAX bytes)
// - dynamic value (u64) - can be multiples
impl Zone {
	pub fn value(&self, value: u64) {
		unsafe {
			sys::___tracy_emit_zone_value(self.ctx, value);
		}
	}

	pub fn text(&self, s: &str) {
		unsafe {
			sys::___tracy_emit_zone_text(self.ctx, s.as_ptr().cast(), s.len())
		}
	}
}

impl Drop for Zone {
	#[inline(always)]
	fn drop(&mut self) {
		// SAFETY: We constructed it properly.
		unsafe {
			sys::___tracy_emit_zone_end(self.ctx);
		}
	}
}

/// A statically allocated location for a zone.
pub struct ZoneLocation {
	data: sys::___tracy_source_location_data,
}

macro_rules! zone_location {
	($color: expr) => {{
		struct X;
		// Tracking issue on the Rust side:
		// https://github.com/rust-lang/rust/issues/63084
		// :UnstableTypeName
		const TYPE_NAME: &'static str = std::any::type_name::<X>();
		// We skip 3 of the '::X' suffix and add 1 for the terminating zero.
		const FUNCTION_LEN: usize = TYPE_NAME.len() - 3 + 1;
		const FUNCTION: &'static [u8] = &details::as_array::<FUNCTION_LEN>(TYPE_NAME);

		static LOC: $crate::ZoneLocation = ZoneLocation {
			data: sys::___tracy_source_location_data {
				name:     concat!("TEST",    '\0').as_ptr().cast(),
				function: FUNCTION.as_ptr().cast(),
				file:     concat!(file!(), '\0').as_ptr().cast(),
				line:     line!(),
				color:    $color,
			},
		};
		&LOC
	}};
}

unsafe impl Send for ZoneLocation {}
unsafe impl Sync for ZoneLocation {}

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

// color is 0xRRGGBB
// predefined colours ( https://en.wikipedia.org/wiki/X11_color_names)
// 0 is not black, it is no-color. 1 is close enough.

// mark_frame at the end vs frame_scope?
// ^ it is optional though

// discontinuous frames aka frame start/end pair with same name pointer

// zone name can be text+size, but won't be stat aggregated

// optional scopes
// - compile-time togglable
// - dynamically togglable

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
	fn playground() {
		unsafe { startup(); }
		while !is_connected() {
			std::thread::yield_now();
		}
		// @Bug This does not work now.
		// let x = 42;
		// set_thread_name!("Worker {}", x);

		{
			let z = zone!(c);
			z.value(1);
			z.value(1);
			z.value(123);
			z.text("hello, sailor");
			z.text("cowabunga!");
			{
				zone!("1st");
				std::thread::sleep(std::time::Duration::from_secs(2));
				zone!("2nd", 0x001230);
				std::thread::sleep(std::time::Duration::from_secs(1));
			}

			{
				zone!("3rd", 0x005015);
				std::thread::sleep(std::time::Duration::from_secs(2));
			}
		}

		unsafe { shutdown(); }
	}
}
