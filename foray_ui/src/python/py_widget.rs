use pyo3::{pyclass, pymethods};

#[pyclass]
pub struct Slider {
    default: f32,
    start: f32,
    stop: f32,
    steps: u32,
}

#[pymethods]
impl Slider {
    ///Create a new slider
    #[new]
    #[pyo3(signature = (default=0.0,start=0.0,stop=10.0,steps=50))]
    fn new(default: f32, start: f32, stop: f32, steps: u32) -> Self {
        Slider {
            default,
            start,
            stop,
            steps,
        }
    }
}
