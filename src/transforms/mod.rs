//! Built-in [`Transform`](crate::transform::Transform) implementations.
//!
//! Each submodule holds one operation (or a small family of closely related
//! ones). New operations should be added as a new submodule and re-exported
//! here, then registered in [`crate::registry::default_registry`].

mod case;
mod clean;
mod extract;
mod identifier;
mod mock;
mod replace;
mod slug;
mod squeeze;
mod strip;
mod titlecase;
mod unslug;

pub use case::{Lower, Upper};
pub use clean::Clean;
pub use extract::Extract;
pub use identifier::{Camel, Constant, Kebab, Pascal, Snake};
pub use mock::Mock;
pub use replace::Replace;
pub use slug::Slug;
pub use squeeze::Squeeze;
pub use strip::Strip;
pub use titlecase::TitleCase;
pub use unslug::Unslug;
