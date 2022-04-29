use std::cmp;
use std::hash;

use pyo3::basic::CompareOp;
use pyo3::prelude::*;

// We can't put a Py<PyAny> directly into a HashMap key
// So to be able to hold references to arbitrary Python objects in HashMap as keys
// we wrap them in a struct that gets the hash() when it receives the object from Python
// and then just echoes back that hash when called Rust needs to hash it
#[derive(Clone)]
pub struct HashedAny(pub Py<PyAny>, isize);

impl<'source> FromPyObject<'source> for HashedAny {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        Ok(HashedAny(ob.into(), ob.hash()?))
    }
}

impl hash::Hash for HashedAny {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        state.write_isize(self.1)
    }
}

impl cmp::PartialEq for HashedAny {
    fn eq(&self, other: &Self) -> bool {
        // This assumes that `self is other` implies `self == other`
        // Which is not necessarily true, e.g. for NaN, but is true in most cases
        // and there's a perf advantage to not calling into Python
        if self.0.is(&other.0) {
            return true;
        }
        Python::with_gil(|py| -> bool {
            self.0
                .as_ref(py)
                .rich_compare(other.0.as_ref(py), CompareOp::Eq)
                .unwrap()
                .is_true()
                .unwrap()
        })
    }
}

impl cmp::Eq for HashedAny {}
