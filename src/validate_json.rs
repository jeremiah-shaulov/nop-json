pub use nop_json_derive::*;

/// During deserialization process, you may want to complain on invalid fields combination in struct.
/// Every type that implements [TryFromJson](trait.TryFromJson.html) must also implement `ValidateJson`.
/// This trait introduces function `validate_json()`. `try_from_json()` calls this function right after
/// it converted JSON string to target type, and deserialization will stop if `validate_json()` returns `Err`.
///
/// `ValidateJson` can be implemented automatically through `#[derive(TryFromJson, ValidateJson)]`.
/// Default implementation always accepts the validation. To validate the type you need to implement this trait manually.
///
/// # Examples
///
/// ```
/// use nop_json::{Reader, TryFromJson};
/// use std::io;
///
/// #[derive(TryFromJson, Debug)]
/// struct FromTo {from: i32, to: i32}
///
/// impl nop_json::ValidateJson for FromTo
/// {	fn validate_json(self) -> Result<Self, String>
/// 	{	if self.from <= self.to
/// 		{	Ok(self)
/// 		}
/// 		else
/// 		{	Err("to must be after from".to_string())
/// 		}
/// 	}
/// }
///
/// let mut reader = Reader::new(r#" {"from": 0, "to": 10}   {"from": 3, "to": -1} "#.bytes());
/// let obj_0: io::Result<FromTo> = reader.read();
/// let obj_1: io::Result<FromTo> = reader.read();
/// assert!(obj_0.is_ok());
/// assert!(obj_1.is_err());
/// ```
pub trait ValidateJson: Sized
{	fn validate_json(self) -> Result<Self, String>
	{	Ok(self)
	}
}
