use core::str::FromStr;


/// Folder of videos, posters, and subtitles
const DEFAULT_FOLDER: &str = ".";
/// Port for the TCP server
const DEFAULT_PORT: u16 = 80;


/// Program mode from CLI arguments
pub enum Mode
{
	Error(String),
	Normal(Config),
	Help,
	Version,
}
use Mode::*;


/// Configuration for this application from JSON
pub struct Config
{
	/// Root folder of the videos, which is "." by default
	pub folder: String,
	/// Port to listen to, which is 80 by default
	pub port: u16,
}


impl Mode
{
	/// Construct from the arguments
	pub fn new() -> Mode
	{
		#[derive(Clone, Copy)]
		enum State
		{
			Begin,
			Folder,
			Port,
		}
		use State::*;

		const FOLDER: &str = "--folder";
		const PORT: &str = "--port";

		let mut state = Begin;

		let mut folder = String::new();
		let mut port = DEFAULT_PORT;

		let mut arg_copy = "";

		for arg in std::env::args().skip(1) {
			match (state, arg.as_str()) {
				(_, "-h" | "--help") => return Help,
				(_, "-v" | "--version") => return Version,
				(Begin, FOLDER) => {
					state = Folder;
					arg_copy = FOLDER;
				},
				(Begin, PORT) => {
					state = Port;
					arg_copy = PORT;
				},
				(Folder, _) => {
					folder = arg;
					state = Begin;
				},
				(Port, _) => {
					port = match u16::from_str(&arg) {
						Ok(port) => port,
						Err(_) => return Error(format!("Expected a port number but got \"{arg}\"\n")),
					};
					state = Begin;
				},
				_ => return Error(format!("Expected a valid argument but got \"{arg}\"\n")),
			}
		}

		if !matches!(state, Begin) {
			return Error(format!("Expected an argument after \"{arg_copy}\"\n"))
		}

		if folder.is_empty() {
			folder = String::from(DEFAULT_FOLDER);
		}

		return Normal(Config {
			folder,
			port,
		});
	}
}
