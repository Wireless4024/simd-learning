pub const MAX_METHOD_LENGTH: usize = 8;
pub const MAX_VERSION_LENGTH: usize = 8;
pub const MAX_PATH_LENGTH: usize = 4096 - 64;// start at 32, + 2 for length prefix
pub const MAX_HEADER_LENGTH: usize = 32;
//pub const MAX_HEADER_VALUE_LENGTH: usize = ?; : 16bit length prefix