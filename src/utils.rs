/// Converts a byte slice to a big-endian u32.
///
/// # Panics
/// Panics if the slice has fewer than 4 bytes.
#[inline(always)]
pub fn slice_as_u32_be(array: &[u8]) -> u32 {
    u32::from_be_bytes([array[0], array[1], array[2], array[3]])
}

/// Converts a 4-byte array to a big-endian u32.
#[inline(always)]
pub fn as_u32_be(array: &[u8; 4]) -> u32 {
    u32::from_be_bytes(*array)
}

/// Converts a 4-byte array to a little-endian u32.
#[inline(always)]
pub fn as_u32_le(array: &[u8; 4]) -> u32 {
    u32::from_le_bytes(*array)
}
