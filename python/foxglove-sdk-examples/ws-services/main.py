"""
This example demonstrates how to use the Foxglove WebSocket API to implement services which can be
called from the Service Call panel in the Foxglove app.

https://docs.foxglove.dev/docs/visualization/panels/service-call
"""

import argparse
import json
import logging

from foxglove import (
    Service,
    ServiceSchema,
    start_server,
    Capability,
    Client,
    ServiceHandler,
)


def logging_handler(
    service_name: str, client: Client, call_id: int, encoding: str, payload: bytes
):
    """
    A handler for the service, adhering to the `ServiceHandler` type,
    called with the following positional arguments:

    1. service_name: The name of the service
    2. topic: The topic name
    3. call_id: The ID of this call
    4. client: The client that called the service
    5. payload: The payload from the client

    The handler should return a bytes object which will be sent back to the client.

    The handler must return quickly.
    """
    log_request(service_name, client, call_id, encoding, payload)
    return b"{}"


# A handler can also be defined as a lambda
greeting_handler: ServiceHandler = lambda *_: json.dumps(
    {"message": "Hello, client!"}
).encode("utf-8")


# ...or any callable
class EchoService:
    def __call__(
        self,
        service_name: str,
        client: Client,
        call_id: int,
        encoding: str,
        payload: bytes,
    ):
        log_request(service_name, client, call_id, encoding, payload)
        return payload


def log_request(
    service_name: str, client: Client, call_id: int, encoding: str, payload: bytes
):
    logging.debug(
        f"[{service_name}] Request {call_id} from {client} on {encoding}: {payload!r}"
    )


def main():
    """
    This example demonstrates how to use the Foxglove WebSocket API to implement services which can
    be called from the Foxglove app.
    """
    parser = argparse.ArgumentParser()
    parser.add_argument("--port", type=int, default=8765)
    parser.add_argument("--host", type=str, default="127.0.0.1")
    args = parser.parse_args()

    logging_service = Service(
        name="logging",
        schema=ServiceSchema(
            name="logging-schema",
        ),
        handler=logging_handler,
    )

    echo_service = Service(
        name="echo",
        schema=ServiceSchema(
            name="echo-schema",
        ),
        handler=EchoService(),
    )

    hello_service = Service(
        name="hello",
        schema=ServiceSchema(
            name="hello-schema",
        ),
        handler=greeting_handler,
    )

    server = start_server(
        name="ws-services-example",
        port=args.port,
        host=args.host,
        capabilities=[Capability.Services],
        # If publishing from Foxglove, add at least one supported encoding (json, ros1, or cdr).
        # These examples use json.
        supported_encodings=["json"],
        # The services to publish
        services=[echo_service, hello_service, logging_service],
    )

    try:
        while True:
            pass
    except KeyboardInterrupt:
        server.stop()


if __name__ == "__main__":
    main()
