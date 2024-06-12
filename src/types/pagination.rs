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
/// use eroteme::types::pagination;
///
/// let mut query = HashMap::new();
/// query.insert("limit".to_string(), "1".to_string());
/// query.insert("offset".to_string(), "10".to_string());
/// let p = pagination::extract_pagination(query).unwrap();
///
/// assert_eq!(p.limit,Some(1));
/// assert_eq!(p.offset, 10);
/// ```
///
/// # Errors
///
/// Will return `Err` if `limit` or `offset` parameters are missing.
#[allow(clippy::missing_panics_doc, clippy::module_name_repetitions)]
pub fn extract_pagination<S: ::std::hash::BuildHasher>(
    params: &HashMap<String, String, S>,
) -> Result<Pagination, Error> {
    // TODO: handle start greater than end
    if params.contains_key("limit") && params.contains_key("offset") {
        return Ok(Pagination {
            // Takes the `limit` parameter and tries to convert it to a number
            limit: Some(
                params
                    .get("limit")
                    .expect("limit param not set in map")
                    .parse::<i32>()
                    .map_err(Error::ParseError)?,
            ),
            // Takes the `offset` parameter and tries to convert it to a number
            offset: params
                .get("offset")
                .expect("offset param not set in map")
                .parse::<i32>()
                .map_err(Error::ParseError)?,
        });
    }

    Err(Error::MissingParameters)
}
