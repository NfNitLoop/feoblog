use actix_web::web::{self, Path, resource, HttpResponse as Response};

// Work around https://github.com/djc/askama/issues/246
// TODO: The above has been fixed, we can probably delete this code.
pub(crate) trait ToResponder
{
    type ResponderType;
    fn responder(self) -> Self::ResponderType;
}

impl<T> ToResponder for T
where T: askama::Template + 
{
    type ResponderType = Response;

    fn responder(self) -> Self::ResponderType {
        let body = self.render().expect(
            "Error rendering?! This shouldn't happen?"
        );
        Response::Ok().body(body)
    }
}