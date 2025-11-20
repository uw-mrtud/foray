use numpy::ndarray::array;
use numpy::pyo3::Python;
use numpy::{PyArrayMethods, ToPyArray};
use pyo3::ffi::c_str;

fn main() {
    println!("Node check!");
    pyo3::prepare_freethreaded_python();

    Python::with_gil(|py| {
        py.run(
            c_str!(
                r#"
import sys
print(sys.executable, sys.path)
"#
            ),
            None,
            None,
        )
        .unwrap();

        let py_array = array![[1i64, 2], [3, 4]].to_pyarray(py);
        assert_eq!(py_array.readonly().as_array(), array![[1i64, 2], [3, 4]]);
    });
}
