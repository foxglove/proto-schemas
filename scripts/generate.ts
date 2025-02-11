import { program } from "commander";
import fs from "fs/promises";
import { spawnSync } from "node:child_process";
import path from "path";
import { rimraf } from "rimraf";

import { generateRosMsg, generateRosMsgDefinition } from "../typescript/schemas/src/internal";
import { exportTypeScriptSchemas } from "../typescript/schemas/src/internal/exportTypeScriptSchemas";
import {
  BYTE_VECTOR_FB,
  DURATION_FB,
  TIME_FB,
  generateFlatbuffers,
} from "../typescript/schemas/src/internal/generateFlatbufferSchema";
import { generateJsonSchema } from "../typescript/schemas/src/internal/generateJsonSchema";
import { generateMarkdown } from "../typescript/schemas/src/internal/generateMarkdown";
import {
  DURATION_IDL,
  TIME_IDL,
  generateOmgIdl,
} from "../typescript/schemas/src/internal/generateOmgIdl";
import { generateProto } from "../typescript/schemas/src/internal/generateProto";
import {
  foxgloveEnumSchemas,
  foxgloveMessageSchemas,
} from "../typescript/schemas/src/internal/schemas";

async function logProgress(message: string, body: () => Promise<void>) {
  process.stderr.write(`${message}... `);
  await body();
  process.stderr.write("done\n");
}

async function main({ clean }: { clean: boolean }) {
  const repoRoot = path.resolve(__dirname, "..");
  const outDir = path.join(repoRoot, "schemas");
  const rosOutDir = path.join(repoRoot, "ros_foxglove_msgs");
  const typescriptTypesDir = path.join(repoRoot, "typescript/schemas/src/types");

  await logProgress("Removing existing output directories", async () => {
    await rimraf(outDir);
    await rimraf(path.join(rosOutDir, "ros1"));
    await rimraf(path.join(rosOutDir, "ros2"));
    await rimraf(typescriptTypesDir);
  });

  if (clean) {
    // we're all done here
    return;
  }

  await logProgress("Generating JSONSchema definitions", async () => {
    await fs.mkdir(path.join(outDir, "jsonschema"), { recursive: true });
    let indexTS = "// Generated by https://github.com/foxglove/schemas\n\n";
    for (const schema of Object.values(foxgloveMessageSchemas)) {
      const json = JSON.stringify(generateJsonSchema(schema), undefined, 2);
      await fs.writeFile(path.join(outDir, "jsonschema", `${schema.name}.json`), json + "\n");
      indexTS += `export const ${schema.name} = ${json};\n\n`;
    }
    await fs.writeFile(path.join(outDir, "jsonschema", `index.ts`), indexTS);
  });

  await logProgress("Generating ROS 1 msg files", async () => {
    await fs.mkdir(path.join(outDir, "ros1"), { recursive: true });
    await fs.mkdir(path.join(rosOutDir, "ros1"), { recursive: true });
    for (const schema of Object.values(foxgloveMessageSchemas)) {
      if (schema.rosEquivalent != undefined) {
        continue;
      }
      const msg = generateRosMsg(generateRosMsgDefinition(schema, { rosVersion: 1 }), {
        rosVersion: 1,
      });
      await fs.writeFile(path.join(outDir, "ros1", `${schema.name}.msg`), msg);
      await fs.writeFile(path.join(rosOutDir, "ros1", `${schema.name}.msg`), msg);
    }
  });

  await logProgress("Generating ROS 2 msg files", async () => {
    await fs.mkdir(path.join(outDir, "ros2"), { recursive: true });
    await fs.mkdir(path.join(rosOutDir, "ros2"), { recursive: true });
    for (const schema of Object.values(foxgloveMessageSchemas)) {
      if (schema.rosEquivalent != undefined) {
        continue;
      }
      const msg = generateRosMsg(generateRosMsgDefinition(schema, { rosVersion: 2 }), {
        rosVersion: 2,
      });
      await fs.writeFile(path.join(outDir, "ros2", `${schema.name}.msg`), msg);
      await fs.writeFile(path.join(rosOutDir, "ros2", `${schema.name}.msg`), msg);
    }
  });

  await logProgress("Generating Protobuf definitions", async () => {
    await fs.mkdir(path.join(outDir, "proto", "foxglove"), { recursive: true });
    for (const schema of Object.values(foxgloveMessageSchemas)) {
      const enums = Object.values(foxgloveEnumSchemas).filter(
        (enumSchema) => enumSchema.parentSchemaName === schema.name,
      );
      await fs.writeFile(
        path.join(outDir, "proto", "foxglove", `${schema.name}.proto`),
        generateProto(schema, enums),
      );
    }
  });

  await logProgress("Generating FlatBuffer definitions", async () => {
    await fs.mkdir(path.join(outDir, "flatbuffer"), { recursive: true });
    await fs.writeFile(path.join(outDir, "flatbuffer", "ByteVector.fbs"), BYTE_VECTOR_FB);
    await fs.writeFile(path.join(outDir, "flatbuffer", "Time.fbs"), TIME_FB);
    await fs.writeFile(path.join(outDir, "flatbuffer", "Duration.fbs"), DURATION_FB);

    for (const schema of Object.values(foxgloveMessageSchemas)) {
      // want enums with their corresponding parent tables for usage
      const enums = Object.values(foxgloveEnumSchemas).filter(
        (enumSchema) => enumSchema.parentSchemaName === schema.name,
      );
      await fs.writeFile(
        path.join(outDir, "flatbuffer", `${schema.name}.fbs`),
        generateFlatbuffers(schema, enums),
      );
    }
  });

  await logProgress("Generating TypeScript definitions", async () => {
    await fs.mkdir(typescriptTypesDir, { recursive: true });

    const schemas = exportTypeScriptSchemas();
    for (const [name, source] of schemas.entries()) {
      await fs.writeFile(path.join(typescriptTypesDir, `${name}.ts`), source);
    }
  });

  await logProgress("Generating OMG IDL definitions", async () => {
    await fs.mkdir(path.join(outDir, "omgidl", "foxglove"), { recursive: true });
    await fs.writeFile(path.join(outDir, "omgidl", "foxglove", "Time.idl"), TIME_IDL);
    await fs.writeFile(path.join(outDir, "omgidl", "foxglove", "Duration.idl"), DURATION_IDL);
    for (const schema of Object.values(foxgloveMessageSchemas)) {
      await fs.writeFile(
        path.join(outDir, "omgidl", "foxglove", `${schema.name}.idl`),
        generateOmgIdl(schema),
      );
    }
    for (const schema of Object.values(foxgloveEnumSchemas)) {
      await fs.writeFile(
        path.join(outDir, "omgidl", "foxglove", `${schema.name}.idl`),
        generateOmgIdl(schema),
      );
    }
  });

  await logProgress("Generating README.md", async () => {
    await fs.writeFile(
      path.join(outDir, "README.md"),
      generateMarkdown(Object.values(foxgloveMessageSchemas), Object.values(foxgloveEnumSchemas)),
    );
  });

  await logProgress("Running yarn test --updateSnapshot", async () => {
    const result = spawnSync("yarn", ["test", "--updateSnapshot"], {
      stdio: "inherit",
    });
    if (result.status !== 0) {
      throw new Error(`yarn test failed with code ${result.status ?? "unknown"}`);
    }
  });
}

program.option("--clean", "remove all generated files").action(main);
program.parseAsync().catch(console.error);
