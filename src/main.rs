use libc;
use std::{fs::File, io::{BufRead, Read}, path::{Path, PathBuf}};

// Event interface: https://kernel.org/doc/html/latest/input/input.html#event-interface
#[repr(C)]
struct InputEvent {
  time: libc::timeval,
  kind: u16,
  code: u16,
  value: u32,
}

// include/uapi/linux/input-event-codes.h.
const EV_KEY: u16 = 0x01;

fn key_log(evdev: PathBuf) {
  let mut file = File::open(evdev).unwrap();
  let mut ev: InputEvent = unsafe { std::mem::zeroed() };
  let size = std::mem::size_of::<InputEvent>();

  loop {
    unsafe {
      let ev_slice = std::slice::from_raw_parts_mut(&mut ev as *mut InputEvent as *mut u8, size);
      file.read_exact(ev_slice).unwrap();
    }
    if ev.kind != EV_KEY || ev.value != 0 {
      continue;
    }
    
    println!("code: {}", ev.code);
  }
}

fn find_kbd_evdev() -> Result<String, String> {
  let file = File::open("/proc/bus/input/devices").unwrap();
  let reader = std::io::BufReader::new(file);
  let mut kbd_section = false;

  for line_result in reader.lines() {
    if let Ok(line) = line_result {
      if line.len() == 0 {
        kbd_section = false;
        continue;
      }
      if line.starts_with("N: Name=") && line.contains("keyboard") {
        kbd_section = true;
        continue;
      }
      if !line.starts_with("H: Handlers=") || !kbd_section {
        continue;
      }

      let evstr = line.strip_prefix("H: Handlers=").unwrap();
      for evdev in evstr.split(" ") {
        if evdev.starts_with("event") {
          return Ok(String::from(evdev));
        }
      }
    }
  }

  Err(String::from("Cannot find the evdev for the keyboard."))
}

fn main() {
  let evdev = Path::new("/dev/input/");
  let name = find_kbd_evdev().unwrap();

  key_log(evdev.join(name));
}
