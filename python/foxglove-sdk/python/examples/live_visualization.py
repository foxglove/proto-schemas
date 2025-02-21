import json
import logging
import math
import struct
import foxglove
import numpy as np
import time

from examples.geometry import euler_to_quaternion

from foxglove import Capability, SchemaDefinition
from foxglove.channels import (
    FrameTransformsChannel,
    PointCloudChannel,
    SceneUpdateChannel,
)
from foxglove.schemas import (
    Color,
    CubePrimitive,
    Duration,
    FrameTransform,
    FrameTransforms,
    PackedElementField,
    PackedElementFieldNumericType,
    PointCloud,
    Pose,
    Quaternion,
    RawImage,
    SceneEntity,
    SceneUpdate,
    Vector3,
)

any_schema = {
    "type": "object",
    "additionalProperties": True,
}

plot_schema = {
    "type": "object",
    "properties": {
        "timestamp": {"type": "number"},
        "y": {"type": "number"},
    },
}


class ExampleListener(foxglove.ServerListener):
    """
    This listener demonstrates receiving messages from the client.
    You can send messages from Foxglove app in the publish panel:
    https://docs.foxglove.dev/docs/visualization/panels/publish
    """

    def on_message_data(
        self,
        client: foxglove.Client,
        channel: foxglove.ClientChannelView,
        data: bytes,
    ) -> None:
        logging.info(f"Message from client {client.id} on channel {channel.topic}")
        logging.info(f"Data: {data!r}")


def main() -> None:
    foxglove.verbose_on()

    server = foxglove.start_server(
        server_listener=ExampleListener(),
        capabilities=[Capability.ClientPublish],
        supported_encodings=["json"],
    )

    # Log messages having well-known Foxglove schemas using the appropriate channel type.
    box_chan = SceneUpdateChannel("/boxes")
    tf_chan = FrameTransformsChannel("/tf")
    point_chan = PointCloudChannel("/pointcloud")

    # Log dicts using JSON encoding
    json_chan = foxglove.Channel(topic="/json", schema=plot_schema)

    # Log messages with a custom schema and any encoding
    sin_chan = foxglove.Channel(
        topic="/sine",
        schema=SchemaDefinition(
            name="sine",
            schema_encoding="jsonschema",
            message_encoding="json",
            schema_data=json.dumps(plot_schema).encode("utf-8"),
        ),
    )

    try:
        counter = 0
        while True:
            counter += 1
            now = time.time()
            y = np.sin(now)

            json_msg = {
                "timestamp": now,
                "y": y,
            }
            sin_chan.log(json.dumps(json_msg).encode("utf-8"))

            json_chan.log(json_msg)

            tf_chan.log(
                FrameTransforms(
                    transforms=[
                        FrameTransform(
                            parent_frame_id="world",
                            child_frame_id="box",
                            rotation=euler_to_quaternion(
                                roll=1, pitch=0, yaw=counter * 0.1
                            ),
                        ),
                        FrameTransform(
                            parent_frame_id="world",
                            child_frame_id="points",
                            translation=Vector3(x=-10, y=-10, z=0),
                        ),
                    ]
                )
            )

            box_chan.log(
                SceneUpdate(
                    entities=[
                        SceneEntity(
                            frame_id="box",
                            id="box_1",
                            lifetime=Duration(seconds=10),
                            cubes=[
                                CubePrimitive(
                                    pose=Pose(
                                        position=Vector3(x=0, y=y, z=3),
                                        orientation=euler_to_quaternion(
                                            roll=0, pitch=0, yaw=counter * -0.1
                                        ),
                                    ),
                                    size=Vector3(x=1, y=1, z=1),
                                    color=Color(r=1.0, g=0, b=0, a=1),
                                )
                            ],
                        ),
                    ]
                )
            )

            point_chan.log(make_point_cloud())

            # Or use high-level log API without needing to manage explicit Channels.
            foxglove.log(
                "/high-level",
                RawImage(
                    data=np.zeros((100, 100, 3), dtype=np.uint8).tobytes(),
                    step=300,
                    width=100,
                    height=100,
                    encoding="rgb8",
                ),
            )

            time.sleep(0.05)

    except KeyboardInterrupt:
        server.stop()


def make_point_cloud() -> PointCloud:
    """
    https://foxglove.dev/blog/visualizing-point-clouds-with-custom-colors
    """
    point_struct = struct.Struct("<fffBBBB")
    f32 = PackedElementFieldNumericType.Float32
    u32 = PackedElementFieldNumericType.Uint32

    t = time.time()
    points = [(x + math.cos(t + y / 5), y, 0) for x in range(20) for y in range(20)]
    buffer = bytearray(point_struct.size * len(points))
    for i, point in enumerate(points):
        x, y, z = point
        r = int(255 * (0.5 + 0.5 * x / 20))
        g = int(255 * y / 20)
        b = int(255 * (0.5 + 0.5 * math.sin(t)))
        a = int(255 * (0.5 + 0.5 * ((x / 20) * (y / 20))))
        point_struct.pack_into(buffer, i * point_struct.size, x, y, z, b, g, r, a)

    return PointCloud(
        frame_id="points",
        pose=Pose(
            position=Vector3(x=0, y=0, z=0),
            orientation=Quaternion(x=0, y=0, z=0, w=1),
        ),
        point_stride=16,  # 4 fields * 4 bytes
        fields=[
            PackedElementField(name="x", offset=0, type=f32),
            PackedElementField(name="y", offset=4, type=f32),
            PackedElementField(name="z", offset=8, type=f32),
            PackedElementField(name="rgba", offset=12, type=u32),
        ],
        data=bytes(buffer),
    )


if __name__ == "__main__":
    main()
