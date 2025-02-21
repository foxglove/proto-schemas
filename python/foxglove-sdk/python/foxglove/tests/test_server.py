import unittest

from foxglove import start_server, Capability, Service, ServiceSchema


class TestServer(unittest.TestCase):
    def test_services_interface(self) -> None:
        server = start_server(
            capabilities=[Capability.Services],
            supported_encodings=["json"],
            services=[
                Service(
                    name="test",
                    schema=ServiceSchema(name="test-schema"),
                    handler=lambda _svc, _client, _cid, _enc, _bytes: b"{}",
                ),
            ],
        )
        server.stop()
