use handle_errors::Error;
use std::collections::HashMap;

/// Pagination struct that is getting extracted from query params
#[derive(Debug, Default)]
pub struct Pagination {
    /// The index of the last item to be returned
    pub limit: Option<i32>,
    /// The index of the first item to be returned
    pub offset: i32,
}

/// Extract query parameters from the `/questions` route
///
/// # Example query
/// GET requests to this route can have a pagination attached so we just
/// return the questions we need
/// `/questions?start=1&end=10`
///
/// # Example usage
/// ```rust
/// use std::collections::HashMap;
/// use q_and_a::types::pagination;
///
/// let mut query = HashMap::new();
/// query.insert("limit".to_string(), "1".to_string());
/// query.insert("offset".to_string(), "10".to_string());
/// let p = pagination::extract_pagination(query).unwrap();
///
/// assert_eq!(p.limit,Some(1));
/// assert_eq!(p.offset, 10);
/// ```
pub fn extract_pagination(params: HashMap<String, String>) -> Result<Pagination, Error> {
    // TODO: handle start greater than end
    if params.contains_key("limit") && params.contains_key("offest") {
        return Ok(Pagination {
            // Takes the "limit" parameter and tries to convert it to a number
            limit: Some(
                params
                    .get("limit")
                    .unwrap()
                    .parse::<i32>()
                    .map_err(Error::ParseError)?,
            ),
            // Takes the "offest" parameter and tries to convert it to a number
            offset: params
                .get("offest")
                .unwrap()
                .parse::<i32>()
                .map_err(Error::ParseError)?,
        });
    }

    Err(Error::MissingParameters)
}
