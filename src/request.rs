/// An HTTP request
pub struct Request
{
	/// Full path
	pub path: String,
	/// Query parameters
	pub query: Vec<QueryParam>,
	/// Start position in a header like `Range: bytes=3702784-`
	pub range_start: usize,
}


/// An optional part after the question mark of a URL
pub struct QueryParam
{
	pub key: String,
	pub value: String,
}


/// The URL parse state
#[derive(Clone, Copy)]
enum UrlPartKind
{
	Path,
	Key,
	Value,
}


impl Request
{
	/// From bytes that could be an HTTP request
	pub fn parse(request: &[u8]) -> Option<Request>
	{
		let mut result = Request::new();

		// Parse a GET method or fail
		const GET: &[u8] = b"GET ";
		if !request.starts_with(GET) {
			return None;
		}
		let mut i = GET.len();

		// Parse the URL
		let mut part_kind = UrlPartKind::Path;
		let mut part_start: usize = i;
		let mut part = Vec::<u8>::new();
		while i < request.len() {
			let byte = request[i];
			match byte {
				// Finish if there's a space
				b' ' => {
					part.extend(&request[part_start..i]);
					result.insert_part(part_kind, &part);
					if result.path.ends_with(".mp4") {
						break;
					}
					else {
						return Some(result);
					}
				},
				// Percent encoding
				b'%' => {
					const PERCENT_LENGTH: usize = 3;
					let char_end = i + PERCENT_LENGTH;
					if char_end > request.len() {
						return None;
					}
					match core::str::from_utf8(&request[i+1..char_end]) {
						Ok(hex_str) => {
							match u8::from_str_radix(hex_str, 16) {
								Ok(hex_num) => {
									part.extend(&request[part_start..i]);
									part.push(hex_num);
									i += PERCENT_LENGTH;
									part_start = i;
									continue;
								},
								Err(_error) => return None,
							}
						},
						Err(_error) => return None,
					}
				}
				// Key of query component delimiter
				b'?' | b'&' => {
					part.extend(&request[part_start..i]);
					result.insert_part(part_kind, &part);
					part.clear();
					part_kind = UrlPartKind::Key;
					part_start = i + 1;
				}
				// Value of query component delimiter
				b'=' => {
					if !matches!(part_kind, UrlPartKind::Path) {
						part.extend(&request[part_start..i]);
						result.insert_part(part_kind, &part);
						part.clear();
						part_kind = UrlPartKind::Value;
						part_start = i + 1;
					}
				}
				// Skip if there's a valid URL character:
				// gen-delims, sub-delims, and unreserved in RFC 3986 Appendix A
				b'!' |
				b'#'..=b'$' |
				b'\''..=b';' |
				b'@'..=b'[' |
				b']' |
				b'_' |
				b'a'..=b'z' |
				b'~' => {

				}
				// Fail if there's anything else
				_ => return None,
			}
			i += 1;
		}

		// Parse the range start
		const SUBSLICE: &[u8] = b"\r\nRange: bytes=";
		let request = &request[i..];
		let mut found = false;
		let mut subslice_i = 0;
		let mut begin = 0;
		let mut end = 0;
		for (slice_i, &c) in request.iter().enumerate() {
			// Parse the subslice
			if !found {
				if c != SUBSLICE[subslice_i] {
					subslice_i = 0;
				}
				else if subslice_i == SUBSLICE.len() - 1 {
					found = true;
				}
				else {
					subslice_i += 1;
				}
			}
			// Parse the number
			else {
				match (begin, c) {
					(0, b'0'..=b'9') => begin = slice_i,
					(_, b'0'..=b'9') => (),
					(0, _) => return None,
					(_, _) => {
						end = slice_i;
						break;
					},
				}
			}
		}
		if !found {
			return Some(result);
		}
		let number_str = match core::str::from_utf8(&request[begin..end]) {
			Ok(number_str) => number_str,
			_ => return Some(result),
		};
		result.range_start = match usize::from_str_radix(number_str, 10) {
			Ok(number) => number,
			_ => return Some(result),
		};

		return Some(result);
	}


	/// Set the path, query key, or query value
	fn insert_part(self: &mut Request, kind: UrlPartKind, part: &[u8])
	{
		match core::str::from_utf8(part) {
			Ok(part) => {
				match kind {
					UrlPartKind::Path => {
						self.path = part.to_string();
					},
					UrlPartKind::Key => {
						self.query.push(QueryParam {
							key: part.to_string(),
							value: String::new(),
						});
					},
					UrlPartKind::Value => {
						if let Some(last) = self.query.last_mut() {
							last.value = part.to_string();
						}
					},
				}
			},
			Err(_error) => {},
		};
	}


	/// A blank URL
	fn new() -> Request
	{
		return Request {
			path: String::new(),
			query: vec![],
			range_start: 0,
		};
	}
}
