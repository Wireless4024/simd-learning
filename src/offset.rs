pub const HEADER_COUNT_OFFSET: usize = 8; // u16
pub const HEADER_LENGTH_OFFSET: usize = 10; // u16
pub const METHOD_OFFSET: usize = 16;
pub const VERSION_OFFSET: usize = 24;
// start at 4096 due possible overlap during copy
pub const PATH_OFFSET: usize = 4096;
pub const HEADER_OFFSET: usize = 32;
// header value has 16bit length prefix, start from PATH_OFFSET+PATH_LENGTH+ 64 bytes padding