/// Read a `bool` value from a `$table` expression at the offset `$offset`.
macro_rules! read_bool {
    ($table: ident, $offset: expr) => {
        u8::from_le($table[$offset]) != 0
    };
}

/// Read a `u8` value from a `$table` expression at the offset `$offset`.
macro_rules! read_u8 {
    ($table: ident, $offset: expr) => {
        u8::from_le($table[$offset])
    };
}

/// Read a `u16` value from a `$table` expression at the offset `$offset`.
macro_rules! read_u16 {
    ($table: ident, $offset: expr) => {
        u16::from_le_bytes($table[$offset..($offset) + 2].try_into().unwrap())
    };
}

/// Read an `i32` value from a `$table` expression at the offset `$offset`.
// macro_rules! read_i32 {
//     ($table: ident, $offset: expr) => {
//         i32::from_le_bytes($table[$offset..($offset) + 4].try_into().unwrap())
//     };
// }

/// Read an `f32` value from a `$table` expression at the offset `$offset`.
macro_rules! read_f32 {
    ($table: ident, $offset: expr) => {
        f32::from_le_bytes($table[$offset..($offset) + 4].try_into().unwrap())
    };
}

pub(crate) use {
    read_bool,
    read_f32,
    // read_i32,
    read_u16,
    read_u8,
};
