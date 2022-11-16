import { FoxgloveEnumSchema, FoxglovePrimitive, FoxgloveSchema } from "./types";

// Flatbuffers only supports nested vectors via table
export const BYTE_VECTOR_FB = `
namespace foxglove;

/// Used for nesting byte vectors
table ByteVector {
  data:[uint8];
}
root_type ByteVector;
`;

// Same as protobuf wellknown types
export const TIME_FB = `
namespace foxglove;

struct Time {
  /// 	Represents seconds of UTC time since Unix epoch 1970-01-01T00:00:00Z
  sec:uint64;
  /// Nano-second fractions from 0 to 999,999,999 inclusive
  nsec:uint32;
}
`;

export const DURATION_FB = `
namespace foxglove;

struct Duration {
  /// Signed seconds of the span of time. Must be from -315,576,000,000 to +315,576,000,000 inclusive.
  sec:int64;
  /// if sec === 0 : -999,999,999 <= nsec <= +999,999,999 
  /// otherwise sign of sec must match sign of nsec or be 0 and abs(nsec) <= 999,999,999
  nsec:int32;
}
`;

function primitiveToFlatbuffers(type: Exclude<FoxglovePrimitive, "time" | "duration">) {
  switch (type) {
    case "uint32":
      return "uint32";
    case "bytes":
      return "[uint8]";
    case "string":
      return "string";
    case "boolean":
      return "bool";
    case "float64":
      return "double";
  }
}

export function generateFlatbuffers(
  schema: FoxgloveSchema,
  nestedEnums: FoxgloveEnumSchema[],
): string {
  const enumDefinitions: string[] = [];
  for (const enumSchema of nestedEnums) {
    const fields = enumSchema.values.map(({ name, value, description }) => {
      if (description != undefined) {
        return `/// ${description}\n  ${name} = ${value},`;
      } else {
        return `${name} = ${value},`;
      }
    });
    enumDefinitions.push(
      // `///` comments required to show up in compiled flatbuffer schemas
      `/// ${enumSchema.description}\nenum ${enumSchema.name} : ubyte {\n  ${fields.join(
        "\n\n  ",
      )}\n}\n`,
    );
  }

  let definition;
  const imports = new Set<string>();
  switch (schema.type) {
    case "enum": {
      const fields = schema.values.map(({ name, value, description }) => {
        if (description != undefined) {
          return `/// ${description}\n  ${name} = ${value},`;
        } else {
          return `${name} = ${value},`;
        }
      });

      // `///` comments required to show up in compiled flatbuffer schemas
      definition = `/// ${schema.description}\nenum ${schema.name} : ubyte {\n  ${fields.join(
        "\n\n  ",
      )}\n}\n`;
      break;
    }
    case "message": {
      const fields = schema.fields.map((field) => {
        const isArray = field.array != undefined;

        let type;
        switch (field.type.type) {
          case "enum":
            type = field.type.enum.name;
            break;
          case "nested":
            type = `foxglove.${field.type.schema.name}`;
            imports.add(field.type.schema.name);
            break;
          case "primitive":
            if (field.type.name === "time") {
              type = "Time";
              imports.add(`Time`);
            } else if (field.type.name === "duration") {
              type = "Duration";
              imports.add(`Duration`);
            } else if (field.type.name === "bytes" && isArray) {
              type = "ByteVector";
              imports.add("ByteVector");
            } else {
              type = primitiveToFlatbuffers(field.type.name);
            }
            break;
        }
        let lengthComment;

        if (typeof field.array === "number") {
          // can't specify length of vector outside of struct, all of these are tables
          lengthComment = `  /// length ${field.array}\n`;
        }
        let defaultValue;
        if (field.defaultValue != undefined) {
          defaultValue = field.defaultValue;
        }

        return `${field.description
          .trim()
          .split("\n")
          .map((line) => `  /// ${line}\n`)
          .join("")}${
          // can't have inline comments, so the lengthComment needs to be above
          lengthComment ?? ""
          // convert field.name to lowercase for flatbuffer compilation compliance
        }  ${field.name.toLowerCase()}:${isArray ? `[${type}]` : type}${
          defaultValue ? ` = ${defaultValue}` : ""
        };`;
      });

      definition = `${enumDefinitions.join("\n\n")}/// ${schema.description}\ntable ${
        schema.name
      } {\n${fields.join("\n\n")}\n}\n\nroot_type ${schema.name};`;
      break;
    }
  }

  const outputSections = [
    `// Generated by https://github.com/foxglove/schemas`,

    Array.from(imports)
      .sort()
      .map((name) => `include "${name}.fbs";`)
      .join("\n"),

    `namespace foxglove;`,

    definition,
  ].filter(Boolean);

  return outputSections.join("\n\n") + "\n";
}