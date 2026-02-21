import json
import sys
from jsonschema import Draft202012Validator

with open("llmd-dcs-dictionary.schema.json", "r", encoding="utf-8") as f:
    schema = json.load(f)

with open(sys.argv[1], "r", encoding="utf-8") as f:
    dictionary = json.load(f)

validator = Draft202012Validator(schema)
errors = sorted(validator.iter_errors(dictionary), key=lambda e: e.path)

if not errors:
    print("✅ Dictionary is valid.")
else:
    print("❌ Dictionary validation failed:")
    for error in errors:
        print(f"- {list(error.path)}: {error.message}")
    sys.exit(1)