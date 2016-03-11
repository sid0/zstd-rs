use std::io;

use time;

pub fn time_fn<R, F: FnOnce() -> io::Result<R>>(f: F) -> io::Result<(u64, R)> {
    let start = time::precise_time_ns();

    let r = try!(f());

    let end = time::precise_time_ns();

    Ok((end - start, r))
}
