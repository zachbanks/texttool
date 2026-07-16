//! The registry that owns every available [`Transform`].
//!
//! The registry is the single place that knows the full set of operations. The
//! CLI is built from it and dispatch looks up transforms in it, so wiring a new
//! operation into the whole application is a one-line addition to
//! [`default_registry`].

use crate::transform::Transform;
use crate::transforms;

/// An ordered collection of registered [`Transform`]s.
///
/// Order is preserved so the `--help` listing reads in the order operations
/// were registered rather than in some arbitrary hash order.
#[derive(Default)]
pub struct Registry {
    transforms: Vec<Box<dyn Transform>>,
}

impl Registry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a transform, returning `&mut self` for fluent chaining.
    ///
    /// # Panics
    ///
    /// Panics if a transform with the same name or alias is already registered.
    /// A name collision is a programming error, not a runtime condition, so it
    /// fails loudly rather than silently shadowing an operation.
    pub fn register<T: Transform + 'static>(&mut self, transform: T) -> &mut Self {
        let name = transform.name();
        if self.get(name).is_some() {
            panic!("duplicate transform registered under name/alias `{name}`");
        }
        for alias in transform.aliases() {
            if self.get(alias).is_some() {
                panic!("duplicate transform registered under name/alias `{alias}`");
            }
        }
        self.transforms.push(Box::new(transform));
        self
    }

    /// All registered transforms, in registration order.
    pub fn all(&self) -> &[Box<dyn Transform>] {
        &self.transforms
    }

    /// Look up a transform by its name or one of its aliases.
    pub fn get(&self, name: &str) -> Option<&dyn Transform> {
        self.transforms
            .iter()
            .find(|t| t.name() == name || t.aliases().contains(&name))
            .map(|boxed| boxed.as_ref())
    }
}

/// Build the registry containing every operation shipped with `texttool`.
///
/// This is the canonical list of built-in operations. To add a new one,
/// implement [`Transform`](crate::transform::Transform) and add a single
/// `.register(...)` line here.
pub fn default_registry() -> Registry {
    let mut registry = Registry::new();
    registry
        .register(transforms::Clean)
        .register(transforms::Squeeze)
        .register(transforms::TitleCase)
        .register(transforms::Slug)
        .register(transforms::Camel)
        .register(transforms::Pascal)
        .register(transforms::Snake)
        .register(transforms::Kebab)
        .register(transforms::Constant)
        .register(transforms::Mock)
        .register(transforms::Upper)
        .register(transforms::Lower);
    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_registry_has_expected_transforms() {
        let registry = default_registry();
        assert!(registry.get("upper").is_some());
        assert!(registry.get("lower").is_some());
        assert!(registry.get("does-not-exist").is_none());
    }

    #[test]
    fn lookup_resolves_aliases() {
        let registry = default_registry();
        // `upper` advertises `uc` as an alias.
        assert!(registry.get("uc").is_some());
    }

    #[test]
    #[should_panic(expected = "duplicate transform")]
    fn duplicate_registration_panics() {
        let mut registry = Registry::new();
        registry
            .register(transforms::Upper)
            .register(transforms::Upper);
    }
}
