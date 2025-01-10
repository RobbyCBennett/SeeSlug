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


/// Run a multi-threaded video server
fn main()
{
	// Handle signals without displaying error messages
	unsafe { libc::signal(libc::SIGINT, handle_interrupt as libc::sighandler_t); }

	// Get the normal program mode from the arguments or exit early
	let config = match Mode::new() {
		Mode::Error(error) => {
			eprint(&error);
			std::process::exit(ExitCode::BadArgs as i32);
		},
		Mode::Normal(config) => config,
		Mode::Help => return print_help(),
		Mode::Version => return print_version(),
	};

	// Create a network listener or fail
	let mut listener = match TcpListener::bind((Ipv4Addr::UNSPECIFIED, config.port)) {
		Ok(listener) => listener,
		Err(error) => {
			eprint(&format!("Failed to start listening to {} ({error})\n", config.port));
			std::process::exit(ExitCode::FailedToListen as i32);
		}
	};
	fix_listener(&mut listener);

	// Treat the data created in main as static
	let root_folder = unsafe { transmute::<&str, &'static str>(&config.folder) };

	// Listen to the clients and have the thread pool handle them
	let pool = ThreadPool::new();
	for stream in listener.incoming() {
		match stream {
			Ok(stream) => pool.execute(|| handle_request(root_folder, stream)),
			Err(_error) => {},
		}
	}
}


/// Prevent the Windows firewall from resetting incoming connections
/// (solution found by randomly changing different socket options)
fn fix_listener(listener: &mut TcpListener)
{
	#[cfg(target_os = "windows")] {
		use std::os::windows::io::AsRawSocket;
		let socket = listener.as_raw_socket() as libc::SOCKET;

		/// Socket level (`SOL_SOCKET`)
		/// https://learn.microsoft.com/en-us/windows/win32/winsock/sol-socket-socket-options
		const OPTION_LEVEL: c_int = 0xffff;
		/// Socket: send buffer size (`SO_SNDBUF`)
		const OPTION_NAME: c_int = 0x1001;

		// A buffer size of zero somehow fixes the issue
		let mut option_value: c_int = 0;
		let option_value = unsafe { transmute(&mut option_value) };

		if unsafe { libc::setsockopt(socket, OPTION_LEVEL, OPTION_NAME, option_value, size_of::<c_int>() as c_int) } != 0 {
			eprint("Failed to configure the server socket\n");
			std::process::exit(ExitCode::FailedToListen as i32);
		}
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
	std::process::exit(ExitCode::Success as i32);
}
