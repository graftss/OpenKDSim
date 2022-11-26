/// Log a formatted strings.
macro_rules! log {
    ($($y: expr),+) => {
        crate::util::debug_log(&format!($($y),+));
    }
}

/// Log a formatted string, then panic.
macro_rules! panic_log {
    ($($y: expr),+) => {
        crate::util::debug_log(&format!($($y),+));
        panic!();
    }
}

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

macro_rules! max_to_none {
    ($ty: ty, $value: expr) => {
        if ($value) == <$ty>::MAX {
            None
        } else {
            Some($value)
        }
    };
}

macro_rules! new_mat4_copy {
    ($ident: ident, $src: expr) => {
        let mut $ident = [0.0; 16];
        mat4::copy(&mut $ident, &$src);
    };
}

macro_rules! max {
    ($expr0: expr, $expr1: expr) => {
        if ($expr0) > ($expr1) {
            $expr0
        } else {
            $expr1
        }
    };
}

macro_rules! min {
    ($expr0: expr, $expr1: expr) => {
        if ($expr0) < ($expr1) {
            $expr0
        } else {
            $expr1
        }
    };
}

macro_rules! inv_lerp {
    ($value: expr, $min: expr, $max: expr) => {
        (($value) - ($min)) / (($max) - ($min))
    };
}

macro_rules! inv_lerp_clamp {
    ($value: expr, $min: expr, $max: expr) => {
        if ($value) <= ($min) {
            0.0
        } else if ($value) >= ($max) {
            1.0
        } else {
            (($value) - ($min)) / (($max) - ($min))
        }
    };
}

pub(crate) use {
    inv_lerp,
    inv_lerp_clamp,
    log,
    max,
    max_to_none,
    min,
    new_mat4_copy,
    panic_log,
    read_bool,
    read_f32,
    // read_i32,
    read_u16,
    read_u8,
};
