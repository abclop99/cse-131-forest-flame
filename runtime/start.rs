use std::{collections::HashSet, env, convert::TryInto};

type SnekVal = u64;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(i64)]
pub enum ErrCode {
    InvalidArgument = 1,
    Overflow = 2,
    IndexOutOfBounds = 3,
    InvalidVecSize = 4,
    OutOfMemory = 5,
}

const TRUE: u64 = 7;
const FALSE: u64 = 3;

static mut HEAP_START: *const u64 = std::ptr::null();
static mut HEAP_END: *const u64 = std::ptr::null();

#[link(name = "our_code")]
extern "C" {
    // The \x01 here is an undocumented feature of LLVM that ensures
    // it does not add an underscore in front of the name.
    // Courtesy of Max New (https://maxsnew.com/teaching/eecs-483-fa22/hw_adder_assignment.html)
    #[link_name = "\x01our_code_starts_here"]
    fn our_code_starts_here(input: u64, heap_start: *const u64, heap_end: *const u64) -> u64;
}

#[export_name = "\x01snek_error"]
pub extern "C" fn snek_error(errcode: i64) {
    if errcode == ErrCode::InvalidArgument as i64 {
        eprintln!("invalid argument");
    } else if errcode == ErrCode::Overflow as i64 {
        eprintln!("overflow");
    } else if errcode == ErrCode::IndexOutOfBounds as i64 {
        eprintln!("index out of bounds");
    } else if errcode == ErrCode::InvalidVecSize as i64 {
        eprintln!("vector size must be non-negative");
    } else {
        eprintln!("an error ocurred {}", errcode);
    }
    std::process::exit(errcode as i32);
}

#[export_name = "\x01snek_print"]
pub unsafe extern "C" fn snek_print(val: SnekVal) -> SnekVal {
    println!("{}", snek_str(val, &mut HashSet::new()));
    val
}

/// This function is called when the program needs to allocate `count` words of memory and there's no
/// space left. The function should try to clean up space by triggering a garbage collection. If there's
/// not enough space to hold `count` words after running the garbage collector, the program should terminate
/// with an `out of memory` error.
///
/// Args:
///     * `count`: The number of words the program is trying to allocate, including an extra word for
///       the size of the vector and an extra word to store metadata for the garbage collector, e.g.,
///       to allocate a vector of size 5, `count` will be 7.
///     * `heap_ptr`: The current position of the heap pointer (i.e., the value stored in `%r15`). It
///       is guaranteed that `heap_ptr + 8 * count > HEAP_END`, i.e., this function is only called if
///       there's not enough space to allocate `count` words.
///     * `stack_base`: A pointer to the "base" of the stack.
///     * `curr_rbp`: The value of `%rbp` in the stack frame that triggered the allocation.
///     * `curr_rsp`: The value of `%rsp` in the stack frame that triggered the allocation.
///
/// Returns:
///
/// The new heap pointer where the program should allocate the vector (i.e., the new value of `%r15`)
///
#[export_name = "\x01snek_try_gc"]
pub unsafe fn snek_try_gc(
    count: isize,
    heap_ptr: *const u64,
    stack_base: *const u64,
    curr_rbp: *const u64,
    curr_rsp: *const u64,
) -> *const u64 {

    // Call the garbage collector here and return the new heap pointer
    let heap_ptr = snek_gc(heap_ptr, stack_base, curr_rbp, curr_rsp);

    // Check if there's enough space to allocate `count` words
    if heap_ptr.add(count as usize) > HEAP_END {
        eprintln!("out of memory");
        std::process::exit(ErrCode::OutOfMemory as i32)
    }

    heap_ptr
}

/// This function should trigger garbage collection and return the updated heap pointer (i.e., the new
/// value of `%r15`). See [`snek_try_gc`] for a description of the meaning of the arguments.
#[export_name = "\x01snek_gc"]
pub unsafe fn snek_gc(
    _heap_ptr: *const u64,
    stack_base: *const u64,
    _curr_rbp: *const u64,
    curr_rsp: *const u64,
) -> *const u64 {
    /// Finds all the roots on the stack. A root is a heap-allocated value
    /// that is directly referenced by a stack value.
    /// Returns a vector of pointers to the pointers on the stack.
    unsafe fn find_root_ptrs(stack_base: &*const u64, curr_rsp: &*const u64) -> Vec<*mut SnekVal> {
        let mut root_ptrs = Vec::new();

        let mut ptr = stack_base.clone();

        while ptr >= curr_rsp.clone() {
            let val = *ptr;
            if val & 1 == 1 && val != 1 {
                let vec_ptr = (val - 1) as *const u64;
                if HEAP_START <= vec_ptr && vec_ptr < HEAP_END {
                    root_ptrs.push(ptr as *mut SnekVal);
                }
            }
            ptr = ptr.sub(1);
        }

        root_ptrs
    }

    /// root_ptr is a pointer to the beginning of a heap-allocated vec.
    /// Returns a vector of pointers to the vecs on the heap.
    unsafe fn mark(root_ptr: *mut u64) -> Vec<*mut u64> {
        let gc_word = &mut *root_ptr;
        let size = *root_ptr.add(1);

        let mut vec_ptrs = Vec::new();

        // Check if the vector has already been marked
        if *gc_word & 1 == 1 {
            return vec_ptrs;
        }

        vec_ptrs.push(root_ptr);

        // Mark the vector
        *gc_word = 1;

        // Mark the elements of the vector
        for i in 0..size {
            let val = *root_ptr.add((2 + i).try_into().unwrap());
            if val & 1 == 1 && val != 1 {
                let val = val - 1;
                vec_ptrs.append(&mut mark(val as *mut u64));
            }
        }

        vec_ptrs
    }

    /// Sets the to addresses in the gc words of the vectors to the new locations
    unsafe fn fwd_headers(move_from: &Vec<*mut u64>) {
        let mut move_to = HEAP_START as *mut u64;

        for from in move_from {
            let size = *(*from).add(1);

            // Set the forwarding pointer to the new location
            debug_assert!(**from == 1, "gc word is not marked or wrong");
            **from = move_to as u64 + 1;

            // Increment the move_to pointer according to the size of the vector
            move_to = move_to.add((2 + size).try_into().unwrap());
        }
    }

    /// Replace a reference to a vector with a reference to the new location of the vector
    unsafe fn fwd_reference(ptr: &mut SnekVal) {
        let gc_word = *((*ptr - 1) as *mut u64);
        *ptr = gc_word;
    }

    /// Replace all references to vectors in a vector with references to the new
    /// locations of the vectors
    unsafe fn fwd_vec(vec_ptr: *mut u64) {
        let size = *vec_ptr.add(1);

        for i in 0..size {
            let val = &mut *vec_ptr.add((2 + i).try_into().unwrap());

            if *val & 1 == 1 && *val != 1 {
                fwd_reference(val);
            }
        }
    }
    
    // Locate roots on the stack
    let root_ptrs = find_root_ptrs(&stack_base, &curr_rsp);

    let mut vec_ptrs: Vec<*mut u64> = Vec::new();

    // Mark all the reachable objects
    for root_ptr in &root_ptrs {
        let root_addr = (**root_ptr - 1) as *mut u64;
        vec_ptrs.append(&mut mark(root_addr));
    }

    // Sort the vector pointers
    vec_ptrs.sort_unstable();
    // Assert that there are no duplicates
    debug_assert!({
        let len = vec_ptrs.len();
        vec_ptrs.dedup();
        len == vec_ptrs.len()
    }, "mark returned duplicate pointers");

    fwd_headers(&vec_ptrs);

    // Forward the references in the stack
    for root_ptr in &root_ptrs {
        fwd_reference(&mut **root_ptr);
    }
    
    // Forward the references in the heap
    for vec_ptr in &vec_ptrs {
        fwd_vec(*vec_ptr);
    }

    // Compact the heap by moving the vectors
    let mut heap_ptr = HEAP_START;
    for vec_ptr in &vec_ptrs {
        let from = *vec_ptr;
        let to = (**vec_ptr - 1) as *mut u64;

        let size = *from.add(1);

        // Set the gc word to 0
        *from = 0;

        // Move the size and elements
        for i in 1..(size + 2) {
            let i = i.try_into().unwrap();
            *to.add(i) = *from.add(i);
        }

        // Set heap_ptr to the next free location
        heap_ptr = heap_ptr.add((2 + size).try_into().unwrap());
    }

    heap_ptr
}

/// A helper function that can called with the `(snek-printstack)` snek function. It prints the stack
/// See [`snek_try_gc`] for a description of the meaning of the arguments.
#[export_name = "\x01snek_print_stack"]
pub unsafe fn snek_print_stack(stack_base: *const u64, _curr_rbp: *const u64, curr_rsp: *const u64) {
    let mut ptr = stack_base;
    println!("-----------------------------------------");
    while ptr >= curr_rsp {
        let val = *ptr;
        println!("{ptr:?}: {:#0x}", val);
        ptr = ptr.sub(1);
    }
    println!("-----------------------------------------");
}

/// A helper function that prints the heap.
pub unsafe fn snek_print_heap() {
    let mut ptr = HEAP_START;
    println!("-----------------------------------------");
    while ptr < HEAP_END {
        let val = *ptr;
        println!("{ptr:?}: {:#x}", val);
        ptr = ptr.add(1);
    }
    println!("-----------------------------------------");
}

unsafe fn snek_str(val: SnekVal, seen: &mut HashSet<SnekVal>) -> String {
    if val == TRUE {
        format!("true")
    } else if val == FALSE {
        format!("false")
    } else if val & 1 == 0 {
        format!("{}", (val as i64) >> 1)
    } else if val == 1 {
        format!("nil")
    } else if val & 1 == 1 {
        if !seen.insert(val) {
            return "[...]".to_string();
        }
        let addr = (val - 1) as *const u64;
        let size = addr.add(1).read() as usize;
        let mut res = "[".to_string();
        for i in 0..size {
            let elem = addr.add(2 + i).read();
            res = res + &snek_str(elem, seen);
            if i < size - 1 {
                res = res + ", ";
            }
        }
        seen.remove(&val);
        res + "]"
    } else {
        format!("unknown value: {val}")
    }
}

fn parse_input(input: &str) -> u64 {
    match input {
        "true" => TRUE,
        "false" => FALSE,
        _ => (input.parse::<i64>().unwrap() << 1) as u64,
    }
}

fn parse_heap_size(input: &str) -> usize {
    input.parse::<usize>().unwrap()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let input = if args.len() >= 2 { &args[1] } else { "false" };
    let heap_size = if args.len() >= 3 { &args[2] } else { "10000" };
    let input = parse_input(&input);
    let heap_size = parse_heap_size(&heap_size);

    // Initialize heap
    let mut heap: Vec<u64> = Vec::with_capacity(heap_size);
    unsafe {
        HEAP_START = heap.as_mut_ptr();
        HEAP_END = HEAP_START.add(heap_size);
    }

    let i: u64 = unsafe { our_code_starts_here(input, HEAP_START, HEAP_END) };
    unsafe { snek_print(i) };
}
