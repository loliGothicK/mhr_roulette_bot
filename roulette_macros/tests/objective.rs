#![feature(format_args_capture)]
#![deny(clippy::all, clippy::pedantic)]
use difference::assert_diff;
use roulette_macros_impl::Objective;

#[derive(Objective)]
pub enum Test { Passed }

#[test]
fn obj() {
    let test = Test::Passed;
    assert_diff!("Dummy", &format!("{test}"), "\n", 0);
}
