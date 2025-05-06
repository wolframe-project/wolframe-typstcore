use serde::{Deserialize, Serialize};
use typst::syntax::package::PackageSpec;
use wasm_bindgen::prelude::wasm_bindgen;


#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct TypstCorePackage {
    pub name: String,
    pub namespace: String,
    pub version: String,
}

impl PartialEq<PackageSpec> for TypstCorePackage {
    fn eq(&self, other: &PackageSpec) -> bool {
        self.name == other.name
            && self.namespace == other.namespace
            && self.version == format!("{:?}", other.version)
    }
}

impl From<PackageSpec> for TypstCorePackage {
    fn from(val: PackageSpec) -> Self {
        TypstCorePackage {
            name: val.name.to_string(),
            namespace: val.namespace.to_string(),
            version: format!("{:?}", val.version),
        }
    }
}

impl PartialEq<TypstCorePackage> for PackageSpec {
    fn eq(&self, other: &TypstCorePackage) -> bool {
        self.name == other.name
            && self.namespace == other.namespace
            && format!("{:?}", self.version) == other.version
    }
}