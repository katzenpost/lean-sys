/*! ST Ref primitives */
use crate::*;

extern "C" {
    pub fn lean_st_mk_ref(a: lean_obj_arg) -> lean_obj_res;
    pub fn lean_st_ref_get(r: b_lean_obj_arg) -> lean_obj_res;
    pub fn lean_st_ref_put(r: b_lean_obj_arg, a: lean_obj_arg) -> lean_obj_res;
    pub fn lean_st_ref_take(r: b_lean_obj_arg) -> lean_obj_res;
    pub fn lean_st_ref_swap(r: b_lean_obj_arg, a: lean_obj_arg) -> lean_obj_res;
}
