use std::cmp;
use std::hash;
use std::fmt;

use pyo3::basic::CompareOp;
use pyo3::prelude::*;


// We can't put a Py<PyAny> directly into a HashMap key
// So to be able to hold references to arbitrary Python objects in HashMap as keys
// we wrap them in a struct that gets the hash() when it receives the object from Python
// and then just echoes back that hash when called Rust needs to hash it
#[derive(Clone)]
pub struct HashedAny {
    pub o: Py<PyAny>,
    pub hash: isize,
    is_equal_to_self: bool,
}

fn check_equal_py(this: &Py<PyAny>, other: &Py<PyAny>) -> bool {
    Python::with_gil(|py| -> PyResult<bool> {
        Ok(this.as_ref(py).rich_compare(other.as_ref(py), CompareOp::Eq)?.is_true()?)
    }).unwrap()
}

// Use the result of calling repr()  and str() on the Python object for string formatting
impl fmt::Debug for HashedAny {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        println!("called");
        Python::with_gil(|py| -> PyResult<fmt::Result> {
            let a = self.o.as_ref(py).repr()?.to_str()?;
            println!("{}", a);
            Ok(write!(f, "{}", a))
        }).unwrap()
    }
}

impl fmt::Display for HashedAny {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        println!("called");
        Python::with_gil(|py| -> PyResult<fmt::Result> {
            let a = self.o.as_ref(py).str()?.to_str()?;
            println!("{}", a);
            Ok(write!(f, "{}", a))
        }).unwrap()
    }
}


impl <'source>FromPyObject<'source> for HashedAny
{
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        let obpy = ob.into();
        let is_equal_to_self: bool;
        {
            let obpyref = &obpy;
            is_equal_to_self = check_equal_py(obpyref,obpyref);
        }
        Ok(
            HashedAny{
                o: obpy,
                hash: ob.hash()?,
                is_equal_to_self: is_equal_to_self,
            }
        )
    }
}

impl hash::Hash for HashedAny {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.hash.hash(state)
    }
}

impl cmp::PartialEq for HashedAny {
    fn eq(&self, other: &Self) -> bool {
        // First check if we can do a pointer comparison in Rust
        if self.is_equal_to_self && self.o.eq(&other.o) {
            return true;
        }
        // Otherwise, call __eq__, which means acquiring the GIL
        return check_equal_py(&self.o, &other.o)
    }
}

impl cmp::Eq for HashedAny {}
