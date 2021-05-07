use std::{cell::Cell, fmt::Display, mem, u8};

pub struct Objects {
    first: Cell<Option<Obj>>,
}

impl Objects {
    pub fn new() -> Self {
        Self {
            first: Cell::new(None)
        }
    }

    pub fn string(&self, s: &str) -> Obj {
        let obj = Obj::string(s, self.first.get());
        self.first.set(Some(obj));
        obj
    }
}

unsafe fn drop_obj(obj: Obj) {
    match (*obj.0).kind {
        ObjKind::String => {
            let obj_pointer: *mut StringObj = mem::transmute(obj.0);
            let obj_box = Box::from_raw(obj_pointer);
            let slice_pointer: *mut [u8] = mem::transmute(obj_box.chars);
            let slice_box = Box::from_raw(slice_pointer);
            drop(slice_box);
            drop(obj_box);
        }
    }
}

impl Drop for Objects {
    fn drop(&mut self) {
        unsafe {
            let mut object = self.first.get();
            loop {
                if let Some(obj) = object {
                    let next = (*obj.0).next;
                    drop_obj(obj);
                    object = next;
                } else { 
                    break
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Obj(*mut BaseObj);

impl Obj {
    fn string(s: &str, next: Option<Obj>) -> Self {
        unsafe {
            assert!(s.len() < u32::MAX as usize);

            let byte_pointer = s.as_bytes().to_owned().into_boxed_slice();
            let byte_pointer = Box::into_raw(byte_pointer);

            let obj = StringObj {
                base: BaseObj {
                    kind: ObjKind::String,
                    next,
                },
                chars: byte_pointer,
            };
            let obj_pointer = Box::new(obj);
            let obj_pointer = Box::into_raw(obj_pointer);

            Obj(mem::transmute(obj_pointer))
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        unsafe {
            let base: &BaseObj = mem::transmute(self.0);
            if base.kind == ObjKind::String {
                let string: &StringObj = mem::transmute(self.0);
                Some(string.as_str())
            } else {
                None
            }
        }
    }
}

impl Display for Obj {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe {
            let base: &BaseObj = mem::transmute(self.0);
            match base.kind {
                ObjKind::String => {
                    let string: &StringObj = mem::transmute(self.0);
                    write!(f, "{}", string.as_str())
                }
            }
        }
    }
}

impl PartialEq for Obj {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            let base_self: &BaseObj = mem::transmute(self.0);
            let other_self: &BaseObj = mem::transmute(other.0);
            match (&base_self.kind, &other_self.kind) {
                (ObjKind::String, ObjKind::String) => {
                    let a: &StringObj = mem::transmute(self.0);
                    let b: &StringObj = mem::transmute(other.0);
                    a.as_str() == b.as_str()
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
enum ObjKind {
    String,
}

#[repr(C)]
struct BaseObj {
    kind: ObjKind,
    next: Option<Obj>,
}

#[repr(C)]
struct StringObj {
    base: BaseObj,
    chars: *const [u8],
}

impl StringObj {
    unsafe fn as_str(&self) -> &str {
        std::str::from_utf8_unchecked(mem::transmute(self.chars))
    }
}


// TODO!
// Each string requires two separate dynamic allocations—one for the ObjString
// and a second for the character array. Accessing the characters from a value
// requires two pointer indirections, which can be bad for performance. A more
// efficient solution relies on a technique called flexible array members.
// Use that to store the ObjString and its character array in a single
// contiguous allocation.

// TODO!
// When we create the ObjString for each string literal, we copy the characters
// onto the heap. That way, when the string is later freed, we know it is safe
// to free the characters too.
// 
// This is a simpler approach but wastes some memory, which might be a problem
// on very constrained devices. Instead, we could keep track of which ObjStrings 
// own their character array and which are “constant strings” that just point 
// back to the original source string or some other non-freeable location. Add 
// support for this.
