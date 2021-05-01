use std::{alloc, fmt::Display, intrinsics::copy_nonoverlapping, mem};

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Obj(*mut BaseObj);

impl Obj {
    pub fn string(s: &str) -> Self {
        unsafe {
            assert!(s.len() < u32::MAX as usize);

            let byte_pointer_layout = alloc::Layout::from_size_align(s.len(), 1).unwrap();
            let byte_pointer: *mut u8 = alloc::alloc(byte_pointer_layout);
            copy_nonoverlapping(s.as_ptr(), byte_pointer, s.len());

            let obj = StringObj {
                base: BaseObj {
                    kind: ObjKind::String,
                },
                length: s.len() as u32,
                chars: byte_pointer,
            };

            let obj_layout = alloc::Layout::from_size_align(
                mem::size_of::<StringObj>(),
                mem::align_of::<StringObj>(),
            )
            .unwrap();
            let obj_pointer: *mut StringObj = mem::transmute(alloc::alloc(obj_layout));
            copy_nonoverlapping(&obj, obj_pointer, 1);

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
}

#[repr(C)]
struct StringObj {
    base: BaseObj,
    length: u32,
    chars: *const u8,
}

impl StringObj {
    unsafe fn as_str(&self) -> &str {
        let slice = std::slice::from_raw_parts(self.chars, self.length as usize);
        std::str::from_utf8_unchecked(slice)
    }
}
