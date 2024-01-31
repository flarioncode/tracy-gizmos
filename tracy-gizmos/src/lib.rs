#![cfg_attr(docsrs, feature(doc_cfg), deny(rustdoc::broken_intra_doc_links))]
#![cfg_attr(any(doc, feature = "enabled"), deny(missing_docs))]
#![cfg_attr(not(feature = "enabled"), allow(unused_variables))]
#![cfg_attr(
	feature = "unstable-function-names",
	allow(incomplete_features),
	feature(const_type_name),
	feature(generic_const_exprs),
)]

//! Bindings for the client library of the
//! [Tracy](https://github.com/wolfpld/tracy) profiler.
//!
//! Refer to the Tracy manual for details and profiler server usage.
//!
//! # How to use
//!
//! Add `tracy-gizmos` to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! tracy-gizmos = { version = "0.0.1", features = ["enabled"] }
//! ```
//!
//! Note that instrumentation is *disabled* by default.
//!
//! The usage is pretty straight-forward (for more details read the docs):
//!
//! ```no_run
//! # fn work() { todo!() }
//! fn main() {
//!     let tracy = tracy_gizmos::start_capture();
//!     tracy_gizmos::zone!("main");
//!     work();
//! }
//! ```
//!
//! The [`#[instrument]`][instrument] attribute provies an easy way to
//! add Tracy zones to functions. A function annotated with
//! `#[instrument]` will create and enter a zone with that function's
//! name everytime the function is called.
//!
//! For example:
//!
//! ```ignore
//! # // this doctest is ignored, because it is impossible
//! # // to run it with cfg(feature = "attributes").
//! #[tracy_gizmos::instrument]
//! fn work() {
//!     // do stuff
//! }
//! ```
//!
//! You can find more examples showing how to use this crate
//! [here][examples].
//!
//! [examples]: https://github.com/den-mentiei/tracy-gizmos/tree/main/examples
//!
//! # Features
//!
//! - **`enabled`** - enables the instrumentation and everything
//! related to it.
//! - **`attributes`** - includes support for the `#[instrument]` attribute.
//! - **`unstable-function-names`** *(nightly only)* -
//! includes the enclosing function name into every zone without
//! additional runtime overhead.
//!
//! # Tracy features
//!
//! Tracy client functionality can be controlled pretty granularly.
//! Refer to the Tracy's manual for more details. A corresponding
//! define is listed for each feature, which can be used to search the
//! Tracy's documentation or source code, if needed.
//!
//! The following features are available:
//!
//! - **`crash-handler`** - enables Tracy's crash handler, which
//! intercepts application crashes and ensures the remaining profiling
//! data is sent to the server together with a crash report details.
//! Influences `TRACY_NO_CRASH_HANDLER`.
//! - **`system-tracing`** - enables system-level tracing information
//! collection (assuming that the profiled program was granted the
//! priveleges needed, e.g. run as root or Administrator). Influences
//! `TRACY_NO_SYSTEM_TRACING`.
//! - **`context-switch`** - enables context switch information
//! collection (assuming having the privelege, as above), which allows
//! to see when a zone was actually executed or was waiting to be
//! resumed. Influences `TRACY_NO_CONTEXT_SWITCH`.
//! - **`sampling`** - enables the callstack sampling to augment
//! instrumented data (requires privelege escalation on Windows).
//! Influences `TRACY_NO_SAMPLING`.
//! - **`callstack-inlines`** - enables the inline frames retrieval in
//! callstacks, which provides more precise information but is
//! magnitude slower. Influences `TRACY_NO_CALLSTACK_INLINES`.
//! - **`hw-counters`** - enables the hardware performance counters
//! sampling (available only on Linux or WSL): IPC, branch
//! mispredicts, cache misses. Influences
//! `TRACY_NO_SAMPLE_RETIREMENT`, `TRACY_NO_SAMPLE_BRANCH` and
//! `TRACY_NO_SAMPLE_CACHE`.
//! - **`code-transfer`** - enables the executable code retrieval,
//! which captures parts of the application code for further analysis
//! in Tracy. Be *extra careful* when working with non-public code!
//! Influences `TRACY_NO_CODE_TRANSFER`.
//! - **`vsync`** - enables the hardware Vsync events capture
//! (assuming having the privilege), which will be reported as frame
//! events per monitor. Influences `TRACY_NO_VSYNC_CAPTURE`.
//! - **`no-exit`** - enables the short-lived application profiling
//! improvement. When `TRACY_NO_EXIT` environment variable is set to
//! `1`, profiled application will wait for the server connection to
//! transfer the data, even if it has already finished executing.
//! Influences `TRACY_NO_EXIT`.
//! - **`broadcast`** - enables the local network announcement, so
//! profiling servers can find the client. Influences
//! `TRACY_NO_BROADCAST`.
//! - **`only-localhost`** *(enabled by default)* - restricts Tracy to
//! only listening on the localhost network interface. Influences
//! `TRACY_ONLY_LOCALHOST`.
//! - **`only-ipv4`** - restricts Tracy to only listenting on IPv4
//! network interfaces. Influences `TRACY_ONLY_IPV4`.

#[cfg(feature = "enabled")]
use std::sync::atomic::{AtomicBool, Ordering};
use std::marker::PhantomData;

#[cfg_attr(docsrs, doc(cfg(feature = "attributes")))]
#[doc(inline)]
#[cfg(feature = "attributes")]
pub use attrs::{instrument, capture};

mod color;
mod memory;
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
	($name:literal) => {
		// @Bug It doesn't work this way.
		#[cfg(feature = "enabled")]
		// SAFETY: We null-terminate the string.
		unsafe {
			$crate::details::set_thread_name(concat!($name, '\0').as_ptr());
		}
	};

	($format:literal, $($args:expr),*) => {
		// @Bug It doesn't work this way.
		#[cfg(feature = "enabled")]
		{
			let name = format!(concat!($format, '\0'), $($args),*).into_bytes();
			// SAFETY: We null-terminated the string during formatting.
			unsafe {
				let name = std::ffi::CString::from_vec_with_nul_unchecked(name);
				$crate::details::set_thread_name(name.as_ptr().cast());
			}
		}
	};
}

/// Sends a message to Tracy's log.
///
/// Fast navigation in large data sets and correlating zones with what
/// was happening in the application may be difficult. To ease these
/// issues, Tracy provides a message log functionality. You can send
/// messages (e.g. debug log output) using this macro.
///
/// # Examples
///
/// Just sending a message is straightforward:
///
/// ```no_run
/// # use tracy_gizmos::*;
/// message!("App started.");
/// ```
///
/// Optionally, a custom [`Color`] could be assigned to the message.
///
/// ```no_run
/// # use tracy_gizmos::*;
/// message!(Color::YELLOW, "App failed to find something.");
/// ```
///
/// ## Dynamic messages
///
/// It is also possible to use dynamic data as the message text.
///
/// The profiler will allocate a temporary internal buffer and copy
/// the passed value there. Hence, this operation involves a small
/// run-time cost.
///
/// Be aware that the passed text couldn't be larger than 64
/// Kb.
///
/// ```no_run
/// # use tracy_gizmos::*;
/// # let i = 0;
/// # let file_path = "file".to_string();
/// message!("Trying {}", i);
/// message!(&file_path);
/// message!(Color::GREEN, "{} is good!", file_path);
/// ```
#[macro_export]
#[cfg(any(doc, feature = "enabled"))]
macro_rules! message {
	($text:literal) => {
		// SAFETY: We null-terminate the string.
		unsafe {
			$crate::details::message(concat!($text, '\0').as_ptr());
		}
	};

	($text:expr) => {
		$crate::details::message_size($text);
	};

	($format:literal, $($args:expr),*) => {
		let _text = format!($format, $($args),*);
		$crate::details::message_size(&_text);
	};

	($color:expr, $text:literal) => {
		// SAFETY: We null-terminate the string.
		unsafe {
			$crate::details::message_color(
				concat!($text, '\0').as_ptr(),
				$color,
			);
		}
	};

	($color:expr, $text:expr) => {
		$crate::details::message_size_color(
			$text,
			$color,
		);
	};

	($color:expr, $format:literal, $($args:expr),*) => {
		let _text = format!($format, $($args),*);
		$crate::details::message_size_color(&_text, $color);
	};
}

#[macro_export]
#[cfg(all(not(doc), not(feature = "enabled")))]
macro_rules! message {
	($text:literal) => {};

	($whatever:expr $(, $text:literal)?) => {
		// Silences unused expression warning.
		_ = $whatever;
	};

	($format:literal, $($args:expr),*) => {
		// Silence unused expression warnings.
		$(
			_ = $args;
		)*
	};

	($color:expr, $text:expr) => {
		// Silence unused expression warnings.
		_ = $color;
		_ = $text;
	};

	($color:expr, $format:literal, $($args:expr),*) => {
		// Silence unused expression warnings.
		_ = $color;
		$(
			_ = $args;
		)*
	};
}

/// Marks the completed frame end moment.
///
/// Program's execution could happen in frame-sized chunks, e.g. a
/// rendering or I/O driven loop.
///
/// Note, that this is fully optional, as lots of applications do not
/// even us the concept of a frame.
///
/// Each frame starts *immediately* after previous one has ended.
///
/// Some types of frames are discontinuous by their natures - they are
/// executed periodically, with a pause between each run. E.g. a
/// physics processing step in a game loop or an audio or save
/// callback on a separate thread. Tracy can also track this kind of
/// frames.
///
/// Frame types *must not* be mixed. For each frame set, identified by
/// an unique name, use either a continuous or discontinuous frame
/// only!
///
/// Under the hood it declares a local [`Frame`].
///
/// # Examples
///
/// Here comes all possible and interesting cases of frame marking.
///
/// ## Main frame
///
/// As simple as:
///
/// ```no_run
/// # use tracy_gizmos::*;
/// # fn update() { todo!() }
/// # fn render() { todo!() }
/// loop {
///     update();
///     render();
///     frame!();
/// }
/// ```
///
/// ## Secondary frame sets
///
/// It works in the same way as the main frame, but requires a unique
/// name to identify this frame set:
///
/// ```no_run
/// # use tracy_gizmos::*;
/// # fn update()      { todo!() }
/// # fn render()      { todo!() }
/// # fn update_bots() { todo!() }
/// let ai = std::thread::spawn(|| {
///    loop {
///        update_bots();
///        frame!("ai");
///    }
/// });
///
/// loop {
///     update();
///     render();
///     frame!();
/// }
/// ```
///
/// ## Discontinuous frames
///
/// As discontinuous frame doesn't start immediately after previous
/// one has ended, you need to manually mark the frame's scope:
///
/// ```no_run
/// # use tracy_gizmos::*;
/// fn do_io_request() {
///     // This declares the `_io` variable containing a guard, which
///     // marks the frame end when dropped.
///     frame!(_io, "IO");
///
///     // do the I/O work.
/// }
/// ```
#[macro_export]
#[cfg(any(doc, feature = "enabled"))]
macro_rules! frame {
	() => {
		// SAFETY: Null pointer means main frame.
		unsafe {
			$crate::details::mark_frame_end(std::ptr::null());
		}
	};

	($name:literal) => {
		// SAFETY: We null-terminate the string.
		unsafe {
			$crate::details::mark_frame_end(concat!($name, '\0').as_ptr());
		}
	};

	($var:ident, $name:literal) => {
		#[allow(unused_variables)]
		// SAFETY: We null-terminate the string.
		let $var = unsafe {
			$crate::details::discontinuous_frame(concat!($name, '\0').as_ptr().cast())
		};
	};
}

#[macro_export]
#[cfg(all(not(doc), not(feature = "enabled")))]
macro_rules! frame {
	($($name:literal)? $($var:ident, $n:literal)?) => {
		// $var could be used to denote a lexically scoped frame or
		// even be manually `drop`-ed. Hence, we need to define it to
		// keep the macro-using code compilable.
		$(
			#[allow(unused_variables)]
			let $var = $crate::Frame();
		)?
	};
}

#[cfg(feature = "enabled")]
static STARTED: AtomicBool = AtomicBool::new(false);

/// Starts the Tracy capture.
///
/// Must be called *before* any other Tracy usage.
///
/// # Panics
///
/// Only one active capture can exist. Hence any consecutive
/// `start_capture()` will panic, unless previously started capture is
/// dropped.
///
/// # Examples
///
/// ```no_run
/// let _tracy = tracy_gizmos::start_capture();
/// ```
pub fn start_capture() -> TracyCapture {
	#[cfg(feature = "enabled")]
	{
		if STARTED.swap(true, Ordering::Acquire) {
			panic!("Tracy capture has been started already.");
		}
		// SAFETY: Check above ensures this happens once.
		unsafe {
			sys::___tracy_startup_profiler();
		}
	}

	TracyCapture(PhantomData)
}

/// Represents an active Tracy capture.
///
/// Obtaining a [`TracyCapture`] is *required* to instrument the code.
/// Otherwise, the behaviour of instrumenting is undefined.
///
/// It is not allowed to have multiple copies of the [`TracyCapture`].
///
/// When it is dropped, the Tracy connection will be shutdown, which
/// will also finish the capture.
pub struct TracyCapture(PhantomData<*mut ()>);

impl TracyCapture {
	/// Returns `true` if a connection is currently established with
	/// the Tracy server.
	///
	/// # Examples
	///
	/// This method can be used to ensure the profiler connection
	/// _before_ doing any profiled work.
	///
	/// ```no_run
	/// let tracy = tracy_gizmos::start_capture();
	/// while !tracy.is_connected() {
	///     std::thread::yield_now();
	/// }
	/// // You can do the profiling here knowing it will reach
	/// // Tracy.
	/// ```
	///
	/// You can also enabled `no-exit` feature instead, so
	/// [`TracyCapture`] will do a blocking wait for the profiling data
	/// to be transfered to the server, when dropped.
	pub fn is_connected(&self) -> bool {
		#[cfg(feature = "enabled")]
		// SAFETY: self could exist only if startup was issued and
		// succeeded.
		unsafe {
			sys::___tracy_connected() != 0
		}

		#[cfg(not(feature = "enabled"))]
		true
	}
}

#[cfg(feature = "enabled")]
impl Drop for TracyCapture {
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
/// Under the hood it declares a local [`Zone`].
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
/// # use tracy_gizmos::*;
/// fn do_stuff() {
///     zone!("stuff");
///     // now actually do stuff :-)
/// }
/// ```
///
/// Optionally, a custom [`Color`] could be assigned for to zone.
/// Note, that the color value will be constant in the recording.
///
/// ```no_run
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
/// # use tracy_gizmos::*;
/// # let file_path = "./main.rs";
/// zone!(parsing, "Parsing");
/// parsing.text(file_path);
/// ```
#[macro_export]
#[cfg(any(doc, feature = "enabled"))]
macro_rules! zone {
	(            $name:literal)                               => { $crate::zone!(_z,   $name, $crate::Color::UNSPECIFIED, enabled:true) };
	($var:ident, $name:literal)                               => { $crate::zone!($var, $name, $crate::Color::UNSPECIFIED, enabled:true) };
	(            $name:literal, $color:expr)                  => { $crate::zone!(_z,   $name, $color,                     enabled:true) };
	($var:ident, $name:literal, $color:expr)                  => { $crate::zone!($var, $name, $color,                     enabled:true) };
	(            $name:literal,              enabled:$e:expr) => { $crate::zone!(_z,   $name, $crate::Color::UNSPECIFIED, enabled:$e)   };
	($var:ident, $name:literal,              enabled:$e:expr) => { $crate::zone!($var, $name, $crate::Color::UNSPECIFIED, enabled:$e)   };
	(            $name:literal, $color:expr, enabled:$e:expr) => { $crate::zone!(_z,   $name, $color,                     enabled:$e)   };
	($var:ident, $name:literal, $color:expr, enabled:$e:expr) => {
		#[allow(unused_variables)]
		// SAFETY: This macro ensures that location & context data are correct.
		let $var = unsafe {
			$crate::details::zone($crate::zone!(@loc $name, $color), if $e {1} else {0})
		};
	};

	(@loc $name:literal, $color: expr) => {{
		// This is an implementation detail and can be changed at any moment.

		#[cfg(feature = "unstable-function-names")]
		struct X;
		#[cfg(feature = "unstable-function-names")]
		const FUNCTION: &'static [u8] = {
			&$crate::details::get_fn_name_from_nested_type::<X>()
		};

		#[cfg(not(feature = "unstable-function-names"))]
		const FUNCTION: &'static [u8] = b"<unavailable>\0";

		// SAFETY: All passed data is created here and is correct.
		static LOC: $crate::ZoneLocation = unsafe {
			$crate::details::zone_location(
				concat!($name, '\0'),
				FUNCTION,
				concat!(file!(), '\0'),
				line!(),
				$crate::Color::as_u32(&$color),
			)
		};
		&LOC
	}};
}

#[macro_export]
#[cfg(all(not(doc), not(feature = "enabled")))]
macro_rules! zone {
	($($var:ident,)? $name:literal, enabled:$e:expr) => {
		// Silences unused enabled expression warning.
		_ = $e;
		$crate::zone!($($var,)? $name, (), enabled:$e);
	};

	($($var:ident,)? $name:literal $(,$color:expr)? $(,enabled:$e:expr)?) => {
		// $var could be used to add dynamic zone data, so we need to
		// define it to keep the macro-using code compilable.
		$(
			#[allow(unused_variables)]
			let $var = $crate::Zone::new();
		)?
		// Silences unused `Color` import warning.
		$(
			_ = $color;
		)?
	};
}

/// Profiling zone.
///
/// Refer to [`zone!`] for the usage how-to.
///
/// It instruments the current scope. Hence, the profiling zone will
/// end when [`Zone`] is dropped.
pub struct Zone {
	#[cfg(feature = "enabled")]
	ctx:     sys::TracyCZoneCtx,
	_unsend: PhantomData<*mut ()>,
}

#[cfg(any(doc, feature = "enabled"))]
impl Drop for Zone {
	#[inline(always)]
	fn drop(&mut self) {
		#[cfg(feature = "enabled")]
		// SAFETY: The only way to have Zone is to construct it via
		// zone! macro, which ensures that ctx value is correct.
		unsafe {
			sys::___tracy_emit_zone_end(self.ctx);
		}
	}
}

impl Zone {
	#[doc(hidden)]
	#[cfg(not(feature = "enabled"))]
	pub fn new() -> Self {
		Self { _unsend: PhantomData }
	}

	/// Allows to control the zone color dynamically.
	///
	/// This can be called multiple times, however only the latest
	/// call will have an effect.
	pub fn color(&self, color: Color) {
		#[cfg(feature = "enabled")]
		// SAFETY: self always contains a valid `ctx`.
		unsafe {
			sys::___tracy_emit_zone_color(self.ctx, color.as_u32());
		}
		#[cfg(not(feature = "enabled"))]
		{
			// Silences unused import warning.
			_ = color;
		}
	}

	/// Adds a custom numeric value that will be displayed along with
	/// the zone information. E.g. a loop iteration or size of the
	/// processed buffer.
	///
	/// This method can be called multiple times, all of the passed
	/// values will be attached to the zone matching the call order.
	#[cfg(feature = "enabled")]
	pub fn number(&self, value: u64) {
		#[cfg(feature = "enabled")]
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
		#[cfg(feature = "enabled")]
		{
			debug_assert!(s.len() < u16::MAX as usize);
			// SAFETY: self always contains a valid `ctx`.
			unsafe {
				sys::___tracy_emit_zone_text(self.ctx, s.as_ptr().cast(), s.len())
			}
		}
	}
}

/// A statically allocated location for a profiling zone.
///
/// It is an implementation detail and can be changed at any moment.
#[doc(hidden)]
#[repr(transparent)]
pub struct ZoneLocation(#[cfg(feature = "enabled")] sys::___tracy_source_location_data);

// SAFETY: It is fully static and constant.
unsafe impl Send for ZoneLocation {}
unsafe impl Sync for ZoneLocation {}

/// Discontinuous frame.
///
/// Refer to [`frame!`] for usage howto.
///
/// It instruments the current frame scope. Hence, the discontinuous
/// frame will be marked as finished when [`Frame`] is dropped.
pub struct Frame(#[cfg(feature = "enabled")] *const i8);

#[cfg(any(doc, feature = "enabled"))]
impl Drop for Frame {
	#[inline(always)]
	fn drop(&mut self) {
		#[cfg(feature = "enabled")]
		// SAFETY: The only way to have Frame is to construct it via
		// frame! macro, which ensures that contained pointer is
		// correct.
		unsafe {
			sys::___tracy_emit_frame_mark_end(self.0);
		}
	}
}

/// Tracy can collect additional information about the profiled
/// application, which will be available in the trace description.
/// This can include data such as the source repository revision,
/// crate version, application's environment, etc.
///
/// This can be called multiple times. Tracy will accumulate all the
/// information and display it altogether.
///
/// Be aware that the passed text slice couldn't be larger than 64
/// Kb.
///
/// ```no_run
/// # use tracy_gizmos::*;
/// app_info("My fancy application");
/// app_info(env!("CARGO_PKG_VERSION"));
/// ```
#[inline(always)]
pub fn app_info(info: &str) {
	#[cfg(feature = "enabled")]
	{
		debug_assert!(info.len() < u16::MAX as usize);
		// SAFETY: Slice should contain valid data and having no
		// terminating zero is fine.
		unsafe {
			sys::___tracy_emit_message_appinfo(info.as_ptr().cast(), info.len());
		}
	}
}

/// Implementation details, do not relay on anything from this module!
///
/// It is public only due to the usage in public macro bodies.
#[doc(hidden)]
#[cfg(feature = "enabled")]
pub mod details {
	use std::ffi::c_void;
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

	#[inline(always)]
	pub unsafe fn message(text: *const u8) {
		sys::___tracy_emit_messageL(
			text.cast(),
			0, // callstack depth, 0 is disabled.
		);
	}

	#[inline(always)]
	pub fn message_size(text: &str) {
		debug_assert!(text.len() < u16::MAX as usize);
		// SAFETY: Dynamic non-zero-terminated string is fine.
		unsafe {
			sys::___tracy_emit_message(
				text.as_ptr().cast(),
				text.len(),
				0, // callstack depth, 0 is disabled.
			);
		}
	}

	#[inline(always)]
	pub fn message_size_color(text: &str, color: Color) {
		debug_assert!(text.len() < u16::MAX as usize);
		// SAFETY: Dynamic non-zero-terminated string is fine.
		unsafe {
			sys::___tracy_emit_messageC(
				text.as_ptr().cast(),
				text.len(),
				color.as_u32(),
				0, // callstack depth, 0 is disabled.
			);
		}
	}

	#[inline(always)]
	pub unsafe fn message_color(text: *const u8, color: Color) {
		sys::___tracy_emit_messageLC(
			text.cast(),
			color.as_u32(),
			0, // callstack depth, 0 is disabled.
		);
	}

	#[inline(always)]
	pub unsafe fn mark_frame_end(name: *const u8) {
		sys::___tracy_emit_frame_mark(name.cast());
	}

	#[inline(always)]
	pub unsafe fn discontinuous_frame(name: *const i8) -> Frame {
		sys::___tracy_emit_frame_mark_start(name);
		Frame(name)
	}

	#[inline(always)]
	pub unsafe fn track_alloc<T>(name: *const u8, ptr: *const T, size: usize) {
		track_alloc_impl(name, ptr.cast(), size);
	}

	#[inline(always)]
	unsafe fn track_alloc_impl(name: *const u8, ptr: *const c_void, size: usize) {
		sys::___tracy_emit_memory_alloc_named(ptr, size, 0, name.cast());
	}

	#[inline(always)]
	pub unsafe fn track_free<T>(name: *const u8, ptr: *const T) {
		track_free_impl(name, ptr.cast());
	}

	#[inline(always)]
	unsafe fn track_free_impl(name: *const u8, ptr: *const c_void) {
		sys::___tracy_emit_memory_free_named(ptr, 0, name.cast());
	}

	// Function name trick only works with an unstable
	// feature, which provides const `type_name`. Tracking
	// issue on the Rust side:
	// https://github.com/rust-lang/rust/issues/63084
	#[cfg(feature = "unstable-function-names")]
	pub const fn get_fn_name_from_nested_type<T>() -> [u8; std::any::type_name::<T>().len() - 2]
	where
		[(); std::any::type_name::<T>().len() - 2]:
	{
		let bytes   = std::any::type_name::<T>().as_bytes();
		// We skip (-3 + 1) of the type name length, to skip the '::X' suffix and add the terminating zero.
		let mut buf = [0; std::any::type_name::<T>().len() - 2];
		let n       = buf.len() - 1;
		let mut i   = 0;

		while i < n {
			buf[i] = bytes[i];
			i += 1;
		}

		buf
	}

	// Function above could be replaced by the following code directly
	// in the macro body, when const_type_name is stable.
	//
	// const FUNCTION: &'static [u8] = {
	// 	struct X;
	// 	const TYPE_NAME: &'static str = std::any::type_name::<X>();
	// 	// We skip 3 of the '::X' suffix and add 1 for the terminating zero.
	// 	const FUNCTION_LEN: usize = TYPE_NAME.len() - 3 + 1;
	// 	&$crate::details::as_array::<FUNCTION_LEN>(TYPE_NAME)
	// };
	// pub const fn as_array<const N: usize>(s: &'static str) -> [u8; N] {
	// 	let bytes   = s.as_bytes();
	// 	let mut buf = [0; N];
	// 	let mut i   = 0;
	// 	while i < N - 1 {
	// 		buf[i] = bytes[i];
	// 		i += 1;
	// 	}
	// 	buf
	// }
}

#[cfg(test)]
mod tests {
	#[cfg(feature = "enabled")]
    use super::*;

	#[cfg(feature = "enabled")]
	#[test]
	fn double_lifecycle() {
		let tracy = start_capture();
		drop(tracy);
		let _tracy = start_capture();
	}

	#[cfg(feature = "enabled")]
	#[test]
	#[should_panic]
	fn double_start_fails() {
		let _tracy1 = start_capture();
		let _tracy2 = start_capture();
	}
}
