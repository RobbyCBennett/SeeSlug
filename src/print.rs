use std::io::Write;


/// Print to stdout without crashing
pub fn print(string: &str)
{
	let _ = std::io::stdout().write_all(string.as_bytes());
}


/// Print to stderr without crashing
pub fn eprint(string: &str)
{
	let _ = std::io::stderr().write_all(string.as_bytes());
}
