/// Marks a memory allocation event.
///
/// Tracy can monitor the memory usage of your application. Knowledge
/// about each performed memory allocation enables the following:
/// - memory usafe graph (like in massif, but interactive)
/// - list of active allocations at program exit
/// - memory map visualization
/// - ability to rewidne the view of active allocations
///   and memory map to any point of program exection
/// - memory statistics of each profiling zone
/// - memory allocation hot-spot tree.
///
/// A custom name is required for the allocation, which allows to
/// identify it later in the Trace visualization. It also allow to
/// support multiple memory allocators/pools/arenas.
///
/// # Danger
///
/// Each tracked memory-free event must also have a corresponding
/// allocation event. Tracy will terminate the profiling session if
/// this assumptions is broken. If this ever happens to you, one may
/// want to check for:
/// - mismatched alloc/free
/// - reporting same memory address allocated twise (without a free
///   in-between)
/// - double-free
/// - untracked external allocations, which are freed by application code
///
/// # Examples
///
/// ```no_run
/// # use tracy_gizmos::*;
/// # fn allocate(size: usize) -> *mut u8 { todo!() }
/// # let size: usize = 1024;
/// let mut buf = allocate(size);
/// emit_alloc!("scratch", buf, size);
/// // ... work with buf ...
/// emit_free!("scratch", buf);
/// ```
#[macro_export]
#[cfg(any(doc, feature = "enabled"))]
macro_rules! emit_alloc {
	($pool:literal, $ptr:expr, $size:expr) => {
		// SAFETY: We null-terminate the string.
		unsafe {
			$crate::details::track_alloc(
				concat!($pool, '\0').as_ptr(),
				$ptr,
				$size,
			);
		}
	};
}

#[macro_export]
#[cfg(all(not(doc), not(feature = "enabled")))]
macro_rules! emit_alloc {
	($pool:literal, $ptr:expr, $size:expr) => {
		// Silences unused enabled expression warnings.
		_ = $ptr;
		_ = $size;
	};
}

/// Marks a memory freeing event.
///
/// Tracy can monitor the memory usage of your application. Knowledge
/// about each performed memory allocation enables the following:
/// - memory usafe graph (like in massif, but interactive)
/// - list of active allocations at program exit
/// - memory map visualization
/// - ability to rewidne the view of active allocations
///   and memory map to any point of program exection
/// - memory statistics of each profiling zone
/// - memory allocation hot-spot tree.
///
/// A custom name is required for the allocation, which allows to
/// identify it later in the Trace visualization. It also allow to
/// support multiple memory allocators/pools/arenas.
///
/// # Danger
///
/// Each tracked memory-free event must also have a corresponding
/// allocation event. Tracy will terminate the profiling session if
/// this assumptions is broken. If this ever happens to you, one may
/// want to check for:
/// - mismatched alloc/free
/// - reporting same memory address allocated twise (without a free
///   in-between)
/// - double-free
/// - untracked external allocations, which are freed by application code
///
/// # Examples
///
/// ```no_run
/// # use tracy_gizmos::*;
/// # fn allocate(size: usize) -> *mut u8 { todo!() }
/// # let size: usize = 1024;
/// let mut buf = allocate(size);
/// emit_alloc!("scratch", buf, size);
/// // ... work with buf ...
/// emit_free!("scratch", buf);
/// ```
#[macro_export]
#[cfg(any(doc, feature = "enabled"))]
macro_rules! emit_free {
	($pool:literal, $ptr:expr) => {
		// SAFETY: We null-terminate the string.
		unsafe {
			$crate::details::track_free(
				concat!($pool, '\0').as_ptr(),
				$ptr,
			);
		}
	};
}

#[macro_export]
#[cfg(all(not(doc), not(feature = "enabled")))]
macro_rules! emit_free {
	($pool:literal, $ptr:expr) => {
		// Silences unused enabled expression warning.
		_ = $ptr;
	};
}

/// Implementation details, do not relay on anything from this module!
///
/// It is public only due to the usage in public macro bodies.
#[doc(hidden)]
#[cfg(feature = "enabled")]
pub mod details {
}
