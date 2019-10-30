# Why This Fork Exists

In my particular use case (a pair of input devices I wanted to read from at any time, but which would generally not be moving much) gilrs did exactly what I wanted (emit events in a nice format) but did so by requiring a busy loop that ate an entire CPU core even when the inputs were not moving. This was unfortunate.

## How it works

In order to prevent the need for a busy-loop this fork adds a `.get_fds()` method to `Gilrs` instances which can be used to get the `RawFd`s of the `/dev/input/event*` files backing the input devices. These fds are used internally by gilrs and can't be used directly with mio/epoll, but fortunately linux will let you `dup(2)` them and put your own polls on those to get the expected results.

Just poll those fds until at least one is readable, and run through all events gilrs can extract, rinse and repeat.

**This breaks parts of gilrs**, just not parts I care about. At the least it interupts the hotplugging feature of gilrs.

## How it could work far better

Upstream Gilrs is already investigating [Add option for blocking reads](https://gitlab.com/gilrs-project/gilrs/issues/63) which would be much more cleanly, but this hack is fairly easy to set up and use for me in the interim.

## Example

```rust
use libc;
use gilrs::{Event, Gilrs};
use mio::*;
use mio::unix::EventedFd;

fn main() {
	let mut gilrs = Gilrs::new().unwrap();

	let poll = Poll::new().unwrap()
	let mut events = Events::with_capacity(1024);
	let token = Token(0);

	loop {
		// drain any events readable now
		while let Some(Event { .. }) = gilrs.next_event() {
			println!("got event");
		}

		// wait for `next_event` to be likely to yield again
		let raw_fds = gilrs.get_fds();
		let ev_fds = raw_fds.iter().map(|fd| 
			let fd = unsafe { libc::dup(fd) };
			EventedFd(&fd)
		);
		for fd in ev_fds.iter() {
			let _ = poll.register(&fd, token, Ready::readable(), PollOpt::edge());
		}
		poll.poll(&mut events, None).unwrap();

		// cleanup the last polling
		for fd in ev_fds.iter() {
			let _ = poll.deregister(&fd);
		}
		for fd in raw_fds.iter() {
			unsafe { libc::close(fd) };
		}
	}
}
```
