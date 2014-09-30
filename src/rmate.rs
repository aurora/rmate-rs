/*
 * rmate-ooc
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

extern crate getopts;
use getopts::{optopt, optflag, OptGroup};
use std::os;

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
 * Main.
 */
fn main() {
    let args    = os::args();
    let program = args[0].clone();
    let opts    = [
        getopts::optopt("H", "host", "Connect to HOST. Use 'auto' to detect the host from SSH.", "HOST"),
        getopts::optopt("p", "port", "Port number to use for connection.", "PORT"),
        getopts::optflag("w", "wait", "Wait for file to be closed by TextMate."),
        getopts::optopt("l", "line", "Place caret on line number after loading file.", "LINE"),
        getopts::optopt("m", "name", "The display name shown in TextMate.", "NAME"),
        getopts::optopt("t", "type", "Treat file as having specified type.", "TYPE"),
        getopts::optflag("f", "force", "Open even if file is not writable."),
        getopts::optflag("v", "verbose", "Verbose logging messages."),
        getopts::optflag("h", "help", "Display usage information."),
        getopts::optflag("", "version", "Show version and exit.")
    ];
    
    showusage(program.as_slice(), opts);
}