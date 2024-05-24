pub type Loc = (usize, usize);

/// Structs can implement this trait to easily return their loc
#[allow(unused)]
pub trait CodeLocation {
    fn loc(&self) -> Loc;
}
