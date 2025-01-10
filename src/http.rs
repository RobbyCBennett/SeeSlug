use core::mem::MaybeUninit;
use std::borrow::Cow;
use std::io::IoSlice;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
use std::net::TcpStream;

use crate::languages::*;
use crate::link_info::*;
use crate::name_parts::*;
use crate::request::*;
use crate::status::*;


/// Maximum size in bytes of an incoming HTTP request
const REQUEST_SIZE: usize = 4096;

// Size in bytes of a portion of a video (don't exceed this)
const VIDEO_BUFFER_SIZE: usize = 1 << 21;


/// Read from the incoming request and either respond or shut it down
pub fn handle_request(root_folder: &str, mut stream: TcpStream)
{
	let stream = &mut stream;

	// Read all bytes into the buffer or fail
	#[allow(invalid_value)]
	let mut request: [u8; REQUEST_SIZE] = unsafe { MaybeUninit::uninit().assume_init() };
	let request_length = match stream.read(&mut request) {
		Ok(request_length @ 0..REQUEST_SIZE) => request_length,
		_ => return,
	};

	// Truncate the request slice
	let request = &request[0..request_length];

	// Parse the Request and respond
	match Request::parse(&request) {
		None => respond_status(stream, Status::BadRequest),
		Some(request) => respond_file(root_folder, stream, request),
	}
}


/// Write a response given a file path
fn respond_file(root_folder: &str, stream: &mut TcpStream, request: Request)
{
	const CONTENT_TYPE_CSS:   &str = "text/css";
	const CONTENT_TYPE_HTML:  &str = "text/html";
	const CONTENT_TYPE_JPG:   &str = "image/jpeg";
	const CONTENT_TYPE_JS:    &str = "text/javascript";
	const CONTENT_TYPE_JSON:  &str = "application/json";
	const CONTENT_TYPE_MP4:   &str = "video/mp4";
	const CONTENT_TYPE_PNG:   &str = "image/png";
	const CONTENT_TYPE_SVG:   &str = "image/svg+xml";
	const CONTENT_TYPE_VTT:   &str = "text/vtt";
	const CONTENT_TYPE_WEBP:  &str = "image/webp";
	const CONTENT_TYPE_WOFF2: &str = "font/woff2";

	let mut buffer = Vec::new();

	let (content_type, content) = match request.path.as_str() {
		"/alata.woff2" => (
			CONTENT_TYPE_WOFF2,
			include_bytes!("../res/alata.woff2").as_slice()),
		"/logo.svg" => (
			CONTENT_TYPE_SVG,
			include_str!("../res/logo.svg").as_bytes()),
		"/logo_circle.svg" => (
			CONTENT_TYPE_SVG,
			include_str!("../res/logo_circle.svg").as_bytes()),
		"/manifest.json" => (
			CONTENT_TYPE_JSON,
			include_str!("../res/manifest.json").as_bytes()),
		"/saira_condensed.woff2" => (
			CONTENT_TYPE_WOFF2,
			include_bytes!("../res/saira_condensed.woff2").as_slice()),
		"/style.css" => (
			CONTENT_TYPE_CSS,
			include_str!("../res/style.css").as_bytes()),
		"/video.js" => (
			CONTENT_TYPE_JS,
			include_str!("../res/video.js").as_bytes()),
		"/videos.js" => (
			CONTENT_TYPE_JS,
			include_str!("../res/videos.js").as_bytes()),
		client_path => {
			if has_parent_dir(client_path) {
				return respond_status(stream, Status::NotFound);
			}

			let full_path = format!("{root_folder}{client_path}");

			match full_path.ends_with("/") {
				// Generated HTML
				true => {
					let mut video_name = "";
					for query in &request.query {
						if query.key != "watch" {
							continue;
						}
						video_name = query.value.as_str();
					}

					if video_name.is_empty() {
						(CONTENT_TYPE_HTML, make_html_videos(&full_path, client_path, &mut buffer))
					}
					else {
						(CONTENT_TYPE_HTML, make_html_video(&full_path, video_name, &mut buffer))
					}
				},
				// File from the filesystem
				false => {
					let content_type = match get_last_extension(&full_path) {
						".jpg" | ".jpeg" => CONTENT_TYPE_JPG,
						".js" => CONTENT_TYPE_JS,
						".png" => CONTENT_TYPE_PNG,
						".webp" => CONTENT_TYPE_WEBP,
						".vtt" => CONTENT_TYPE_VTT,
						".mp4" => {
							// Get the file or fail
							let mut file = match std::fs::File::open(&full_path) {
								Ok(file) => file,
								Err(_) => return respond_status(stream, Status::NotFound),
							};

							// Seek or fail
							let seek = match request.range_start.try_into() {
								Ok(seek) => seek,
								Err(_) => return respond_status(stream, Status::InternalServerError),
							};
							match file.seek_relative(seek) {
								Ok(()) => (),
								Err(_) => return respond_status(stream, Status::InternalServerError),
							}

							// Get the size or fail
							let total_size = match file.metadata() {
								Ok(metadata) => metadata.len(),
								Err(_) => return respond_status(stream, Status::InternalServerError),
							};

							// Allocate space or fail
							let mut buffer = Vec::new();
							match buffer.try_reserve(VIDEO_BUFFER_SIZE) {
								Ok(()) => (),
								Err(_) => return respond_status(stream, Status::InternalServerError),
							}

							// Read the bytes or fail
							unsafe { buffer.set_len(VIDEO_BUFFER_SIZE) }
							match file.read(&mut buffer) {
								Ok(0) => return respond_status(stream, Status::InternalServerError),
								Ok(size) => unsafe { buffer.set_len(size) },
								Err(_) => return respond_status(stream, Status::InternalServerError),
							}

							// Respond as partial content
							// (needed to support seeking and big files)
							let begin = request.range_start;
							let end = begin + buffer.len() - 1;
							return respond_partial_content(stream, CONTENT_TYPE_MP4, &buffer, begin, end, total_size);
						},
						_ => return respond_status(stream, Status::NotFound),
					};

					buffer = match std::fs::read(&full_path) {
						Ok(buffer) => buffer,
						Err(_) => return respond_status(stream, Status::NotFound),
					};

					(content_type, buffer.as_slice())
				}
			}
		},
	};

	respond_status_and_content(stream, Status::Okay, content_type, content);
}


/// Write a response given a status code
fn respond_status(stream: &mut TcpStream, status: Status)
{
	let _ = stream.write_all(status.to_response().as_bytes());
}


/// Write a response given some non-video content
fn respond_status_and_content(stream: &mut TcpStream, status: Status, content_type: &str, content: &[u8])
{
	let _ = stream.write_vectored(&[
		IoSlice::new(b"HTTP/1.1 "),
		IoSlice::new(status.to_str().as_bytes()),
		IoSlice::new(b"\r\nContent-Length: "),
		IoSlice::new(content.len().to_string().as_bytes()),
		IoSlice::new(b"\r\nContent-Type: "),
		IoSlice::new(content_type.as_bytes()),
		IoSlice::new(b"\r\n\r\n"),
		IoSlice::new(content),
	]);
}


/// Write a response given some non-video content
fn respond_partial_content(stream: &mut TcpStream, content_type: &str, content: &[u8], begin: usize, end: usize, total_size: u64)
{
	let _ = stream.write_vectored(&[
		IoSlice::new(b"HTTP/1.1 206 Partial Content\r\nContent-Length: "),
		IoSlice::new(content.len().to_string().as_bytes()),
		IoSlice::new(b"\r\nContent-Range: bytes "),
		IoSlice::new(begin.to_string().as_bytes()),
		IoSlice::new(b"-"),
		IoSlice::new(end.to_string().as_bytes()),
		IoSlice::new(b"/"),
		IoSlice::new(total_size.to_string().as_bytes()),
		IoSlice::new(b"\r\nContent-Type: "),
		IoSlice::new(content_type.as_bytes()),
		IoSlice::new(b"\r\n\r\n"),
		IoSlice::new(content),
	]);
}


/// Whether the path has the parent directory in it ("..")
fn has_parent_dir(path: &str) -> bool
{
	enum State
	{
		Begin,
		DotOne,
		DotTwo,
		Other,
	}

	let mut state = State::Begin;

	for c in path.chars() {
		state = match (state, c) {
			(State::Begin, '.') => State::DotOne,
			(State::Begin, '/' | '\\') => State::Begin,
			(State::Begin, _) => State::Other,

			(State::DotOne, '.') => State::DotTwo,
			(State::DotOne, '/' | '\\') => State::Begin,
			(State::DotOne, _) => State::Other,

			(State::DotTwo, '/' | '\\') => return true,
			(State::DotTwo, _) => State::Other,

			(State::Other, '/' | '\\') => State::Begin,
			(State::Other, _) => State::Other,
		};
	}

	return matches!(state, State::DotTwo);
}


/// Escape the special HTML characters from a string
fn escape_html(slice: &str) -> Cow<str>
{
	// Copy and escape if any escaped characters are found
	let mut escaped_string = String::new();
	let mut found_special = false;
	for (i, c) in slice.char_indices() {
		let escaped_char = match c {
			'"' => "&quot;",
			'&' => "&amp;",
			'\'' => "&apos;",
			'<' => "&lt;",
			'>' => "&gt;",
			_ => {
				match found_special {
					true => escaped_string.push(c),
					false => (),
				}
				continue;
			},
		};
		if !found_special {
			found_special = true;
			escaped_string.push_str(&slice[0..i]);
		}
		escaped_string.push_str(escaped_char);
	}

	// Borrow or own
	return match escaped_string.is_empty() {
		true => Cow::Borrowed(slice),
		false => Cow::Owned(escaped_string),
	};
}


/// Get the last part of the extension, for example "/Movie.en.vtt" is ".vtt"
fn get_last_extension(path: &str) -> &str
{
	let mut result = "";

	for (i, c) in path.char_indices() {
		match c {
			'/' => result = "",
			'.' => result = &path[i..],
			_ => (),
		}
	}

	return result;
}


/// Make the HTML page for a folder which lists videos
fn make_html_videos<'a>(full_folder: &str, client_folder: &str, buffer: &'a mut Vec<u8>) -> &'a [u8]
{
	let folder_name = get_folder_name(client_folder);
	let page_subtitle = match folder_name.is_empty() {
		true => String::new(),
		false => format!("{} - ", escape_html(folder_name)),
	};

	let mut video_links = String::new();
	for link_info in LinkInfo::list(full_folder) {
		let basename = escape_html(&link_info.basename);
		let poster = match link_info.poster_extension {
			"" => String::new(),
			extension => format!("<img src='{}{}' loading='lazy'>", basename, extension),
		};
		let video_link = match link_info.is_folder {
			true => format!(concat!(
				"<div class='col-6 col-sm-4 col-md-3 col-lg-2'>",
					"<a class='video_link' href='{}/'>",
						"<div class='poster'>",
							"{}",
							"<div class='overlay'></div>",
						"</div>",
						"<p>{}</p>",
					"</a>",
				"</div>",
				), basename, poster, basename),
			false => format!(concat!(
				"<div class='col-6 col-sm-4 col-md-3 col-lg-2'>",
					"<a class='video_link' href='?watch={}'>",
						"<div class='poster'>",
							"{}",
							"<div class='overlay'></div>",
						"</div>",
						"<p>{}</p>",
					"</a>",
				"</div>",
				), basename, poster, basename),
		};
		video_links.push_str(&video_link);
	}
	let video_links = video_links;

	buffer.extend(format!(concat!(
		"<!DOCTYPE html>",
		"<html lang='en' dir='ltr'>",
			"<head>",
				"<meta charset='utf-8'>",
				"<meta name='viewport' content='width=device-width, initial-scale=1'>",
				"<title>{}See Slug</title>",
				"<link type='text/css' rel='stylesheet' href='/style.css' as='style'>",
				"<link type='font/woff2' rel='preload' href='/alata.woff2' as='font' crossorigin>",
				"<link type='font/woff2' rel='preload' href='/saira_condensed.woff2' as='font' crossorigin>",
				"<link type='image/svg+xml' rel='icon' href='/logo_circle.svg'>",
				"<link rel='manifest' href='/manifest.json' />",
			"</head>",
			"<body id='videos_body'>",
				"<header>",
					"<a href='/'>",
						"<img src='/logo.svg' alt='Logo for See Slug' loading='lazy'>",
						"<h1>See Slug</h1>",
					"</a>",
				"</header>",
				"<div class='container g-0'>",
					"<div class='row g-4'>{}</div>",
				"</div>",
				"<script src='/videos.js'></script>",
			"</body>",
		"</html>",
		), &page_subtitle, &video_links).as_bytes());

	return buffer.as_slice();
}


fn make_html_video<'a>(folder: &str, video_name: &str, buffer: &'a mut Vec<u8>) -> &'a [u8]
{
	let subtitles = list_subtitles(folder, video_name);

	let video_name = escape_html(video_name);

	buffer.extend(format!(concat!(
		"<!DOCTYPE html>",
		"<html lang='en' dir='ltr'>",
			"<head>",
				"<meta charset='utf-8'>",
				"<meta name='viewport' content='width=device-width, initial-scale=1'>",
				"<title>{} - See Slug</title>",
				"<link type='text/css' rel='stylesheet' href='/style.css' as='style'>",
				"<link type='font/woff2' href='/alata.woff2' as='font'>",
				"<link type='image/svg+xml' rel='icon' href='/logo_circle.svg'>",
				"<link rel='manifest' href='/manifest.json' />",
			"</head>",
			"<body id='video_body'>",
				"<video id='video' src='{}.mp4' autoplay controls>{}</video>",
				"<section id='controls' class='hidden'>",
					"<div id='buttons'>",
						"<button id='play_pause' title='Pause - Space'>",
							"<svg id='play' class='hidden' viewBox='0 0 32 32'>",
								"<path d='M10.0718 8L23.9281 16L10.0718 24V8Z'/>",
							"</svg>",
							"<svg id='pause' viewBox='0 0 32 32'>",
								"<path d='M9.06995 23.9999V8M22.9299 24V8.00006'/>",
							"</svg>",
						"</button>",
						"<button id='captions' title='Captions Off - C'>",
							"<svg id='captions_on' class='hidden' viewBox='0 0 32 32'>",
								"<path d='M14.9097 13.884C13.2968 12.982 11.0515 13.6619 11.0515 16C11.0515 18.338 13.2968 19.018 14.9097 18.1161M20.9485 13.884C19.3356 12.982 17.0903 13.6619 17.0903 16C17.0903 18.338 19.3356 19.018 20.9485 18.1161M8 8H24V24H8V8Z'/>",
							"</svg>",
							"<svg id='captions_off' viewBox='0 0 32 32'>",
								"<path d='M14.9097 13.884C13.2968 12.982 11.0515 13.6619 11.0515 16C11.0515 18.338 13.2968 19.018 14.9097 18.1161M20.9486 13.884C19.3356 12.982 17.0903 13.6619 17.0903 16C17.0903 18.338 19.3356 19.018 20.9486 18.1161M8 8H24V24H8V8Z'/>",
								"<path d='M8 8L24 24'/>",
							"</svg>",
						"</button>",
						"<button id='picture_in_picture' title='Picture in Picture - P'>",
							"<svg viewBox='0 0 32 32'>",
								"<path d='M11.6266 24H6V8H26V11.2495M14.75 15H26V24H14.75V15Z'/>",
							"</svg>",
						"</button>",
						"<button id='download' title='Download'>",
							"<svg viewBox='0 0 32 32'>",
								"<path d='M16 8V23.9983M24 16L16 24L8 16'/>",
							"</svg>",
						"</button>",
						"<button id='fullscreen' title='Fullscreen - F'>",
							"<svg id='enter_fullscreen' viewBox='0 0 32 32'>",
								"<path d='M13 8L8 8L8 13M24 13L24 8L19 8M19 24L24 24L24 19M8 19L8 24L13 24'/>",
							"</svg>",
							"<svg id='exit_fullscreen' class='hidden' viewBox='0 0 32 32'>",
								"<path d='M8 13H13V8M19 8V13H24M24 19H19V24M13 24L13 19H8'/>",
							"</svg>",
						"</button>",
					"</div>",
					"<div id='times_and_progress' class='invisible'>",
						"<div class='time'>",
							"<span id='current_time'></span>",
						"</div>",
						"<div id='progress_track' title='Seek'>",
							"<div id='progress'></div>",
						"</div>",
						"<div class='time right'>",
							"<span id='total_time'></span>",
						"</div>",
					"</div>",
				"</section>",
				"<dialog id='dialog'>",
					"<p id='dialog_message'></p>",
					"<form method='dialog'>",
						"<button>Close</button>",
					"</form>",
				"</dialog>",
				"<script src='/video.js'></script>",
			"</body>",
		"</html>",
		), video_name, video_name, subtitles).as_bytes());

	return buffer.as_slice();
}


/// Given a path like "/Star Wars/Prequels" or "/" get "Prequels" or ""
fn get_folder_name<'a>(path: &'a str) -> &'a str
{
	let mut begin = 0;
	let mut end = 0;

	// Get the characters between the slashes, where addition is okay because
	// it's always on the index of '/'
	for (i, c) in path.char_indices() {
		if c != '/' {
			continue;
		}
		if begin == 0 {
			begin = i + 1;
		}
		else if end == 0 {
			end = i;
		}
		else {
			begin = end + 1;
			end = i;
		}
	}

	if begin == 0 || end == 0 {
		return "";
	}

	return &path[begin..end];
}


/// List the file names of the subtitles for the video in the folder
fn list_subtitles(folder: &str, video_name: &str) -> String
{
	const VTT: &str = ".vtt";

	let mut result = String::new();

	let dir = match std::fs::read_dir(folder) {
		Ok(dir) => dir,
		Err(_) => return result,
	};

	// Find the subtitles for this video
	for entry in dir {
		let entry = match entry {
			Ok(entry) => entry,
			Err(_) => continue,
		};

		let name = match entry.file_name().into_string() {
			Ok(name) => name,
			Err(_) => continue,
		};

		let parts = NameParts::new(&name);

		// Skip if the not a subtitle for this video
		let ext = parts.extension;
		if parts.basename != video_name || !ext.ends_with(VTT) {
			continue;
		}

		// Parse the subtitle extension
		// - NAME.vtt
		// - NAME.default.vtt
		// - NAME.default.LANG.vtt
		// - NAME.LANG.vtt
		const DEFAULT: &str = ".default.";
		let (default, language_short) = match ext.starts_with(DEFAULT) {
			true => match ext.len() > DEFAULT.len() + VTT.len() {
				true => ("default", &ext[DEFAULT.len() .. (ext.len()-VTT.len())]),
				false => ("default", ""),
			},
			false => match ext.len() > VTT.len() {
				true => ("", &ext[1 .. (ext.len()-VTT.len())]),
				false => ("", ""),
			},
		};

		let language_long = language_abbrevation_to_name(language_short);

		result += &format!("<track {} kind='subtitles' srclang='{}' label='{}' src='{}'>",
			default, escape_html(language_short), language_long, escape_html(&name));
	}

	return result;
}
