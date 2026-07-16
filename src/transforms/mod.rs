//! Built-in [`Transform`](crate::transform::Transform) implementations.
//!
//! Each submodule holds one operation (or a small family of closely related
//! ones). New operations should be added as a new submodule and re-exported
//! here, then registered in [`crate::registry::default_registry`].

mod case;
mod clean;
mod identifier;
mod mock;
mod slug;
mod titlecase;

pub use case::{Lower, Upper};
pub use clean::Clean;
pub use identifier::{Camel, Constant, Kebab, Pascal, Snake};
pub use mock::Mock;
pub use slug::Slug;
pub use titlecase::TitleCase;
