// Generated by https://github.com/foxglove/schemas

import { Color } from "./Color";
import { Pose } from "./Pose";
import { Vector3 } from "./Vector3";

/** (Experimental, subject to change) A primitive representing a 3D model file loaded from an external URL or embedded data */
export type ModelPrimitive = {
  /** Origin of model relative to reference frame */
  pose: Pose;

  /** Scale factor to apply to the model along each axis */
  scale: Vector3;

  /** Solid color to use for the whole model if `override_color` is true. */
  color: Color;

  /** Whether to use the color specified in `color` instead of any materials embedded in the original model. */
  override_color: boolean;

  /** URL pointing to model file. One of `url` or `data` should be provided. */
  url: string;

  /** [Media type](https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types) of embedded model (e.g. `model/gltf-binary`). Required if `data` is provided instead of `url`. Overrides the inferred media type if `url` is provided. */
  media_type: string;

  /** Embedded model. One of `url` or `data` should be provided. If `data` is provided, `media_type` must be set to indicate the type of the data. */
  data: Uint8Array;
};