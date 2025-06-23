pub mod py_node;
pub mod py_widget;

//#[cfg(test)]
//mod test {
//    use pyo3::prepare_freethreaded_python;
//
//    use crate::nodes::port::PortType;
//
//    use super::*;
//    #[test]
//    fn simple_config() {
//        prepare_freethreaded_python();
//
//        let port_def = gpipy_read_config(
//            "test",
//            r#"
//def config():
//    class out:
//        inputs = {"p1": "Real", "p2": "ArrayReal"}
//        outputs =  "Complex"
//        parameters =  {}
//    return out
//            "#,
//            "".into(),
//        );
//
//        assert_eq!(
//            PyNode {
//                name: "test".into(),
//                path: "".into(),
//                ports: Ok(PortDef {
//                    inputs: PortType::Object(
//                        [
//                            ("p1".to_string(), PortType::Real),      //PortType::Real),
//                            ("p2".to_string(), PortType::ArrayReal)  //PortType::Real2d)
//                        ]
//                        .into()
//                    )
//                    .into(),
//                    outputs: PortType::Complex
//                }),
//                parameters: Ok([].into())
//            },
//            port_def
//        );
//    }
//}
