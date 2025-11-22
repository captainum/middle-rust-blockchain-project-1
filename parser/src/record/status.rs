use super::errors::StatusParseError;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Success,
    Failure,
    Pending,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Success => "SUCCESS",
            Self::Failure => "FAILURE",
            Self::Pending => "PENDING",
        };

        write!(f, "{s}")
    }
}

impl TryFrom<&str> for Status {
    type Error = StatusParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "SUCCESS" => Ok(Self::Success),
            "FAILURE" => Ok(Self::Failure),
            "PENDING" => Ok(Self::Pending),
            _ => Err(StatusParseError::InvalidStatus(s.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        assert_eq!(Status::Success.to_string(), "SUCCESS");
        assert_eq!(Status::Failure.to_string(), "FAILURE");
        assert_eq!(Status::Pending.to_string(), "PENDING");
    }

    #[test]
    fn test_try_from() {
        assert_eq!(Status::try_from("SUCCESS").unwrap(), Status::Success);
        assert_eq!(Status::try_from("FAILURE").unwrap(), Status::Failure);
        assert_eq!(Status::try_from("PENDING").unwrap(), Status::Pending);
        assert!(Status::try_from("").is_err_and(|e| e.to_string() == "Invalid status: "));
        assert!(
            Status::try_from("INVALID").is_err_and(|e| e.to_string() == "Invalid status: INVALID")
        );
    }
}
