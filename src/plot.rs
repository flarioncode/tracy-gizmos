//! @Incomplete Document this.

use std::ffi::CStr;

use crate::Color;

/// @Incomplete Document this.
///
/// Tracy can capture and draw numeric value changes over time. You
/// may use it to analyze draw call counts, number of performed
/// queries, high-mark of temp arena memory, etc.
#[repr(transparent)]
pub struct Plot(&'static CStr);

/// @Incomplete document this.
#[macro_export]
macro_rules! plot {
	($name:literal, $value:expr) => {
	};
}

impl Plot {
	/// @Incomplete Document this.
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
}

/// @Incomplete Document this.
pub trait PlotEmit<T> {
	/// @Incomplete Document this.
	fn emit(&self, value: T);
}

macro_rules! impl_emit {
	($ty:ident, $with:ident) => {
		impl PlotEmit<$ty> for Plot {
			#[inline(always)]
			fn emit(&self, value: $ty) {
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

/// @Incomplete Document this.
#[derive(Debug, Clone, Copy)]
pub struct PlotConfig {
	/// @Incomplete Document this.
	pub format: PlotFormat,
	/// @Incomplete Document this.
	pub style:  PlotStyle,
	/// @Incomplete Document this.
	pub color:  Color,
	/// @Incomplete Document this.
	pub filled: bool,
}

/// An enum representing the plot values display format.
///
/// Typical usage is to pass this to `plot_config` @Incomplete Finish
/// this comment to specify the actual usage.
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

/// An enum representing the plot style.
///
/// @Incomplete Add typical usage note.
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
