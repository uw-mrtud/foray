use crate::node::{Dict, PortData};

//TODO: is this useful/necessary?
pub trait NodeSpec {
    // type PortType;
    // type PortData;
    // type UIParameter;

    fn compute(
        input_data: Dict<String, PortData>,
        parameters: Dict<String, PortData>,
    ) -> Dict<String, PortData>;
}
