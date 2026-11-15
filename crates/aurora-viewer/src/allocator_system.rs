//! fork: what stands in for allocator.rs without the `mimalloc` feature, and
//! always in the library: no global allocator is set, the system's is used.

/// The allocator in use, for the log.
pub fn name() -> &'static str {
    "system"
}
