pub trait Satisfied {}

pub struct Condition<const B: bool>;
impl Satisfied for Condition<true> {}

pub trait SameAs<T> {}
impl<T> SameAs<T> for T {}
