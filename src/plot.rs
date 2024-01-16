use std::ffi::CStr;

use crate::Color;

/// Emits a new value for the specific plot.
///
/// Supported value types are: `i64`, `f64`, `f32`.
///
/// # Examples
/// ```no_run
/// # use tracy_gizmos::*;
/// # let mut draw_calls = 0;
/// draw_calls += 1;
/// plot!("Draw calls", draw_calls);
/// ```
#[macro_export]
macro_rules! plot {
	($name:literal, $value:expr) => {
		#[cfg(feature = "enabled")]
		{
			use $crate::PlotEmit;
			$crate::Plot::new(
				// SAFETY: We null-terminate the string.
				unsafe {
					std::ffi::CStr::from_bytes_with_nul_unchecked(concat!($name, '\0').as_bytes())
				},
			).emit($value);
		}
		#[cfg(not(feature = "enabled"))]
		{
			_ = $value;
		}
	};
}

/// Creates and configures the plot.
///
/// It allows to create a plot and configure it.
/// If you are fine with the plot defaults, you can just use [`plot`].
///
/// Value could be emitted to the plot after that.
///
/// # Examples
/// ```no_run
/// # use tracy_gizmos::*;
/// let plot_config = PlotConfig { filled: true, ..Default::default() };
/// let draw_calls  = make_plot!("Draw calls", plot_config);
/// draw_calls.emit(10);
/// ```
#[macro_export]
macro_rules! make_plot {
	($name:literal, $config:expr) => {
		$crate::Plot::with_config(
			// SAFETY: We null-terminate the string.
			unsafe {
				std::ffi::CStr::from_bytes_with_nul_unchecked(concat!($name, '\0').as_bytes())
			},
			$config
		)
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
	#[cfg(feature = "enabled")]
	pub fn new(name: &'static CStr) -> Self {
		Self(name)
	}
	#[inline(always)]
	#[cfg(not(feature = "enabled"))]
	pub fn new(_: &'static CStr) -> Self {
		Self()
	}

	#[inline(always)]
	#[cfg(feature = "enabled")]
	pub fn with_config(name: &'static CStr, config: PlotConfig) -> Self {
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

		Self(name)
	}
	#[inline(always)]
	#[cfg(not(feature = "enabled"))]
	pub fn with_config(_: &'static CStr, _: PlotConfig) -> Self {
		Self()
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
			#[cfg(feature = "enabled")]
			#[inline(always)]
			fn emit(&self, value: $ty) {
				// SAFETY: `Plot` creation ensures the name correctness.
				unsafe {
					sys::$with(self.0.as_ptr(), value);
				}
			}

			#[cfg(not(feature = "enabled"))]
			#[inline(always)]
			fn emit(&self, _: $ty) {
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
