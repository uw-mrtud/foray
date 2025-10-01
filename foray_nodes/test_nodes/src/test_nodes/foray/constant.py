from foray import ForayConfig, Port, NumberField


def config():
    return (
        ForayConfig()
        .outputs(
            {
                "out": Port.float,
            }
        )
        .parameters({"constant": NumberField(0.0)})
    )


def compute(_, parameters):
    return {"out": parameters["constant"]}
