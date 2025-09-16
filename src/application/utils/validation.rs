use validator::{ValidationError, ValidationErrors};
use url::Url;

pub fn validate_http_url(opt: &Option<String>) -> Result<(), ValidationError> {
    if let Some(u) = opt {
        let parsed = Url::parse(u).map_err(|_| ValidationError::new("invalid_url"))?;
        let scheme = parsed.scheme();
        if scheme != "http" && scheme != "https" {
            return Err(ValidationError::new("unsupported_scheme"));
        }
    }
    Ok(())
}

pub fn aggregate(errors: Vec<(&'static str, ValidationError)>) -> Result<(), ValidationErrors> {
    if errors.is_empty() { return Ok(()); }
    let mut ve = ValidationErrors::new();
    for (field, err) in errors {
        ve.add(field, err);
    }
    Err(ve)
}
