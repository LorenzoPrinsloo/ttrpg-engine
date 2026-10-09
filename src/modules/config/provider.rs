use figment::value::{Dict, Map, Value};
use figment::{Error, Metadata, Profile, Provider};

// Puts everything `inner` provides under `section.name`, so many files can be
// merged into one document without their keys colliding.
pub(crate) struct Nested<P> {
    section: &'static str,
    name: String,
    inner: P,
}

impl<P: Provider> Nested<P> {
    pub(crate) fn new(section: &'static str, name: impl Into<String>, inner: P) -> Self {
        Self {
            section,
            name: name.into(),
            inner,
        }
    }

    // `{ items: [...] }` becomes `{ section: { name: { items: [...] } } }`.
    fn wrap(&self, contents: Dict) -> Dict {
        let file = Dict::from([(self.name.clone(), Value::from(contents))]);
        Dict::from([(self.section.to_string(), Value::from(file))])
    }
}

impl<P: Provider> Provider for Nested<P> {
    fn metadata(&self) -> Metadata {
        self.inner.metadata()
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        let data = self
            .inner
            .data()?
            .into_iter()
            .map(|(profile, contents)| (profile, self.wrap(contents)))
            .collect();

        Ok(data)
    }
}
