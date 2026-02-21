import fs from "fs";
import Ajv from "ajv";

const schema = JSON.parse(
  fs.readFileSync("llmd-dcs-dictionary.schema.json", "utf8")
);

const dict = JSON.parse(
  fs.readFileSync(process.argv[2], "utf8")
);

const ajv = new Ajv({
  allErrors: true,
  strict: true
});

const validate = ajv.compile(schema);
const valid = validate(dict);

if (valid) {
  console.log("✅ Dictionary is valid.");
} else {
  console.error("❌ Dictionary validation failed:");
  console.error(validate.errors);
  process.exit(1);
}