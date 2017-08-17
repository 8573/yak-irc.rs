use std::fmt;

pub mod debug_repr {
    pub struct Fn;
}

impl fmt::Debug for debug_repr::Fn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("<function>")
    }
}
