// Generated by https://github.com/foxglove/foxglove-sdk
// Options: {}

import { Time } from "./Time";

/** A raw image */
export type RawImage = {
  /** Timestamp of image */
  timestamp: Time;

  /** Frame of reference for the image. The origin of the frame is the optical center of the camera. +x points to the right in the image, +y points down, and +z points into the plane of the image. */
  frame_id: string;

  /** Image width */
  width: number;

  /** Image height */
  height: number;

  /**
   * Encoding of the raw image data
   * 
   * Supported values: `8UC1`, `8UC3`, `16UC1` (little endian), `32FC1` (little endian), `bayer_bggr8`, `bayer_gbrg8`, `bayer_grbg8`, `bayer_rggb8`, `bgr8`, `bgra8`, `mono8`, `mono16`, `rgb8`, `rgba8`, `uyvy` or `yuv422`, `yuyv` or `yuv422_yuy2`
   */
  encoding: string;

  /** Byte length of a single row */
  step: number;

  /** Raw image data */
  data: Uint8Array;
};
