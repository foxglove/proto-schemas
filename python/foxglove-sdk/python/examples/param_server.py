import logging
from typing import Optional
import foxglove
import time


from foxglove import Capability
from foxglove import ParameterType, ParameterValue, Parameter


class ParameterStore(foxglove.ServerListener):
    def __init__(self, parameters: list[Parameter]) -> None:
        self.parameters = parameters

    # Foxglove server callback
    def on_get_parameters(
        self,
        client: foxglove.Client,
        param_names: list[str],
        request_id: Optional[str] = None,
    ) -> list[Parameter]:
        logging.debug(f"on_get_parameters: {param_names}, {client.id}, {request_id}")
        return self.parameters

    def on_set_parameters(
        self,
        client: foxglove.Client,
        parameters: list[foxglove.Parameter],
        request_id: str | None = None,
    ) -> list[foxglove.Parameter]:
        logging.debug(f"on_set_parameters: {parameters}, {client.id}, {request_id}")
        self.parameters = parameters
        return parameters


def main() -> None:
    foxglove.verbose_on()

    initial_values: list[Parameter] = [
        Parameter(
            "param1",
            value=ParameterValue.Dict(
                {
                    "a": ParameterValue.Number(1),
                    "b": ParameterValue.Bool(True),
                    "c": ParameterValue.Bytes(b"hello"),
                    "arr": ParameterValue.Array(
                        [ParameterValue.Number(1), ParameterValue.Bool(True)]
                    ),
                }
            ),
        ),
        Parameter("param2"),
        Parameter("p3", value=ParameterValue.Number(0.124), type=ParameterType.Float64),
    ]

    store = ParameterStore(initial_values)

    websocket_server = foxglove.start_server(
        server_listener=store,
        capabilities=[Capability.Parameters],
    )

    try:
        while True:
            websocket_server.publish_parameter_values(store.parameters)
            time.sleep(1)
    except KeyboardInterrupt:
        # pass
        exit(0)
    except Exception as e:
        print(e)

    # breakpoint()


if __name__ == "__main__":
    main()
