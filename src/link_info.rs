use core::cmp::Ordering;
use std::collections::HashMap;

use crate::name_parts::*;


/// Information for a link to a video or video collection
pub struct LinkInfo
{
	/// Whether the name is a video collection
	pub is_folder: bool,
	/// Basename of a video file or folder
	pub basename: String,
	/// ".jpg", ".jpeg", ".png", or ".webp"
	pub poster_extension: &'static str,
}


impl LinkInfo
{
	/// List all entires in the path
	pub fn list(folder: &str) -> Vec<LinkInfo>
	{
		let dir = match std::fs::read_dir(folder) {
			Ok(dir) => dir,
			Err(_) => return Vec::new(),
		};

		let mut result = Vec::new();
		let mut poster_extensions = HashMap::new();

		for entry in dir {
			let entry = match entry {
				Ok(entry) => entry,
				Err(_) => continue,
			};

			let mut name = match entry.file_name().into_string() {
				Ok(name) => name,
				Err(_) => continue,
			};

			let parts = NameParts::new(&name);
			match parts.extension {
				"" => {
					result.push(LinkInfo {
						is_folder: true,
						basename: name,
						poster_extension: "",
					});
				},
				".mp4" => {
					name.truncate(parts.basename.len());
					result.push(LinkInfo {
						is_folder: false,
						basename: name,
						poster_extension: "",
					});
				},
				".jpg" => {
					name.truncate(parts.basename.len());
					poster_extensions.insert(name, ".jpg");
				},
				".jpeg" => {
					name.truncate(parts.basename.len());
					poster_extensions.insert(name, ".jpeg");
				},
				".png" => {
					name.truncate(parts.basename.len());
					poster_extensions.insert(name, ".png");
				},
				".webp" => {
					name.truncate(parts.basename.len());
					poster_extensions.insert(name, ".webp");
				},
				_ => (),
			}
		}

		result.sort();

		for link_info in &mut result {
			match poster_extensions.get(&link_info.basename) {
				Some(ext) => link_info.poster_extension = ext,
				None => (),
			}
		}

		return result;
	}
}


impl PartialEq for LinkInfo
{
	fn eq(&self, other: &LinkInfo) -> bool
	{
		return self.basename == other.basename;
	}
}


impl Eq for LinkInfo
{}


impl PartialOrd for LinkInfo
{
	fn partial_cmp(&self, other: &LinkInfo) -> Option<Ordering>
	{
		return self.basename.partial_cmp(&other.basename);
	}
}


impl Ord for LinkInfo
{
	fn cmp(&self, other: &LinkInfo) -> Ordering
	{
		return self.basename.cmp(&other.basename);
	}
}
