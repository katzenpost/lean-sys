/*! Array of scalars */
use crate::*;

pub mod byte;
pub mod float;
pub use byte::*;
pub use float::*;

#[inline]
pub fn lean_alloc_sarray_would_overflow(elem_size: c_uint, capacity: usize) -> bool {
    match (elem_size as usize).checked_mul(capacity) {
        Some(n) => lean_usize_add_would_overflow(core::mem::size_of::<lean_sarray_object>(), n),
        None => true,
    }
}

#[inline]
pub unsafe fn lean_alloc_sarray(elem_size: c_uint, size: usize, capacity: usize) -> lean_obj_res {
    if elem_size >= LEAN_LINEAR_MARK_MASK {
        lean_internal_panic(
            c"scalar array element size is larger than 2^7"
                .as_ptr()
                .cast(),
        )
    }
    let o = lean_alloc_object(lean_usize_add_checked(
        core::mem::size_of::<lean_sarray_object>(),
        lean_usize_mul_checked(elem_size as usize, capacity),
    )) as *mut lean_sarray_object;
    lean_set_st_header(o as *mut _, LeanScalarArray as u32, elem_size);
    (raw_field!(o, lean_sarray_object, m_size) as *mut usize).write(size);
    (raw_field!(o, lean_sarray_object, m_capacity) as *mut usize).write(capacity);
    o as *mut _
}

#[inline(always)]
pub unsafe fn lean_sarray_elem_size(o: *const lean_object) -> c_uint {
    debug_assert!(lean_is_sarray(o));
    lean_ptr_other(o) & !LEAN_LINEAR_MARK_MASK
}

#[inline(always)]
pub unsafe fn lean_sarray_capacity(o: *const lean_object) -> usize {
    *raw_field!(lean_to_sarray(o as *mut _), lean_sarray_object, m_capacity)
}

#[inline]
pub unsafe fn lean_sarray_byte_size(o: *const lean_object) -> usize {
    size_of::<lean_sarray_object>() + lean_sarray_elem_size(o) as usize * lean_sarray_capacity(o)
}

#[inline(always)]
pub unsafe fn lean_sarray_size(o: *const lean_object) -> usize {
    *raw_field!(lean_to_sarray(o as *mut _), lean_sarray_object, m_size)
}

#[inline(always)]
pub unsafe fn lean_sarray_data_byte_size(o: *const lean_object) -> usize {
    size_of::<lean_sarray_object>() + lean_sarray_elem_size(o) as usize * lean_sarray_size(o)
}

#[inline(always)]
pub unsafe fn lean_sarray_set_size(o: u_lean_obj_arg, sz: usize) {
    debug_assert!(lean_is_exclusive(o));
    debug_assert!(sz <= lean_sarray_capacity(o));
    (raw_field!(lean_to_sarray(o), lean_sarray_object, m_size) as *mut usize).write(sz);
}

#[inline(always)]
pub unsafe fn lean_sarray_cptr(o: *mut lean_object) -> *mut u8 {
    raw_field!(lean_to_sarray(o), lean_sarray_object, m_data) as *mut _
}

#[inline(always)]
pub unsafe fn lean_sarray_is_marked_linear(o: b_lean_obj_arg) -> bool {
    debug_assert!(lean_is_sarray(o));
    lean_is_marked_linear_core(o)
}

#[inline(always)]
pub unsafe fn lean_sarray_mark_linear_core(o: u_lean_obj_arg) {
    debug_assert!(lean_is_sarray(o));
    lean_mark_linear_core(o)
}

#[inline]
pub unsafe fn lean_sarray_ensure_exclusive(a: lean_obj_arg) -> lean_obj_res {
    if lean_is_exclusive(a) {
        a
    } else {
        lean_copy_sarray_nonlinear(a, lean_sarray_capacity(a))
    }
}

#[inline]
pub unsafe fn lean_sarray_mark_linear(a: lean_obj_arg) -> lean_obj_res {
    let r = lean_sarray_ensure_exclusive(a);
    lean_sarray_mark_linear_core(r);
    r
}

#[inline]
pub unsafe fn lean_sarray_propagate_mark(src: b_lean_obj_arg, dst: lean_obj_arg) -> lean_obj_res {
    if !lean_sarray_is_marked_linear(src) {
        dst
    } else {
        lean_sarray_mark_linear(dst)
    }
}

#[inline]
pub unsafe fn lean_sarray_eq(a1: b_lean_obj_arg, a2: b_lean_obj_arg) -> bool {
    debug_assert_eq!(lean_sarray_elem_size(a1), lean_sarray_elem_size(a2));
    a1 == a2 || (lean_sarray_size(a1) == lean_sarray_size(a2) && lean_sarray_eq_cold(a1, a2))
}

#[inline]
pub unsafe fn lean_sarray_dec_eq(a1: b_lean_obj_arg, a2: b_lean_obj_arg) -> u8 {
    lean_sarray_eq(a1, a2) as u8
}

extern "C" {
    pub fn lean_copy_sarray(a: lean_obj_arg, cap: usize) -> lean_obj_res;
    /// Equivalent to [`lean_copy_sarray`] but used as a gadget to spot scalar array
    /// non-linearities in profiles. Panics if `a` is marked linear.
    pub fn lean_copy_sarray_nonlinear(a: lean_obj_arg, cap: usize) -> lean_obj_res;
    #[cold]
    pub fn lean_sarray_eq_cold(a1: b_lean_obj_arg, a2: b_lean_obj_arg) -> bool;
}
