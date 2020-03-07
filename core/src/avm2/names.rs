//! AVM2 names & namespacing

use crate::avm2::value::{abc_string, abc_string_option};
use crate::avm2::{Avm2, Error};
use gc_arena::Collect;
use swf::avm2::types::{
    AbcFile, Index, Multiname as AbcMultiname, Namespace as AbcNamespace,
    NamespaceSet as AbcNamespaceSet,
};

/// Represents the name of a namespace.
#[derive(Clone, Collect, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[collect(no_drop)]
pub enum Namespace {
    Namespace(String),
    Package(String),
    PackageInternal(String),
    Protected(String),
    Explicit(String),
    StaticProtected(String),
    Private(String),
    Any,
}

impl Namespace {
    /// Read a namespace declaration from the ABC constant pool and copy it to
    /// a namespace value.
    pub fn from_abc_namespace(
        file: &AbcFile,
        namespace_index: Index<AbcNamespace>,
    ) -> Result<Self, Error> {
        if namespace_index.0 == 0 {
            return Ok(Self::Any);
        }

        let actual_index = namespace_index.0 as usize - 1;
        let abc_namespace: Result<&AbcNamespace, Error> = file
            .constant_pool
            .namespaces
            .get(actual_index)
            .ok_or_else(|| format!("Unknown namespace constant {}", namespace_index.0).into());

        Ok(match abc_namespace? {
            AbcNamespace::Namespace(idx) => Self::Namespace(abc_string(file, idx.clone())?),
            AbcNamespace::Package(idx) => Self::Package(abc_string(file, idx.clone())?),
            AbcNamespace::PackageInternal(idx) => {
                Self::PackageInternal(abc_string(file, idx.clone())?)
            }
            AbcNamespace::Protected(idx) => Self::Protected(abc_string(file, idx.clone())?),
            AbcNamespace::Explicit(idx) => Self::Explicit(abc_string(file, idx.clone())?),
            AbcNamespace::StaticProtected(idx) => {
                Self::StaticProtected(abc_string(file, idx.clone())?)
            }
            AbcNamespace::Private(idx) => Self::Private(abc_string(file, idx.clone())?),
        })
    }

    pub fn public_namespace() -> Self {
        Namespace::Package("".to_string())
    }

    pub fn package(package_name: &str) -> Self {
        Namespace::Package(package_name.to_string())
    }

    pub fn is_any(&self) -> bool {
        match self {
            Self::Any => true,
            _ => false,
        }
    }

    pub fn is_private(&self) -> bool {
        match self {
            Self::Private(_) => true,
            _ => false,
        }
    }
}

/// A `QName`, likely "qualified name", consists of a namespace and name string.
///
/// This is technically interchangeable with `xml::XMLName`, as they both
/// implement `QName`; however, AVM2 and XML have separate representations.
///
/// A property cannot be retrieved or set without first being resolved into a
/// `QName`. All other forms of names and multinames are either versions of
/// `QName` with unspecified parameters, or multiple names to be checked in
/// order.
#[derive(Clone, Collect, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[collect(no_drop)]
pub struct QName {
    ns: Namespace,
    name: String,
}

impl QName {
    pub fn new(ns: Namespace, name: &str) -> Self {
        Self {
            ns,
            name: name.to_string(),
        }
    }

    pub fn dynamic_name(local_part: &str) -> Self {
        Self {
            ns: Namespace::public_namespace(),
            name: local_part.to_string(),
        }
    }

    /// Pull a `QName` from the multiname pool.
    ///
    /// This function returns an Err if the multiname does not exist or is not
    /// a `QName`.
    pub fn from_abc_multiname(
        file: &AbcFile,
        multiname_index: Index<AbcMultiname>,
    ) -> Result<Self, Error> {
        let actual_index = multiname_index.0 as usize - 1;
        let abc_multiname: Result<&AbcMultiname, Error> = file
            .constant_pool
            .multinames
            .get(actual_index)
            .ok_or_else(|| format!("Unknown multiname constant {}", multiname_index.0).into());

        Ok(match abc_multiname? {
            AbcMultiname::QName { namespace, name } => Self {
                ns: Namespace::from_abc_namespace(file, namespace.clone())?,
                name: abc_string(file, name.clone())?,
            },
            _ => return Err("Attempted to pull QName from non-QName multiname".into()),
        })
    }

    pub fn local_name(&self) -> &str {
        &self.name
    }

    pub fn namespace(&self) -> &Namespace {
        &self.ns
    }
}

/// A `Multiname` consists of a name which could be resolved in one or more
/// potential namespaces.
///
/// All unresolved names are of the form `Multiname`, and the name resolution
/// process consists of searching each name space for a given name.
///
/// The existence of a `name` of `None` indicates the `Any` name.
#[derive(Debug)]
pub struct Multiname {
    ns: Vec<Namespace>,
    name: Option<String>,
}

impl Multiname {
    /// Read a namespace set from the ABC constant pool, and return a list of
    /// copied namespaces.
    fn abc_namespace_set(
        file: &AbcFile,
        namespace_set_index: Index<AbcNamespaceSet>,
    ) -> Result<Vec<Namespace>, Error> {
        if namespace_set_index.0 == 0 {
            //TODO: What is namespace set zero?
            return Ok(vec![]);
        }

        let actual_index = namespace_set_index.0 as usize - 1;
        let ns_set: Result<&AbcNamespaceSet, Error> = file
            .constant_pool
            .namespace_sets
            .get(actual_index)
            .ok_or_else(|| {
                format!("Unknown namespace set constant {}", namespace_set_index.0).into()
            });
        let mut result = vec![];

        for ns in ns_set? {
            result.push(Namespace::from_abc_namespace(file, ns.clone())?)
        }

        Ok(result)
    }

    /// Read a multiname from the ABC constant pool, copying it into the most
    /// general form of multiname.
    pub fn from_abc_multiname(
        file: &AbcFile,
        multiname_index: Index<AbcMultiname>,
        avm: &mut Avm2<'_>,
    ) -> Result<Self, Error> {
        let actual_index = multiname_index.0 as usize - 1;
        let abc_multiname: Result<&AbcMultiname, Error> = file
            .constant_pool
            .multinames
            .get(actual_index)
            .ok_or_else(|| format!("Unknown multiname constant {}", multiname_index.0).into());

        Ok(match abc_multiname? {
            AbcMultiname::QName { namespace, name } | AbcMultiname::QNameA { namespace, name } => {
                Self {
                    ns: vec![Namespace::from_abc_namespace(file, namespace.clone())?],
                    name: abc_string_option(file, name.clone())?,
                }
            }
            AbcMultiname::RTQName { name } | AbcMultiname::RTQNameA { name } => {
                let ns = avm.pop().as_namespace()?.clone();
                Self {
                    ns: vec![ns],
                    name: abc_string_option(file, name.clone())?,
                }
            }
            AbcMultiname::RTQNameL | AbcMultiname::RTQNameLA => {
                let ns = avm.pop().as_namespace()?.clone();
                let name = avm.pop().as_string()?.clone();
                Self {
                    ns: vec![ns],
                    name: Some(name),
                }
            }
            AbcMultiname::Multiname {
                namespace_set,
                name,
            }
            | AbcMultiname::MultinameA {
                namespace_set,
                name,
            } => Self {
                ns: Self::abc_namespace_set(file, namespace_set.clone())?,
                name: abc_string_option(file, name.clone())?,
            },
            AbcMultiname::MultinameL { namespace_set }
            | AbcMultiname::MultinameLA { namespace_set } => {
                let name = avm.pop().as_string()?.clone();
                Self {
                    ns: Self::abc_namespace_set(file, namespace_set.clone())?,
                    name: Some(name),
                }
            }
        })
    }

    /// Read a static multiname from the ABC constant pool
    ///
    /// This function prohibits the use of runtime-qualified and late-bound
    /// names. Runtime multinames will instead result in an error.
    pub fn from_abc_multiname_static(
        file: &AbcFile,
        multiname_index: Index<AbcMultiname>,
    ) -> Result<Self, Error> {
        let actual_index = multiname_index.0 as usize - 1;
        let abc_multiname: Result<&AbcMultiname, Error> = file
            .constant_pool
            .multinames
            .get(actual_index)
            .ok_or_else(|| format!("Unknown multiname constant {}", multiname_index.0).into());

        Ok(match abc_multiname? {
            AbcMultiname::QName { namespace, name } | AbcMultiname::QNameA { namespace, name } => {
                Self {
                    ns: vec![Namespace::from_abc_namespace(file, namespace.clone())?],
                    name: abc_string_option(file, name.clone())?,
                }
            }
            AbcMultiname::Multiname {
                namespace_set,
                name,
            }
            | AbcMultiname::MultinameA {
                namespace_set,
                name,
            } => Self {
                ns: Self::abc_namespace_set(file, namespace_set.clone())?,
                name: abc_string_option(file, name.clone())?,
            },
            _ => return Err(format!("Multiname {} is not static", multiname_index.0).into()),
        })
    }

    pub fn namespace_set(&self) -> impl Iterator<Item = &Namespace> {
        self.ns.iter()
    }

    pub fn local_name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}
