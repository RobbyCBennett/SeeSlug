/// An HTTP status code
pub enum Status
{
	Okay                = 200,
	BadRequest          = 400,
	NotFound            = 404,
	RangeNotSatisfiable = 416,
	InternalServerError = 500,
}
use Status::*;


impl Status
{
	/// Turn the status code into a string which is a number and phrase
	pub fn to_str(&self) -> &'static str
	{
		return match self {
			Okay                => "200 Ok",
			BadRequest          => "400 Bad Request",
			NotFound            => "404 Not Found",
			RangeNotSatisfiable => "416 Range Not Satisfiable",
			InternalServerError => "500 Internal Server Error",
		};
	}


	/// Turn the status code into an entire HTTP response
	pub fn to_response(&self) -> &'static str
	{
		return match self {
			Okay                => "HTTP/1.1 200 Ok\r\n\r\n",
			BadRequest          => "HTTP/1.1 400 Bad Request\r\n\r\n",
			NotFound            => "HTTP/1.1 404 Not Found\r\n\r\n",
			RangeNotSatisfiable => "HTTP/1.1 416 Range Not Satisfiable\r\n\r\n",
			InternalServerError => "HTTP/1.1 500 Internal Server Error\r\n\r\n",
		};
	}
}
