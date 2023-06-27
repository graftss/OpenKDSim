/// Log a formatted string to the main debug log file. Used temporarily while debugging.
#[allow(unused_macros)]
macro_rules! debug_log {
    ($($y: expr),+) => {
        crate::util::debug_log(&format!($($y),+));
    }
}

/// Log a formatted string to the main debug log file. Used temporarily while debugging.
#[allow(unused_macros)]
macro_rules! temp_debug_log {
    ($($y: expr),+) => {
        crate::util::debug_log(&format!($($y),+));
    }
}

/// Log a formatted string to an arbitrary file. Used temporarily while debugging.
#[allow(unused_macros)]
macro_rules! temp_debug_write {
    ($path: expr, $($y: expr),+) => {
        crate::util::debug_write($path, &format!($($y),+));
    }
}

/// Log a formatted string.
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

/// Linear map on `$t` induced by `[0, 1] -> [$min, $max]`.
macro_rules! lerp {
    ($t: expr, $min: expr, $max: expr) => {
        ($min) + ($t) * ($max - $min)
    };
}

/// Linear map on `$value` induced by `[$min, $max] -> [0, 1]`.
macro_rules! inv_lerp {
    ($value: expr, $min: expr, $max: expr) => {
        ((($value) - ($min)) / (($max) - ($min)))
    };
}

/// Piecewise linear map on `$value`:
/// [-inf, $min] -> 0
/// [$min, $max] -> [0, 1]
/// [$max, +inf] -> 1
macro_rules! inv_lerp_clamp {
    ($value: expr, $min: expr, $max: expr) => {{
        debug_assert!(
            ($min) <= ($max),
            "tried to use `inv_lerp_clamp` with reversed bounds."
        );

        if ($value) <= ($min) {
            0.0
        } else if ($value) >= ($max) {
            1.0
        } else {
            (($value) - ($min)) / (($max) - ($min))
        }
    }};
}

/// Linearly rescale the value `$val` from the interval `[$val_min, $val_max]`
/// to the interval `[$out_min, $out_max]`.
/// That is:
///   $val_min -> $out_min,
///   $val_max -> $out_max,
///   and intermediate values are obtained from linear interpolation.
macro_rules! rescale {
    ($val: expr, $val_min: expr, $val_max: expr, $out_min: expr, $out_max: expr) => {
        crate::macros::lerp!(
            crate::macros::inv_lerp!($val, $val_min, $val_max),
            $out_min,
            $out_max
        )
    };
}

/// Creates a new `Vec3` obtained by applying the binary operator `$op` to
/// the elements of the vectors `$a` and `$b`.
macro_rules! vec3_from {
    ($op: tt, $a: expr, $b: expr) => {
        [$a[0] $op $b[0], $a[1] $op $b[1], $a[2] $op $b[2]]
    }
}

/// Creates a new `Vec3` obtained by normalizing just the x and z components of `vec`.
macro_rules! vec3_unit_xz {
    ($vec: expr) => {{
        let mut unit_xz = [$vec[0], 0.0, $vec[2]];
        crate::math::vec3_inplace_normalize(&mut unit_xz);
        unit_xz
    }};
}

/// Sets the translation of the matrix `$mat` to the vector `$trans`.
macro_rules! set_translation {
    ($mat: expr, $trans: expr) => {
        $mat[12] = $trans[0];
        $mat[13] = $trans[1];
        $mat[14] = $trans[2];
    };
}

/// Set the `y` coordinate of the vector `$vec` to `$value`.
macro_rules! set_y {
    ($vec: expr, $value: expr) => {
        $vec[1] = $value;
    };
}

macro_rules! mark_address {
    ($addr: literal) => {}
}

#[allow(unused_imports)]
pub(crate) use {
    debug_log,
    inv_lerp,
    inv_lerp_clamp,
    lerp,
    log,
    mark_address,
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
    rescale,
    set_translation,
    set_y,
    temp_debug_log,
    temp_debug_write,
    vec3_from,
    vec3_unit_xz,
};
