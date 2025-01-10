/// The file name and extension
pub struct NameParts<'a>
{
	/// Before the first dot
	pub basename: &'a str,
	/// The first dot until the end
	pub extension: &'a str,
}


impl<'a> NameParts<'a>
{
	/// Split the name
	pub fn new(name: &'a str) -> NameParts<'a>
	{
		let mut basename = name;
		let mut extension = "";

		for (i, c) in name.char_indices() {
			if c == '.' {
				basename = &name[..i];
				extension = &name[i..];
				break;
			}
		}

		return NameParts {
			basename,
			extension,
		};
	}
}
