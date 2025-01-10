const ASSUMED_LANGUAGE: &str = "English";


/// Lengthen a language, for example `"en"` to `"English"`
/// https://raw.githubusercontent.com/r12a/r12a.github.io/refs/heads/master/apps/subtags/languages.js
pub fn language_abbrevation_to_name(abbreviation: &str) -> &str
{
	return match abbreviation {
		""   => ASSUMED_LANGUAGE,
		"ar" => "Arabic",
		"bn" => "Bengali",
		"de" => "German",
		"en" => "English",
		"es" => "Spanish",
		"fr" => "French",
		"hi" => "Hindi",
		"id" => "Indonesian",
		"ja" => "Japanese",
		"pt" => "Portuguese",
		"ru" => "Russian",
		"ur" => "Urdu",
		"zh" => "Chinese",
		_ => "Other",
	};
}
