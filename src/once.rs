/*! Initialize-once cells used by compiled code, and other runtime entry points */
use crate::*;

#[repr(C)]
pub struct lean_once_cell_t {
    pub state: AtomicI32,
    pub lock: AtomicI32,
}

impl lean_once_cell_t {
    pub const fn new() -> Self {
        lean_once_cell_t {
            state: AtomicI32::new(0),
            lock: AtomicI32::new(0),
        }
    }
}

impl Default for lean_once_cell_t {
    fn default() -> Self {
        Self::new()
    }
}

macro_rules! once {
    ($name:ident, $cold:ident, $t:ty) => {
        extern "C" {
            pub fn $cold(
                loc: *mut $t,
                tok: *mut lean_once_cell_t,
                init: extern "C" fn() -> $t,
            ) -> $t;
        }

        #[inline]
        pub unsafe fn $name(
            loc: *mut $t,
            tok: *mut lean_once_cell_t,
            init: extern "C" fn() -> $t,
        ) -> $t {
            if (*tok).state.load(Ordering::SeqCst) == 1 {
                return *loc;
            }
            $cold(loc, tok, init)
        }
    };
}

once!(lean_obj_once, lean_obj_once_cold, *mut lean_object);
once!(lean_uint8_once, lean_uint8_once_cold, u8);
once!(lean_uint16_once, lean_uint16_once_cold, u16);
once!(lean_uint32_once, lean_uint32_once_cold, u32);
once!(lean_uint64_once, lean_uint64_once_cold, u64);
once!(lean_usize_once, lean_usize_once_cold, usize);
once!(lean_float32_once, lean_float32_once_cold, f32);
once!(lean_float_once, lean_float_once_cold, f64);

#[inline(always)]
pub fn lean_runtime_hold(_a: b_lean_obj_arg) -> lean_obj_res {
    lean_box(0)
}

extern "C" {
    pub fn lean_run_main(
        main_fn: extern "C" fn(c_int, *mut *mut c_char) -> *mut lean_object,
        argc: c_int,
        argv: *mut *mut c_char,
    ) -> *mut lean_object;
}
