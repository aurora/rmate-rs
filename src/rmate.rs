/*
 * rmate-rs
 * Copyright (C) 2014 by Harald Lapp <harald@octris.org>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 *
 *
 * This software can be found at:
 * https://github.com/aurora/rmate-rs
 *
 * Thanks very much to all users and contributors! All bug-reports,
 * feature-requests, patches, etc. are greatly appreciated! :-)
 *
 */

#![allow(unused_must_use)]

extern crate libc;
extern crate getopts;

use getopts::{optopt, optflag, OptGroup};
use std::os;
use std::io;
use std::io::fs::PathExtensions;
use std::io::TcpStream;
use std::str;

static VERSION: &'static str = "0.0.1";
static VERSION_DATE: &'static str = "0000-00-00";

// default values
static mut VERBOSE: bool = false;

static HOST: &'static str = "localhost";
static PORT: &'static str = "52698";

/**
 * Determine hostname.
 * @see     https://github.com/uutils/coreutils/blob/master/src/hostname/hostname.rs
 */
fn gethostname() -> String {
    extern {
        fn gethostname(name: *mut libc::c_char, namelen: libc::size_t) -> libc::c_int;
    }

    let namelen = 256u;
    let mut name = Vec::from_elem(namelen, 0u8);

    let err = unsafe {
        gethostname (name.as_mut_ptr() as *mut libc::c_char,
                                        namelen as libc::size_t)
    };

    if err != 0 {
        log("Cannot determine hostname");
    }

    let last_char = name.iter().position(|byte| *byte == 0).unwrap_or(namelen);

    str::from_utf8(name.slice_to(last_char)).unwrap().to_string()
}

/*** WORKAROUND FOR MISSING realpath (ref: https://github.com/rust-lang/rust/issues/11857#issuecomment-55329505) ***/

#[cfg(unix)]
fn realpath(p: Path) -> Path {
    use libc::{c_char};
    use std::c_str::{CString};
    extern {
        fn realpath(path: *const c_char, resolved: *mut c_char) -> *const c_char;
    }
    let mut p = p.into_vec();
    p.push(0);
    let new_p = unsafe { realpath(p.as_ptr() as *const c_char, 0 as *mut c_char) };
    unsafe { Path::new(CString::new(new_p, true).as_bytes_no_nul()) }
}

#[cfg(windows)]
fn realpath(p: Path) -> Path {
    // TODO
    p
}

/*******************************************************************************************************************/

/**
 * Show usage information.
 */
fn showusage(program: &str, opts: &[OptGroup]) {
    println!("usage: {program} [options] file-path  edit specified file
   or: {program} [options] -          read text from stdin
   
{usage}",
    program = program,
    usage   = getopts::usage("Open a file in TextMate.", opts));
}

/**
 * Message logging.
 */
fn log(msg: &str) {
    unsafe {
        if VERBOSE {
            let mut out = io::stderr();
            out.write_str(format!("{}\n", msg).as_slice());
        }
    }
}

/**
 * Main.
 */
fn main() {
    let args    = os::args();
    let program = args[0].as_slice();
    
    let mut host = match os::getenv("RMATE_HOST") {
        Some(val) => val,
        None      => HOST.to_string()
    };
    
    let mut port = match os::getenv("RMATE_PORT") {
        Some(val) => val,
        None      => PORT.to_string()
    };
    
    let opts    = [
        getopts::optopt("H", "host", format!("Connect to HOST. Use 'auto' to detect the host from SSH. Defaults to {}.", host).as_slice(), "HOST"),
        getopts::optopt("p", "port", format!("Port number to use for connection. Defaults to {}.", port).as_slice(), "PORT"),
        getopts::optflag("w", "wait", "Wait for file to be closed by TextMate."),
        getopts::optopt("l", "line", "Place caret on line number after loading file.", "LINE"),
        getopts::optopt("m", "name", "The display name shown in TextMate.", "NAME"),
        getopts::optopt("t", "type", "Treat file as having specified type.", "TYPE"),
        getopts::optflag("f", "force", "Open even if file is not writable."),
        getopts::optflag("v", "verbose", "Verbose logging messages."),
        getopts::optflag("h", "help", "Display usage information."),
        getopts::optflag("", "version", "Show version and exit.")
    ];
    
    let matches = match getopts::getopts(args.tail(), opts) {
        Ok(m)  => m,
        Err(_) => {
            showusage(program, opts);
            os::set_exit_status(1);
        
            return;
        }
    };

    unsafe {
        VERBOSE = matches.opt_present("verbose");
    }

    if matches.free.is_empty() {
        showusage(program, opts);
        os::set_exit_status(1);
        
        return;
    }
    
    let filepath = matches.free[0].as_slice();
        
    if matches.free.len() > 1 {
        log(format!("There are more than one files specified. Opening only {} and ignoring other.", filepath).as_slice());
    }

    let (resolvedpath, displayname) = match filepath {
        "-" => (
                    "".to_string(),
                    format!("{}:untitled", gethostname())
                ),
        _   => {
            let tmp = Path::new(filepath);
            (
                (if tmp.exists() { realpath(tmp).as_str().to_string() } else { "".to_string() }),
                format!("{}:{}", gethostname(), filepath)
            )
        }
    };

    if matches.opt_present("help") {
        showusage(program, opts);
        os::set_exit_status(1);
        
        return;
    } else if matches.opt_present("version") {
        println!("rmate-rs {} ({})", VERSION, VERSION_DATE);
        os::set_exit_status(1);
        
        return;
    }
    
    match matches.opt_str("host") {
        Some(val) => host = val,
        None      => ()
    }
    match matches.opt_str("port") {
        Some(val) => port = val,
        None      => ()
    }
    let port = match from_str::<u16>(port.as_slice()) {
        Some(val) => val,
        None      => {
            log(format!("Invalid port specified {}.", port).as_slice());
            os::set_exit_status(1);
        
            return;
        }
    };
    
    let selection = match matches.opt_str("line") {
        Some(val) => val,
        None      => "".to_string()
    };
    let filetype = match matches.opt_str("type") {
        Some(val) => val,
        None      => "".to_string()
    };
    let verbose = matches.opt_present("verbose");
    let wait    = matches.opt_present("wait");
    let force   = matches.opt_present("force");
    
    // connect to TextMate
    let stream = TcpStream::connect(host.as_slice(), port);
    drop(stream);
}
