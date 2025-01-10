/// An HTTP status code
pub enum Status
{
	Okay                = 200,
	BadRequest          = 400,
	NotFound            = 404,
	InternalServerError = 500,
}


impl Status
{
	/// Turn the status code into a string which is a number and phrase
	pub fn to_str(&self) -> &'static str
	{
		return match self {
			Status::Okay                => "200 Ok",
			Status::BadRequest          => "400 Bad Request",
			Status::NotFound            => "404 Not Found",
			Status::InternalServerError => "500 Not Found",
		};
	}


	/// Turn the status code into an entire HTTP response
	pub fn to_response(&self) -> &'static str
	{
		return match self {
			Status::Okay                => "HTTP/1.1 200 Ok\r\n\r\n",
			Status::BadRequest          => "HTTP/1.1 400 Bad Request\r\n\r\n",
			Status::NotFound            => "HTTP/1.1 404 Not Found\r\n\r\n",
			Status::InternalServerError => "HTTP/1.1 500 Not Found\r\n\r\n",
		};
	}
}
