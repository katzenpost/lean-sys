//! Objects built by these bindings must match the ones the Lean runtime builds itself.

use core::ffi::c_int;
use lean_sys::*;

extern "C" {
    fn lean_io_realpath(fname: lean_obj_arg) -> lean_obj_res;
    fn lean_io_remove_file(fname: b_lean_obj_arg) -> lean_obj_res;
    #[link_name = "lean_uint64_mix_hash"]
    fn runtime_uint64_mix_hash(h: u64, k: u64) -> u64;
}

fn init() {
    unsafe { lean_initialize_runtime_module_locked() }
}

unsafe fn set_rc(o: *mut lean_object, rc: c_int) {
    *lean_get_rc_mt_addr(o) = rc;
}

unsafe fn rc(o: *mut lean_object) -> c_int {
    *lean_get_rc_mt_addr(o)
}

#[test]
fn io_ok_result_matches_runtime() {
    init();
    unsafe {
        let theirs = lean_io_realpath(lean_mk_string(c"/".as_ptr().cast()));
        let ours = lean_io_result_mk_ok(lean_box(0));
        assert!(lean_io_result_is_ok(theirs) && lean_io_result_is_ok(ours));
        assert_eq!(lean_ctor_num_objs(ours), lean_ctor_num_objs(theirs));
        assert_eq!(lean_object_byte_size(ours), lean_object_byte_size(theirs));
        lean_dec(theirs);
        lean_dec(ours);
    }
}

#[test]
fn io_error_result_matches_runtime() {
    init();
    unsafe {
        let path = lean_mk_string(c"/nonexistent/lean-sys-test".as_ptr().cast());
        let theirs = lean_io_remove_file(path);
        assert!(lean_io_result_is_error(theirs));
        let err = lean_io_result_get_error(theirs);
        lean_inc(err);
        let ours = lean_io_result_mk_error(err);
        assert_eq!(lean_ctor_num_objs(ours), lean_ctor_num_objs(theirs));
        assert_eq!(lean_object_byte_size(ours), lean_object_byte_size(theirs));
        lean_dec(ours);
        lean_dec(theirs);
        lean_dec(path);
    }
}

#[test]
fn st_ref_round_trip() {
    init();
    unsafe {
        let r = lean_st_mk_ref(lean_box(5));
        assert_eq!(lean_unbox(lean_st_ref_get(r)), 5);
        lean_st_ref_put(r, lean_box(7));
        assert_eq!(lean_unbox(lean_st_ref_swap(r, lean_box(9))), 7);
        assert_eq!(lean_unbox(lean_st_ref_get(r)), 9);
        lean_dec(r);
    }
}

/// Objects allocated here are freed by the runtime, so both must use the same allocator.
#[test]
fn runtime_frees_what_we_allocate() {
    init();
    unsafe {
        for n in 0..64 {
            let c = lean_alloc_ctor(0, n, 7);
            for i in 0..n {
                lean_ctor_set(c, i, lean_io_result_mk_ok(lean_box(i as usize)));
            }
            lean_dec(c);
        }
        for fixed in 0..15 {
            let f = lean_alloc_closure(core::ptr::null_mut(), 16, fixed);
            for i in 0..fixed {
                lean_closure_set(f, i, lean_box(0));
            }
            assert_eq!(lean_object_byte_size(f), lean_closure_byte_size(f));
            lean_dec(f);
        }
    }
}

#[test]
fn ctor_release_clears_the_named_field() {
    init();
    unsafe {
        let a = lean_io_result_mk_ok(lean_box(1));
        let b = lean_io_result_mk_ok(lean_box(2));
        let c = lean_alloc_ctor(0, 2, 0);
        lean_ctor_set(c, 0, a);
        lean_ctor_set(c, 1, b);
        lean_ctor_release(c, 1);
        assert_eq!(lean_ctor_get(c, 0), a);
        assert_eq!(lean_ctor_get(c, 1), lean_box(0));
        lean_dec(c);
    }
}

#[test]
fn overflowing_count_sticks() {
    init();
    unsafe {
        let o = lean_alloc_ctor(0, 0, 0);
        set_rc(o, c_int::MAX);
        lean_inc_ref(o);
        let frozen = rc(o);
        assert!(frozen <= LEAN_RC_STICKY);
        lean_inc_ref(o);
        lean_dec_ref(o);
        lean_dec_ref(o);
        assert_eq!(rc(o), frozen);
        set_rc(o, 1);
        lean_dec_ref(o);
    }
}

#[test]
fn huge_increments_go_to_the_runtime() {
    init();
    unsafe {
        let o = lean_alloc_ctor(0, 0, 0);
        lean_inc_ref_n(o, LEAN_RC_INC_MAX + 1);
        assert_eq!(rc(o), 1 + (LEAN_RC_INC_MAX + 1) as c_int);
        set_rc(o, 1);
        lean_dec_ref(o);
    }
}

#[test]
fn mix_hash_matches_runtime() {
    for (h, k) in [
        (0, 0),
        (1, 2),
        (u64::MAX, 7),
        (0xdead_beef, 0x1234_5678_9abc_def0),
    ] {
        assert_eq!(lean_uint64_mix_hash(h, k), unsafe {
            runtime_uint64_mix_hash(h, k)
        });
    }
}

#[test]
fn array_get_borrows_the_default() {
    init();
    unsafe {
        let def = lean_io_result_mk_ok(lean_box(0));
        let a = lean_mk_empty_array();
        let a = lean_array_push(a, lean_box(3));
        let got = lean_array_get(def, a, lean_box(0));
        assert_eq!(lean_unbox(got), 3);
        assert_eq!(rc(def), 1);
        lean_dec(def);
        lean_dec(a);
    }
}

#[test]
fn sarray_and_string_equality() {
    init();
    unsafe {
        let mk = |bytes: &[u8]| {
            let o = lean_alloc_sarray(1, bytes.len(), bytes.len());
            core::ptr::copy_nonoverlapping(bytes.as_ptr(), lean_sarray_cptr(o), bytes.len());
            o
        };
        let (a, b, c) = (mk(b"abc"), mk(b"abc"), mk(b"abd"));
        assert!(lean_sarray_eq(a, b));
        assert!(!lean_sarray_eq(a, c));
        let s = lean_mk_string(c"abc".as_ptr().cast());
        let t = lean_mk_string(c"abd".as_ptr().cast());
        assert_eq!(lean_string_uget_byte_fast(s, 2), b'c');
        assert_eq!(lean_string_compare(s, t), 0);
        for o in [a, b, c, s, t] {
            lean_dec(o);
        }
    }
}

/// Constructors on both sides of mimalloc's small pages, which `lean_alloc_small_object_core` and
/// `mi_malloc` allocate: the runtime reads the size the allocation recorded, and frees them.
#[cfg(feature = "mimalloc")]
#[test]
fn small_objects_on_both_allocation_paths() {
    init();
    unsafe {
        for (fields, scalars) in [
            (0, 3),
            (1, 0),
            (126, 0),
            (127, 0),
            (128, 0),
            (200, 5),
            (255, 1000),
        ] {
            let c = lean_alloc_ctor(0, fields, scalars);
            let sz = size_of::<lean_ctor_object>()
                + fields as usize * size_of::<usize>()
                + scalars as usize;
            assert_eq!(rc(c), 1);
            assert_eq!(
                lean_object_byte_size(c),
                lean_align(sz, LEAN_OBJECT_SIZE_DELTA)
            );
            for i in 0..fields {
                lean_ctor_set(c, i, lean_box(i as usize));
            }
            lean_dec(c);
        }
    }
}

#[test]
fn st_ref_put_and_take() {
    init();
    unsafe {
        let r = lean_st_mk_ref(lean_box(5));
        lean_st_ref_put(r, lean_box(7));
        assert_eq!(lean_unbox(lean_st_ref_take(r)), 7);
        lean_dec(r);
    }
}

/// A linear mark set here is the runtime's: its copies keep it, and it reads element and object
/// sizes without it.
#[test]
fn marked_sarray_matches_runtime() {
    init();
    unsafe {
        let a = lean_alloc_sarray(1, 3, 3);
        core::ptr::copy_nonoverlapping(b"abc".as_ptr(), lean_sarray_cptr(a), 3);
        let a = lean_sarray_mark_linear(a);
        assert!(lean_sarray_is_marked_linear(a));
        assert_eq!(lean_sarray_elem_size(a), 1);
        assert_eq!(lean_object_byte_size(a), lean_sarray_byte_size(a));
        // Shared, so the write goes to the runtime's copy.
        lean_inc(a);
        let b = lean_byte_array_uset(a, 0, b'x');
        assert_ne!(a, b);
        assert!(lean_sarray_is_marked_linear(b));
        assert_eq!(lean_sarray_elem_size(b), 1);
        assert_eq!(lean_object_byte_size(b), lean_sarray_byte_size(b));
        assert_eq!(*lean_sarray_cptr(a), b'a');
        assert_eq!(*lean_sarray_cptr(b), b'x');

        let f = lean_sarray_mark_linear(lean_alloc_sarray(8, 1, 1));
        assert_eq!(lean_sarray_elem_size(f), 8);
        lean_inc(f);
        let g = lean_float_array_uset(f, 0, 2.5);
        assert!(lean_sarray_is_marked_linear(g));
        assert_eq!(lean_sarray_elem_size(g), 8);
        assert_eq!(lean_float_array_uget(g, 0), 2.5);

        let plain = lean_alloc_sarray(1, 0, 1);
        let c = lean_sarray_propagate_mark(a, lean_alloc_sarray(1, 0, 1));
        let d = lean_sarray_propagate_mark(plain, lean_alloc_sarray(1, 0, 1));
        assert!(lean_sarray_is_marked_linear(c));
        assert!(!lean_sarray_is_marked_linear(d));
        for o in [a, b, f, g, plain, c, d] {
            lean_dec(o);
        }
    }
}

#[test]
fn marked_string_matches_runtime() {
    init();
    unsafe {
        let s = lean_string_mark_linear(lean_mk_string(c"abc".as_ptr().cast()));
        assert!(lean_string_is_marked_linear(s));
        assert_eq!(lean_object_byte_size(s), lean_string_byte_size(s));
        lean_inc(s);
        let t = lean_string_ensure_exclusive(s);
        assert_ne!(s, t);
        assert!(lean_string_is_marked_linear(t));
        assert_eq!(lean_string_size(t), 4);
        // An `Ordering`: 1 is `eq`.
        assert_eq!(lean_string_compare(s, t), 1);
        let u = lean_string_propagate_mark(s, lean_mk_string(c"d".as_ptr().cast()));
        assert!(lean_string_is_marked_linear(u));
        for o in [s, t, u] {
            lean_dec(o);
        }
    }
}

#[test]
fn marked_array_matches_runtime() {
    init();
    unsafe {
        let a = lean_array_mark_linear(lean_array_push(lean_mk_empty_array(), lean_box(1)));
        assert!(lean_array_is_marked_linear(a));
        lean_inc(a);
        let b = lean_array_uset(a, 0, lean_box(2));
        assert_ne!(a, b);
        assert!(lean_array_is_marked_linear(b));
        assert_eq!(lean_unbox(lean_array_get_core(a, 0)), 1);
        assert_eq!(lean_unbox(lean_array_get_core(b, 0)), 2);
        lean_dec(a);
        lean_dec(b);
    }
}

#[test]
fn checked_sizes_panic_on_overflow() {
    assert!(lean_usize_mul_would_overflow(usize::MAX, 2));
    assert!(lean_usize_add_would_overflow(usize::MAX, 1));
    assert!(!lean_alloc_sarray_would_overflow(1, 16));
    assert!(lean_alloc_sarray_would_overflow(8, usize::MAX / 4));
    assert_eq!(lean_usize_add_checked(2, 3), 5);
    assert_eq!(lean_usize_mul_checked(2, 3), 6);
}
