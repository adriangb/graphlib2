use pyo3::prelude::*;


#[pyclass]
struct Iter {
    src: Py<T>,
    cb: impl FnMut(Py<T>) -> Option<PyObject>,
}


#[pyproto]
impl PyIterProtocol for Iter {
    fn __iter__(self: PyRef<Self>) -> Py<Iter> {
        self
    }
    fn __next__(mut slf: PyRefMut<Self>) -> Option<PyObject> {
        self.cb(self.src)
    }
}
