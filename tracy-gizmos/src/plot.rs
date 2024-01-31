use std::ffi::CStr;

use crate::Color;

/// Takes a value, emits it into the specific plot and returns the
/// value back.
///
/// Supported value types are: `i64`, `f64`, `f32`.
///
/// Invoking the macro on an expression moves and takes ownership of
/// it before returning the evaluated expression unchanged. As all
/// supported types implement `Copy`, you can stop caring about
/// ownership.
///
/// # Examples
/// ```no_run
/// # use tracy_gizmos::*;
/// # let mut draw_calls = 0;
/// # fn get_size() -> i64 { todo!() }
/// draw_calls += 1;
/// plot!("Draw calls", draw_calls);
/// let size = plot!("Current Size", get_size());
/// ```
#[macro_export]
macro_rules! plot {
	($name:literal, $value:expr) => {
		// match works as `let .. in` and is required to properly
		// manage lifetimes.
		match $value {
			tmp => {
				use $crate::PlotEmit;
				$crate::Plot::new(
					// SAFETY: We null-terminate the string.
					unsafe {
						std::ffi::CStr::from_bytes_with_nul_unchecked(concat!($name, '\0').as_bytes())
					},
				).emit(tmp);
				tmp
			}
		}
	};

	($plot:ident, $value:expr) => {{
		// match works as `let .. in` and is required to properly
		// manage lifetimes.
		match $value {
			tmp => {
				use $crate::PlotEmit;
				$plot.emit(tmp);
				tmp
			}
		}
	}};
}

/// Creates and configures the plot.
///
/// If you are fine with the plot defaults, you can just use [`plot`].
///
/// Value could be emitted to the plot after that.
///
/// # Examples
/// ```no_run
/// # use tracy_gizmos::*;
/// let plot_config = PlotConfig { filled: true, ..Default::default() };
/// make_plot!(draws, "Draw calls", plot_config);
/// plot!(draws, 10);
/// ```
#[macro_export]
#[cfg(any(doc, feature = "enabled"))]
macro_rules! make_plot {
	($var:ident, $name:literal, $config:expr) => {
		#[allow(unused_variables)]
		let $var = $crate::Plot::with_config(
			// SAFETY: We null-terminate the string.
			unsafe {
				std::ffi::CStr::from_bytes_with_nul_unchecked(concat!($name, '\0').as_bytes())
			},
			$config
		);
	};
}

#[macro_export]
#[cfg(all(not(doc), not(feature = "enabled")))]
macro_rules! make_plot {
	($var:ident, $name:literal, $config:expr) => {
		// $var could be used with further `plot!` emissions,
		// define it to keep the macro-using code compilable.
		#[allow(unused_variables)]
		let $var = $crate::Plot();
		// Silences unused `Plot*` imports warning.
		_ = $config;
	};
}

/// Plotting mechanism.
///
/// Tracy can capture and draw numeric value changes over time. You
/// may use it to analyze draw call counts, number of performed
/// queries, high-mark of temp arena memory, etc.
///
/// Take a look into [`plot`], [`make_plot`].
///
/// # Examples
///
/// Unique plot identifier, which can be used to emit values for the
/// plot.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Plot(#[cfg(feature = "enabled")] &'static CStr);

#[doc(hidden)]
impl Plot {
	#[inline(always)]
	pub const fn new(name: &'static CStr) -> Self {
		Self(#[cfg(feature = "enabled")] name)
	}

	#[inline(always)]
	pub fn with_config(name: &'static CStr, config: PlotConfig) -> Self {
		#[cfg(feature = "enabled")]
		// SAFETY: `PlotConfig` ensures values are correct.
		unsafe {
			sys::___tracy_emit_plot_config(
				name.as_ptr(),
				config.format as i32,
				config.style  as i32,
				config.filled as i32,
				config.color .as_u32(),
			);
		}

		Self(#[cfg(feature = "enabled")] name)
	}
}

/// The `PlotEmit` trait allows for value emission into a plot.
///
/// It is used to get overloading for `emit`s with the supported value
/// types.
pub trait PlotEmit<T> {
	/// Emits a value.
	fn emit(&self, value: T);
}

macro_rules! impl_emit {
	($ty:ident, $with:ident) => {
		impl PlotEmit<$ty> for Plot {
			#[inline(always)]
			fn emit(&self, value: $ty) {
				#[cfg(feature = "enabled")]
				// SAFETY: `Plot` creation ensures the name correctness.
				unsafe {
					sys::$with(self.0.as_ptr(), value);
				}
			}
		}
	};
}

// Yay, overloading.
impl_emit!(f64, ___tracy_emit_plot);
impl_emit!(f32, ___tracy_emit_plot_float);
impl_emit!(i64, ___tracy_emit_plot_int);

/// A plot configuration, which controls the way plot will be
/// displayed.
#[derive(Debug, Clone, Copy)]
pub struct PlotConfig {
	/// Format controls how plot values are displayed.
	pub format: PlotFormat,
	/// Style controls how plot lines are displayed.
	pub style:  PlotStyle,
	/// Color of the plot.
	pub color:  Color,
	/// If `true`, the area below the plot will be filled with a solid
	/// color.
	pub filled: bool,
}

impl Default for PlotConfig {
	fn default() -> Self {
		Self {
			format: PlotFormat::Number,
			style:  PlotStyle::Smooth,
			color:  Color::UNSPECIFIED,
			filled: false,
		}
	}
}

/// An enum representing the plot values display format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum PlotFormat {
	/// Values will be displayed as plain numbers.
	Number = 0,
	/// Treats the values as memory sizes.
	///
	/// Tracy will display kilobytes, megabytes, etc.
	Memory = 1,
	/// Values will be displayed as percentage.
	///
	/// With a value of `100` being equal to 100%.
	Percentage = 2,
	/// Values will be displayed as watts.
	///
	/// E.g. `5` will be displayed as `5 W`.
	Watts = 3,
}

#[cfg(feature = "enabled")]
mod check_plot_format {
	macro_rules! const_assert {
		($x:expr $(,)?) => {
			const _: [(); 0 - !{ const ASSERT: bool = $x; ASSERT } as usize] = [];
		};
	}

	use super::PlotFormat::*;
	const_assert!(Number     as i32 == sys::TracyPlotFormatNumber);
	const_assert!(Memory     as i32 == sys::TracyPlotFormatMemory);
	const_assert!(Percentage as i32 == sys::TracyPlotFormatPercentage);
	const_assert!(Watts      as i32 == sys::TracyPlotFormatWatt);
}

/// An enum representing the plot style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum PlotStyle {
	/// Plot will smoothly change between the points.
	///
	/// ```text
	///                   o
	///                  / \
	///                 /   \
	///       o        /     \
	///      | \      |       |
	///      /  \     /       \
	///     /    \   /         o
	///    |      \ /
	///    /       o
	///   |
	///   o
	/// ```
	Smooth = 0,
	/// Plot will be displayed as a staircase aka step function.
	///
	/// ```text
	///                   o----+
	///                   |    |
	///                   |    |
	///       o----+      |    |
	///       |    |      |    |
	///       |    |      |    |
	///       |    |      |    o-----
	/// --+   |    |      |
	///   |   |    o------+
	///   |   |
	///   o---+
	/// ```
	Staircase = 1,
}
