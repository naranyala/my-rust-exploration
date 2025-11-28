// reactive/mod.rs

use core::{cmp, mem::MaybeUninit, ptr, str};

const MAX_DEPS: usize = 8;
const MAX_STR: usize = 256;
const MAX_SIGNALS: usize = 256;


#[allow(unused)]
#[derive(Copy, Clone, PartialEq)]
pub enum SignalType {
    Int,
    Double,
    String,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Signal {
    pub(crate) ty: SignalType,
    pub(crate) dirty: bool,

    // "Union" fields
    pub(crate) val_i: i32,
    pub(crate) val_d: f64,
    pub(crate) val_s: [u8; MAX_STR],

    pub(crate) deps: [*mut Signal; MAX_DEPS],
    pub(crate) dep_count: usize,

    pub(crate) compute: Option<fn(&mut Signal)>,
}

// Static pool and registry (no dynamic allocation)
static mut POOL: [MaybeUninit<Signal>; MAX_SIGNALS] =
    [MaybeUninit::<Signal>::uninit(); MAX_SIGNALS];
static mut POOL_USED: usize = 0;

static mut SIGNALS: [*mut Signal; MAX_SIGNALS] = [ptr::null_mut(); MAX_SIGNALS];
static mut SIGNAL_COUNT: usize = 0;

fn register_signal(s: *mut Signal) {
    unsafe {
        if SIGNAL_COUNT < MAX_SIGNALS {
            SIGNALS[SIGNAL_COUNT] = s;
            SIGNAL_COUNT += 1;
        }
    }
}

fn alloc_signal(initial: Signal) -> *mut Signal {
    unsafe {
        if POOL_USED >= MAX_SIGNALS {
            return ptr::null_mut();
        }
        let slot = &mut POOL[POOL_USED];
        let p = slot.as_mut_ptr();
        ptr::write(p, initial);
        POOL_USED += 1;
        register_signal(p);
        p
    }
}

// ====================== Creation ======================
pub fn signal_int(value: i32) -> *mut Signal {
    let s = Signal {
        ty: SignalType::Int,
        dirty: false,
        val_i: value,
        val_d: 0.0,
        val_s: [0; MAX_STR],
        deps: [ptr::null_mut(); MAX_DEPS],
        dep_count: 0,
        compute: None,
    };
    alloc_signal(s)
}

#[allow(unused)]
pub fn signal_double(value: f64) -> *mut Signal {
    let s = Signal {
        ty: SignalType::Double,
        dirty: false,
        val_i: 0,
        val_d: value,
        val_s: [0; MAX_STR],
        deps: [ptr::null_mut(); MAX_DEPS],
        dep_count: 0,
        compute: None,
    };
    alloc_signal(s)
}

pub fn signal_string(value: &str) -> *mut Signal {
    let mut val_s = [0u8; MAX_STR];
    let bytes = value.as_bytes();
    let len = cmp::min(bytes.len(), MAX_STR - 1);
    val_s[..len].copy_from_slice(&bytes[..len]);
    val_s[len] = 0;

    let s = Signal {
        ty: SignalType::String,
        dirty: false,
        val_i: 0,
        val_d: 0.0,
        val_s,
        deps: [ptr::null_mut(); MAX_DEPS],
        dep_count: 0,
        compute: None,
    };
    alloc_signal(s)
}

// ====================== Getters ======================
#[inline]
pub fn get_int(s: *mut Signal) -> i32 {
    unsafe {
        if s.is_null() {
            return 0;
        }
        if let Some(compute) = (*s).compute {
            if (*s).dirty {
                compute(&mut *s);
                (*s).dirty = false;
            }
        }
        (*s).val_i
    }
}

#[inline]
#[allow(unused)]
pub fn get_double(s: *mut Signal) -> f64 {
    unsafe {
        if s.is_null() {
            return 0.0;
        }
        if let Some(compute) = (*s).compute {
            if (*s).dirty {
                compute(&mut *s);
                (*s).dirty = false;
            }
        }
        (*s).val_d
    }
}

#[inline]
pub fn get_string(s: *mut Signal, out: &mut [u8]) -> usize {
    unsafe {
        if s.is_null() {
            if !out.is_empty() {
                out[0] = 0;
            }
            return 0;
        }
        if let Some(compute) = (*s).compute {
            if (*s).dirty {
                compute(&mut *s);
                (*s).dirty = false;
            }
        }
        let sig = &*s;
        let len = sig
            .val_s
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(MAX_STR);
        let copy_len = cmp::min(len, if out.len() > 0 { out.len() - 1 } else { 0 });
        if copy_len > 0 {
            out[..copy_len].copy_from_slice(&sig.val_s[..copy_len]);
        }
        if !out.is_empty() {
            out[copy_len] = 0;
        }
        copy_len
    }
}

// ====================== Setters ======================
#[inline]
pub fn set_int(s: *mut Signal, value: i32) {
    unsafe {
        if s.is_null() {
            return;
        }
        if (*s).val_i != value || (*s).ty != SignalType::Int {
            (*s).ty = SignalType::Int;
            (*s).val_i = value;
            (*s).dirty = true;
            propagate(s);
        }
    }
}

#[inline]
#[allow(unused)]
pub fn set_double(s: *mut Signal, value: f64) {
    unsafe {
        if s.is_null() {
            return;
        }
        if (*s).val_d != value || (*s).ty != SignalType::Double {
            (*s).ty = SignalType::Double;
            (*s).val_d = value;
            (*s).dirty = true;
            propagate(s);
        }
    }
}

#[inline]
pub fn set_string(s: *mut Signal, value: &str) {
    unsafe {
        if s.is_null() {
            return;
        }
        let sig = &mut *s;
        let bytes = value.as_bytes();
        let len = cmp::min(bytes.len(), MAX_STR - 1);

        let curr_len = sig
            .val_s
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(MAX_STR);

        let mut changed = curr_len != len;
        if !changed {
            for i in 0..len {
                if sig.val_s[i] != bytes[i] {
                    changed = true;
                    break;
                }
            }
        }

        if changed || sig.ty != SignalType::String {
            sig.ty = SignalType::String;
            sig.val_s[..len].copy_from_slice(&bytes[..len]);
            sig.val_s[len] = 0;
            sig.dirty = true;
            propagate(s);
        }
    }
}

// ====================== Computed Signals ======================
pub fn signal_computed(compute: fn(&mut Signal), deps: &[*mut Signal]) -> *mut Signal {
    let dep_count = deps.len().min(MAX_DEPS);
    let mut deps_arr = [ptr::null_mut(); MAX_DEPS];
    deps_arr[..dep_count].copy_from_slice(&deps[..dep_count]);

    let s = Signal {
        ty: SignalType::Int, // Default type, compute function should set this
        dirty: true, // Start as dirty to force first computation
        val_i: 0,
        val_d: 0.0,
        val_s: [0; MAX_STR],
        deps: deps_arr,
        dep_count,
        compute: Some(compute),
    };

    alloc_signal(s)
}

// Safe compute function for doubling
pub fn compute_double(signal: &mut Signal) {
    if signal.dep_count == 0 {
        return;
    }
    
    let counter = signal.deps[0];
    if counter.is_null() {
        return;
    }
    
    // Get the current counter value and compute doubled
    let v = get_int(counter);
    signal.ty = SignalType::Int;
    signal.val_i = v * 2;
}

// ====================== Propagation ======================
fn propagate(changed: *mut Signal) {
    unsafe {
        if changed.is_null() {
            return;
        }
        for i in 0..SIGNAL_COUNT {
            let candidate = SIGNALS[i];
            if candidate.is_null() || candidate == changed {
                continue;
            }

            let mut is_dependent = false;
            for j in 0..(*candidate).dep_count {
                if (*candidate).deps[j] == changed {
                    is_dependent = true;
                    break;
                }
            }

            if is_dependent {
                (*candidate).dirty = true;
            }
        }
    }
}

// ====================== Utilities ======================
pub fn signals_reset() {
    unsafe {
        // Just reset the counters - Signal doesn't need destructors
        for i in 0..POOL_USED {
            SIGNALS[i] = ptr::null_mut();
        }
        POOL_USED = 0;
        SIGNAL_COUNT = 0;
    }
}
