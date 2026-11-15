//! The few `meshopt` calls of this crate, on meshopt-pure: the Rust port of
//! meshoptimizer 1.3 (the version the C++ `meshopt` crate wrapped), with the
//! names and signatures the callers already use. Its results are the C++
//! library's, index for index.

use std::ops::BitOr;

pub trait DecodePosition {
    fn decode_position(&self) -> [f32; 3];
}

/// `meshopt_SimplifyX` bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimplifyOptions(u32);

#[allow(non_upper_case_globals, reason = "the names of the meshopt crate")]
impl SimplifyOptions {
    pub const None: Self = Self(0);
    pub const LockBorder: Self = Self(meshoptimizer::SIMPLIFY_LOCK_BORDER);
    pub const Prune: Self = Self(meshoptimizer::SIMPLIFY_PRUNE);
    pub const Permissive: Self = Self(meshoptimizer::SIMPLIFY_PERMISSIVE);
}

impl BitOr for SimplifyOptions {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

fn positions<T: DecodePosition>(vertices: &[T]) -> Vec<f32> {
    vertices.iter().flat_map(|v| v.decode_position()).collect()
}

/// The index buffer a call wrote, or none when it refused its input (the
/// C++ library asserts there): indices out of range, a count not of triangles.
fn written(mut result: Vec<u32>, count: meshoptimizer::Result<usize>) -> Vec<u32> {
    result.truncate(count.unwrap_or(0));
    result
}

pub fn optimize_vertex_cache(indices: &[u32], vertex_count: usize) -> Vec<u32> {
    let mut result = vec![0; indices.len()];
    let done = meshoptimizer::optimize_vertex_cache(&mut result, indices, indices.len(), vertex_count);
    written(result, done.map(|()| indices.len()))
}

pub fn simplify_decoder<T: DecodePosition>(
    indices: &[u32],
    vertices: &[T],
    target_count: usize,
    target_error: f32,
    options: SimplifyOptions,
    result_error: Option<&mut f32>,
) -> Vec<u32> {
    let mut result = vec![0; indices.len()];
    let count = meshoptimizer::simplify(
        &mut result,
        indices,
        indices.len(),
        &positions(vertices),
        vertices.len(),
        12,
        target_count,
        target_error,
        options.0,
        result_error,
    );
    written(result, count)
}

pub fn simplify_sloppy_decoder<T: DecodePosition>(
    indices: &[u32],
    vertices: &[T],
    target_count: usize,
    target_error: f32,
    result_error: Option<&mut f32>,
) -> Vec<u32> {
    let mut result = vec![0; indices.len()];
    let count = meshoptimizer::simplify_sloppy(
        &mut result,
        indices,
        indices.len(),
        &positions(vertices),
        vertices.len(),
        12,
        None,
        target_count,
        target_error,
        result_error,
    );
    written(result, count)
}
