mod arguments;
mod http;
mod languages;
mod link_info;
mod name_parts;
mod print;
mod request;
mod status;
mod thread_pool;


use core::ffi::c_int;
use core::mem::transmute;
use core::net::Ipv4Addr;
use std::net::TcpListener;

use crate::arguments::*;
use crate::http::*;
use crate::thread_pool::*;
use crate::print::*;


/// Number to send to the shell
enum ExitCode
{
	Success,
	BadArgs,
	FailedToListen,
}
use ExitCode::*;


/// Run a multi-threaded video server
fn main()
{
	const SIGINT: core::ffi::c_int = 2;
	unsafe extern "C"
	{
		fn signal(signum: core::ffi::c_int, handler: usize) -> usize;
	}

	// Handle signals without displaying error messages
	unsafe { signal(SIGINT, handle_interrupt as usize); }

	// Get the normal program mode from the arguments or exit early
	let config = match Mode::new() {
		Mode::Error(error) => {
			eprint(&error);
			std::process::exit(BadArgs as i32);
		},
		Mode::Normal(config) => config,
		Mode::Help => return print_help(),
		Mode::Version => return print_version(),
	};

	// Create a network listener or fail
	let mut listener = match TcpListener::bind((Ipv4Addr::UNSPECIFIED, config.port)) {
		Ok(listener) => listener,
		Err(error) => {
			eprint(&format!("Failed to start listening to port {} - {error}\nHint: try another number with the --port argument\n", config.port));
			std::process::exit(FailedToListen as i32);
		}
	};
	#[cfg(target_os = "windows")]
	fix_listener(&mut listener);

	// Treat the data created in main as static
	let root_folder = unsafe { transmute::<&str, &'static str>(&config.folder) };

	// Listen to the clients and have the thread pool handle them
	let pool = ThreadPool::new();
	for stream in listener.incoming() {
		match stream {
			Ok(stream) => pool.execute(|| handle_request(root_folder, stream)),
			Err(_) => (),
		}
	}
}


/// Prevent the Windows firewall from resetting incoming connections
/// (solution found by randomly changing different socket options)
#[cfg(target_os = "windows")]
fn fix_listener(listener: &mut TcpListener)
{
	use core::ffi::c_char;
	use std::os::windows::io::AsRawSocket;
	use std::os::windows::raw::SOCKET;

	unsafe extern "C"
	{
		// https://learn.microsoft.com/en-us/windows/win32/api/winsock/nf-winsock-setsockopt
		fn setsockopt(s: SOCKET, level: c_int, optname: c_int, optval: *const c_char, optlen: c_int) -> c_int;
	}

	let socket = listener.as_raw_socket() as SOCKET;

	/// Socket level (`SOL_SOCKET`)
	/// https://learn.microsoft.com/en-us/windows/win32/winsock/sol-socket-socket-options
	const OPTION_LEVEL: c_int = 0xffff;
	/// Socket: send buffer size (`SO_SNDBUF`)
	const OPTION_NAME: c_int = 0x1001;

	// A buffer size of zero somehow fixes the issue
	let mut option_value: c_int = 0;
	let option_value = unsafe { transmute(&mut option_value) };

	if unsafe { setsockopt(socket, OPTION_LEVEL, OPTION_NAME, option_value, size_of::<c_int>() as c_int) } != 0 {
		eprint("Warning: Failed to configure the port to prevent Windows from resetting connections\n");
	}
}


/// Print help for this app
fn print_help()
{
	print(concat!(
		"See Slug\n",
		"Web server to stream or download videos\n",
		"\n",
		"https://github.com/RobbyCBennett/SeeSlug\n",
		"\n",
		"Config arguments:\n",
		"    --folder  (default: \".\")\n",
		"    --port    (default: 80)\n",
		"\n",
		"Other arguments:\n",
		"    --help or -h\n",
		"    --version or -v\n",
	));
}


/// Print the version of this app
fn print_version()
{
	print(concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"), "\n"));
}


/// Immediately exit
extern "C" fn handle_interrupt(_signal: c_int)
{
	std::process::exit(Success as i32);
}
