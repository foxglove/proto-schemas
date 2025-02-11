from enum import Enum
from typing import List

class ArrowPrimitive:
    """
    A primitive representing an arrow
    """

    def __new__(
        cls,
        pose: "Pose",
        shaft_length: "float",
        shaft_diameter: "float",
        head_length: "float",
        head_diameter: "float",
        color: "Color",
    ) -> "ArrowPrimitive": ...

class CameraCalibration:
    """
    Camera calibration parameters
    """

    def __new__(
        cls,
        timestamp: "Timestamp",
        frame_id: "str",
        width: "int",
        height: "int",
        distortion_model: "str",
        D: "List[float]",
        K: "List[float]",
        R: "List[float]",
        P: "List[float]",
    ) -> "CameraCalibration": ...

class CircleAnnotation:
    """
    A circle annotation on a 2D image
    """

    def __new__(
        cls,
        timestamp: "Timestamp",
        position: "Point2",
        diameter: "float",
        thickness: "float",
        fill_color: "Color",
        outline_color: "Color",
    ) -> "CircleAnnotation": ...

class Color:
    """
    A color in RGBA format
    """

    def __new__(cls, r: "float", g: "float", b: "float", a: "float") -> "Color": ...

class CompressedImage:
    """
    A compressed image
    """

    def __new__(
        cls, timestamp: "Timestamp", frame_id: "str", data: "bytes", format: "str"
    ) -> "CompressedImage": ...

class CompressedVideo:
    """
    A single frame of a compressed video bitstream
    """

    def __new__(
        cls, timestamp: "Timestamp", frame_id: "str", data: "bytes", format: "str"
    ) -> "CompressedVideo": ...

class CubePrimitive:
    """
    A primitive representing a cube or rectangular prism
    """

    def __new__(
        cls, pose: "Pose", size: "Vector3", color: "Color"
    ) -> "CubePrimitive": ...

class CylinderPrimitive:
    """
    A primitive representing a cylinder, elliptic cylinder, or truncated cone
    """

    def __new__(
        cls,
        pose: "Pose",
        size: "Vector3",
        bottom_scale: "float",
        top_scale: "float",
        color: "Color",
    ) -> "CylinderPrimitive": ...

class Duration:
    """
    A duration in seconds and nanoseconds
    """

    def __new__(
        cls,
        seconds: int,
        nanos: int,
    ) -> "Duration": ...

class FrameTransform:
    """
    A transform between two reference frames in 3D space
    """

    def __new__(
        cls,
        timestamp: "Timestamp",
        parent_frame_id: "str",
        child_frame_id: "str",
        translation: "Vector3",
        rotation: "Quaternion",
    ) -> "FrameTransform": ...

class FrameTransforms:
    """
    An array of FrameTransform messages
    """

    def __new__(cls, transforms: "List[FrameTransform]") -> "FrameTransforms": ...

class GeoJson:
    """
    GeoJSON data for annotating maps
    """

    def __new__(cls, geojson: "str") -> "GeoJson": ...

class Grid:
    """
    A 2D grid of data
    """

    def __new__(
        cls,
        timestamp: "Timestamp",
        frame_id: "str",
        pose: "Pose",
        column_count: "int",
        cell_size: "Vector2",
        row_stride: "int",
        cell_stride: "int",
        fields: "List[PackedElementField]",
        data: "bytes",
    ) -> "Grid": ...

class ImageAnnotations:
    """
    Array of annotations for a 2D image
    """

    def __new__(
        cls,
        circles: "List[CircleAnnotation]",
        points: "List[PointsAnnotation]",
        texts: "List[TextAnnotation]",
    ) -> "ImageAnnotations": ...

class KeyValuePair:
    """
    A key with its associated value
    """

    def __new__(cls, key: "str", value: "str") -> "KeyValuePair": ...

class LaserScan:
    """
    A single scan from a planar laser range-finder
    """

    def __new__(
        cls,
        timestamp: "Timestamp",
        frame_id: "str",
        pose: "Pose",
        start_angle: "float",
        end_angle: "float",
        ranges: "List[float]",
        intensities: "List[float]",
    ) -> "LaserScan": ...

class LinePrimitive:
    """
    A primitive representing a series of points connected by lines
    """

    def __new__(
        cls,
        type: "LinePrimitiveLineType",
        pose: "Pose",
        thickness: "float",
        scale_invariant: "bool",
        points: "List[Point3]",
        color: "Color",
        colors: "List[Color]",
        indices: "List[int]",
    ) -> "LinePrimitive": ...

class LinePrimitiveLineType(Enum):
    """
    An enumeration indicating how input points should be interpreted to create lines
    """

    LineStrip = 0
    LineLoop = 1
    LineList = 2

class LocationFix:
    """
    A navigation satellite fix for any Global Navigation Satellite System
    """

    def __new__(
        cls,
        timestamp: "Timestamp",
        frame_id: "str",
        latitude: "float",
        longitude: "float",
        altitude: "float",
        position_covariance: "List[float]",
        position_covariance_type: "LocationFixPositionCovarianceType",
    ) -> "LocationFix": ...

class LocationFixPositionCovarianceType(Enum):
    """
    Type of position covariance
    """

    Unknown = 0
    Approximated = 1
    DiagonalKnown = 2
    Known = 3

class Log:
    """
    A log message
    """

    def __new__(
        cls,
        timestamp: "Timestamp",
        level: "LogLevel",
        message: "str",
        name: "str",
        file: "str",
        line: "int",
    ) -> "Log": ...

class LogLevel(Enum):
    """
    Log level
    """

    Unknown = 0
    Debug = 1
    Info = 2
    Warning = 3
    Error = 4
    Fatal = 5

class ModelPrimitive:
    """
    A primitive representing a 3D model file loaded from an external URL or embedded data
    """

    def __new__(
        cls,
        pose: "Pose",
        scale: "Vector3",
        color: "Color",
        override_color: "bool",
        url: "str",
        media_type: "str",
        data: "bytes",
    ) -> "ModelPrimitive": ...

class PackedElementField:
    """
    A field present within each element in a byte array of packed elements.
    """

    def __new__(
        cls, name: "str", offset: "int", type: "PackedElementFieldNumericType"
    ) -> "PackedElementField": ...

class PackedElementFieldNumericType(Enum):
    """
    Numeric type
    """

    Unknown = 0
    Uint8 = 1
    Int8 = 2
    Uint16 = 3
    Int16 = 4
    Uint32 = 5
    Int32 = 6
    Float32 = 7
    Float64 = 8

class Point2:
    """
    A point representing a position in 2D space
    """

    def __new__(cls, x: "float", y: "float") -> "Point2": ...

class Point3:
    """
    A point representing a position in 3D space
    """

    def __new__(cls, x: "float", y: "float", z: "float") -> "Point3": ...

class PointCloud:
    """
    A collection of N-dimensional points, which may contain additional fields with information like normals, intensity, etc.
    """

    def __new__(
        cls,
        timestamp: "Timestamp",
        frame_id: "str",
        pose: "Pose",
        point_stride: "int",
        fields: "List[PackedElementField]",
        data: "bytes",
    ) -> "PointCloud": ...

class PointsAnnotation:
    """
    An array of points on a 2D image
    """

    def __new__(
        cls,
        timestamp: "Timestamp",
        type: "PointsAnnotationType",
        points: "List[Point2]",
        outline_color: "Color",
        outline_colors: "List[Color]",
        fill_color: "Color",
        thickness: "float",
    ) -> "PointsAnnotation": ...

class PointsAnnotationType(Enum):
    """
    Type of points annotation
    """

    Unknown = 0
    Points = 1
    LineLoop = 2
    LineStrip = 3
    LineList = 4

class Pose:
    """
    A position and orientation for an object or reference frame in 3D space
    """

    def __new__(cls, position: "Vector3", orientation: "Quaternion") -> "Pose": ...

class PoseInFrame:
    """
    A timestamped pose for an object or reference frame in 3D space
    """

    def __new__(
        cls, timestamp: "Timestamp", frame_id: "str", pose: "Pose"
    ) -> "PoseInFrame": ...

class PosesInFrame:
    """
    An array of timestamped poses for an object or reference frame in 3D space
    """

    def __new__(
        cls, timestamp: "Timestamp", frame_id: "str", poses: "List[Pose]"
    ) -> "PosesInFrame": ...

class Quaternion:
    """
    A [quaternion](https://eater.net/quaternions) representing a rotation in 3D space
    """

    def __new__(
        cls, x: "float", y: "float", z: "float", w: "float"
    ) -> "Quaternion": ...

class RawImage:
    """
    A raw image
    """

    def __new__(
        cls,
        timestamp: "Timestamp",
        frame_id: "str",
        width: "int",
        height: "int",
        encoding: "str",
        step: "int",
        data: "bytes",
    ) -> "RawImage": ...

class SceneEntity:
    """
    A visual element in a 3D scene. An entity may be composed of multiple primitives which all share the same frame of reference.
    """

    def __new__(
        cls,
        timestamp: "Timestamp",
        frame_id: "str",
        id: "str",
        lifetime: "Duration",
        frame_locked: "bool",
        metadata: "List[KeyValuePair]",
        arrows: "List[ArrowPrimitive]",
        cubes: "List[CubePrimitive]",
        spheres: "List[SpherePrimitive]",
        cylinders: "List[CylinderPrimitive]",
        lines: "List[LinePrimitive]",
        triangles: "List[TriangleListPrimitive]",
        texts: "List[TextPrimitive]",
        models: "List[ModelPrimitive]",
    ) -> "SceneEntity": ...

class SceneEntityDeletion:
    """
    Command to remove previously published entities
    """

    def __new__(
        cls, timestamp: "Timestamp", type: "SceneEntityDeletionType", id: "str"
    ) -> "SceneEntityDeletion": ...

class SceneEntityDeletionType(Enum):
    """
    An enumeration indicating which entities should match a SceneEntityDeletion command
    """

    MatchingId = 0
    All = 1

class SceneUpdate:
    """
    An update to the entities displayed in a 3D scene
    """

    def __new__(
        cls, deletions: "List[SceneEntityDeletion]", entities: "List[SceneEntity]"
    ) -> "SceneUpdate": ...

class SpherePrimitive:
    """
    A primitive representing a sphere or ellipsoid
    """

    def __new__(
        cls, pose: "Pose", size: "Vector3", color: "Color"
    ) -> "SpherePrimitive": ...

class TextAnnotation:
    """
    A text label on a 2D image
    """

    def __new__(
        cls,
        timestamp: "Timestamp",
        position: "Point2",
        text: "str",
        font_size: "float",
        text_color: "Color",
        background_color: "Color",
    ) -> "TextAnnotation": ...

class TextPrimitive:
    """
    A primitive representing a text label
    """

    def __new__(
        cls,
        pose: "Pose",
        billboard: "bool",
        font_size: "float",
        scale_invariant: "bool",
        color: "Color",
        text: "str",
    ) -> "TextPrimitive": ...

class Timestamp:
    """
    A timestamp in seconds and nanoseconds
    """

    def __new__(
        cls,
        seconds: int,
        nanos: int,
    ) -> "Timestamp": ...

class TriangleListPrimitive:
    """
    A primitive representing a set of triangles or a surface tiled by triangles
    """

    def __new__(
        cls,
        pose: "Pose",
        points: "List[Point3]",
        color: "Color",
        colors: "List[Color]",
        indices: "List[int]",
    ) -> "TriangleListPrimitive": ...

class Vector2:
    """
    A vector in 2D space that represents a direction only
    """

    def __new__(cls, x: "float", y: "float") -> "Vector2": ...

class Vector3:
    """
    A vector in 3D space that represents a direction only
    """

    def __new__(cls, x: "float", y: "float", z: "float") -> "Vector3": ...
